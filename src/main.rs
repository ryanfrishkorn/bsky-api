mod task;

use axum::extract::Path;
use axum::response::sse::{Event, Sse};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use duckdb::{Config, Connection};
use futures::stream::Stream;
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use task::{Process, Task, TaskResult, TaskStatus};
use tokio::io::{AsyncBufReadExt, BufReader};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone, Debug, Serialize)]
struct AppState {
    // #[serde(skip)]
    // db: Option<Arc<Mutex<Connection>>>,
    #[serde(skip)]
    tasks: Arc<Mutex<usize>>,
    source_duckdb: bool,
    source_sqlite: bool,
    source_json: bool,
    start_time: std::time::SystemTime,
}

#[derive(Debug, serde::Serialize)]
struct JsonData {
    unix_ts: f64,
    msg: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        // db: None,
        tasks: Arc::new(Mutex::new(0)),
        source_json: false,
        source_sqlite: false,
        source_duckdb: false,
        start_time: std::time::SystemTime::now(),
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .format_file(true)
        .format_line_number(true)
        .init();
    let bind_address = "0.0.0.0";
    info!("starting with bind address {}", bind_address);
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    info!("cors_layer: {:?}", cors_layer);
    let app = Router::new()
        .route("/", get(root))
        .route("/state", get(api_state))
        .route("/task/{process}", get(run_task))
        .route("/task/{process}/stream", get(stream_task))
        .route("/search/{term}/{order_field}", get(search))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(cors_layer));
    let listener = tokio::net::TcpListener::bind(format!("{bind_address}:3000"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(State(state): State<AppState>) -> impl IntoResponse {
    let unix_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let tasks_locked = state.tasks.lock().expect("locking AppState tasks");
    let response = JsonData {
        unix_ts,
        msg: format!("tasks processed by api: {}", *tasks_locked),
    };
    info!("response: {:?}", response);

    Json(response)
}

async fn file_exists(path: &str) -> bool {
    let p = std::path::Path::new(path);
    p.exists()
}

async fn api_state(State(mut state): State<AppState>) -> impl IntoResponse {
    // search for database files
    state.source_duckdb = file_exists("data/jetstream.duckdb").await;
    state.source_json = file_exists("data/jetstream.json").await;
    state.source_sqlite = file_exists("data/jetstream.sqlite3").await;

    Json(state)
}

#[axum::debug_handler]
async fn run_task(Path(process): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let mut task = match process.as_str() {
        "uname" => Task::new(Process::Uname),
        "date" => Task::new(Process::Date),
        "jetstream" => Task::new(Process::Jetstream),
        _ => panic!("junk"),
    };

    task.status = TaskStatus::Running;
    // spawn task
    info!("task: {:?}", task);
    let mut cmd = Command::new(task.cmd.clone());
    for arg in task.args {
        cmd.arg(arg);
    }
    // spawn for blocking
    let result = tokio::task::spawn_blocking(move || {
        let output = cmd.output().expect("could not get output");
        let result: TaskResult = match cmd.status() {
            Ok(_) => {
                let s = String::from_utf8_lossy(&output.stdout).to_string();
                TaskResult::Success(s)
            }
            Err(_) => {
                let s = String::from_utf8_lossy(&output.stdout).to_string();
                TaskResult::Fail(s)
            }
        };
        info!("result: {:?}", result);
        result
    })
    .await
    .expect("execution error");
    let mut tasks_locked = state.tasks.lock().expect("locking AppState tasks");
    *tasks_locked += 1;

    Json(result)
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    did: String,
    cid: String,
    created_at: String,
    score: f64,
    text: String,
}

#[axum::debug_handler]
async fn search(
    Path(params): Path<(String, String)>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let (term, order_field) = params;
    let query_limit = 1000.to_string();
    // let order_field = "created_at";
    let mut posts: Vec<Post> = Vec::new();
    let db_cfg = Config::default()
        .access_mode(duckdb::AccessMode::ReadOnly)
        .expect("db config");
    let db = Connection::open_with_flags("data/jetstream.duckdb", db_cfg).expect("opening duckdb");

    // Set DuckDB configuration to allow match_bm25 macro to work
    db.execute("SET scalar_subquery_error_on_multiple_rows=false", [])
        .expect("setting scalar_subquery configuration");

    info!("searching for term: {}", term);
    let mut query = r#"
        select distinct(fts_main_posts.match_bm25(cid, ?, fields := 'text', k := 1.2, b := 0.75, conjunctive := 0)) as score, did, cid, created_at, text
        from posts
        where fts_main_posts.match_bm25(cid, ?, fields := 'text', k := 1.2, b := 0.75, conjunctive := 0) is not null
        and score >= 3.0
    "#.trim().to_string();
    query = match order_field.as_str() {
        "created_at" => format!("{} {}", query, "order by created_at desc"),
        _ => format!("{} {}", query, "order by score desc"),
    };
    query = format!("{} limit ?", query);
    debug!(
        "query: {}",
        query
            .split('\n')
            .map(|x| x.trim().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    );

    let mut stmt = db
        .prepare(&query)
        .map_err(|e| {
            log::error!("{}", e);
            panic!("panicked preparing statement");
        })
        .expect("logged error");

    let posts_iter = stmt
        .query_map([&term, &term, &query_limit], |row| {
            Ok(Post {
                score: row.get(0)?,
                did: row.get(1)?,
                cid: row.get(2)?,
                created_at: row.get(3)?,
                text: row.get(4)?,
            })
        })
        .expect("during query_map");

    for p in posts_iter {
        match p {
            Ok(good_post) => {
                trace!("post: {:?}", good_post);
                posts.push(good_post);
            }
            Err(e) => error!("{e}"),
        }
    }
    Json(posts)
}

#[axum::debug_handler]
async fn stream_task(
    Path(process): Path<Process>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let task = Task::new(process);
    info!("streaming task: {:?}", task);

    // Create the stream
    let stream = async_stream::stream! {
        // Spawn the process with piped stdout
        let mut cmd = tokio::process::Command::new(&task.cmd);
        for arg in &task.args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                yield Ok(Event::default().data(format!("Error spawning process: {}", e)));
                return;
            }
        };

        // Get stdout and create async line reader
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                info!("streaming line: {}", line);
                yield Ok(Event::default().data(line));
            }
        }

        // Wait for process to complete
        match child.wait().await {
            Ok(status) => {
                info!("process completed with status: {:?}", status);
                yield Ok(Event::default().data(format!("[DONE] Exit status: {}", status)));
            }
            Err(e) => {
                yield Ok(Event::default().data(format!("[ERROR] Process error: {}", e)));
            }
        }

        // Update task counter
        let mut tasks_locked = state.tasks.lock().expect("locking AppState tasks");
        *tasks_locked += 1;
    };

    Sse::new(stream)
}

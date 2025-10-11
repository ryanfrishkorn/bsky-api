mod task;

use axum::extract::Path;
use axum::{Json, Router, response::IntoResponse, routing::get};
use log::info;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use task::{Process, Task, TaskResult, TaskStatus};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, serde::Serialize)]
struct JsonData {
    unix_ts: f64,
    msg: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .format_file(true)
        .format_line_number(true)
        .init();
    let bind_address = "127.0.0.1";
    info!("starting with bind address {}", bind_address);
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    info!("cors_layer: {:?}", cors_layer);
    let app = Router::new()
        .route("/", get(root))
        .route("/task/{process}", get(run_task))
        .layer(ServiceBuilder::new().layer(cors_layer));
    let listener = tokio::net::TcpListener::bind(format!("{bind_address}:3000"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    let unix_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let response = JsonData {
        unix_ts,
        msg: "message from api".to_string(),
    };
    info!("response: {:?}", response);

    Json(response)
}

async fn run_task(Path(process): Path<String>) -> impl IntoResponse {
    let mut task = match process.as_str() {
        "uname" => Task::new(Process::Uname),
        "date" => Task::new(Process::Date),
        _ => panic!("junk"),
    };

    task.status = TaskStatus::Running;

    // spawn task
    info!("task: {:?}", task);
    let mut cmd = Command::new(task.cmd.clone());
    for arg in task.args {
        cmd.arg(arg);
    }
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

    Json(result)
}

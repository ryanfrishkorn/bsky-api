use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub process: Process, // for now
    pub cmd: String,
    pub args: Vec<String>,
    pub status: TaskStatus,
    pub result: Option<TaskResult>,
}

impl Task {
    pub fn new(process: Process) -> Task {
        match process {
            Process::ArchiveJetstream => Self {
                process,
                cmd: "./scripts/archive-jetstream".to_string(),
                args: Vec::new(),
                status: TaskStatus::Created,
                result: None,
            },
            Process::BskyTrending => Self {
                process,
                cmd: "./bin/bsky-trending".to_string(),
                args: [
                    "--db",
                    "data/jetstream.duckdb",
                    "--limit",
                    "500",
                    "--min",
                    "2",
                    "--max",
                    "4",
                ]
                .iter_mut()
                .map(|x| x.to_string())
                .collect(),
                status: TaskStatus::Created,
                result: None,
            },
            Process::BuildDuckDb => Self {
                process,
                cmd: "./scripts/build-duckdb".to_string(),
                args: ["data/jetstream.duckdb", "data/jetstream.sqlite3"]
                    .iter_mut()
                    .map(|x| x.to_string())
                    .collect(),
                status: TaskStatus::Created,
                result: None,
            },
            Process::ListData => Self {
                process,
                cmd: "./scripts/list-data".to_string(),
                args: vec![],
                status: TaskStatus::Created,
                result: None,
            },
            Process::ClearData => Self {
                process,
                cmd: "./scripts/clear-data".to_string(),
                args: vec![],
                status: TaskStatus::Created,
                result: None,
            },
            Process::Date => Self {
                process,
                cmd: "uname".to_string(),
                args: vec!["-snr".to_string()],
                status: TaskStatus::Created,
                result: None,
            },
            Process::Jetstream => Self {
                process,
                cmd: "./bin/jetstream-client".to_string(),
                args: [
                    "--json",
                    "data/jetstream.json",
                    "--db",
                    "data/jetstream.sqlite3",
                ]
                .iter_mut()
                .map(|x| x.to_string())
                .collect(),
                status: TaskStatus::Created,
                result: None,
            },
            Process::Uname => Self {
                process,
                cmd: "date".to_string(),
                args: Vec::new(),
                status: TaskStatus::Created,
                result: None,
            },
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Created,
    Finished(TaskResult),
    Queued,
    Running,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskResult {
    Success(String),
    Fail(String), // exit status, msg
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Process {
    ArchiveJetstream,
    BskyTrending,
    BuildDuckDb,
    ListData,
    ClearData,
    Date,
    Jetstream,
    Uname,
}

#[allow(dead_code)]
pub struct TaskQueue {
    tasks: Vec<Task>,
}

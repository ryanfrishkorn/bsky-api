use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
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
            Process::Date => Self {
                process,
                cmd: "uname".to_string(),
                args: vec!["-snr".to_string()],
                status: TaskStatus::Created,
                result: None,
            },
            Process::Uname => Self {
                process,
                cmd: "date".to_string(),
                args: vec![],
                status: TaskStatus::Created,
                result: None,
            },
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub enum TaskStatus {
    Created,
    Finished(TaskResult),
    Queued,
    Running,
}

#[derive(Clone, Debug, Serialize)]
pub enum TaskResult {
    Success(String),
    Fail(String), // exit status, msg
}

#[derive(Clone, Debug, Serialize)]
pub enum Process {
    Date,
    Uname,
}

#[allow(dead_code)]
pub struct TaskQueue {
    tasks: Vec<Task>,
}

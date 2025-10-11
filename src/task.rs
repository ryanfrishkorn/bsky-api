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
            // get system information
            Process::Uname => {
                let cmd = "uname".to_string();
                let args = vec!["-a".to_string()];

                Self {
                    process,
                    cmd,
                    args,
                    status: TaskStatus::Created,
                    result: None,
                }
            }
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
    Uname,
}

#[allow(dead_code)]
pub struct TaskQueue {
    tasks: Vec<Task>,
}

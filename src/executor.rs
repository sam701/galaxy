use std::fmt;
use std::io::{Read, Write};
use std::sync::Arc;

use ssh2::Session;

use crate::config::{Config, ConfigTaskId, Task, TaskContent};
use crate::Error;

pub struct ExecutionContext {
    pub host: Arc<String>,
    pub config: Arc<Config>,
    pub session: Session,
}

// impl ExecutionContext {
//     fn task(&self, task_id: &TaskId) -> Option<&Task> {
//         self.config.tasks.get(task_id)
//     }
// }

#[derive(Clone)]
pub struct TaskId(Arc<TaskIdInner>);

struct TaskIdInner {
    inner: ConfigTaskId,
    parent: Option<TaskId>,
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.parent {
            Some(parent) => write!(f, "{}.{:?}", parent, &self.0.inner),
            None => write!(f, "{}", &self.0.inner),
        }
    }
}

impl TaskId {
    pub fn new(task_id: &ConfigTaskId) -> Self {
        Self(Arc::new(TaskIdInner {
            inner: task_id.clone(),
            parent: None,
        }))
    }
}

pub fn execute_task(ctx: &ExecutionContext, id: TaskId, task: &Task) -> Result<(), Error> {
    let span = debug_span!("task", %id);
    let _enter = span.enter();

    debug!("start");
    match &task.content {
        TaskContent::Exec(cmd) => exec(&ctx.session, cmd).map(|r| {
            info!(exit_code=%r.exit_code, "done");
        }),
        _ => todo!(),
    }
}

fn exec(sess: &Session, script: &str) -> Result<ExecResult, Error> {
    let mut channel = sess.channel_session()?;
    channel.exec("sudo bash")?;
    channel.write_all("set -e \n".as_bytes())?;
    channel.write_all(script.as_bytes())?;
    channel.send_eof()?;

    let mut stdout = String::new();
    channel.read_to_string(&mut stdout)?;
    let exit_code = channel.exit_status().unwrap();

    channel.close()?;
    // channel.wait_close()?;
    Ok(ExecResult { stdout, exit_code })
}

struct ExecResult {
    stdout: String,
    exit_code: i32,
}

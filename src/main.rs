#[macro_use]
extern crate tracing;

use std::borrow::Cow;
use std::env;
use std::fs::canonicalize;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use ssh2::Session;
use syconf_serde::from_file;
use tracing::Level;

use crate::config::Config;
use crate::executor::{ExecutionContext, TaskId, execute_task};
use std::sync::Arc;

mod config;
mod executor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SSH: {}", .0)]
    Ssh(#[from] ssh2::Error),
    #[error("IO: {}", .0)]
    Io(#[from] std::io::Error),
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("galaxy=DEBUG")
        .init();

    let config_file = "./test.sy";
    match from_file::<Config>(config_file) {
        Ok(config) => {
            debug!("loaded configuration from {}: {:?}", &config_file, &config);

            let host = &config.hosts[0];
            let span = info_span!("remote", host=?host.host);
            let _enter = span.enter();

            let tcp =
                TcpStream::connect(format!("{}:{}", host.host, host.port.unwrap_or(22))).unwrap();

            let mut sess = Session::new().unwrap();
            sess.set_tcp_stream(tcp);
            sess.handshake().unwrap();

            let keys = &host.keys;
            let public = keys.public.as_ref().map(|x| ssh_key_path(&x));
            sess.userauth_pubkey_file(
                &host.username,
                public.as_ref().map(|x|&**x),
                &*ssh_key_path(&keys.private),
                None,
            )
            .unwrap();
            info!("Connected");


            let ctx = ExecutionContext {
                host: host.host.clone(),
                session: sess.clone(),
                config: Arc::new(config),
            };
            let (task_id, task) = ctx.config.tasks.iter().next().unwrap();
            let task_id = TaskId::new(task_id);
            execute_task(&ctx, task_id, task).unwrap();
        }
        Err(e) => println!("ERROR: {}", e.to_string()),
    }
}

fn ssh_key_path(key: &str) -> Cow<Path> {
    let p = Path::new(key);
    if p.is_absolute() {
        Cow::Borrowed(p)
    } else {
        let ssh_dir = env::var("HOME").expect("home dir");
        Cow::Owned(Path::new(&ssh_dir).join(".ssh").join(p))
    }
}

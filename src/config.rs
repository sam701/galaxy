use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use serde::Deserialize;
use syconf_serde::Function;
use std::ops::Deref;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct Task {
    #[serde(flatten)]
    pub content: TaskContent,
    #[serde(default = "default_public")]
    pub public: bool,
    #[serde(default)]
    pub state_type: StateType,
    #[serde(default)]
    pub when: When,
    pub requires: Option<Vec<ConfigTaskId>>,
}

fn default_public() -> bool {
    false
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskContent {
    Exec(Command),
    Group(HashMap<ConfigTaskId, Task>),
    Func(Function),
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Command(String);

impl Deref for Command {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum StateType {
    ContentHash,
    /// Raw stdout content
    StdoutString,
    /// Stdout content hashed
    StdoutHash,

    /// The function takes a single argument, the stdout as string
    Func(Function),

    /// Hash of the child states.
    AllChildrenHash,
    /// Hash of a child with the provided ID.
    ChildState(String),
}

impl Default for StateType {
    fn default() -> Self {
        StateType::ContentHash
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum When {
    Always,
    Func(Function),
}

impl Default for When {
    fn default() -> Self {
        When::Always
    }
}

#[derive(Debug, Deserialize, Hash, Eq, PartialEq, Clone)]
#[serde(transparent)]
pub struct ConfigTaskId(Arc<String>);

impl fmt::Display for ConfigTaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub tasks: HashMap<ConfigTaskId, Task>,
    pub hosts: Vec<Host>,
}

#[derive(Debug, Deserialize)]
pub struct Host {
    pub host: Arc<String>,
    pub port: Option<i32>,
    pub keys: SshKeys,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SshKeys {
    pub private: String,
    pub public: Option<String>,
}

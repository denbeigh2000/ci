use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const DEFAULT_TIMEOUT_MINUTES: u16 = 20;

fn default_timeout() -> u16 {
    DEFAULT_TIMEOUT_MINUTES
}

#[derive(Deserialize, Serialize)]
pub struct CommandStep {
    #[serde(default)]
    allow_dependency_failure: bool,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    concurrency_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(default = "default_timeout")]
    timeout_in_minutes: u16,
}

impl CommandStep {
    pub fn builder() -> CommandStepBuilder {
        CommandStepBuilder::default()
    }
}

fn format_command(cmd: Vec<String>) -> String {
    let utf_vec: Vec<_> = cmd
        .into_iter()
        .map(|v| String::from_utf8(shell_quote::Bash::quote(&v)).unwrap())
        .collect();
    utf_vec.join(" ")
}

#[derive(Default)]
pub struct CommandStepBuilder {
    allow_dependency_failure: bool,
    concurrency_group: Option<String>,
    depends_on: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    label: Option<String>,
    timeout_in_minutes: Option<u16>,
}

impl CommandStepBuilder {
    pub fn set_allow_dependency_failure(&mut self, val: bool) -> &mut Self {
        self.allow_dependency_failure = val;
        self
    }

    pub fn set_concurrency_group(&mut self, val: String) -> &mut Self {
        self.concurrency_group = Some(val);
        self
    }

    pub fn set_depends_on(&mut self, val: Vec<String>) -> &mut Self {
        self.depends_on = Some(val);
        self
    }

    pub fn set_env(&mut self, key: String, val: String) -> &mut Self {
        if self.env.is_none() {
            self.env = Some(HashMap::new());
        }

        self.env.as_mut().unwrap().insert(key, val);
        self
    }

    pub fn set_label(&mut self, val: String) -> &mut Self {
        self.label = Some(val);
        self
    }

    pub fn set_timeout_in_minutes(&mut self, val: u16) -> &mut Self {
        self.timeout_in_minutes = Some(val);
        self
    }

    pub fn build(self, key: String, command: Vec<String>) -> CommandStep {
        CommandStep {
            key,
            command: format_command(command),
            allow_dependency_failure: self.allow_dependency_failure,
            concurrency_group: self.concurrency_group,
            depends_on: self.depends_on,
            env: self.env,
            label: self.label,
            timeout_in_minutes: self.timeout_in_minutes.unwrap_or(DEFAULT_TIMEOUT_MINUTES),
        }
    }
}

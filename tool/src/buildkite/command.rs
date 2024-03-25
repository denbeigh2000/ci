use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const DEFAULT_TIMEOUT_MINUTES: u16 = 20;

#[derive(Deserialize, Serialize)]
pub struct CommandStep {
    allow_depdendency_failure: bool,
    command: Vec<String>,
    concurrency_group: Option<String>,
    depends_on: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    key: String,
    label: Option<String>,
    timeout_in_minutes: u16,
}

impl CommandStep {
    pub fn builder() -> CommandStepBuilder {
        CommandStepBuilder::default()
    }
}

#[derive(Default)]
pub struct CommandStepBuilder {
    allow_depdendency_failure: bool,
    concurrency_group: Option<String>,
    depends_on: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    label: Option<String>,
    timeout_in_minutes: Option<u16>,
}

impl CommandStepBuilder {
    pub fn set_allow_dependency_failure(&mut self, val: bool) -> &mut Self {
        self.allow_depdendency_failure = val;
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
            command,
            allow_depdendency_failure: self.allow_depdendency_failure,
            concurrency_group: self.concurrency_group,
            depends_on: self.depends_on,
            env: self.env,
            label: self.label,
            timeout_in_minutes: self.timeout_in_minutes.unwrap_or(DEFAULT_TIMEOUT_MINUTES),
        }
    }
}

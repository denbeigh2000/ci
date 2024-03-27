use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TriggerStep {
    r#async: bool,
    build: TriggerBuildSpec,
}

#[derive(Deserialize, Serialize)]
pub struct TriggerBuildSpec {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta_data: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depends_on: Option<Vec<String>>,
}

impl TriggerBuildSpec {
    pub fn builder() -> TriggerBuildSpecBuilder {
        TriggerBuildSpecBuilder::default()
    }
}

#[derive(Default)]
pub struct TriggerBuildSpecBuilder {
    branch: Option<String>,
    commit: Option<String>,
    env: Option<HashMap<String, String>>,
    label: Option<String>,
    message: Option<String>,
    meta_data: Option<HashMap<String, String>>,
    depends_on: Option<Vec<String>>,
}

impl TriggerBuildSpecBuilder {
    pub fn set_branch(&mut self, branch: String) -> &mut Self {
        self.branch = Some(branch);
        self
    }

    pub fn set_commit(&mut self, commit: String) -> &mut Self {
        self.commit = Some(commit);
        self
    }

    pub fn set_env(&mut self, key: String, val: String) -> &mut Self {
        if self.env.is_none() {
            self.env = Some(HashMap::new());
        }

        self.env.as_mut().unwrap().insert(key, val);
        self
    }

    pub fn set_label(mut self, label: String) {
        self.label = Some(label);
    }

    pub fn set_message(mut self, message: String) {
        self.message = Some(message);
    }

    pub fn set_meta_data(&mut self, key: String, val: String) -> &mut Self {
        if self.meta_data.is_none() {
            self.meta_data = Some(HashMap::new());
        }

        self.meta_data.as_mut().unwrap().insert(key, val);
        self
    }

    pub fn add_depends_on(&mut self, item: String) -> &mut Self {
        if self.depends_on.is_none() {
            self.depends_on = Some(Vec::new());
        }

        self.depends_on.as_mut().unwrap().push(item);
        self
    }

    pub fn build(self, key: String) -> TriggerBuildSpec {
        TriggerBuildSpec {
            key,
            branch: self.branch,
            commit: self.commit,
            env: self.env,
            label: self.label,
            message: self.message,
            meta_data: self.meta_data,
            depends_on: self.depends_on,
        }
    }
}

use std::collections::HashMap;

use serde::{de::Visitor, Deserialize, Serialize};

mod command;
mod trigger;

pub use command::CommandStep;
pub use trigger::{TriggerBuildSpec, TriggerStep};

// NOTE: only supporting what we need here. Unfortunately, the buildkite client
// library and JSON-Schema codegen ecosystems in rust are both....limited.
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Step {
    Block(BlockStep),
    Command(CommandStep),
    Trigger(TriggerStep),
    Wait(WaitStep),
}

pub enum BlockState {
    Passed,
    Failed,
    Running,
}

impl Serialize for BlockState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Running => "running",
        };

        serializer.serialize_str(v)
    }
}

struct BlockStateVisitor;

impl<'de> Visitor<'de> for BlockStateVisitor {
    type Value = BlockState;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("one of [\"passed\", \"failed\", \"running\"]")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "passed" => Ok(BlockState::Passed),
            "failed" => Ok(BlockState::Failed),
            "running" => Ok(BlockState::Running),
            _ => Err(E::custom(format!("unexpected value {v}"))),
        }
    }
}

impl<'de> Deserialize<'de> for BlockState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(BlockStateVisitor)
    }
}

#[derive(Deserialize, Serialize)]
pub struct BlockStep {
    /// Label of the block step
    block: String,
    blocked_state: BlockState,
    depends_on: Option<Vec<String>>,
    // TODO: fields?
    label: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct WaitStep {
    allow_dependency_failure: bool,
    continue_on_failure: bool,
    depends_on: Option<Vec<String>>,
    key: String,
}

impl WaitStep {
    pub fn builder() -> WaitStepBuilder {
        WaitStepBuilder::default()
    }
}

#[derive(Default)]
pub struct WaitStepBuilder {
    allow_dependency_failure: bool,
    continue_on_failure: bool,
    depends_on: Option<Vec<String>>,
}

impl WaitStepBuilder {
    pub fn set_allow_dependency_failure(&mut self, val: bool) -> &mut Self {
        self.allow_dependency_failure = val;
        self
    }

    pub fn set_continue_on_failure(&mut self, val: bool) -> &mut Self {
        self.continue_on_failure = val;
        self
    }

    pub fn set_depends_on(&mut self, val: Vec<String>) -> &mut Self {
        self.depends_on = Some(val);
        self
    }

    pub fn build(self, key: String) -> WaitStep {
        WaitStep {
            key,
            allow_dependency_failure: self.allow_dependency_failure,
            continue_on_failure: self.continue_on_failure,
            depends_on: self.depends_on,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Pipeline {
    steps: Vec<Step>,
    env: Option<HashMap<String, String>>,
}

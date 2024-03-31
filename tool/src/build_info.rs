use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::buildkite::Step;
use crate::flags::BuildkiteArgs;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const SYSTEM: &str = "aarch64-darwin";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const SYSTEM: &str = "aarch64-linux";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const SYSTEM: &str = "x86_64-darwin";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const SYSTEM: &str = "x86_64-linux";

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildTargetType {
    Package,
    DevShell,
    #[serde(rename = "nixos")]
    NixOSConfiguration,
    #[serde(rename = "darwin")]
    NixDarwinConfiguration,
    // maybe one day?
    #[serde(rename = "home")]
    HomeManagerConfiguration,
}

#[derive(Deserialize)]
pub struct FoundDerivationBuild {
    pub name: String,
    pub build_type: BuildTargetType,
    pub path: PathBuf,
    pub tag: String,
}

impl FoundDerivationBuild {
    pub fn label(&self) -> String {
        let emoji = match self.build_type {
            BuildTargetType::Package => "package",
            BuildTargetType::NixDarwinConfiguration => "mac",
            BuildTargetType::HomeManagerConfiguration => "house_with_garden",
            BuildTargetType::NixOSConfiguration => "nix",
            BuildTargetType::DevShell => "terminal",
        };

        format!(":hammer_and_wrench: :{emoji}: {}", self.name)
    }
}

#[derive(Deserialize, Serialize)]
struct PipelineInfo {
    id: String,
    slug: String,
}

#[derive(Deserialize)]
pub struct BuildEvaluation {
    pub builds: HashMap<String, FoundDerivationBuild>,
    pub steps: Vec<Step>,
}

#[derive(thiserror::Error, Debug)]
pub enum EvaluationError {
    #[error("Error running `nix eval`: {0}")]
    LaunchingNix(std::io::Error),
    #[error("Error parsing JSON from nix: {0}")]
    ParsingJSON(#[from] serde_json::Error),
}

impl BuildEvaluation {
    pub fn from_env(path: &Path) -> Result<Self, EvaluationError> {
        let target = format!(".#ci.{SYSTEM}.config.evaluation");
        let data = std::process::Command::new("nix")
            .args(["eval", "--json", &target])
            .current_dir(path)
            .output()
            .map_err(EvaluationError::LaunchingNix)?;

        if !data.stderr.is_empty() {
            let stderr = String::from_utf8(data.stderr).unwrap();
            eprintln!("{stderr}");
        }

        // TODO: will we need to dedupe in the future for (e.g.) default
        // targets?
        let eval: Self = serde_json::from_slice(&data.stdout)?;
        Ok(eval)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CIRunStateWriteToFileError {
    #[error("creating state file: {0}")]
    Creating(std::io::Error),
    #[error("writing to file: {0}")]
    Writing(std::io::Error),
    #[error("converting to json::Value: {0}")]
    Converting(#[from] serde_json::Error),
}

#[derive(Deserialize, Serialize)]
pub struct CIRunState {
    commit: String,
    branch: Option<String>,
    tag: Option<String>,
    pipeline: PipelineInfo,

    repo: String,
}

impl CIRunState {
    pub fn from_args(args: BuildkiteArgs) -> Self {
        CIRunState {
            commit: args.commit,
            branch: args.branch,
            tag: args.tag,

            pipeline: PipelineInfo {
                id: args.pipeline_id,
                slug: args.pipeline_slug,
            },

            repo: args.repository,
        }
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), CIRunStateWriteToFileError> {
        {
            let json_val = serde_json::to_value(self).unwrap();
            let json_data = json_digest::canonical_json(&json_val).unwrap();
            let mut state_file =
                File::create(path).map_err(CIRunStateWriteToFileError::Creating)?;
            state_file
                .write_all(json_data.as_bytes())
                .map_err(CIRunStateWriteToFileError::Writing)?;
        };

        Ok(())
    }
}

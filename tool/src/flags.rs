use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::LevelFilter;

lazy_static::lazy_static! {
    static ref DEFAULT_LOG_LEVEL: LevelFilter = {
        let ci_str = std::env::var("CI");
        match ci_str.as_ref().map(|m| m.as_str()) {
            Err(_) | Ok("0") | Ok("false") | Ok("no") | Ok("n") => LevelFilter::Warn,
            Ok(_) => LevelFilter::Info,
        }
    };
}

#[derive(Clone)]
pub struct BuildkiteArgs {
    pub commit: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub repository: String,
    pub path: PathBuf,

    pub pipeline_id: String,
    pub pipeline_slug: String,
}

#[derive(Parser)]
pub struct CliArgs {
    #[arg(long, env = "BUILDKITE_COMMIT")]
    pub commit: String,
    #[arg(long, env = "BUILDKITE_REPO")]
    pub repository: String,
    #[arg(long, env = "BUILDKITE_BUILD_CHECKOUT_PATH")]
    pub path: PathBuf,

    #[arg(long, env = "BUILDKITE_PIPELINE_ID")]
    pub pipeline_id: String,
    #[arg(long, env = "BUILDKITE_PIPELINE_SLUG")]
    pub pipeline_slug: String,

    #[arg(long, env = "BUILDKITE_BRANCH")]
    pub branch: Option<String>,
    #[arg(long, env = "BUILDKITE_TAG")]
    pub tag: Option<String>,

    #[arg(long, env = "CI_COMMAND", default_value = "ci")]
    pub ci_cmd: String,

    #[arg(long, env = "LOG_LEVEL", default_value_t = *DEFAULT_LOG_LEVEL)]
    pub log_level: LevelFilter,

    #[command(subcommand)]
    pub action: Action,
}
impl CliArgs {
    pub fn into_parts(self) -> (String, LevelFilter, Action, BuildkiteArgs) {
        (
            self.ci_cmd,
            self.log_level,
            self.action,
            BuildkiteArgs {
                commit: self.commit,
                branch: self.branch,
                tag: self.tag,
                repository: self.repository,
                path: self.path,
                pipeline_id: self.pipeline_id,
                pipeline_slug: self.pipeline_slug,
            },
        )
    }
}

#[derive(Subcommand)]
pub enum Action {
    /// Collect all results at the end of a CI run.
    Collect,
    /// Evaluate the derivations to be built for this commit
    Evaluate,
    /// Execute a build target
    Execute { target: String },
    /// Build a derivation
    Build { target: String },
}

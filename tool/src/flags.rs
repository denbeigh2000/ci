use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    #[command(subcommand)]
    pub action: Action,

    #[arg(env = "BUILDKITE_COMMIT")]
    pub commit: String,
    #[arg(env = "BUILDKITE_BRANCH")]
    pub branch: Option<String>,
    #[arg(env = "BUILDKITE_TAG")]
    pub tag: Option<String>,
    #[arg(env = "BUILDKITE_REPO")]
    pub repository: String,
    #[arg(env = "BUILDKITE_BUILD_CHECKOUT_PATH")]
    pub path: PathBuf,

    #[arg(env = "BUILDKITE_PIPELINE_ID")]
    pub pipeline_id: String,
    #[arg(env = "BUILDKITE_PIPELINE_SLUG")]
    pub pipeline_slug: String,
}
impl CliArgs {
    pub fn into_parts(self) -> (Action, BuildkiteArgs) {
        (
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
    Capture,
    Evaluate,
    Execute {
        #[arg(short)]
        target: String,
    },
}

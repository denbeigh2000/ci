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

    #[command(subcommand)]
    pub action: Action,
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
    Collect,
    Evaluate,
    Execute {
        #[arg(short)]
        target: String,
    },
}

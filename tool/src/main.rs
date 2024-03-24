use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const SYSTEM: &str = "aarch64-darwin";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const SYSTEM: &str = "aarch64-linux";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const SYSTEM: &str = "x86_64-darwin";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const SYSTEM: &str = "x86_64-linux";

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(env = "BUILDKITE_COMMIT")]
    commit: String,
    #[arg(env = "BUILDKITE_BRANCH")]
    branch: String,
    #[arg(env = "BUILDKITE_REPO")]
    repository: String,
    #[arg(env = "BUILDKITE_BUILD_CHECKOUT_PATH")]
    path: PathBuf,
}

// #[derive(Subcommand)]
// enum Action {
//     Evaluate,
// }

#[derive(Deserialize)]
struct FoundDerivationBuild {
    name: String,
    path: PathBuf,
    tag: String,
}

type BuildkiteStep = serde_json::Value;

#[derive(Serialize)]
struct BuildkitePipeline {
    steps: Vec<BuildkiteStep>,
}

#[derive(Deserialize)]
struct BuildEvaluation {
    builds: HashMap<String, FoundDerivationBuild>,
    steps: Vec<BuildkiteStep>,
}

#[derive(thiserror::Error, Debug)]
enum EvaluationError {
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

        let eval: Self = serde_json::from_slice(&data.stdout)?;
        Ok(eval)
    }
}

fn main() {
    let args = Args::parse();
    let _eval = BuildEvaluation::from_env(&args.path).unwrap();

    // let mut builds = create_builds(eval);
    // builds.extend(args.steps)p;

    // TODO:
    //  - create JSON file and `git add` it to repo when evaluating
    //    - write comment explaining caching/ephemerality balance so I feel
    //      better about doing it
    //  - add build caching mechanism, and use it to find derivations to skip
    //    building
    //    - initially just be naive and always build, later use a server with
    //      a db
    //  - create buildkite JSON for triggered new builds, prepend it to
    //    explicit steps
    //    - remember to set previously-assumed step_key
    //  - upload steps to buildkite
    //  - write nix wrapper for tool
}

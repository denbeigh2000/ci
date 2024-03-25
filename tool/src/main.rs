use std::io::Write;
use std::path::Path;
use std::process::Command;

use build_info::{CIRunStateWriteToFileError, EvaluationError};
use buildkite::WaitStep;
use clap::Parser;
use flags::{Action, BuildkiteArgs};
use git::{
    apply_patch, fetch_patch, ApplyPatchError, CreateCommitError, FetchPatchError,
    UploadingPatchError,
};
use serde::Serialize;

use crate::build_info::{BuildEvaluation, CIRunState};
use crate::buildkite::{CommandStep, Step};
use crate::flags::CliArgs;
use crate::git::{create_commit, upload_patch};

mod build_info;
#[allow(dead_code)]
mod buildkite;
mod flags;
mod git;

#[derive(Serialize)]
struct BuildkitePipeline {
    steps: Vec<Step>,
}

#[derive(thiserror::Error, Debug)]
enum CaptureError {
    #[error("error writing CI state to file: {0}")]
    WritingToFile(#[from] CIRunStateWriteToFileError),
    #[error("error creating git commit: {0}")]
    CreatingCommit(#[from] CreateCommitError),
    #[error("error uploading patch file: {0}")]
    UploadingPatch(#[from] UploadingPatchError),
}

fn capture_buildkite_state(args: BuildkiteArgs) -> Result<(), CaptureError> {
    let path = args.path.clone();

    let state = CIRunState::from_args(args);
    let state_file_path = Path::new("./nix/build-info.json");
    state.write_to_file(state_file_path)?;

    create_commit(&path, state_file_path)?;
    upload_patch()?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum ApplyError {
    #[error("error fetching patch: {0}")]
    FetchingPatch(#[from] FetchPatchError),
    #[error("error applying patch: {0}")]
    ApplyingPatch(#[from] ApplyPatchError),
}

fn apply(args: &BuildkiteArgs) -> Result<(), ApplyError> {
    fetch_patch()?;
    apply_patch(&args.path)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum EvaluateError {
    #[error("error capturing CI state in git: {0}")]
    CapturingGitState(#[from] CaptureError),
    #[error("error evaluating CI state: {0}")]
    EvaluatingState(#[from] EvaluationError),
    #[error("error encoding pipeline to JSON: {0}")]
    Encoding(#[from] serde_json::Error),
    #[error("error starting `buildkite-agent`: {0}")]
    InvokingBKAgent(std::io::Error),
    #[error("error writing JSON data to `buildkite-agent`: {0}")]
    WritingToBKAgent(std::io::Error),
    #[error("error waiting for `buildkite-agent` to finish: {0}")]
    WaitingForBKAgent(std::io::Error),
    #[error("`buildkite-agent` exited unsuccessfully (status {0:?}):\n{1}")]
    BKAgentExitState(Option<i32>, String),
}

fn evaluate(args: BuildkiteArgs) -> Result<i32, EvaluateError> {
    capture_buildkite_state(args.clone())?;
    let eval = BuildEvaluation::from_env(&args.path)?;

    // start with all the steps building our derivations
    // TODO: check which derivations have been built already
    let mut steps: Vec<_> = eval
        .builds
        .into_iter()
        .map(|(k, v)| {
            let mut b = CommandStep::builder();
            let args = Vec::from([
                // NOTE: assuming this exists?
                "ci".to_string(),
                "execute".to_string(),
                format!(".#{}", v.tag),
            ]);
            b.set_label(format!("build {}", v.name));
            Step::Command(b.build(k, args))
        })
        .collect();

    let mut b = CommandStep::builder();
    b.set_label(":thinking: collecting results".to_string());
    b.set_timeout_in_minutes(3);

    // add a wait step so all builds run first (necessary?)
    steps.push(Step::Wait(
        WaitStep::builder().build("wait-builds".to_string()),
    ));
    // add the additional requested ones from our evaluated config
    // (likely releases, deployments, other automated actions)
    steps.extend(eval.steps);

    // Add a collection step
    let final_step = Step::Command(b.build(
        "collect-results".to_string(),
        Vec::from(["ci".to_string(), "collect".to_string()]),
    ));

    // append one more step to upload state at the end
    steps.push(final_step);

    let pipeline = BuildkitePipeline { steps };
    let json_data = serde_json::to_vec(&pipeline)?;

    // upload them all to buildkite!
    let mut handle = std::process::Command::new("buildkite-agent")
        .args(["pipeline", "upload"])
        .spawn()
        .map_err(EvaluateError::InvokingBKAgent)?;
    let mut stdin = handle.stdin.take().unwrap();
    stdin
        .write_all(&json_data)
        .map_err(EvaluateError::WritingToBKAgent)?;

    let output = handle
        .wait_with_output()
        .map_err(EvaluateError::WaitingForBKAgent)?;
    if !output.status.success() {
        return Err(EvaluateError::BKAgentExitState(
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(0)
}

#[derive(thiserror::Error, Debug)]
enum ExecuteError {
    #[error("error applying git state: {0}")]
    ApplyingPatch(#[from] ApplyError),
    #[error("spawning `nix run` subprocess: {0}")]
    SpawiningProcess(std::io::Error),
    #[error("waiting for subprocess to finish: {0}")]
    AwaitingProcess(std::io::Error),
}

fn execute(args: BuildkiteArgs, target: String) -> Result<i32, ExecuteError> {
    apply(&args)?;
    // We need to do the execution through here, after applying the commit with
    // our trigger info in it, so that we evaluate the versions of the scripts
    // with information potentially embedded (tags, etc).
    let target_str = format!(".#{target}");
    let res = Command::new("nix")
        .args(["run", &target_str])
        .spawn()
        .map_err(ExecuteError::SpawiningProcess)?
        .wait()
        .map_err(ExecuteError::AwaitingProcess)?;

    // NOTE: I think missing a code here means something is not ok?
    Ok(res.code().unwrap_or(1))
}

#[derive(thiserror::Error, Debug)]
pub enum CollectError {}

fn collect_final_pipeline_state(_args: BuildkiteArgs) -> Result<i32, CollectError> {
    Ok(0)
    // TODO: Collect results from the builds and send them back to DB for
    // caching
}

#[derive(thiserror::Error, Debug)]
enum MainError {
    #[error("error evaluating CI state: {0}")]
    Evaluating(#[from] EvaluateError),
    #[error("error executing CI step: {0}")]
    Executing(#[from] ExecuteError),
    #[error("error collecting final pipeline state: {0}")]
    Collecting(#[from] CollectError),
}

fn real_main() -> Result<i32, MainError> {
    let args = CliArgs::parse();
    let (action, bk) = args.into_parts();
    let code = match action {
        Action::Evaluate => evaluate(bk)?,
        Action::Execute { target } => execute(bk, target)?,
        // TODO: need to have this collect information about the CI job after
        // all steps have finished
        Action::Collect => collect_final_pipeline_state(bk)?,
    };

    Ok(code)

    // TODO:
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

fn main() {
    let code = match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    };

    std::process::exit(code);
}

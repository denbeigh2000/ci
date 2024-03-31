use std::io::Write;
use std::process::{Command, Stdio};

use build_info::{CIRunStateWriteToFileError, EvaluationError};
use buildkite::WaitStep;
use clap::Parser;
#[cfg(debug_assertions)]
use develop::{print_cmd, IS_DEVELOP_MODE};
use flags::{Action, BuildkiteArgs};
use git::{
    apply_patch, fetch_patch, ApplyPatchError, CreateCommitError, FetchPatchError,
    UploadingPatchError,
};
use serde::Serialize;

use crate::build_info::{BuildEvaluation, CIRunState};
use crate::buildkite::{CommandStep, Step};
use crate::flags::CliArgs;
use crate::git::{create_state_commit, upload_patch};

mod build_info;
#[allow(dead_code)]
mod buildkite;
#[cfg(debug_assertions)]
mod develop;
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
    let state_file_path = path.join("./nix/build-info.json");
    state.write_to_file(&state_file_path)?;

    create_state_commit(&path, &state_file_path)?;
    upload_patch(&path)?;

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
enum DerivePipelineError {
    #[error("error capturing CI state in git: {0}")]
    CapturingGitState(#[from] CaptureError),
    #[error("error evaluating CI state: {0}")]
    EvaluatingState(#[from] EvaluationError),
}

#[derive(thiserror::Error, Debug)]
enum EvaluateError {
    #[error("error deriving pipeline: {0}")]
    Deriving(#[from] DerivePipelineError),
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

// TODO: should this have its' own error type?
fn make_buildkite_pipeline(args: BuildkiteArgs) -> Result<BuildkitePipeline, DerivePipelineError> {
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
                // TODO: should we also be wrapping this?
                "nix".to_string(),
                "build".to_string(),
                format!(".#{}", v.tag),
            ]);
            b.set_label(format!(":hammer_and_wrench: build {}", v.name));
            Step::Command(b.build(format!("build-{k}"), args))
        })
        .collect();

    // add a wait step so all builds run first (necessary?)
    steps.push(Step::Wait(
        WaitStep::builder().build("wait-builds".to_string()),
    ));
    // add the additional requested ones from our evaluated config
    // (likely releases, deployments, other automated actions)
    steps.extend(eval.steps);

    let mut wait_step_b = WaitStep::builder();
    wait_step_b
        .set_allow_dependency_failure(true)
        .set_continue_on_failure(true);
    let wait_step = wait_step_b.build("wait-final".to_string());

    // Add a collection step for after all the other steps are done
    let mut cmd_step_b = CommandStep::builder();
    cmd_step_b
        .set_label(":shopping_trolley: collect results".to_string())
        .set_timeout_in_minutes(3)
        .set_allow_dependency_failure(true);
    let cmd_step = cmd_step_b.build(
        "collect-results".to_string(),
        ["nix", "run", ".#tool", "--", "collect"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    steps.extend([Step::Wait(wait_step), Step::Command(cmd_step)]);

    Ok(BuildkitePipeline { steps })
}

fn evaluate(args: BuildkiteArgs) -> Result<i32, EvaluateError> {
    let pipeline = make_buildkite_pipeline(args)?;
    let json_data = serde_json::to_vec(&pipeline)?;

    let mut cmd = std::process::Command::new("buildkite-agent");
    cmd.args(["pipeline", "upload"]);
    #[cfg(debug_assertions)]
    if *IS_DEVELOP_MODE {
        print_cmd("buildkite-agent", &cmd);
        let data = String::from_utf8(json_data).unwrap();
        println!("data: {data}");
        return Ok(0);
    }
    cmd.stdin(Stdio::piped());
    let mut handle = cmd.spawn().map_err(EvaluateError::InvokingBKAgent)?;
    {
        let mut stdin = handle.stdin.take().unwrap();
        stdin
            .write_all(&json_data)
            .map_err(EvaluateError::WritingToBKAgent)?;
    }

    let data = String::from_utf8(json_data).unwrap();
    eprintln!("{data}");

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

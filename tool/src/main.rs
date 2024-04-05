use std::process::Command;

use build_info::{CIRunStateWriteToFileError, EvaluationError};
use buildkite::{RunError, WaitStep};
use clap::Parser;
use flags::{Action, BuildkiteArgs};
use git::{
    apply_patch, fetch_patch, ApplyPatchError, CreateCommitError, FetchPatchError,
    UploadingPatchError,
};
use serde::Serialize;
use simple_logger::SimpleLogger;

use crate::build_info::{BuildEvaluation, CIRunState};
use crate::buildkite::{Cli, CommandStep, Step};
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
    let state_file_path = path.join("./build-info.json");
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
    log::info!("fetching patch");
    fetch_patch()?;
    log::info!("applying patch {}", args.path.to_string_lossy());
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
    #[error("error running buildkite-agent: {0}")]
    UploadingPipeline(#[from] RunError),
}

// TODO: should this have its' own error type?
fn make_buildkite_pipeline(
    cmd: String,
    args: BuildkiteArgs,
) -> Result<BuildkitePipeline, DerivePipelineError> {
    capture_buildkite_state(args.clone())?;
    let mut eval = BuildEvaluation::from_env(&args.path)?;

    // start with all the steps building our derivations
    // TODO: check which derivations have been built already
    let mut steps: Vec<_> = eval
        .builds
        .into_iter()
        .map(|(k, v)| {
            let mut b = CommandStep::builder();
            let args = format!("$CI_COMMAND build {}", v.tag);
            b.set_label(v.label());
            Step::Command(b.build(format!("build-{k}"), args))
        })
        .collect();

    if !eval.steps.is_empty() {
        // add a wait step so all builds run first (necessary?)
        steps.push(Step::Wait(
            WaitStep::builder().build("wait-builds".to_string()),
        ));

        eval.steps.iter_mut().for_each(|mut step| {
            if let Step::Command(ref mut s) = &mut step {
                s.command = s.command.replace("@tool@", &cmd);
            }
        });

        // add the additional requested ones from our evaluated config
        // (likely releases, deployments, other automated actions)
        steps.extend(eval.steps);
    }

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
        [cmd, "collect".to_string()].join(" "),
    );

    steps.extend([Step::Wait(wait_step), Step::Command(cmd_step)]);

    Ok(BuildkitePipeline { steps })
}

fn evaluate(cmd_name: String, args: BuildkiteArgs) -> Result<i32, EvaluateError> {
    log::info!("Evaluating pipeline");
    let pipeline = make_buildkite_pipeline(cmd_name, args)?;
    log::trace!("Encoding to JSON");
    let json_data = serde_json::to_vec(&pipeline)?;

    log::info!("Uploading buildkite pipeline");
    Cli.pipeline_upload_bytes(&json_data)?;

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

fn nix_action(
    action: &[&'static str],
    args: BuildkiteArgs,
    target: String,
) -> Result<i32, ExecuteError> {
    let msg = action.join(" ");
    log::info!("preparing `nix {msg}`");
    apply(&args)?;
    let target_str = format!(".#{target}");
    log::info!("running `nix {msg} {target_str}`");
    let res = Command::new("nix")
        .args(action)
        .arg(&target_str)
        .spawn()
        .map_err(ExecuteError::SpawiningProcess)?
        .wait()
        .map_err(ExecuteError::AwaitingProcess)?;

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

    let (cmd, log_level, action, bk) = args.into_parts();
    SimpleLogger::new()
        .with_level(log_level)
        .init()
        .expect("failed to set logging");
    let code = match action {
        Action::Evaluate => evaluate(cmd, bk)?,
        Action::Execute { target } => nix_action(&["run"], bk, target)?,
        Action::Build { target } => nix_action(&["build", "--no-link"], bk, target)?,
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

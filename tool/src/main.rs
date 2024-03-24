use std::path::Path;

use build_info::{CIRunStateWriteToFileError, EvaluationError};
use clap::Parser;
use flags::{Action, BuildkiteArgs};
use git::{
    apply_patch, fetch_patch, ApplyPatchError, CreateCommitError, FetchPatchError,
    UploadingPatchError,
};
use serde::Serialize;

use crate::build_info::{BuildEvaluation, BuildkiteStep, CIRunState};
use crate::flags::CliArgs;
use crate::git::{create_commit, upload_patch};

mod build_info;
mod flags;
mod git;

#[derive(Serialize)]
struct BuildkitePipeline {
    steps: Vec<BuildkiteStep>,
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

fn capture(args: BuildkiteArgs) -> Result<(), CaptureError> {
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
}

fn evaluate(args: BuildkiteArgs) -> Result<(), EvaluateError> {
    capture(args.clone())?;
    let eval = BuildEvaluation::from_env(&args.path)?;

    // TODO:
    // - check which derivations have been built already
    // - create buildkite steps for derivations
    // - prepend them to the additional ones from nix
    // - append one more step to upload state at the end
    // - upload them all to buildkite!

    unimplemented!()
}

#[derive(thiserror::Error, Debug)]
enum ExecuteError {
    #[error("error applying git state: {0}")]
    ApplyingPatch(#[from] ApplyError),
}

fn execute(args: BuildkiteArgs, target: String) -> Result<(), ExecuteError> {
    apply(&args)?;

    unimplemented!()
}

#[derive(thiserror::Error, Debug)]
enum MainError {
    #[error("error evaluating CI state: {0}")]
    Evaluating(#[from] EvaluateError),
    #[error("error executing CI step: {0}")]
    Executing(#[from] ExecuteError),
}

fn real_main() -> Result<(), MainError> {
    let args = CliArgs::parse();
    let (action, bk) = args.into_parts();
    match action {
        Action::Evaluate => evaluate(bk)?,
        Action::Execute { target } => execute(bk, target)?,
        // TODO: need to have this collect information about the CI job after
        // all steps have finished
        Action::Capture => unimplemented!(),
    };

    unimplemented!()

    // TODO:
    //  - create JSON file with meta-info in it, commit it and create a patch
    //    file for it, upload to buildkite as an artifact. Apply it when
    //    evaluating to get full benefits of cache re-use, and maybe cache
    //    previous runs?
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
    if let Err(e) = real_main() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

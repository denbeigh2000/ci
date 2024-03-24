use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

lazy_static::lazy_static! {
    static ref PATCH_PATHBUF: PathBuf = PathBuf::from_str(PATCH_PATH).unwrap();
}

const PATCH_PATH: &str = "./ci-data.patch";

const GIT_NAME: &str = "CI Bot";
const GIT_EMAIL: &str = "ci@denbeigh.cloud";

#[derive(thiserror::Error, Debug)]
pub enum UploadingPatchError {
    #[error("failed to invoke `buildkite-agent`: {0}")]
    InvokingBKAgent(#[from] std::io::Error),
    #[error("error status {0:?} from `buildkite-agent`: {1}")]
    BKAgentStatus(Option<i32>, String),
}

pub fn upload_patch() -> Result<(), UploadingPatchError> {
    let upload_result = std::process::Command::new("buildkite-agent")
        .args(["artifact", "upload", PATCH_PATH])
        .output()?;

    if !upload_result.status.success() {
        let out = String::from_utf8_lossy(&upload_result.stderr).to_string();
        return Err(UploadingPatchError::BKAgentStatus(
            upload_result.status.code(),
            out,
        ));
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum CreateCommitError {
    #[error("error running `git add`: {0}")]
    InvokingGitAdd(std::io::Error),
    #[error("`git add` exited with {0:?}: {1}")]
    GitAddStatus(Option<i32>, String),

    #[error("error running `git commit`: {0}")]
    InvokingGitCommit(std::io::Error),
    #[error("`git commit` exited with {0:?}: {1}")]
    GitCommitStatus(Option<i32>, String),
}

pub fn create_commit(repo: &Path, state_file: &Path) -> Result<(), CreateCommitError> {
    let res = Command::new("git")
        .current_dir(repo)
        .args(["add", &state_file.to_string_lossy()])
        .output()
        .map_err(CreateCommitError::InvokingGitAdd)?;

    if !res.status.success() {
        return Err(CreateCommitError::GitAddStatus(
            res.status.code(),
            String::from_utf8_lossy(&res.stderr).to_string(),
        ));
    }

    let commit_res = Command::new("git")
        .current_dir(repo)
        .args(["commit", "--message", "automated CI state commit"])
        .env("GIT_COMMITTER_EMAIL", GIT_EMAIL)
        .env("GIT_COMMITTER_NAME", GIT_NAME)
        .output()
        .map_err(CreateCommitError::InvokingGitCommit)?;

    if !commit_res.status.success() {
        return Err(CreateCommitError::GitCommitStatus(
            commit_res.status.code(),
            String::from_utf8_lossy(&res.stderr).to_string(),
        ));
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum FetchPatchError {
    #[error("error invoking BK Agent: {0}")]
    InvokingBKAgent(std::io::Error),
    #[error("error status {0:?}, from BK Agent while downloading/writing: {1}")]
    Transferring(Option<i32>, String),
}

pub fn fetch_patch() -> Result<(), FetchPatchError> {
    let output = Command::new("buildkite-agent")
        .args(["artifact", "download", PATCH_PATH])
        .output()
        .map_err(FetchPatchError::InvokingBKAgent)?;

    let status = output.status;
    if !status.success() {
        let st = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(FetchPatchError::Transferring(status.code(), st));
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ApplyPatchError {
    #[error("Starting git: {0}")]
    SpawningGit(#[from] std::io::Error),
    #[error("git exited with code {code:?}\n{stderr}")]
    GitError { code: Option<i32>, stderr: String },
}

pub fn apply_patch(repo_path: &Path) -> Result<(), ApplyPatchError> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args([
            "apply-mailbox",
            PATCH_PATH,
            "--committer-date-is-author-date",
        ])
        .env("GIT_COMMITTER_EMAIL", GIT_EMAIL)
        .env("GIT_COMMITTER_NAME", GIT_NAME)
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        let code = output.status.code();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ApplyPatchError::GitError { code, stderr });
    }

    Ok(())
}

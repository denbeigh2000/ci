use std::process::Command;
use std::{io::Write, path::Path};

#[cfg(debug_assertions)]
use crate::develop::{print_cmd, IS_DEVELOP_MODE};

const PATCH_PATH: &str = "./ci-data.patch";

const GIT_NAME: &str = "CI Bot";
const GIT_EMAIL: &str = "ci@denbeigh.cloud";

#[derive(thiserror::Error, Debug)]
pub enum UploadingPatchError {
    #[error("failed to invoke `buildkite-agent`: {0}")]
    InvokingBKAgent(#[from] std::io::Error),
    #[error("error status {0:?} from `buildkite-agent`: {1}")]
    BKAgentStatus(Option<i32>, String),
    #[error("failed to invoke `git`: {0}")]
    InvokingGit(std::io::Error),
    #[error("error status {0:?} from `git`: {1}")]
    GitStatus(Option<i32>, String),
}

fn format_patch(repo: &Path) -> Result<Vec<u8>, UploadingPatchError> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["format-patch", "-n1", "--stdout"])
        .current_dir(repo);

    let output = cmd.output().map_err(UploadingPatchError::InvokingGit)?;
    if !output.status.success() {
        let out = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(UploadingPatchError::GitStatus(output.status.code(), out));
    }

    Ok(output.stdout)
}

pub fn upload_patch(repo: &Path) -> Result<(), UploadingPatchError> {
    let patch_data = format_patch(repo)?;
    {
        let mut f = std::fs::File::create(repo.join(PATCH_PATH)).expect("creating file");
        f.write_all(&patch_data).expect("writing data");
    }

    let mut cmd = std::process::Command::new("buildkite-agent");
    cmd.args(["artifact", "upload", PATCH_PATH]);
    #[cfg(debug_assertions)]
    if *IS_DEVELOP_MODE {
        print_cmd("buildkite-agent", &cmd);
        return Ok(());
    }

    let upload_result = cmd.output()?;
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

fn add_file(repo: &Path, state_file: &Path) -> Result<(), CreateCommitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo)
        .args(["add", &state_file.to_string_lossy()]);

    #[cfg(debug_assertions)]
    if *IS_DEVELOP_MODE {
        print_cmd("git", &cmd);
        return Ok(());
    }
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

    Ok(())
}

fn create_commit(repo: &Path) -> Result<(), CreateCommitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo)
        .args(["commit", "--message", "automated CI state commit"])
        .env("GIT_COMMITTER_EMAIL", GIT_EMAIL)
        .env("GIT_COMMITTER_NAME", GIT_NAME);

    #[cfg(debug_assertions)]
    if *IS_DEVELOP_MODE {
        print_cmd("git", &cmd);
        return Ok(());
    }

    let res = cmd.output().map_err(CreateCommitError::InvokingGitCommit)?;

    if !res.status.success() {
        return Err(CreateCommitError::GitCommitStatus(
            res.status.code(),
            String::from_utf8_lossy(&res.stderr).to_string(),
        ));
    }

    Ok(())
}

pub fn create_state_commit(repo: &Path, state_file: &Path) -> Result<(), CreateCommitError> {
    add_file(repo, state_file)?;
    create_commit(repo)?;
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
    let mut cmd = Command::new("buildkite-agent");
    cmd.args(["artifact", "download", PATCH_PATH]);

    #[cfg(debug_assertions)]
    if *IS_DEVELOP_MODE {
        print_cmd("buildkite-agent", &cmd);
        return Ok(());
    }

    let output = cmd.output().map_err(FetchPatchError::InvokingBKAgent)?;

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
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_path)
        .args([
            "apply-mailbox",
            PATCH_PATH,
            "--committer-date-is-author-date",
        ])
        .env("GIT_COMMITTER_EMAIL", GIT_EMAIL)
        .env("GIT_COMMITTER_NAME", GIT_NAME)
        .current_dir(repo_path);

    #[cfg(debug_assertions)]
    if *IS_DEVELOP_MODE {
        print_cmd("git", &cmd);
        return Ok(());
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let code = output.status.code();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ApplyPatchError::GitError { code, stderr });
    }

    Ok(())
}

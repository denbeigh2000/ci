use std::{
    io::Write,
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus, Output, Stdio},
};

#[cfg(debug_assertions)]
use crate::develop::{print_cmd, IS_DEVELOP_MODE};

#[derive(thiserror::Error, Debug)]
pub enum RunError {
    #[error("error spawning buildkite-agent: {0}")]
    Spawning(std::io::Error),
    #[error("buildkite-agent exited with error code: ({0:?})\n{1}")]
    ExitedWithError(Option<i32>, String),
    #[error("error writing input to stdin: {0}")]
    WritingInput(std::io::Error),
    #[error("error awaiting process: {0}")]
    Awaiting(std::io::Error),
}

#[derive(Default)]
pub struct Cli;

impl Cli {
    fn run(self, args: &[&str], input: Option<&[u8]>) -> Result<Output, RunError> {
        let mut cmd = Command::new("buildkite-agent");
        cmd.args(args);
        if input.is_some() {
            cmd.stdin(Stdio::piped());
        }

        log::debug!("executing: `buildkite-agent {}`", args.join(" "));

        #[cfg(debug_assertions)]
        if *IS_DEVELOP_MODE {
            print_cmd("buildkite-agent", &cmd);
            return Ok(Output {
                status: ExitStatus::from_raw(0),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }

        let mut child = cmd.spawn().map_err(RunError::Spawning)?;

        if let Some(i) = input {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(i).map_err(RunError::WritingInput)?;
        }

        let output = child.wait_with_output().map_err(RunError::Awaiting)?;
        let status = output.status;
        if !status.success() {
            let st = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(RunError::ExitedWithError(status.code(), st));
        }

        Ok(output)
    }

    pub fn upload(self, paths: &[&str]) -> Result<(), RunError> {
        log::debug!("uploading buildkite artifacts: {}", paths.join(" "));
        let mut args = Vec::from(["buildkite-agent", "artifact", "upload"]);
        for path in paths {
            args.push(path);
        }

        self.run(&args, None)?;

        Ok(())
    }

    pub fn download(self, query: &str, dest: &str) -> Result<(), RunError> {
        log::debug!("downloading buildkite artifacts `{query}` to `{dest}`");
        let args = ["buildkite-agent", "artifact", "download", query, dest];
        self.run(&args, None)?;

        Ok(())
    }

    pub fn pipeline_upload_bytes(self, data: &[u8]) -> Result<(), RunError> {
        log::debug!("Uploading buildkite pipeline {} bytes", data.len());
        self.run(&["pipeline", "upload"], Some(data))?;

        Ok(())
    }
}

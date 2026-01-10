use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::result::Result as StdResult;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tracing::{error, info, warn};

/// Error type for process spawning and management operations
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("failed to spawn child process: {0}")]
    SpawnFailed(#[from] io::Error),
    #[error("failed to send signal to child process: {0}")]
    SignalFailed(nix::errno::Errno),
    #[error("no command specified")]
    EmptyCommand,
}

pub type Result<T> = std::result::Result<T, ProcessError>;

pub struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    pub fn spawn(command: &[String]) -> Result<Self> {
        if command.is_empty() {
            return Err(ProcessError::EmptyCommand);
        }

        info!(command = ?command, "Spawning child process");

        let child = Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        info!(pid = child.id(), "Child process spawned");

        Ok(Self { child })
    }

    pub fn send_sigusr1(&self) -> Result<()> {
        let pid = Pid::from_raw(self.child.id().try_into().expect("child had no valid pid"));
        info!(pid = ?pid, "Sending SIGUSR1 to child");
        kill(pid, Signal::SIGUSR1).map_err(ProcessError::SignalFailed)?;
        Ok(())
    }

    pub fn wait(mut self) -> std::io::Result<ExitStatus> {
        info!("Waiting for child process to exit");
        let status = self.child.wait()?;
        info!(status = ?status, "Child process exited");
        Ok(status)
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub fn force_kill(&mut self) {
        warn!("Force killing child process");
        if let Err(e) = self.child.kill() {
            error!(error = %e, "Failed to force kill child process");
        }
    }
}

/// Error type for holding possibilities when running a child process to termination
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("child process exited with failure exit status: {0:?}")]
    Status(std::process::ExitStatus),
}

/// Convenience trait giving a 1-liner for the execution and checking phase of running a
/// [`Command`]
pub trait CommandExt {
    /// Check the status of the command and return an error if it failed, but in one step
    fn status_checked(&mut self) -> StdResult<(), CommandError>;
}

impl CommandExt for Command {
    #[inline]
    fn status_checked(&mut self) -> StdResult<(), CommandError> {
        self.status()
            .map_err(From::from)
            .and_then(|status| if status.success() {
                Ok(())
            } else {
                Err(CommandError::Status(status))
            })
    }
}

/// Simply shells out to `killall`
pub fn killall<S: AsRef<OsStr>>(process_name: S, signal_type: Option<&str>) -> StdResult<(), CommandError> {
    Command::new("killall")
        .args(signal_type)
        .arg(process_name)
        .status_checked()
}

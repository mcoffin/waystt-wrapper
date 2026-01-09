use std::process::{Child, Command, ExitStatus, Stdio};

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tracing::{error, info, warn};

use crate::error::{Result, WaysttWrapperError};

pub struct ChildProcess {
    child: Child,
}

impl ChildProcess {
    pub fn spawn(command: &[String]) -> Result<Self> {
        if command.is_empty() {
            return Err(WaysttWrapperError::ConfigError(
                "No command specified".to_string(),
            ));
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
        let pid = Pid::from_raw(self.child.id() as i32);
        info!(pid = ?pid, "Sending SIGUSR1 to child");
        kill(pid, Signal::SIGUSR1)?;
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

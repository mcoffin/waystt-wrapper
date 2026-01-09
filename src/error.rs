use thiserror::Error;

#[derive(Error, Debug)]
pub enum WaysttWrapperError {
    #[error("Failed to spawn waystt process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("Failed to send signal to child process: {0}")]
    SignalFailed(#[from] nix::errno::Errno),

    #[error("Layer shell not supported on this compositor")]
    LayerShellNotSupported,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, WaysttWrapperError>;

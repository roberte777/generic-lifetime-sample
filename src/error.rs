//! Error for VCT Utils crate
use thiserror::Error;

/// Represents errors that can occur within the toolbox.
#[derive(Debug, Error)]
pub enum ToolboxError {
    /// This variant is used when there's an error loading the configuration, encapsulating the error details.
    #[error("Error loading config {source}")]
    Config {
        /// wrapper for the underlying error source
        #[from]
        source: config::ConfigError,
    },
    /// Used to indicate a failure when converting types
    #[error("Conversion error {0}")]
    Conversion(
        /// Information about the failure
        String,
    ),
    /// Actor closed error
    #[error("Unable to communicate with actor {0}")]
    ActorClosed(String),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ToolboxError {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        ToolboxError::ActorClosed(format!("Unable to communicate with actor {}", value))
    }
}

/// Encapsulates the outcome of an operation that might produce a result of type `T` or an error of type `ToolboxError`.
pub type ToolboxResult<T = ()> = Result<T, ToolboxError>;

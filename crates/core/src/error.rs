//! The single error type that crosses crate boundaries.

/// Errors returned by core, the store and every tool.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The tool exists in the catalog but its engine is not written yet.
    #[error("{0} is not implemented yet")]
    NotImplemented(&'static str),

    /// The job was cancelled by the user before it finished.
    #[error("cancelled")]
    Cancelled,

    /// Bad input from the user (wrong file type, output would overwrite input, ...).
    #[error("invalid input: {0}")]
    Invalid(String),

    /// Network request failed (offline, timeout, HTTP error).
    #[error("network: {0}")]
    Network(String),

    /// File system error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Local database error.
    #[cfg(feature = "sqlite")]
    #[error(transparent)]
    Db(#[from] rusqlite::Error),

    /// A bug: a tool panicked or an invariant broke. The UI keeps running.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

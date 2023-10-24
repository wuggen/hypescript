//! Virtual machine runtime errors

use std::fmt::{self, Display, Formatter};

/// A result type specialized to runtime errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Categories of runtime error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    DivideByZero,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "divide by zero"),
        }
    }
}

/// VM runtime errors
#[derive(Debug, thiserror::Error)]
#[error("runtime error at program counter {pc}: {kind}")]
pub struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) pc: u64,
}

impl Error {
    /// Get the kind of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Get the value of the program counter at which the error occurred.
    pub fn program_counter(&self) -> u64 {
        self.pc
    }
}

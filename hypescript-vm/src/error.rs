//! Virtual machine runtime errors

use std::fmt::{self, Display, Formatter};

use hypescript_bytecode::Instruction;

use crate::trace::{format_trace, Snapshot};

/// A result type specialized to runtime errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Categories of runtime error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    StackUnderflow,
    OutOfBoundsVariableReference,
    DivideByZero,
    IncompleteLiteral,
    AllocationError,
    NoInputStream,
    InputError,
    OutputError,
    ParseError,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::OutOfBoundsVariableReference => write!(f, "out of bounds variable reference"),
            Self::DivideByZero => write!(f, "divide by zero"),
            Self::IncompleteLiteral => write!(f, "incomplete literal"),
            Self::AllocationError => write!(f, "host memory allocation error"),
            Self::NoInputStream => write!(f, "no input stream configured"),
            Self::InputError => write!(f, "could not read input stream"),
            Self::OutputError => write!(f, "could not write to output stream"),
            Self::ParseError => write!(f, "could not parse integer value"),
        }
    }
}

/// VM runtime errors
#[derive(Debug, thiserror::Error)]
//#[error("runtime error at program counter {program_counter}: {kind}")]
pub struct Error {
    /// The error kind.
    pub kind: ErrorKind,

    /// The value of the program counter when the error was encountered.
    pub program_counter: usize,

    /// The instruction whose execution caused the error, if relevant.
    pub instr: Option<Instruction>,

    /// A trace of the program execution, if the context was configured to record it.
    pub trace: Option<Vec<Snapshot>>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error at pc {}", self.program_counter)?;
        if let Some(instr) = self.instr {
            write!(f, " ({})", instr)?;
        }
        write!(f, ": {}", self.kind)?;
        if let Some(trace) = self.trace.as_ref() {
            writeln!(f, "\nTrace:")?;
            format_trace(f, trace)?;
        }

        Ok(())
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            kind,
            program_counter: 0,
            instr: None,
            trace: None,
        }
    }
}

//! Types and functions for program execution traces.

use std::fmt::{self, Display, Formatter};

use hypescript_bytecode::Instruction;

use crate::value::Value;

/// A snapshot of the machine state before executing an instruction.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// The current program counter, the address of the next instruction.
    pub program_counter: usize,

    /// The instruction about to be executed in the current state.
    pub next_instruction: Instruction,

    /// The current operand stack.
    pub stack: Vec<Value>,

    /// The current local variables array.
    pub local_variables: Vec<Value>,
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "pc {}", self.program_counter)?;
        writeln!(f, "{}", self.next_instruction)?;

        writeln!(f, "stack")?;
        format_stack(f, &self.stack)?;

        writeln!(f, "vars")?;
        format_vars(f, &self.local_variables)
    }
}

pub fn format_stack<W: fmt::Write>(stream: &mut W, stack: &[Value]) -> fmt::Result {
    for (i, v) in stack.iter().rev().enumerate() {
        writeln!(stream, " {i:2}: {v:x}\t\t{v}\t{v:-}")?;
    }

    Ok(())
}

pub fn format_vars<W: fmt::Write>(stream: &mut W, vars: &[Value]) -> fmt::Result {
    for (i, v) in vars.iter().enumerate() {
        writeln!(stream, " {i:2}: {v:x}\t\t{v}\t{v:-}")?;
    }

    Ok(())
}

pub fn format_trace<W: fmt::Write>(stream: &mut W, trace: &[Snapshot]) -> fmt::Result {
    let mut first = true;
    for snapshot in trace {
        if !first {
            writeln!(stream)?;
        } else {
            first = false;
        }

        write!(stream, "{snapshot}")?;
    }

    Ok(())
}

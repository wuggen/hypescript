//! # The HypeScript Virtual Machine
//!
//! This crate implements the HypeScript VM execution engine.

use crate::error::*;
use crate::trace::{format_trace, format_vars};

use std::fmt::{self, Debug, Display, Formatter};
use std::io::{BufRead, Write};

use hypescript_bytecode::{Instruction, Opcode};
use trace::{format_stack, Snapshot};
use value::Value;

pub mod error;
pub mod trace;
pub mod value;

/// Execution context for a HypeScript program.
///
/// This contains the machine state for a running HypeScript program. The input and output streams
/// are configurable, and can be directed to any streams desired, or left unconfigured. If the
/// input stream is unconfigured and the machine encounters a `read` or `reads` instruction,
/// execution will halt with an error. If the output stream is unconfigured and the machine
/// encounters a `print` or `prints` instruction, that instruction will pop its operands from the
/// stack but otherwise be treated as a no-op.
///
/// The context borrows the program data as a byte slice. The input and output streams may be owned
/// or borrowed.
///
/// The lifetime parameters are as follows:
///
/// - `'p`: The lifetime of the program data.
/// - `'i`: The lifetime of the input stream (`'static` if it is owned or not configured).
/// - `'o`: The lifetime of the output stream (`'static` if it is owned or not configured).
pub struct ExecutionContext<'p, 'i, 'o> {
    program: &'p [u8],
    program_counter: usize,
    stack: Vec<Value>,
    local_vars: Vec<Value>,
    input_stream: Option<Box<dyn BufRead + 'i>>,
    input_buffer: Vec<String>,
    output_stream: Option<Box<dyn Write + 'o>>,
    trace: Option<Vec<Snapshot>>,
}

impl Debug for ExecutionContext<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionContext")
            .field("program", &self.program)
            .field("program_counter", &self.program_counter)
            .field("stack", &self.stack)
            .field("local_vars", &self.local_vars)
            .finish()
    }
}

impl<'p, 'i, 'o> ExecutionContext<'p, 'i, 'o> {
    /// Create a new `ExecutionContext` with the given program data.
    ///
    /// The new context will have its internal state initialized to the starting state of the VM:
    /// program counter 0, empty stack and local variables array. Additionally, the input and
    /// output streams will be unconfigured; use [`ExecutionContext::with_input_stream`] and
    /// [`ExecutionContext::with_output_stream`] to configure them.
    pub fn new(program: &'p [u8]) -> Self {
        Self {
            program,
            program_counter: 0,
            stack: Vec::new(),
            local_vars: Vec::new(),
            output_stream: None,
            input_stream: None,
            input_buffer: Vec::new(),
            trace: None,
        }
    }

    /// A builder method to set the input stream for this execution context.
    pub fn with_input_stream<R: BufRead + 'i>(self, stream: R) -> Self {
        Self {
            input_stream: Some(Box::new(stream)),
            ..self
        }
    }

    /// A builder method to set the output stream for this execution context.
    pub fn with_output_stream<W: Write + 'o>(self, stream: W) -> Self {
        Self {
            output_stream: Some(Box::new(stream)),
            ..self
        }
    }

    /// Enable recording a trace of the execution of the program.
    ///
    /// If tracing is enabled, a snapshot of the machine state will be saved before each
    /// instruction, and the full list of snapshots will be returned with the execution summary and
    /// with any runtime errors.
    pub fn with_trace(self) -> Self {
        Self {
            trace: Some(Vec::new()),
            ..self
        }
    }

    /// Consume the context, and execute the loaded program.
    pub fn run(mut self) -> Result<ExecutionSummary> {
        while (self.program_counter) < self.program.len() {
            let pc = self.program_counter;
            let mut stream = &self.program[pc..];
            let instr = Instruction::decode_from_stream(&mut stream).map_err(|err| {
                debug_assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
                Error {
                    kind: ErrorKind::IncompleteLiteral,
                    program_counter: self.program_counter,
                    instr: None,
                    trace: self.trace.clone(),
                }
            })?;

            if self.trace.is_some() {
                let snapshot = self.generate_snapshot(instr);
                self.trace.as_mut().unwrap().push(snapshot);
            }

            let advance = self.execute_instruction(instr).map_err(|err| Error {
                program_counter: self.program_counter,
                instr: Some(instr),
                trace: self.trace.clone(),
                ..err
            })?;
            if advance == 0 {
                break;
            } else {
                self.program_counter += advance;
            }
        }

        Ok(ExecutionSummary {
            program_counter: self.program_counter,
            stack: self.stack,
            local_vars: self.local_vars,
            trace: self.trace,
        })
    }

    fn generate_snapshot(&self, next_instruction: Instruction) -> Snapshot {
        Snapshot {
            program_counter: self.program_counter,
            next_instruction,
            stack: self.stack.clone(),
            local_variables: self.local_vars.clone(),
        }
    }
}

/// A snapshot of the machine state at the end of program execution.
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub program_counter: usize,
    pub stack: Vec<Value>,
    pub local_vars: Vec<Value>,
    pub trace: Option<Vec<Snapshot>>,
}

impl Display for ExecutionSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(trace) = self.trace.as_ref() {
            writeln!(f, "== EXECUTION TRACE ==")?;
            format_trace(f, trace)?;

            writeln!(f)?;
        }

        writeln!(f, "== EXECUTION END STATE ==")?;
        writeln!(f, "pc {}", self.program_counter)?;

        writeln!(f, "stack")?;
        format_stack(f, &self.stack)?;

        writeln!(f, "vars")?;
        format_vars(f, &self.local_vars)
    }
}

impl ExecutionContext<'_, '_, '_> {
    fn execute_instruction(&mut self, instr: Instruction) -> Result<usize> {
        match instr.opcode {
            Opcode::VarSt => self.varst(),
            Opcode::VarLd => self.varld(),
            Opcode::VarRes => self.varres(),
            Opcode::VarDisc => self.vardisc(),
            Opcode::NumVars => {
                self.numvars();
                Ok(())
            }
            Opcode::Push8
            | Opcode::Push8S
            | Opcode::Push16
            | Opcode::Push16S
            | Opcode::Push32
            | Opcode::Push32S
            | Opcode::Push64 => {
                self.pushn(Value::from_u64(instr.literal));
                Ok(())
            }
            Opcode::Dup0 => self.dupn(0),
            Opcode::Dup1 => self.dupn(1),
            Opcode::Dup2 => self.dupn(2),
            Opcode::Dup3 => self.dupn(3),
            Opcode::Pop => self.pop(),
            Opcode::Swap => self.swap(),
            Opcode::Add => self.binop_infallible(Value::add),
            Opcode::Sub => self.binop_infallible(Value::sub),
            Opcode::Mul => self.binop_infallible(Value::mul),
            Opcode::Mod => self.binop_fallible(Value::mod_),
            Opcode::Div => self.binop_fallible(Value::div_unsigned),
            Opcode::DivS => self.binop_fallible(Value::div_signed),
            Opcode::Gt => self.binop_infallible(Value::greater_unsigned),
            Opcode::GtS => self.binop_infallible(Value::greater_signed),
            Opcode::Lt => self.binop_infallible(Value::less_unsigned),
            Opcode::LtS => self.binop_infallible(Value::less_signed),
            Opcode::Ge => self.binop_infallible(Value::greater_or_eq_unsigned),
            Opcode::GeS => self.binop_infallible(Value::greater_or_eq_signed),
            Opcode::Le => self.binop_infallible(Value::less_or_eq_unsigned),
            Opcode::LeS => self.binop_infallible(Value::less_or_eq_signed),
            Opcode::Eq => self.binop_infallible(Value::eq),
            Opcode::And => self.binop_infallible(Value::and),
            Opcode::Or => self.binop_infallible(Value::or),
            Opcode::Xor => self.binop_infallible(Value::xor),
            Opcode::Not => self.unop(Value::not),
            Opcode::Inv => self.unop(Value::inv),
            Opcode::Jump => self.jump(),
            Opcode::JCond => self.jcond(),
            Opcode::Read => self.read(false),
            Opcode::ReadS => self.read(true),
            Opcode::Print => self.print(false),
            Opcode::PrintS => self.print(true),
            Opcode::Halt => return Ok(0),
        }?;

        Ok(1 + instr.opcode.literal_len())
    }

    fn pop_stack(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| Error::from(ErrorKind::StackUnderflow))
    }

    fn push_stack(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn read_var(&self, n: Value) -> Result<Value> {
        let n = n.as_u64() as usize;
        self.local_vars
            .get(n)
            .copied()
            .ok_or_else(|| Error::from(ErrorKind::OutOfBoundsVariableReference))
    }

    fn write_var(&mut self, n: Value, x: Value) -> Result<()> {
        let n = n.as_u64() as usize;
        self.local_vars
            .get_mut(n)
            .map(|var| *var = x)
            .ok_or_else(|| Error::from(ErrorKind::OutOfBoundsVariableReference))
    }

    fn varst(&mut self) -> Result<()> {
        let n = self.pop_stack()?;
        let x = self.pop_stack()?;
        self.write_var(n, x)
    }

    fn varld(&mut self) -> Result<()> {
        let n = self.pop_stack()?;
        let x = self.read_var(n)?;
        self.push_stack(x);
        Ok(())
    }

    fn varres(&mut self) -> Result<()> {
        let n = self.pop_stack()?.as_u64() as usize;
        self.local_vars
            .try_reserve(n)
            .map_err(|_| Error::from(ErrorKind::AllocationError))?;
        self.local_vars
            .resize(self.local_vars.len() + n, Value::default());

        Ok(())
    }

    fn vardisc(&mut self) -> Result<()> {
        let n = self.pop_stack()?.as_u64() as usize;
        if n < self.local_vars.len() {
            self.local_vars.truncate(self.local_vars.len() - n);
        } else {
            self.local_vars.clear();
        }
        Ok(())
    }

    fn numvars(&mut self) {
        self.push_stack(Value::from_u64(self.local_vars.len() as u64));
    }

    fn pushn(&mut self, value: Value) {
        self.push_stack(value);
    }

    fn dupn(&mut self, n: usize) -> Result<()> {
        if n < self.stack.len() {
            let v = self.stack[self.stack.len() - 1 - n];
            self.push_stack(v);
            Ok(())
        } else {
            Err(Error::from(ErrorKind::StackUnderflow))
        }
    }

    fn pop(&mut self) -> Result<()> {
        self.pop_stack()?;
        Ok(())
    }

    fn swap(&mut self) -> Result<()> {
        if self.stack.len() < 2 {
            Err(Error::from(ErrorKind::StackUnderflow))
        } else {
            let len = self.stack.len();
            self.stack.swap(len - 1, len - 2);
            Ok(())
        }
    }

    fn binop_infallible(&mut self, op: fn(Value, Value) -> Value) -> Result<()> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.push_stack(op(a, b));
        Ok(())
    }

    fn binop_fallible(&mut self, op: fn(Value, Value) -> Result<Value>) -> Result<()> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.push_stack(op(a, b)?);
        Ok(())
    }

    fn unop(&mut self, op: fn(Value) -> Value) -> Result<()> {
        let a = self.pop_stack()?;
        self.push_stack(op(a));
        Ok(())
    }

    fn jump(&mut self) -> Result<()> {
        let n = self.pop_stack()?.as_i64() as isize;
        self.program_counter = self.program_counter.wrapping_add_signed(n);
        Ok(())
    }

    fn jcond(&mut self) -> Result<()> {
        let n = self.pop_stack()?.as_i64() as isize;
        let b = self.pop_stack()?.as_u64();

        if b != 0 {
            self.program_counter = self.program_counter.wrapping_add_signed(n);
        }

        Ok(())
    }

    fn fill_input_buffer(&mut self) -> Result<()> {
        if let Some(input) = self.input_stream.as_mut() {
            if self.input_buffer.is_empty() {
                let mut line = String::new();
                input
                    .read_line(&mut line)
                    .map_err(|_| Error::from(ErrorKind::InputError))?;
                self.input_buffer = line.split_whitespace().rev().map(String::from).collect();
            }

            Ok(())
        } else {
            Err(Error::from(ErrorKind::NoInputStream))
        }
    }

    fn read(&mut self, signed: bool) -> Result<()> {
        self.fill_input_buffer()?;
        let input = self.input_buffer.pop().unwrap();
        let val = if signed {
            Value::from_i64(
                input
                    .parse()
                    .map_err(|_| Error::from(ErrorKind::ParseError))?,
            )
        } else {
            Value::from_u64(
                input
                    .parse()
                    .map_err(|_| Error::from(ErrorKind::ParseError))?,
            )
        };
        self.push_stack(val);
        Ok(())
    }

    fn print(&mut self, signed: bool) -> Result<()> {
        let val = self.pop_stack()?;
        if let Some(output) = self.output_stream.as_mut() {
            if signed {
                writeln!(output, "{}", val.as_i64())
                    .map_err(|_| Error::from(ErrorKind::OutputError))?;
            } else {
                writeln!(output, "{}", val.as_u64())
                    .map_err(|_| Error::from(ErrorKind::OutputError))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use hypescript_bytecode::consts::*;

    use super::*;

    fn val_vec(values: &[u64]) -> Vec<Value> {
        values.iter().copied().map(Value::from_u64).collect()
    }

    fn test_program(program: &[u8], input: &str, validate: fn(ExecutionSummary, &str)) {
        let mut output = Vec::<u8>::new();
        let res = ExecutionContext::new(program)
            .with_input_stream(&mut input.as_bytes())
            .with_output_stream(&mut output)
            .with_trace()
            .run();

        match res {
            Err(err) => {
                eprintln!("{err}");
                panic!("VM encountered a runtime error");
            }

            Ok(summary) => {
                println!("{}", summary);
                let output = String::from_utf8(output).unwrap();
                validate(summary, &output);
            }
        }
    }

    #[test]
    fn example1() {
        // Translated example from the challenge document:
        // uint a = 5;
        // uint b = 4 + a;
        // print a;
        // print b;
        #[rustfmt::skip]
        test_program(
            &[
                PUSH8, 2,
                VARRES,
                PUSH8, 5,
                PUSH8, 0,
                VARST,
                PUSH8, 4,
                PUSH8, 0,
                VARLD,
                ADD,
                PUSH8, 1,
                VARST,
                PUSH8, 0,
                VARLD,
                PRINT,
                PUSH8, 1,
                VARLD,
                PRINT,
            ],
            "",
            |summary, output| {
                assert_eq!(summary.local_vars, &[Value::from_u8(5), Value::from_u8(9)]);
                assert!(summary.stack.is_empty());
                assert_eq!(summary.program_counter, 25);
                assert_eq!(output, "5\n9\n");
            },
        );
    }

    #[test]
    fn example2() {
        // Translated example from the challenge document:
        //
        // uint a = 1;
        // uint b = 0;
        // if b == a {
        //     print 0;
        // }
        // if a > b {
        //     print 2;
        // }
        // print a + b;
        #[rustfmt::skip]
        test_program(
            &[
                PUSH8, 2,
                VARRES, // pc 3
                PUSH8, 1,
                PUSH8, 0,
                VARST,

                // var1 is initialized to zero, nothing needs doing

                // pc 8
                PUSH8, 0,
                VARLD,
                PUSH8, 1,
                VARLD,
                EQ, NOT,
                PUSH8, 3,
                JCOND,
                PUSH8, 0,
                PRINT,

                // pc 22
                PUSH8, 0,
                VARLD,
                PUSH8, 1,
                VARLD,
                LE,
                PUSH8, 3,
                JCOND,
                PUSH8, 2,
                PRINT,

                // pc 35
                PUSH8, 0,
                VARLD,
                PUSH8, 1,
                VARLD,
                ADD,
                PRINT,
                // pc 43
            ],
            "",
            |summary, output| {
                assert_eq!(summary.local_vars, &[Value::from_u8(1), Value::from_u8(0)]);
                assert!(summary.stack.is_empty());
                assert_eq!(summary.program_counter, 43);
                assert_eq!(output, "2\n1\n");
            },
        );
    }

    #[test]
    fn counter() {
        // Basic counter, storing locals purely on stack
        //
        // uint a = 0;
        // while a < 10 {
        //     a = a + 1;
        //     print a;
        // }
        #[rustfmt::skip]
        test_program(
            &[
                PUSH8, 0,
                PUSH8, 5,
                JUMP,

                PUSH8, 1,
                ADD,
                DUP0,
                PRINT,

                DUP0,
                PUSH8, 10,
                LT,
                PUSH8S, (-12i8) as u8,
                JCOND,

                POP,
            ],
            "",
            |summary, output| {
                assert!(summary.local_vars.is_empty());
                assert!(summary.stack.is_empty(), "stack = {:?}", summary.stack);
                assert_eq!(output, "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
            },
        );
    }

    #[test]
    fn var_management() {
        #[rustfmt::skip]
        test_program(
            &[
                // pc 0, trace step 0
                PUSH8, 4,
                VARRES,

                // pc 3, tr 2
                NUMVARS,
                POP,

                // pc 5, tr 4
                PUSH8, 2,
                VARDISC,

                // pc 8, tr 6
                NUMVARS,
                POP,

                // pc 10, tr 8
                PUSH8, 12,
                PUSH8, 0,
                VARST,

                // pc 15, tr 11
                PUSH8, 15,
                PUSH8, 1,
                VARST,

                // pc 20, tr 14
                PUSH8, 0,
                VARLD,
                POP,

                // pc 24, tr 17
                PUSH8, 1,
                VARLD,
                POP,

                // pc 28, tr 20
            ],
            "",
            |summary, _| {
                let trace = summary.trace.as_ref().unwrap();

                // Reserve 4 vars, push num vars
                assert_eq!(trace[2].local_variables, &[Value::from_u8(0); 4]);
                assert_eq!(*trace[3].stack.last().unwrap(), Value::from_u8(4));

                // Discard 2 vars, push num vars
                assert_eq!(trace[6].local_variables, &[Value::from_u8(0); 2]);
                assert_eq!(*trace[7].stack.last().unwrap(), Value::from_u8(2));

                // Store 12 in var 0, 15 in var 1
                assert_eq!(trace[11].local_variables, &[Value::from_u8(12), Value::from_u8(0)]);
                assert_eq!(trace[14].local_variables, &[Value::from_u8(12), Value::from_u8(15)]);

                // Load var 0, var 1
                assert_eq!(*trace[16].stack.last().unwrap(), Value::from_u8(12));
                assert_eq!(*trace[19].stack.last().unwrap(), Value::from_u8(15));
            }
        );
    }

    #[test]
    fn stack_management() {
        #[rustfmt::skip]
        test_program(
            &[
                // tr 0
                PUSH8, 4,
                PUSH8S, 0xff,

                // tr 2
                PUSH16, 0xbe, 0xef,
                PUSH16S, 0xca, 0xfe,

                // tr 4
                PUSH32, 0, 0, 0xde, 0xed,
                PUSH32S, 0xde, 0xad, 0xbe, 0xef,

                // tr 6
                PUSH64, 0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xfa, 0xce,

                // tr 7
                SWAP, SWAP,

                // tr 9
                DUP0, POP,

                // tr 11
                DUP1, POP,

                // tr 13
                DUP2, POP,

                // tr 15
                DUP3, POP,

                // tr 17
            ],
            "",
            |summary, _ | {
                let trace = summary.trace.as_ref().unwrap();

                // push8, push8s
                assert_eq!(trace[2].stack, val_vec(&[4, 0xffffffffffffffff]));

                // push16, push16s
                assert_eq!(trace[4].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                ]));

                // push32, push32s
                assert_eq!(trace[6].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xffffffffdeadbeef,
                ]));

                // push64
                assert_eq!(trace[7].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xffffffffdeadbeef,
                        0xdeadbeefcafeface,
                ]));

                // swap
                assert_eq!(trace[8].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xdeadbeefcafeface,
                        0xffffffffdeadbeef,
                ]));

                // swap, dup0
                assert_eq!(trace[10].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xffffffffdeadbeef,
                        0xdeadbeefcafeface,
                        0xdeadbeefcafeface,
                ]));

                // pop, dup1
                assert_eq!(trace[12].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xffffffffdeadbeef,
                        0xdeadbeefcafeface,
                        0xffffffffdeadbeef,
                ]));

                // pop, dup2
                assert_eq!(trace[14].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xffffffffdeadbeef,
                        0xdeadbeefcafeface,
                        0xdeed,
                ]));

                // pop, dup3
                assert_eq!(trace[16].stack, val_vec(&[
                        4,
                        0xffffffffffffffff,
                        0xbeef,
                        0xffffffffffffcafe,
                        0xdeed,
                        0xffffffffdeadbeef,
                        0xdeadbeefcafeface,
                        0xffffffffffffcafe,
                ]));
            },
        );
    }

    #[test]
    fn arithmetic() {
        #[rustfmt::skip]
        test_program(
            &[
                // tr 0
                PUSH8, 2,
                PUSH8, 6,
                ADD,

                // tr 3
                POP,
                PUSH8, 12,
                PUSH8, 5,
                SUB,

                // tr 7
                POP,
                PUSH8, 7,
                PUSH8, 6,
                MUL,

                // tr 11
                POP,
                PUSH8, 17,
                PUSH8, 8,
                MOD,

                // tr 15
                POP,
                PUSH8, 88,
                PUSH8, 11,
                DIV,

                // tr 19
                POP,
                PUSH8S, -20_i8 as u8,
                PUSH8, 5,
                DIVS,

                // tr 23
            ],
            "",
            |summary, _| {
                let trace = summary.trace.as_ref().unwrap();

                // 2 + 6
                assert_eq!(trace[3].stack, val_vec(&[8]));

                // 12 - 5
                assert_eq!(trace[7].stack, val_vec(&[7]));

                // 7 * 6
                assert_eq!(trace[11].stack, val_vec(&[42]));

                // 17 % 8
                assert_eq!(trace[15].stack, val_vec(&[1]));

                // 88 / 11
                assert_eq!(trace[19].stack, val_vec(&[8]));

                // -20 / 5
                assert_eq!(summary.stack, val_vec(&[-4_i64 as u64]));
            },
        );
    }

    // TODO: other instructions, and runtime errors
}

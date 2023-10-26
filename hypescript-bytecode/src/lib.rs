//! # HypeScript bytecode definitions
//!
//! This crate provides types and functions for working with HypeScript bytecode. This includes
//! writing and parsing bytecode, and querying information about opcodes, but not execution; see
//! the `nilscript-vm` crate for an execution engine.

pub mod consts;

use consts::*;
use hypescript_util::array_from_slice;
use std::fmt::{self, Display, Formatter};
use std::io;

/// Opcodes recognized by the NilScript VM.
///
/// This enum can be converted to the binary forms of opcodes via `u8::from` or primitive
/// conversion to a `u8`.
///
/// Conversely, the binary forms of opcodes can be parsed into this enum via [`Opcode::try_from`]
/// or [`Opcode::from_u8`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    VarSt = VARST,
    VarLd = VARLD,
    VarRes = VARRES,
    VarDisc = VARDISC,
    NumVars = NUMVARS,
    Push8 = PUSH8,
    Push8S = PUSH8S,
    Push16 = PUSH16,
    Push16S = PUSH16S,
    Push32 = PUSH32,
    Push32S = PUSH32S,
    Push64 = PUSH64,
    Dup0 = DUP0,
    Dup1 = DUP1,
    Dup2 = DUP2,
    Dup3 = DUP3,
    Pop = POP,
    Swap = SWAP,
    Add = ADD,
    Sub = SUB,
    Mul = MUL,
    Mod = MOD,
    Div = DIV,
    DivS = DIVS,
    Gt = GT,
    GtS = GTS,
    Lt = LT,
    LtS = LTS,
    Ge = GE,
    GeS = GES,
    Le = LE,
    LeS = LES,
    Eq = EQ,
    And = AND,
    Or = OR,
    Xor = XOR,
    Not = NOT,
    Inv = INV,
    Jump = JUMP,
    JCond = JCOND,
    Read = READ,
    ReadS = READS,
    Print = PRINT,
    PrintS = PRINTS,
    Halt = HALT,
}

impl Opcode {
    /// Convert an opcode encoded as a `u8` into an `Opcode`.
    ///
    /// Returns `None` if the given byte is not recognized as an opcode.
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            VARST => Some(Self::VarSt),
            VARLD => Some(Self::VarLd),
            VARRES => Some(Self::VarRes),
            VARDISC => Some(Self::VarDisc),
            NUMVARS => Some(Self::NumVars),
            PUSH8 => Some(Self::Push8),
            PUSH8S => Some(Self::Push8S),
            PUSH16 => Some(Self::Push16),
            PUSH16S => Some(Self::Push16S),
            PUSH32 => Some(Self::Push32),
            PUSH32S => Some(Self::Push32S),
            PUSH64 => Some(Self::Push64),
            DUP0 => Some(Self::Dup0),
            DUP1 => Some(Self::Dup1),
            DUP2 => Some(Self::Dup2),
            DUP3 => Some(Self::Dup3),
            POP => Some(Self::Pop),
            SWAP => Some(Self::Swap),
            ADD => Some(Self::Add),
            SUB => Some(Self::Sub),
            MUL => Some(Self::Mul),
            MOD => Some(Self::Mod),
            DIV => Some(Self::Div),
            DIVS => Some(Self::DivS),
            GT => Some(Self::Gt),
            GTS => Some(Self::GtS),
            LT => Some(Self::Lt),
            LTS => Some(Self::LtS),
            GE => Some(Self::Ge),
            GES => Some(Self::GeS),
            LE => Some(Self::Le),
            LES => Some(Self::LeS),
            EQ => Some(Self::Eq),
            AND => Some(Self::And),
            OR => Some(Self::Or),
            XOR => Some(Self::Xor),
            NOT => Some(Self::Not),
            INV => Some(Self::Inv),
            JUMP => Some(Self::Jump),
            JCOND => Some(Self::JCond),
            READ => Some(Self::Read),
            READS => Some(Self::ReadS),
            PRINT => Some(Self::Print),
            PRINTS => Some(Self::PrintS),
            HALT => Some(Self::Halt),
            _ => None,
        }
    }

    /// Translate an opcode mnemonic into an `Opcode`.
    ///
    /// This function accepts mnemonics spelled with any combination of upper or lower case
    /// letters, and with any amount or kind of leading or trailing whitespace.
    pub fn from_mnemonic(mnemonic: &str) -> Option<Self> {
        let mut s = String::from(mnemonic);
        s.make_ascii_lowercase();
        match s.trim() {
            "varst" => Some(Self::VarSt),
            "varld" => Some(Self::VarLd),
            "varres" => Some(Self::VarRes),
            "vardisc" => Some(Self::VarDisc),
            "numvars" => Some(Self::NumVars),
            "push8" => Some(Self::Push8),
            "push8s" => Some(Self::Push8S),
            "push16" => Some(Self::Push16),
            "push16s" => Some(Self::Push16S),
            "push32" => Some(Self::Push32),
            "push32s" => Some(Self::Push32S),
            "push64" => Some(Self::Push64),
            "dup0" => Some(Self::Dup0),
            "dup1" => Some(Self::Dup1),
            "dup2" => Some(Self::Dup2),
            "dup3" => Some(Self::Dup3),
            "pop" => Some(Self::Pop),
            "swap" => Some(Self::Swap),
            "add" => Some(Self::Add),
            "sub" => Some(Self::Sub),
            "mul" => Some(Self::Mul),
            "mod" => Some(Self::Mod),
            "div" => Some(Self::Div),
            "divs" => Some(Self::DivS),
            "gt" => Some(Self::Gt),
            "gts" => Some(Self::GtS),
            "lt" => Some(Self::Lt),
            "lts" => Some(Self::LtS),
            "ge" => Some(Self::Ge),
            "ges" => Some(Self::GeS),
            "le" => Some(Self::Le),
            "les" => Some(Self::LeS),
            "eq" => Some(Self::Eq),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "xor" => Some(Self::Xor),
            "not" => Some(Self::Not),
            "inv" => Some(Self::Inv),
            "jump" => Some(Self::Jump),
            "jcond" => Some(Self::JCond),
            "read" => Some(Self::Read),
            "reads" => Some(Self::ReadS),
            "print" => Some(Self::Print),
            "prints" => Some(Self::PrintS),
            "halt" => Some(Self::Halt),
            _ => None,
        }
    }

    /// Get the lowercase mnemonic of this opcode.
    pub fn mnemonic(self) -> &'static str {
        match self {
            Self::VarSt => "varst",
            Self::VarLd => "varld",
            Self::VarRes => "varres",
            Self::VarDisc => "vardisc",
            Self::NumVars => "numvars",
            Self::Push8 => "push8",
            Self::Push8S => "push8s",
            Self::Push16 => "push16",
            Self::Push16S => "push16s",
            Self::Push32 => "push32",
            Self::Push32S => "push32s",
            Self::Push64 => "push64",
            Self::Dup0 => "dup0",
            Self::Dup1 => "dup1",
            Self::Dup2 => "dup2",
            Self::Dup3 => "dup3",
            Self::Pop => "pop",
            Self::Swap => "swap",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Mod => "mod",
            Self::Div => "div",
            Self::DivS => "divs",
            Self::Gt => "gt",
            Self::GtS => "gts",
            Self::Lt => "lt",
            Self::LtS => "lts",
            Self::Ge => "ge",
            Self::GeS => "ges",
            Self::Le => "le",
            Self::LeS => "les",
            Self::Eq => "eq",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Not => "not",
            Self::Inv => "inv",
            Self::Jump => "jump",
            Self::JCond => "jcond",
            Self::Read => "read",
            Self::ReadS => "reads",
            Self::Print => "print",
            Self::PrintS => "prints",
            Self::Halt => "halt",
        }
    }

    /// Get the number of bytes in the inline literal expected by this opcode.
    ///
    /// This will be 0, 1, 2, 4, or 8.
    pub fn literal_len(self) -> usize {
        match self {
            Opcode::Push8 | Opcode::Push8S => 1,
            Opcode::Push16 | Opcode::Push16S => 2,
            Opcode::Push32 | Opcode::Push32S => 4,
            Opcode::Push64 => 8,
            _ => 0,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(value: Opcode) -> Self {
        value as u8
    }
}

/// Error returned by [`Opcode::try_from`].
#[derive(Debug, thiserror::Error)]
#[error("Invalid opcode")]
pub struct InvalidOpcodeError;

impl TryFrom<u8> for Opcode {
    type Error = InvalidOpcodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Opcode::from_u8(value).ok_or(InvalidOpcodeError)
    }
}

/// A decoded bytecode instruction.
///
/// This includes the opcode and, if applicable, the literal value.
///
/// Note that all values of this struct have a literal, even though most opcodes do not expect a
/// literal. Note also that the literal stored in this struct is of constant bit width, even though
/// not all opcodes that expect literals expect the same size of literal. These apparent
/// discrepancies are handled as follows:
///
/// - During decoding, any opcode that does not expect a literal will cause the `literal` field to
///   be set to 0. Any literals shorter than 64 bits will be copied into the low order bits of the
///   `literal` field, with sign extension as appropriate.
/// - During encoding, any opcode that does not expect a literal will cause the `literal` field to
///   be ignored. Any literals shorter than 64 bits will be truncated from the `literal` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub opcode: Opcode,
    pub literal: u64,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode.mnemonic())?;
        match self.opcode.literal_len() {
            0 => Ok(()),
            1 => write!(f, " 0x{:02x}", self.literal as u8),
            2 => write!(f, " 0x{:04x}", self.literal as u16),
            4 => write!(f, " 0x{:08x}", self.literal as u32),
            8 => write!(f, " 0x{:016x}", self.literal),
            _ => unreachable!(),
        }
    }
}

impl From<Opcode> for Instruction {
    fn from(value: Opcode) -> Self {
        Self::new(value, 0)
    }
}

impl Instruction {
    pub fn from_opcode(opcode: Opcode) -> Self {
        Self::new(opcode, 0)
    }

    /// Create a new `Instruction`.
    pub fn new(opcode: Opcode, literal: u64) -> Self {
        Self { opcode, literal }
    }

    /// Decode a single instruction from a stream.
    ///
    /// This function makes very small reads. It is recommended to use it on buffered streams to
    /// improve performance.
    ///
    /// # Errors
    ///
    /// If the given stream returns an error, this function will return that error unmodified.
    ///
    /// If there is an error in decoding, (e.g. an unrecognized opcode,) this function will return
    /// an error with error kind `Other`, whose wrapped error is downcastable to [`DecodeError`].
    pub fn decode_from_stream<R: io::Read>(stream: &mut R) -> io::Result<Self> {
        let mut buf = [0; 8];
        stream.read_exact(&mut buf[..1])?;
        let opcode = Opcode::from_u8(buf[0])
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, DecodeError::UnrecognizedOpcode))?;

        let lit_len = opcode.literal_len();
        let literal = if lit_len > 0 {
            stream.read_exact(&mut buf[..lit_len])?;
            match opcode {
                Opcode::Push8 => buf[0] as u64,
                Opcode::Push8S => buf[0] as i8 as u64,
                Opcode::Push16 => u16::from_be_bytes(array_from_slice(&buf[..2])) as u64,
                Opcode::Push16S => i16::from_be_bytes(array_from_slice(&buf[..2])) as u64,
                Opcode::Push32 => u32::from_be_bytes(array_from_slice(&buf[..4])) as u64,
                Opcode::Push32S => i32::from_be_bytes(array_from_slice(&buf[..4])) as u64,
                Opcode::Push64 => u64::from_be_bytes(buf),
                _ => unreachable!(),
            }
        } else {
            0
        };

        Ok(Instruction { opcode, literal })
    }

    /// Encode an instruction into a stream.
    ///
    /// This function makes very small writes. It is recommended to use it on buffered streams to
    /// improve performance.
    ///
    /// # Errors
    ///
    /// Any errors returned from the stream will be returned unmodified.
    pub fn encode_to_stream<W: io::Write>(&self, stream: &mut W) -> io::Result<()> {
        stream.write_all(&[self.opcode as u8])?;

        let lit_len = self.opcode.literal_len();
        if lit_len > 0 {
            let buf = self.literal.to_be_bytes();
            stream.write_all(&buf[8 - lit_len..])?;
        }

        Ok(())
    }

    /// Get the number of bytes in the encoded form of this instruction.
    pub fn encoded_len(&self) -> usize {
        1 + self.opcode.literal_len()
    }

    /// Get the combined encoded length of a series of instructions.
    pub fn combined_len(instructions: &[Self]) -> usize {
        instructions.iter().map(|inst| inst.encoded_len()).sum()
    }

    /// Construct an unsigned push instruction of optimal size for the value.
    pub fn optimal_push(value: u64) -> Self {
        let opcode = if value <= u8::MAX as u64 {
            Opcode::Push8
        } else if value <= u16::MAX as u64 {
            Opcode::Push16
        } else if value <= u32::MAX as u64 {
            Opcode::Push32
        } else {
            Opcode::Push64
        };

        Self::new(opcode, value)
    }

    /// Construct a signed push instruction of optimal size for the value.
    pub fn optimal_pushs(value: i64) -> Self {
        let opcode = if i8::MIN as i64 <= value && value <= i8::MAX as i64 {
            Opcode::Push8S
        } else if i16::MIN as i64 <= value && value <= i16::MAX as i64 {
            Opcode::Push16S
        } else if i32::MIN as i64 <= value && value <= i32::MAX as i64 {
            Opcode::Push32S
        } else {
            Opcode::Push64
        };

        Self::new(opcode, value as u64)
    }
}

pub fn write_instructions<W: io::Write>(stream: &mut W, instrs: &[Instruction]) -> io::Result<()> {
    for instr in instrs {
        instr.encode_to_stream(stream)?;
    }

    Ok(())
}

pub fn instructions_to_vec(instrs: &[Instruction]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_instructions(&mut bytes, instrs).unwrap();
    bytes
}

/// Error returned by [`Instruction`] encoding and decoding.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Unrecognized opcode")]
    UnrecognizedOpcode,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode() -> io::Result<()> {
        let pairs: &[(_, &[u8])] = &[
            (Instruction::new(Opcode::Gt, 0), &[GT]),
            (Instruction::new(Opcode::Div, 14), &[DIV]), // Lits are ignored for standalone opcodes
            (Instruction::new(Opcode::Push8, 0), &[PUSH8, 0]),
            (Instruction::new(Opcode::Push8, 155), &[PUSH8, 155]),
            (
                Instruction::new(Opcode::Push16, 0x12345), // Lits are truncated for shorter lits
                &[PUSH16, 0x23, 0x45],
            ),
            (
                Instruction::new(Opcode::Push16S, 0x8455), // Signed lits are written correctly
                &[PUSH16S, 0x84, 0x55],
            ),
            (
                Instruction::new(Opcode::Push32, 0xdeadbeef),
                &[PUSH32, 0xde, 0xad, 0xbe, 0xef],
            ),
            (
                Instruction::new(Opcode::Push64, 0xdeadbeef),
                &[PUSH64, 0, 0, 0, 0, 0xde, 0xad, 0xbe, 0xef],
            ),
        ];

        let mut stream = Vec::new();
        for (instr, bytes) in pairs {
            stream.clear();

            instr.encode_to_stream(&mut stream)?;
            assert_eq!(&stream, bytes);
        }

        Ok(())
    }

    #[test]
    fn decode_successful() -> io::Result<()> {
        let pairs: &[(_, &[u8])] = &[
            (Instruction::new(Opcode::Gt, 0), &[GT]),
            // Only extracts first instruction:
            (Instruction::new(Opcode::Div, 0), &[DIV, 3, 4, 5, 6]),
            (Instruction::new(Opcode::Push8, 0), &[PUSH8, 0]),
            // Only extracts first instruction in presence of literal:
            (Instruction::new(Opcode::Push8, 155), &[PUSH8, 155, 3, 4, 5]),
            (
                Instruction::new(Opcode::Push16, 0x2345),
                &[PUSH16, 0x23, 0x45],
            ),
            (
                Instruction::new(Opcode::Push16S, 0xffffffffffff8455), // Signed lits are sign-extended
                &[PUSH16S, 0x84, 0x55],
            ),
            (
                Instruction::new(Opcode::Push32, 0xdeadbeef),
                &[PUSH32, 0xde, 0xad, 0xbe, 0xef],
            ),
            (
                Instruction::new(Opcode::Push64, 0xdeadbeef), // Unsigned lits are zero-extended
                &[PUSH64, 0, 0, 0, 0, 0xde, 0xad, 0xbe, 0xef],
            ),
        ];

        for (instr, bytes) in pairs {
            let mut stream = *bytes;

            let decoded = Instruction::decode_from_stream(&mut stream)?;
            let expected_amt_read = 1 + instr.opcode.literal_len();

            assert_eq!(*instr, decoded);
            assert_eq!(&bytes[expected_amt_read..], stream);
        }

        Ok(())
    }

    #[test]
    fn decode_unsuccessful() {
        // Empty stream
        let mut stream: &[u8] = &[];
        let err = Instruction::decode_from_stream(&mut stream).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);

        // Missing literal
        let mut stream: &[u8] = &[PUSH8];
        let err = Instruction::decode_from_stream(&mut stream).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);

        // Incomplete literal
        let mut stream: &[u8] = &[PUSH32, 1, 2, 3];
        let err = Instruction::decode_from_stream(&mut stream).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);

        // Unrecognized opcode
        let mut stream: &[u8] = &[0x20];
        let err = Instruction::decode_from_stream(&mut stream).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        let err = err
            .into_inner()
            .expect("Error has no inner err")
            .downcast::<DecodeError>()
            .expect("Downcast failed");
        assert!(matches!(*err, DecodeError::UnrecognizedOpcode));
    }
}

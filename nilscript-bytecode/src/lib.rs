//! # NilScript bytecode definitions
//!
//! This crate provides types and functions for working with NilScript bytecode. This includes
//! writing and parsing bytecode, and querying information about opcodes, but not execution; see
//! the `nilscript-vm` crate for an execution engine.

use std::io;

// TODO: refactor this into a separate util crate or something
fn array_from_slice<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr = [0; N];
    arr.copy_from_slice(slice);
    arr
}

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
    VarSt = 0x18,
    VarLd = 0x1a,
    VarRes = 0x1c,
    VarDisc = 0x1d,
    NumVars = 0x1e,
    Push8 = 0x28,
    Push8S = 0x29,
    Push16 = 0x2a,
    Push16S = 0x2b,
    Push32 = 0x2c,
    Push32S = 0x2d,
    Push64 = 0x2e,
    Dup0 = 0x30,
    Dup1 = 0x31,
    Dup2 = 0x32,
    Dup3 = 0x33,
    Pop = 0x34,
    Swap = 0x35,
    Add = 0x38,
    Sub = 0x39,
    Mul = 0x3a,
    MulS = 0x3b,
    Div = 0x3c,
    DivS = 0x3d,
    Mod = 0x3e,
    ModS = 0x3f,
    Gt = 0x50,
    GtS = 0x51,
    Lt = 0x52,
    LtS = 0x53,
    Ge = 0x54,
    GeS = 0x55,
    Le = 0x56,
    LeS = 0x57,
    Eq = 0x58,
    And = 0x59,
    Or = 0x5a,
    Xor = 0x5b,
    Not = 0x5c,
    Inv = 0x5d,
    Jump = 0x60,
    JCond = 0x61,
    Read = 0xfa,
    ReadS = 0xfb,
    Print = 0xfc,
    PrintS = 0xfd,
    Halt = 0xff,
}

impl Opcode {
    /// Convert an opcode encoded as a `u8` into an `Opcode`.
    ///
    /// Returns `None` if the given byte is not recognized as an opcode.
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x18 => Some(Self::VarSt),
            0x1a => Some(Self::VarLd),
            0x1c => Some(Self::VarRes),
            0x1d => Some(Self::VarDisc),
            0x1e => Some(Self::NumVars),
            0x28 => Some(Self::Push8),
            0x29 => Some(Self::Push8S),
            0x2a => Some(Self::Push16),
            0x2b => Some(Self::Push16S),
            0x2c => Some(Self::Push32),
            0x2d => Some(Self::Push32S),
            0x2e => Some(Self::Push64),
            0x30 => Some(Self::Dup0),
            0x31 => Some(Self::Dup1),
            0x32 => Some(Self::Dup2),
            0x33 => Some(Self::Dup3),
            0x34 => Some(Self::Pop),
            0x35 => Some(Self::Swap),
            0x38 => Some(Self::Add),
            0x39 => Some(Self::Sub),
            0x3a => Some(Self::Mul),
            0x3b => Some(Self::MulS),
            0x3c => Some(Self::Div),
            0x3d => Some(Self::DivS),
            0x3e => Some(Self::Mod),
            0x3f => Some(Self::ModS),
            0x50 => Some(Self::Gt),
            0x51 => Some(Self::GtS),
            0x52 => Some(Self::Lt),
            0x53 => Some(Self::LtS),
            0x54 => Some(Self::Ge),
            0x55 => Some(Self::GeS),
            0x56 => Some(Self::Le),
            0x57 => Some(Self::LeS),
            0x58 => Some(Self::Eq),
            0x59 => Some(Self::And),
            0x5a => Some(Self::Or),
            0x5b => Some(Self::Xor),
            0x5c => Some(Self::Not),
            0x5d => Some(Self::Inv),
            0x60 => Some(Self::Jump),
            0x61 => Some(Self::JCond),
            0xfa => Some(Self::Read),
            0xfb => Some(Self::ReadS),
            0xfc => Some(Self::Print),
            0xfd => Some(Self::PrintS),
            0xff => Some(Self::Halt),
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
            "muls" => Some(Self::MulS),
            "div" => Some(Self::Div),
            "divs" => Some(Self::DivS),
            "mod" => Some(Self::Mod),
            "mods" => Some(Self::ModS),
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

    /// Get the signedness of the inline literal expected by this opcode.
    ///
    /// For opcodes that do not expect an inline literal, this will return `Unsigned`.
    pub fn literal_signedness(self) -> Signedness {
        match self {
            Self::Push8S | Self::Push16S | Self::Push32S => Signedness::Signed,
            _ => Signedness::Unsigned,
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

/// The signedness of a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Signedness {
    #[default]
    Unsigned,
    Signed,
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

impl Instruction {
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

    pub fn encode_to_stream<W: io::Write>(&self, stream: &mut W) -> io::Result<()> {
        stream.write_all(&[self.opcode as u8])?;

        let lit_len = self.opcode.literal_len();
        if lit_len > 0 {
            let buf = self.literal.to_be_bytes();
            stream.write_all(&buf[8 - lit_len..])?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Unrecognized opcode")]
    UnrecognizedOpcode,
}

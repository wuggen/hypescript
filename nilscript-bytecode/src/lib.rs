//! # NilScript bytecode definitions
//!
//! This crate provides types and functions for working with NilScript bytecode. This includes
//! writing and parsing bytecode, and querying information about opcodes, but not execution; see
//! the `nilscript-vm` crate for an execution engine.

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

    /// Get the number of bytes in the inline literal expected by this opcode, if any.
    ///
    /// For opcodes that do not expect an inline literal value, this will return 0.
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

//! # The NilScript Virtual Machine
//!
//! This crate implements the NilScript VM execution engine.

pub mod vars;

use std::ops::*;

// TODO: refactor this into a separate util crate or something
fn array_from_slice<const N: usize>(slice: &[u8]) -> [u8; N] {
    let mut arr = [0; N];
    arr.copy_from_slice(slice);
    arr
}

/// A value in a stack or variable slot.
///
/// This wraps an array of 8 bytes, and provides utility methods for manipulating and retrieving
/// its value as various types.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Value([u8; 8]);

impl Value {
    /// Get the low-order byte of this value.
    pub fn as_u8(&self) -> u8 {
        self.0[7]
    }

    /// Get the low-order byte of this value, as an `i8`.
    pub fn as_i8(&self) -> i8 {
        self.as_u8() as i8
    }

    /// Get the low-order two bytes of this value as a `u16`.
    pub fn as_u16(&self) -> u16 {
        u16::from_be_bytes(array_from_slice(&self.0[6..]))
    }

    /// Get the low-order two bytes of this value as an `i16`.
    pub fn as_i16(&self) -> i16 {
        self.as_u16() as i16
    }

    /// Get the low-order four bytes of this value as a `u32`.
    pub fn as_u32(&self) -> u32 {
        u32::from_be_bytes(array_from_slice(&self.0[4..]))
    }

    /// Get the low-order four bytes of this value as an `i32`.
    pub fn as_i32(&self) -> i32 {
        self.as_u32() as i32
    }

    /// Get this value as a `u64`.
    pub fn as_u64(&self) -> u64 {
        u64::from_be_bytes(self.0)
    }

    /// Get this value as an `i64`.
    pub fn as_i64(&self) -> i64 {
        self.as_u64() as i64
    }

    /// Get a slice of this value's bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Create a `Value` from a `u8`.
    pub fn from_u8(val: u8) -> Self {
        Self::from_u64(val as u64)
    }

    /// Create a `Value` from an `i8`.
    pub fn from_i8(val: i8) -> Self {
        Self::from_i64(val as i64)
    }

    /// Create a `Value` from a `u16`.
    pub fn from_u16(val: u16) -> Self {
        Self::from_u64(val as u64)
    }

    /// Create a `Value` from an `i16`.
    pub fn from_i16(val: i16) -> Self {
        Self::from_i64(val as i64)
    }

    /// Create a `Value` from a `u32`.
    pub fn from_u32(val: u32) -> Self {
        Self::from_u64(val as u64)
    }

    /// Create a `Value` from an `i32`.
    pub fn from_i32(val: i32) -> Self {
        Self::from_i64(val as i64)
    }

    /// Create a `Value` from a `u64`.
    pub fn from_u64(val: u64) -> Self {
        Value(val.to_be_bytes())
    }

    /// Create a `Value` from an `i64`.
    pub fn from_i64(val: i64) -> Self {
        Value(val.to_be_bytes())
    }

    /// Create a `Value` from a byte slice.
    ///
    /// The given slice must be of length 1, 2, 4, or 8. If it is shorter than 8 bytes, it will be
    /// copied into the last bytes of the resulting `Value`, and the first bytes will be zero.
    ///
    /// # Panics
    ///
    /// This function will panic if the given slice is not of length 1, 2, 4, or 8.
    pub fn from_slice(val: &[u8]) -> Self {
        let mut arr = [0; 8];
        match val.len() {
            1 => arr[7] = val[0],
            2 => arr[6..].copy_from_slice(val),
            4 => arr[4..].copy_from_slice(val),
            8 => arr.copy_from_slice(val),
            _ => panic!("invalid value length"),
        }

        Value(arr)
    }

    /// Create a `Value` from a byte slice, performing sign extension.
    ///
    /// This will interpret the bytes of the given slice as a signed integer in big-endian byte
    /// order, sign-extend to an `i64`, and create a `Value` from the big-endian bytes of the
    /// result.
    ///
    /// # Panics
    ///
    /// This function will panic if the given slice is not of length 1, 2, 4, or 8.
    pub fn from_slice_signed(val: &[u8]) -> Self {
        match val.len() {
            1 => Self::from_i8(val[0] as i8),
            2 => Self::from_i16(i16::from_be_bytes(array_from_slice(val))),
            4 => Self::from_i32(i32::from_be_bytes(array_from_slice(val))),
            8 => Self::from_i64(i64::from_be_bytes(array_from_slice(val))),
            _ => panic!("invalid value length"),
        }
    }

    pub fn add(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_add(rhs.as_u64()))
    }

    pub fn sub(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_sub(rhs.as_u64()))
    }

    pub fn mul_unsigned(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_mul(rhs.as_u64()))
    }

    pub fn mul_signed(self, rhs: Self) -> Self {
        Self::from_i64(self.as_i64().wrapping_mul(rhs.as_i64()))
    }

    pub fn div_unsigned(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_div(rhs.as_u64()))
    }

    pub fn div_signed(self, rhs: Self) -> Self {
        Self::from_i64(self.as_i64().wrapping_div(rhs.as_i64()))
    }

    pub fn mod_unsigned(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64() % rhs.as_u64())
    }

    pub fn mod_signed(self, rhs: Self) -> Self {
        Self::from_i64(self.as_i64() % rhs.as_i64())
    }

    pub fn greater_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() > rhs.as_u64()) as u64)
    }

    pub fn greater_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() > rhs.as_i64()) as u64)
    }

    pub fn less_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() < rhs.as_u64()) as u64)
    }

    pub fn less_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() < rhs.as_i64()) as u64)
    }

    pub fn greater_or_eq_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() >= rhs.as_u64()) as u64)
    }

    pub fn greater_or_eq_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() >= rhs.as_i64()) as u64)
    }

    pub fn less_or_eq_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() <= rhs.as_u64()) as u64)
    }

    pub fn less_or_eq_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() <= rhs.as_i64()) as u64)
    }

    pub fn eq(self, rhs: Self) -> Self {
        Self::from_u64((self.0 == rhs.0) as u64)
    }

    pub fn and(self, rhs: Self) -> Self {
        let mut res = self;
        for (a, b) in res.0.iter_mut().zip(rhs.0.iter().copied()) {
            *a &= b;
        }
        res
    }

    pub fn or(self, rhs: Self) -> Self {
        let mut res = self;
        for (a, b) in res.0.iter_mut().zip(rhs.0.iter().copied()) {
            *a |= b;
        }
        res
    }

    pub fn xor(self, rhs: Self) -> Self {
        let mut res = self;
        for (a, b) in res.0.iter_mut().zip(rhs.0.iter().copied()) {
            *a ^= b;
        }
        res
    }

    pub fn not(self) -> Self {
        todo!()
    }
}

impl AsRef<[u8]> for Value {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

pub struct ExecutionContext {}

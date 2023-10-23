//! # The HypeScript Virtual Machine
//!
//! This crate implements the HypeScript VM execution engine.

pub mod vars;

use hypescript_util::array_from_slice;

/// A value in a stack or variable slot.
///
/// This wraps a `u64`, and provides utility methods for manipulating and retrieving its value as
/// various types.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Value(u64);

macro_rules! as_method {
    ($(($method_name:ident $type:ident))*) => {
        $(#[doc = concat!("Get this value as a `", stringify!($type), "`.")]
        pub fn $method_name(&self) -> $type {
            self.0 as $type
        })*
    };
}

macro_rules! from_method {
    ($(($method_name:ident $type:ident))*) => {
        $(#[doc = concat!("Create a `Value` from a `", stringify!($type), "`.")]
        pub fn $method_name(val: $type) -> Self {
            Self(val as u64)
        })*
    };
}

// In many circumstances, a `Value` can be regarded as signed or unsigned; since a single
// implementation of a trait from `std::ops` would be insufficient in these cases, we make these
// operations (e.g. comparisons, division) inherent methods suffixed with `_signed` or `_unsigned`.
// For consistency, those operations that are bitwise identical between signed and unsigned are
// also implemented as inherent methods, but without the suffixes, rather than via the trait
// implementations. Their names are then identical to trait methods from `std::ops`, which clippy
// complains about by default. So we silence it.
#[allow(clippy::should_implement_trait)]
impl Value {
    as_method! {
        (as_u8 u8)
        (as_i8 i8)
        (as_u16 u16)
        (as_i16 i16)
        (as_u32 u32)
        (as_i32 i32)
        (as_u64 u64)
        (as_i64 i64)
    }

    /// Get an array of this value's bytes, in big-endian order.
    pub fn as_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    from_method! {
        (from_u8 u8)
        (from_i8 i8)
        (from_u16 u16)
        (from_i16 i16)
        (from_u32 u32)
        (from_i32 i32)
        (from_u64 u64)
        (from_i64 i64)
    }

    /// Create a `Value` from a byte slice.
    ///
    /// This will interpret the bytes of the given slice as an unsigned integer in big-endian byte
    /// order, zero-extend to a `u64`, and create a `Value` from the result.
    ///
    /// # Panics
    ///
    /// This function will panic if the given slice is not of length 1, 2, 4, or 8.
    pub fn from_slice(val: &[u8]) -> Self {
        match val.len() {
            1 => Self::from_u8(val[0]),
            2 => Self::from_u16(u16::from_be_bytes(array_from_slice(val))),
            4 => Self::from_u32(u32::from_be_bytes(array_from_slice(val))),
            8 => Self::from_u64(u64::from_be_bytes(array_from_slice(val))),
            _ => panic!("invalid value length"),
        }
    }

    /// Create a `Value` from a byte slice, performing sign extension.
    ///
    /// This will interpret the bytes of the given slice as a signed integer in big-endian byte
    /// order, sign-extend to an `i64`, and create a `Value` from the result.
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

    /// Add two values as integers, wrapping on overflow.
    pub fn add(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_add(rhs.as_u64()))
    }

    /// Subtract two values as integers, wrapping on underflow.
    pub fn sub(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_sub(rhs.as_u64()))
    }

    /// Multiply two values as integers, wrapping on overflow.
    pub fn mul(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_mul(rhs.as_u64()))
    }

    /// Divide two values as unsigned integers.
    ///
    /// # Panics
    ///
    /// This function will panic if `rhs` is zero.
    pub fn div_unsigned(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64().wrapping_div(rhs.as_u64()))
    }

    /// Divide two values as signed integers.
    ///
    /// # Panics
    ///
    /// This function will panic if `rhs` is zero.
    pub fn div_signed(self, rhs: Self) -> Self {
        Self::from_i64(self.as_i64().wrapping_div(rhs.as_i64()))
    }

    /// Take the modulo of two values as unsigned integers.
    ///
    /// # Panics
    ///
    /// This function will panic if `rhs` is zero.
    pub fn mod_(self, rhs: Self) -> Self {
        Self::from_u64(self.as_u64() % rhs.as_u64())
    }

    /// Check if `self` is greater than `rhs`, as unsigned integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn greater_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() > rhs.as_u64()) as u64)
    }

    /// Check if `self` is greater than `rhs`, as signed integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn greater_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() > rhs.as_i64()) as u64)
    }

    /// Check if `self` is less than `rhs`, as unsigned integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn less_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() < rhs.as_u64()) as u64)
    }

    /// Check if `self` is less than `rhs`, as signed integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn less_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() < rhs.as_i64()) as u64)
    }

    /// Check if `self` is greater than or equal to `rhs`, as unsigned integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn greater_or_eq_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() >= rhs.as_u64()) as u64)
    }

    /// Check if `self` is greater than or equal to `rhs`, as signed integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn greater_or_eq_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() >= rhs.as_i64()) as u64)
    }

    /// Check if `self` is less than or equal to `rhs`, as unsigned integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn less_or_eq_unsigned(self, rhs: Self) -> Self {
        Self::from_u64((self.as_u64() <= rhs.as_u64()) as u64)
    }

    /// Check if `self` is less than or equal to `rhs`, as signed integers.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn less_or_eq_signed(self, rhs: Self) -> Self {
        Self::from_u64((self.as_i64() <= rhs.as_i64()) as u64)
    }

    /// Check if `self` is equal to `rhs`.
    ///
    /// Returns a value of 1 for true, and 0 for false.
    pub fn eq(self, rhs: Self) -> Self {
        Self::from_u64((self.0 == rhs.0) as u64)
    }

    /// Compute the bitwise AND of two values.
    pub fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    /// Compute the bitwise OR of two values.
    pub fn or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    /// Compute the bitwise XOR of two values.
    pub fn xor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    /// Get the logical negation of a value.
    ///
    /// Returns a value of 1 if `self` is 0, and a value of 0 otherwise.
    pub fn not(self) -> Self {
        Self::from_u64((self.0 == 0) as u64)
    }

    /// Compute the bitwise NOT of a value.
    pub fn inv(self) -> Self {
        Self::from_u64(!self.as_u64())
    }
}

pub struct ExecutionContext {}

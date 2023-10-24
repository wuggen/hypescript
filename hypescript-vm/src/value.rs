//! Stack and variable values.

use crate::error::*;

use hypescript_util::array_from_slice;

/// A value in a stack or variable slot.
///
/// This wraps a `u64`, and provides utility methods for manipulating and retrieving its value as
/// various types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
    /// # Errors
    ///
    /// If `rhs` is zero, this function will return an error with kind [`ErrorKind::DivideByZero`],
    /// and program counter set to zero.
    pub fn div_unsigned(self, rhs: Self) -> Result<Self> {
        Ok(Self::from_u64(
            self.as_u64().checked_div(rhs.as_u64()).ok_or(Error {
                kind: ErrorKind::DivideByZero,
                pc: 0,
            })?,
        ))
    }

    /// Divide two values as signed integers.
    ///
    /// # Errors
    ///
    /// If `rhs` is zero, this function will return an error with kind [`ErrorKind::DivideByZero`],
    /// and program counter set to zero.
    pub fn div_signed(self, rhs: Self) -> Result<Self> {
        Ok(Self::from_i64(
            self.as_i64().checked_div(rhs.as_i64()).ok_or(Error {
                kind: ErrorKind::DivideByZero,
                pc: 0,
            })?,
        ))
    }

    /// Take the modulo of two values as unsigned integers.
    ///
    /// # Errors
    ///
    /// If `rhs` is zero, this function will return an error with kind [`ErrorKind::DivideByZero`],
    /// and program counter set to zero.
    pub fn mod_(self, rhs: Self) -> Result<Self> {
        Ok(Self::from_u64(
            self.as_u64().checked_rem(rhs.as_u64()).ok_or(Error {
                kind: ErrorKind::DivideByZero,
                pc: 0,
            })?,
        ))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_unsigned() {
        assert_eq!(Value::from_u8(0x8f).as_u64(), 0x8f);
        assert_eq!(Value::from_u16(0x1234).as_u64(), 0x1234);
        assert_eq!(Value::from_u16(0x44).as_u64(), 0x44);
        assert_eq!(Value::from_u32(0xdeadbeef).as_u64(), 0xdeadbeef);
        assert_eq!(
            Value::from_u64(0x1234567890abcdef).as_u64(),
            0x1234567890abcdef
        );
    }

    #[test]
    fn from_signed() {
        assert_eq!(Value::from_i8(0x34).as_u64(), 0x34);
        assert_eq!(Value::from_i8(0x8f_u8 as i8).as_u64(), 0xffffffffffffff8f);
        assert_eq!(Value::from_i16(0x1234).as_u64(), 0x1234);
        assert_eq!(
            Value::from_i16(0x8234_u16 as i16).as_u64(),
            0xffffffffffff8234
        );
        assert_eq!(Value::from_i32(0x7eadbeef).as_u64(), 0x7eadbeef);
        assert_eq!(
            Value::from_i32(0xdeadbeef_u32 as i32).as_u64(),
            0xffffffffdeadbeef
        );
        assert_eq!(
            Value::from_i64(0x1234567890abcdef).as_u64(),
            0x1234567890abcdef
        );
        assert_eq!(
            Value::from_i64(0x890abcdef1234567_u64 as i64).as_u64(),
            0x890abcdef1234567
        );
    }

    #[test]
    fn bytes_conversions() {
        assert_eq!(
            Value::from_u32(0xdeadbeef).as_bytes(),
            [0, 0, 0, 0, 0xde, 0xad, 0xbe, 0xef]
        );
        assert_eq!(Value::from_slice(&[125]).as_u64(), 125);
        assert_eq!(Value::from_slice(&[0x12, 0x34]).as_u64(), 0x1234);
        assert_eq!(
            Value::from_slice(&[0xde, 0xad, 0xbe, 0xef]).as_u64(),
            0xdeadbeef
        );
        assert_eq!(
            Value::from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef]).as_u64(),
            0x1234567890abcdef
        );
    }

    #[test]
    fn addition() {
        assert_eq!(
            Value::from_u64(4).add(Value::from_u64(6)),
            Value::from_u64(10)
        );
        assert_eq!(
            Value::from_u64(100).add(Value::from_i64(-25)),
            Value::from_u64(75)
        );
        assert_eq!(
            Value::from_i64(-1).add(Value::from_u64(1)),
            Value::from_u64(0)
        );
    }

    #[test]
    fn subtraction() {
        assert_eq!(
            Value::from_u64(1150).sub(Value::from_u64(150)),
            Value::from_u64(1000)
        );
        assert_eq!(
            Value::from_u64(1234).sub(Value::from_i64(-6)),
            Value::from_u64(1240)
        );
        assert_eq!(
            Value::from_u64(0).sub(Value::from_u64(1)),
            Value::from_i64(-1)
        );
    }

    #[test]
    fn multiplication() {
        assert_eq!(
            Value::from_u64(8).mul(Value::from_u64(3)),
            Value::from_u64(24)
        );
        assert_eq!(
            Value::from_u64(12).mul(Value::from_i64(-2)),
            Value::from_i64(-24)
        );
        assert_eq!(
            Value::from_i64(-25).mul(Value::from_i64(-4)),
            Value::from_u64(100)
        );
    }

    #[test]
    fn unsigned_division() {
        assert_eq!(
            Value::from_u64(12)
                .div_unsigned(Value::from_u64(3))
                .unwrap(),
            Value::from_u64(4)
        );
        assert_eq!(
            Value::from_u64(15)
                .div_unsigned(Value::from_u64(4))
                .unwrap(),
            Value::from_u64(3)
        );

        assert_eq!(
            Value::from_u64(1526)
                .div_unsigned(Value::from_u64(0))
                .unwrap_err()
                .kind(),
            ErrorKind::DivideByZero,
        );
    }

    #[test]
    fn signed_division() {
        assert_eq!(
            Value::from_u64(12).div_signed(Value::from_i64(-3)).unwrap(),
            Value::from_i64(-4)
        );
        assert_eq!(
            Value::from_i64(-36)
                .div_signed(Value::from_i64(-18))
                .unwrap(),
            Value::from_u64(2)
        );

        assert_eq!(
            Value::from_i64(-162456)
                .div_signed(Value::from_i64(0))
                .unwrap_err()
                .kind(),
            ErrorKind::DivideByZero
        );
    }

    #[test]
    fn modulo() {
        assert_eq!(
            Value::from_u64(64).mod_(Value::from_u64(5)).unwrap(),
            Value::from_u64(4)
        );
        assert_eq!(
            Value::from_u64(121).mod_(Value::from_u64(11)).unwrap(),
            Value::from_u64(0)
        );

        assert_eq!(
            Value::from_u64(1234)
                .mod_(Value::from_u64(0))
                .unwrap_err()
                .kind(),
            ErrorKind::DivideByZero
        );
    }

    // TODO: tests for the rest of these methods :P
}

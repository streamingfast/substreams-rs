use hex;
use num_bigint;
use stable_hash::utils::AsInt;
use stable_hash::{FieldAddress, StableHash};
use stable_hash_legacy::SequenceNumber;
use thiserror::Error;

use bigdecimal::{One, ParseBigDecimalError, ToPrimitive, Zero};

use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Deref, Div, Mul, Neg, Sub};
use std::str::FromStr;

pub use num_bigint::Sign as BigIntSign;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BigDecimal(bigdecimal::BigDecimal);

impl From<bigdecimal::BigDecimal> for BigDecimal {
    fn from(big_decimal: bigdecimal::BigDecimal) -> Self {
        BigDecimal(big_decimal).normalized()
    }
}

impl BigDecimal {
    /// These are the limits of IEEE-754 decimal128, a format we may want to switch to. See
    /// https://en.wikipedia.org/wiki/Decimal128_floating-point_format.
    pub const MIN_EXP: i32 = -6143;
    pub const MAX_EXP: i32 = 6144;
    pub const MAX_SIGNFICANT_DIGITS: i32 = 34;

    pub fn new(digits: BigInt, exp: i64) -> Self {
        // bigdecimal uses `scale` as the opposite of the power of ten, so negate `exp`.
        Self::from(bigdecimal::BigDecimal::new(digits.0, -exp))
    }

    pub fn parse_bytes(bytes: &[u8]) -> Option<Self> {
        bigdecimal::BigDecimal::parse_bytes(bytes, 10).map(Self)
    }

    pub fn zero() -> BigDecimal {
        BigDecimal::from(0)
    }

    pub fn one() -> BigDecimal {
        BigDecimal::from(1)
    }

    pub fn as_bigint_and_exponent(&self) -> (num_bigint::BigInt, i64) {
        self.0.as_bigint_and_exponent()
    }

    pub fn digits(&self) -> u64 {
        self.0.digits()
    }

    // Copy-pasted from `bigdecimal::BigDecimal::normalize`. We can use the upstream version once it
    // is included in a released version supported by Diesel.
    #[must_use]
    pub fn normalized(&self) -> BigDecimal {
        if self == &BigDecimal::zero() {
            return BigDecimal::zero();
        }

        // Round to the maximum significant digits.
        let big_decimal = self.0.with_prec(Self::MAX_SIGNFICANT_DIGITS as u64);

        let (bigint, exp) = big_decimal.as_bigint_and_exponent();
        let (sign, mut digits) = bigint.to_radix_be(10);
        let trailing_count = digits.iter().rev().take_while(|i| **i == 0).count();
        digits.truncate(digits.len() - trailing_count);
        let int_val = num_bigint::BigInt::from_radix_be(sign, &digits, 10).unwrap();
        let scale = exp - trailing_count as i64;

        BigDecimal(bigdecimal::BigDecimal::new(int_val, scale))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn with_prec(&self, prec: u64) -> BigDecimal {
        BigDecimal::from(self.0.with_prec(prec))
    }

    pub fn neg(&self) -> BigDecimal {
        BigDecimal::from(self.0.clone().neg())
    }
}

impl Display for BigDecimal {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl fmt::Debug for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BigDecimal({})", self.0)
    }
}

impl FromStr for BigDecimal {
    type Err = <bigdecimal::BigDecimal as FromStr>::Err;

    fn from_str(s: &str) -> Result<BigDecimal, Self::Err> {
        Ok(Self::from(bigdecimal::BigDecimal::from_str(s)?))
    }
}

impl Into<bigdecimal::BigDecimal> for BigDecimal {
    fn into(self) -> bigdecimal::BigDecimal {
        self.0
    }
}

impl From<i32> for BigDecimal {
    fn from(n: i32) -> Self {
        Self::from(bigdecimal::BigDecimal::from(n))
    }
}

impl From<u32> for BigDecimal {
    fn from(n: u32) -> Self {
        Self::from(bigdecimal::BigDecimal::from(n))
    }
}

impl From<i64> for BigDecimal {
    fn from(n: i64) -> Self {
        Self::from(bigdecimal::BigDecimal::from(n))
    }
}

impl From<u64> for BigDecimal {
    fn from(n: u64) -> Self {
        Self::from(bigdecimal::BigDecimal::from(n))
    }
}

impl From<BigInt> for BigDecimal {
    fn from(n: BigInt) -> Self {
        Self::from(bigdecimal::BigDecimal::from(n.0))
    }
}

impl TryFrom<f64> for BigDecimal {
    type Error = ParseBigDecimalError;

    #[inline]
    fn try_from(n: f64) -> Result<Self, Self::Error> {
        BigDecimal::from_str(&format!(
            "{:.PRECISION$e}",
            n,
            PRECISION = ::std::f64::DIGITS as usize
        ))
    }
}

impl Add for BigDecimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from(self.0.add(other.0))
    }
}

impl Sub for BigDecimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::from(self.0.sub(other.0))
    }
}

impl Mul for BigDecimal {
    type Output = Self;

    fn mul(self, rhs: BigDecimal) -> BigDecimal {
        Self::from(self.0.mul(rhs.0))
    }
}

impl Div for BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: BigDecimal) -> BigDecimal {
        if other == BigDecimal::from(0) {
            panic!("Cannot divide by zero-valued `BigDecimal`!")
        }

        Self::from(self.0.div(other.0))
    }
}

impl bigdecimal::ToPrimitive for BigDecimal {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }
}

impl stable_hash_legacy::StableHash for BigDecimal {
    fn stable_hash<H: stable_hash_legacy::StableHasher>(
        &self,
        mut sequence_number: H::Seq,
        state: &mut H,
    ) {
        let (int, exp) = self.as_bigint_and_exponent();
        // This only allows for backward compatible changes between
        // BigDecimal and unsigned ints
        stable_hash_legacy::StableHash::stable_hash(&exp, sequence_number.next_child(), state);
        stable_hash_legacy::StableHash::stable_hash(&BigInt(int), sequence_number, state);
    }
}

impl StableHash for BigDecimal {
    fn stable_hash<H: stable_hash::StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        // This implementation allows for backward compatible changes from integers (signed or unsigned)
        // when the exponent is zero.
        let (int, exp) = self.as_bigint_and_exponent();
        StableHash::stable_hash(&exp, field_address.child(1), state);
        // Normally it would be a red flag to pass field_address in after having used a child slot.
        // But, we know the implementation of StableHash for BigInt will not use child(1) and that
        // it will not in the future due to having no forward schema evolutions for ints and the
        // stability guarantee.
        //
        // For reference, ints use child(0) for the sign and write the little endian bytes to the parent slot.
        BigInt(int).stable_hash(field_address, state);
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BigInt(num_bigint::BigInt);

impl stable_hash_legacy::StableHash for BigInt {
    #[inline]
    fn stable_hash<H: stable_hash_legacy::StableHasher>(
        &self,
        sequence_number: H::Seq,
        state: &mut H,
    ) {
        stable_hash_legacy::utils::AsInt {
            is_negative: self.0.sign() == BigIntSign::Minus,
            little_endian: &self.to_bytes_le().1,
        }
        .stable_hash(sequence_number, state)
    }
}

impl StableHash for BigInt {
    fn stable_hash<H: stable_hash::StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        AsInt {
            is_negative: self.0.sign() == BigIntSign::Minus,
            little_endian: &self.to_bytes_le().1,
        }
        .stable_hash(field_address, state)
    }
}

#[derive(Error, Debug)]
pub enum BigIntOutOfRangeError {
    #[error("Cannot convert negative BigInt into type")]
    Negative,
    #[error("BigInt value is too large for type")]
    Overflow,
}

impl<'a> TryFrom<&'a BigInt> for u64 {
    type Error = BigIntOutOfRangeError;
    fn try_from(value: &'a BigInt) -> Result<u64, BigIntOutOfRangeError> {
        let (sign, bytes) = value.to_bytes_le();

        if sign == num_bigint::Sign::Minus {
            return Err(BigIntOutOfRangeError::Negative);
        }

        if bytes.len() > 8 {
            return Err(BigIntOutOfRangeError::Overflow);
        }

        // Replace this with u64::from_le_bytes when stabilized
        let mut n = 0u64;
        let mut shift_dist = 0;
        for b in bytes {
            n |= (b as u64) << shift_dist;
            shift_dist += 8;
        }
        Ok(n)
    }
}

impl TryFrom<BigInt> for u64 {
    type Error = BigIntOutOfRangeError;
    fn try_from(value: BigInt) -> Result<u64, BigIntOutOfRangeError> {
        (&value).try_into()
    }
}

impl fmt::Debug for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BigInt({})", self)
    }
}

impl BigInt {
    pub fn zero() -> BigInt {
        BigInt::from(0)
    }

    pub fn one() -> BigInt {
        BigInt::from(1)
    }

    pub fn from_unsigned_bytes_le(bytes: &[u8]) -> Self {
        BigInt(num_bigint::BigInt::from_bytes_le(
            num_bigint::Sign::Plus,
            bytes,
        ))
    }

    pub fn from_signed_bytes_le(bytes: &[u8]) -> Self {
        BigInt(num_bigint::BigInt::from_signed_bytes_le(bytes))
    }

    pub fn from_signed_bytes_be(bytes: &[u8]) -> Self {
        BigInt(num_bigint::BigInt::from_signed_bytes_be(bytes))
    }

    pub fn to_bytes_le(&self) -> (BigIntSign, Vec<u8>) {
        self.0.to_bytes_le()
    }

    pub fn to_bytes_be(&self) -> (BigIntSign, Vec<u8>) {
        self.0.to_bytes_be()
    }

    pub fn to_signed_bytes_le(&self) -> Vec<u8> {
        self.0.to_signed_bytes_le()
    }

    pub fn to_signed_bytes_be(&self) -> Vec<u8> {
        self.0.to_signed_bytes_be()
    }

    /// Deprecated. Use try_into instead
    pub fn to_u64(&self) -> u64 {
        self.try_into().unwrap()
    }

    // pub fn from_unsigned_u256(n: &U256) -> Self {
    //     let mut bytes: [u8; 32] = [0; 32];
    //     n.to_little_endian(&mut bytes);
    //     BigInt::from_unsigned_bytes_le(&bytes)
    // }
    //
    // pub fn from_signed_u256(n: &U256) -> Self {
    //     let mut bytes: [u8; 32] = [0; 32];
    //     n.to_little_endian(&mut bytes);
    //     BigInt::from_signed_bytes_le(&bytes)
    // }
    //
    // pub fn to_signed_u256(&self) -> U256 {
    //     let bytes = self.to_signed_bytes_le();
    //     if self < &BigInt::from(0) {
    //         assert!(
    //             bytes.len() <= 32,
    //             "BigInt value does not fit into signed U256"
    //         );
    //         let mut i_bytes: [u8; 32] = [255; 32];
    //         i_bytes[..bytes.len()].copy_from_slice(&bytes);
    //         U256::from_little_endian(&i_bytes)
    //     } else {
    //         U256::from_little_endian(&bytes)
    //     }
    // }
    //
    // pub fn to_unsigned_u256(&self) -> U256 {
    //     let (sign, bytes) = self.to_bytes_le();
    //     assert!(
    //         sign == BigIntSign::NoSign || sign == BigIntSign::Plus,
    //         "negative value encountered for U256: {}",
    //         self
    //     );
    //     U256::from_little_endian(&bytes)
    // }

    pub fn pow(self, exponent: u32) -> Self {
        use num_traits::pow::Pow;

        BigInt(self.0.pow(exponent))
    }

    pub fn bits(&self) -> usize {
        self.0.bits() as usize
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.0.is_one()
    }

    pub fn neg(&self) -> BigInt {
        BigInt::from(self.0.clone().neg())
    }
}

impl Display for BigInt {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl From<num_bigint::BigInt> for BigInt {
    fn from(big_int: num_bigint::BigInt) -> BigInt {
        BigInt(big_int)
    }
}

impl Into<num_bigint::BigInt> for BigInt {
    fn into(self) -> num_bigint::BigInt {
        self.0
    }
}

impl Into<i32> for BigInt {
    fn into(self) -> i32 {
        self.0.to_i32().unwrap()
    }
}

impl From<i32> for BigInt {
    fn from(i: i32) -> BigInt {
        BigInt(i.into())
    }
}

impl From<u64> for BigInt {
    fn from(i: u64) -> BigInt {
        BigInt(i.into())
    }
}

impl From<i64> for BigInt {
    fn from(i: i64) -> BigInt {
        BigInt(i.into())
    }
}

impl From<[u8; 32]> for BigInt {
    fn from(i: [u8; 32]) -> Self {
        BigInt::from_signed_bytes_be(&i)
    }
}

// impl From<U64> for BigInt {
//     /// This implementation assumes that U64 represents an unsigned U64,
//     /// and not a signed U64 (aka int64 in Solidity). Right now, this is
//     /// all we need (for block numbers). If it ever becomes necessary to
//     /// handle signed U64s, we should add the same
//     /// `{to,from}_{signed,unsigned}_u64` methods that we have for U64.
//     fn from(n: U64) -> BigInt {
//         BigInt::from(n.as_u64())
//     }
// }
//
// impl From<U128> for BigInt {
//     /// This implementation assumes that U128 represents an unsigned U128,
//     /// and not a signed U128 (aka int128 in Solidity). Right now, this is
//     /// all we need (for block numbers). If it ever becomes necessary to
//     /// handle signed U128s, we should add the same
//     /// `{to,from}_{signed,unsigned}_u128` methods that we have for U256.
//     fn from(n: U128) -> BigInt {
//         let mut bytes: [u8; 16] = [0; 16];
//         n.to_little_endian(&mut bytes);
//         BigInt::from_unsigned_bytes_le(&bytes)
//     }
// }

impl FromStr for BigInt {
    type Err = <num_bigint::BigInt as FromStr>::Err;

    fn from_str(s: &str) -> Result<BigInt, Self::Err> {
        num_bigint::BigInt::from_str(s).map(BigInt)
    }
}

impl Add for BigInt {
    type Output = BigInt;

    fn add(self, other: BigInt) -> BigInt {
        BigInt(self.0.add(other.0))
    }
}

impl Sub for BigInt {
    type Output = BigInt;

    fn sub(self, other: BigInt) -> BigInt {
        BigInt(self.0.sub(other.0))
    }
}

impl Mul for BigInt {
    type Output = BigInt;

    fn mul(self, other: BigInt) -> BigInt {
        BigInt(self.0.mul(other.0))
    }
}

impl Div for BigInt {
    type Output = BigInt;

    fn div(self, other: BigInt) -> BigInt {
        if other == BigInt::from(0) {
            panic!("Cannot divide by zero-valued `BigInt`!")
        }

        BigInt(self.0.div(other.0))
    }
}

/// A byte array that's serialized as a hex string prefixed by `0x`.
#[derive(Clone, PartialEq, Eq)]
pub struct Bytes(Box<[u8]>);

impl Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Bytes(0x{})", hex::encode(&self.0))
    }
}

impl Bytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for Bytes {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Bytes, Self::Err> {
        hex::decode(s.trim_start_matches("0x")).map(|x| Bytes(x.into()))
    }
}

impl<'a> From<&'a [u8]> for Bytes {
    fn from(array: &[u8]) -> Self {
        Bytes(array.into())
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(array: [u8; N]) -> Bytes {
        Bytes(array.into())
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Bytes(vec.into())
    }
}

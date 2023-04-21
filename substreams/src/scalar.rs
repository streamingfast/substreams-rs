use num_bigint::{Sign, ToBigInt};
use num_traits::{FromPrimitive, Signed};
use {
    bigdecimal::{One, ParseBigDecimalError, ToPrimitive, Zero},
    num_bigint::{BigUint, ParseBigIntError, Sign as BigIntSign},
    std::{
        convert::{TryFrom, TryInto},
        fmt::{self, Display, Formatter},
        ops::{Add, Div, Mul, Neg, Sub},
        str,
        str::FromStr,
    },
    thiserror::Error,
};

// ---------- BigDecimal ---------- //
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BigDecimal(bigdecimal::BigDecimal);

impl BigDecimal {
    /// These are the limits of IEEE-754 decimal128, a format we may want to switch to. See
    /// https://en.wikipedia.org/wiki/Decimal128_floating-point_format.
    pub const MIN_EXP: i32 = -6143;
    pub const MAX_EXP: i32 = 6144;
    pub const MAX_SIGNIFICANT_DIGITS: i32 = 34;

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

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn with_prec(&self, prec: u64) -> BigDecimal {
        BigDecimal::from(self.0.with_prec(prec))
    }

    pub fn neg(&self) -> BigDecimal {
        BigDecimal::from(self.0.clone().neg())
    }

    pub fn from_store_bytes(bytes: &[u8]) -> BigDecimal {
        if bytes.len() == 0 {
            return BigDecimal::zero();
        }

        let bytes_as_str = str::from_utf8(bytes.as_ref()).unwrap();
        return BigDecimal::from_str(bytes_as_str).unwrap();
    }

    pub fn divide_by_decimals(big_decimal_amount: BigDecimal, decimals: u64) -> BigDecimal {
        // FIXME: Should we think about using a table of pre-made BigDecimal for a range of decimals between 0 -> 20?
        big_decimal_amount.div(BigDecimal::new(BigInt::one(), decimals as i64))
    }

    pub fn absolute(&self) -> BigDecimal {
        // TODO: implement as a a trit
        BigDecimal::from(self.0.abs())
    }

    pub fn to_bigint(&self) -> BigInt {
        BigInt(self.0.to_bigint().unwrap())
    }
}

impl AsRef<BigDecimal> for BigDecimal {
    fn as_ref(&self) -> &BigDecimal {
        &self
    }
}

impl ToPrimitive for BigDecimal {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
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

impl TryFrom<String> for BigDecimal {
    type Error = ParseBigDecimalError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        BigDecimal::try_from(value.as_str())
    }
}

impl TryFrom<&String> for BigDecimal {
    type Error = ParseBigDecimalError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        BigDecimal::try_from(value.as_str())
    }
}

impl TryFrom<&str> for BigDecimal {
    type Error = ParseBigDecimalError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        bigdecimal::BigDecimal::from_str(value).map(|bd| BigDecimal(bd))
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

impl From<usize> for BigDecimal {
    fn from(n: usize) -> Self {
        match bigdecimal::BigDecimal::from_usize(n) {
            None => {
                panic!("creating big decimal from invalid usize value {}", n)
            }
            Some(bd) => BigDecimal(bd),
        }
    }
}

impl From<BigInt> for BigDecimal {
    fn from(n: BigInt) -> Self {
        Self::from(bigdecimal::BigDecimal::from(n.0))
    }
}

impl From<BigUint> for BigDecimal {
    fn from(val: BigUint) -> Self {
        BigInt(num_bigint::BigInt::from(val)).into()
    }
}

impl From<bigdecimal::BigDecimal> for BigDecimal {
    fn from(big_decimal: bigdecimal::BigDecimal) -> Self {
        BigDecimal(big_decimal)
    }
}

impl From<&bigdecimal::BigDecimal> for BigDecimal {
    fn from(big_decimal: &bigdecimal::BigDecimal) -> Self {
        BigDecimal(big_decimal.clone())
    }
}

impl TryFrom<f32> for BigDecimal {
    type Error = ParseBigDecimalError;

    #[inline]
    fn try_from(n: f32) -> Result<Self, Self::Error> {
        BigDecimal::from_str(&format!(
            "{:.PRECISION$e}",
            n,
            PRECISION = ::std::f32::DIGITS as usize
        ))
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

impl Into<String> for &BigDecimal {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Into<String> for BigDecimal {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Into<bigdecimal::BigDecimal> for BigDecimal {
    fn into(self) -> bigdecimal::BigDecimal {
        self.0
    }
}

impl<T> Add<T> for BigDecimal
where
    T: Into<BigDecimal>,
{
    type Output = BigDecimal;

    fn add(self, other: T) -> BigDecimal {
        BigDecimal(self.0 + other.into().0)
    }
}

impl<T> Sub<T> for BigDecimal
where
    T: Into<BigDecimal>,
{
    type Output = BigDecimal;

    fn sub(self, other: T) -> BigDecimal {
        BigDecimal(self.0 - other.into().0)
    }
}

impl<T> Mul<T> for BigDecimal
where
    T: Into<BigDecimal>,
{
    type Output = BigDecimal;

    fn mul(self, rhs: T) -> BigDecimal {
        BigDecimal(self.0 * rhs.into().0)
    }
}

impl<T> Div<T> for BigDecimal
where
    T: Into<BigDecimal>,
{
    type Output = BigDecimal;

    fn div(self, rhs: T) -> BigDecimal {
        let rhs = rhs.into();
        if rhs.is_zero() {
            panic!("attempt to divide by zero");
        }

        BigDecimal(self.0 / rhs.0)
    }
}

impl Div<&BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn div(self, other: &BigDecimal) -> BigDecimal {
        if other.is_zero() {
            panic!("Cannot divide by zero-valued `BigDecimal`!")
        }

        Self::from(self.0.div(&other.0))
    }
}

// ---------- BigInt ---------- //
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BigInt(num_bigint::BigInt);

#[derive(Error, Debug)]
pub enum BigIntOutOfRangeError {
    #[error("Cannot convert negative BigInt into type")]
    Negative,
    #[error("BigInt value is too large for type")]
    Overflow,
}

impl fmt::Debug for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BigInt({})", self)
    }
}

impl Display for BigInt {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl AsRef<BigInt> for BigInt {
    fn as_ref(&self) -> &BigInt {
        &self
    }
}

impl BigInt {
    pub fn new(sign: Sign, digits: Vec<u32>) -> BigInt {
        return BigInt(num_bigint::BigInt::new(sign, digits));
    }

    pub fn zero() -> BigInt {
        BigInt::from(0)
    }

    pub fn one() -> BigInt {
        BigInt::from(1)
    }

    pub fn from_unsigned_bytes_be(bytes: &[u8]) -> Self {
        BigInt(num_bigint::BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            bytes,
        ))
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

    pub fn from_bytes_le(sign: BigIntSign, bytes: &[u8]) -> Self {
        BigInt(num_bigint::BigInt::from_bytes_le(sign, bytes))
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

    pub fn to_u64(&self) -> u64 {
        self.try_into().unwrap()
    }

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

    pub fn from_store_bytes(bytes: &[u8]) -> BigInt {
        let bytes = bytes.as_ref();

        if bytes.len() == 0 {
            return BigInt::zero();
        }
        let bytes_as_str = str::from_utf8(bytes).unwrap();
        return BigInt::from_str(bytes_as_str).unwrap();
    }

    pub fn to_decimal(&self, decimals: u64) -> BigDecimal {
        // FIXME: Should we think about using a table of pre-made BigDecimal for a range of decimals between 0 -> 20?
        let big_decimal_amount: BigDecimal = self.into();
        return big_decimal_amount.div(BigDecimal::new(BigInt::one(), decimals as i64));
    }

    pub fn absolute(&self) -> BigInt {
        BigInt::from(self.0.abs())
    }
}

impl FromStr for BigInt {
    type Err = <num_bigint::BigInt as FromStr>::Err;

    fn from_str(s: &str) -> Result<BigInt, Self::Err> {
        num_bigint::BigInt::from_str(s).map(BigInt)
    }
}

impl From<u32> for BigInt {
    fn from(i: u32) -> BigInt {
        BigInt(i.into())
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

impl From<usize> for BigInt {
    fn from(i: usize) -> BigInt {
        BigInt(i.into())
    }
}

impl From<isize> for BigInt {
    fn from(i: isize) -> BigInt {
        BigInt(i.into())
    }
}

impl TryFrom<String> for BigInt {
    type Error = ParseBigIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        BigInt::from_str(value.as_str())
    }
}

impl TryFrom<&String> for BigInt {
    type Error = ParseBigIntError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        BigInt::from_str(value.as_str())
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

impl TryFrom<BigInt> for u64 {
    type Error = BigIntOutOfRangeError;
    fn try_from(value: BigInt) -> Result<u64, BigIntOutOfRangeError> {
        (&value).try_into()
    }
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

impl Into<u32> for BigInt {
    fn into(self) -> u32 {
        self.0.to_u32().unwrap()
    }
}

impl Into<i32> for BigInt {
    fn into(self) -> i32 {
        self.0.to_i32().unwrap()
    }
}

impl Into<String> for BigInt {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Into<String> for &BigInt {
    fn into(self) -> String {
        self.to_string()
    }
}

impl Into<BigDecimal> for &BigInt {
    fn into(self) -> BigDecimal {
        BigDecimal(bigdecimal::BigDecimal::from(self.0.clone()))
    }
}

impl<T> Add<T> for BigInt
where
    T: Into<BigInt>,
{
    type Output = BigInt;

    fn add(self, other: T) -> BigInt {
        BigInt(self.0.add(other.into().0))
    }
}

impl Add<BigDecimal> for BigInt {
    type Output = BigDecimal;

    fn add(self, other: BigDecimal) -> BigDecimal {
        let lhs: BigDecimal = self.into();
        lhs.add(other)
    }
}

macro_rules! impl_add_floats_bigint {
    ($($t:ty),*) => {
        $(
            impl Add<$t> for BigInt
            {
                type Output = BigDecimal;

                fn add(self, other: $t) -> BigDecimal {
                    let rhs: BigDecimal = other.try_into().unwrap();
                    self.add(rhs)
                }
            }
        )*
    }
}
impl_add_floats_bigint!(f32, f64);

macro_rules! impl_add_bigint_int {
    ($($t:ty),*) => {
        $(
            impl Add<BigInt> for $t
            {
                type Output = BigInt;

                fn add(self, other: BigInt) -> BigInt {
                    let lhs: BigInt = self.into();
                    lhs.add(other)
                }
            }
        )*
    }
}
impl_add_bigint_int!(u32, i32, u64, i64, usize, isize);

macro_rules! impl_add_bigint_float {
    ($($t:ty),*) => {
        $(
            impl Add<BigInt> for $t
            {
                type Output = BigDecimal;

                fn add(self, other: BigInt) -> BigDecimal {
                    let lhs: BigDecimal = match self.try_into() {
                        Ok(v) => v,
                        Err(_) => panic!("Cannot convert {} to BigDecimal", self)
                    };
                    let rhs: BigDecimal = other.into();
                    lhs.add(rhs)
                }
            }
        )*
    }
}
impl_add_bigint_float!(f32, f64);

impl<T> Sub<T> for BigInt
where
    T: Into<BigInt>,
{
    type Output = BigInt;

    fn sub(self, other: T) -> BigInt {
        BigInt(self.0.sub(other.into().0))
    }
}

impl Sub<BigDecimal> for BigInt {
    type Output = BigDecimal;

    fn sub(self, other: BigDecimal) -> BigDecimal {
        let lhs: BigDecimal = self.into();
        lhs.sub(other)
    }
}

macro_rules! impl_sub_floats_bigint {
    ($($t:ty),*) => {
        $(
            impl Sub<$t> for BigInt
            {
                type Output = BigDecimal;

                fn sub(self, other: $t) -> BigDecimal {
                    let rhs: BigDecimal = other.try_into().unwrap();
                    self.sub(rhs)
                }
            }
        )*
    }
}
impl_sub_floats_bigint!(f32, f64);

macro_rules! impl_sub_bigint_int {
    ($($t:ty),*) => {
        $(
            impl Sub<BigInt> for $t
            {
                type Output = BigInt;

                fn sub(self, other: BigInt) -> BigInt {
                    let lhs: BigInt = self.into();
                    lhs.sub(other)
                }
            }
        )*
    }
}
impl_sub_bigint_int!(u32, i32, u64, i64, usize, isize);

macro_rules! impl_sub_bigint_float {
    ($($t:ty),*) => {
        $(
            impl Sub<BigInt> for $t
            {
                type Output = BigDecimal;

                fn sub(self, other: BigInt) -> BigDecimal {
                    let lhs: BigDecimal = match self.try_into() {
                        Ok(v) => v,
                        Err(_) => panic!("Cannot convert {} to BigDecimal", self)
                    };
                    let rhs: BigDecimal = other.into();
                    lhs.sub(rhs)
                }
            }
        )*
    }
}
impl_sub_bigint_float!(f32, f64);

impl<T> Mul<T> for BigInt
where
    T: Into<BigInt>,
{
    type Output = BigInt;

    fn mul(self, other: T) -> BigInt {
        BigInt(self.0.mul(other.into().0))
    }
}

impl Mul<BigDecimal> for BigInt {
    type Output = BigDecimal;

    fn mul(self, other: BigDecimal) -> BigDecimal {
        let lhs: BigDecimal = self.into();
        lhs.mul(other)
    }
}

macro_rules! impl_mul_floats_bigint {
    ($($t:ty),*) => {
        $(
            impl Mul<$t> for BigInt
            {
                type Output = BigDecimal;

                fn mul(self, other: $t) -> BigDecimal {
                    let rhs: BigDecimal = other.try_into().unwrap();
                    self.mul(rhs)
                }
            }
        )*
    }
}
impl_mul_floats_bigint!(f32, f64);

macro_rules! impl_mul_bigint_int {
    ($($t:ty),*) => {
        $(
            impl Mul<BigInt> for $t
            {
                type Output = BigInt;

                fn mul(self, other: BigInt) -> BigInt {
                    let lhs: BigInt = self.into();
                    lhs.mul(other)
                }
            }
        )*
    }
}
impl_mul_bigint_int!(u32, i32, u64, i64, usize, isize);

macro_rules! impl_mul_bigint_float {
    ($($t:ty),*) => {
        $(
            impl Mul<BigInt> for $t
            {
                type Output = BigDecimal;

                fn mul(self, other: BigInt) -> BigDecimal {
                    let lhs: BigDecimal = match self.try_into() {
                        Ok(v) => v,
                        Err(_) => panic!("Cannot convert {} to BigDecimal", self)
                    };
                    let rhs: BigDecimal = other.into();
                    lhs.mul(rhs)
                }
            }
        )*
    }
}
impl_mul_bigint_float!(f32, f64);

impl<T> Div<T> for BigInt
where
    T: Into<BigInt>,
{
    type Output = BigInt;

    fn div(self, other: T) -> BigInt {
        let rhs: BigInt = other.into();
        if rhs.is_zero() {
            panic!("Cannot divide by zero-valued `BigInt`!")
        }

        BigInt(self.0.div(rhs.0))
    }
}

impl Div<BigDecimal> for BigInt {
    type Output = BigDecimal;

    fn div(self, other: BigDecimal) -> BigDecimal {
        if other.is_zero() {
            panic!("Cannot divide by zero-valued `BigDecimal`!")
        }
        let lhs: BigDecimal = self.into();
        lhs.div(other)
    }
}

macro_rules! impl_div_floats_bigint {
    ($($t:ty),*) => {
        $(
            impl Div<$t> for BigInt
            {
                type Output = BigDecimal;

                fn div(self, other: $t) -> BigDecimal {
                    if other.is_zero() {
                        panic!("Cannot divide by zero-valued `BigDecimal`!")
                    }
                    let rhs: BigDecimal = other.try_into().unwrap();
                    self.div(rhs)
                }
            }
        )*
    }
}
impl_div_floats_bigint!(f32, f64);

macro_rules! impl_div_bigint_int {
    ($($t:ty),*) => {
        $(
            impl Div<BigInt> for $t
            {
                type Output = BigInt;

                fn div(self, other: BigInt) -> BigInt {
                    if other.is_zero() {
                        panic!("Cannot divide by zero-valued `BigInt`!")
                    }
                    let lhs: BigInt = self.into();
                    lhs.div(other)
                }
            }
        )*
    }
}
impl_div_bigint_int!(u32, i32, u64, i64, usize, isize);

macro_rules! impl_div_bigint_float {
    ($($t:ty),*) => {
        $(
            impl Div<BigInt> for $t
            {
                type Output = BigDecimal;

                fn div(self, other: BigInt) -> BigDecimal {
                    let lhs: BigDecimal = match self.try_into() {
                        Ok(v) => v,
                        Err(_) => panic!("Cannot convert {} to BigDecimal", self)
                    };

                    let rhs: BigDecimal = other.into();
                    if rhs.is_zero() {
                        panic!("Cannot divide by zero-valued `BigDecimal`!")
                    }
                    lhs.div(rhs)
                }
            }
        )*
    }
}
impl_div_bigint_float!(f32, f64);

#[cfg(test)]
mod tests {
    use super::BigDecimal;
    use super::BigInt;
    use std::convert::TryFrom;

    fn big_decimal(input: f64) -> BigDecimal {
        BigDecimal::try_from(input).unwrap()
    }

    fn big_uint(input: u64) -> BigInt {
        BigInt::try_from(input).unwrap()
    }

    fn big_int(input: i64) -> BigInt {
        BigInt::try_from(input).unwrap()
    }

    #[test]
    fn bigint_op_int() {
        assert_eq!(big_int(1) + 1 as i32, big_int(2));
        assert_eq!(big_int(1) + 1 as i64, big_int(2));
        assert_eq!(big_int(1) + 1 as u32, big_int(2));
        assert_eq!(big_int(1) + 1 as u64, big_int(2));
        assert_eq!(big_int(1) + 1 as isize, big_int(2));
        assert_eq!(big_int(1) + 1 as usize, big_int(2));
        assert_eq!(big_int(1) + 1, big_int(2));
        assert_eq!(big_int(1) - 1 as i32, big_int(0));
        assert_eq!(big_int(1) - 1 as i64, big_int(0));
        assert_eq!(big_int(1) - 1 as u32, big_int(0));
        assert_eq!(big_int(1) - 1 as u64, big_int(0));
        assert_eq!(big_int(1) - 1 as isize, big_int(0));
        assert_eq!(big_int(1) - 1 as usize, big_int(0));
        assert_eq!(big_int(1) - 1, big_int(0));
        assert_eq!(big_int(2) * 2 as i32, big_int(4));
        assert_eq!(big_int(2) * 2 as i64, big_int(4));
        assert_eq!(big_int(2) * 2 as u32, big_int(4));
        assert_eq!(big_int(2) * 2 as u64, big_int(4));
        assert_eq!(big_int(2) * 2 as isize, big_int(4));
        assert_eq!(big_int(2) * 2 as usize, big_int(4));
        assert_eq!(big_int(2) * 2, big_int(4));
        assert_eq!(big_int(4) / 2 as i32, big_int(2));
        assert_eq!(big_int(4) / 2 as i64, big_int(2));
        assert_eq!(big_int(4) / 2 as u32, big_int(2));
        assert_eq!(big_int(4) / 2 as u64, big_int(2));
        assert_eq!(big_int(4) / 2 as isize, big_int(2));
        assert_eq!(big_int(4) / 2 as usize, big_int(2));
        assert_eq!(big_int(4) / 2, big_int(2));
        assert_eq!(big_int(3) / 2 as i32, big_int(1));
        assert_eq!(big_int(3) / 2 as i64, big_int(1));
        assert_eq!(big_int(3) / 2 as u32, big_int(1));
        assert_eq!(big_int(3) / 2 as u64, big_int(1));
        assert_eq!(big_int(3) / 2 as isize, big_int(1));
        assert_eq!(big_int(3) / 2 as usize, big_int(1));
        assert_eq!(big_int(3) / 2, big_int(1));
    }

    #[test]
    fn big_uint_minus_int_is_signed() {
        assert_eq!(big_uint(1) - 2 as i32, big_int(-1));
        assert_eq!(big_uint(1) - 2 as i64, big_int(-1));
        assert_eq!(big_uint(1) - 2 as u32, big_int(-1));
        assert_eq!(big_uint(1) - 2 as u64, big_int(-1));
        assert_eq!(big_uint(1) - 2 as isize, big_int(-1));
        assert_eq!(big_uint(1) - 2 as usize, big_int(-1));
        assert_eq!(big_uint(1) - 2, big_int(-1));
    }

    //
    #[test]
    fn int_op_bigint() {
        assert_eq!(1 as i32 + big_int(1), big_int(2));
        assert_eq!(1 as i64 + big_int(1), big_int(2));
        assert_eq!(1 as u32 + big_int(1), big_int(2));
        assert_eq!(1 as u64 + big_int(1), big_int(2));
        assert_eq!(1 as isize + big_int(1), big_int(2));
        assert_eq!(1 as usize + big_int(1), big_int(2));
        assert_eq!(1 + big_int(1), big_int(2));
        assert_eq!(1 as i32 - big_int(1), big_int(0));
        assert_eq!(1 as i64 - big_int(1), big_int(0));
        assert_eq!(1 as u32 - big_int(1), big_int(0));
        assert_eq!(1 as u64 - big_int(1), big_int(0));
        assert_eq!(1 as isize - big_int(1), big_int(0));
        assert_eq!(1 as usize - big_int(1), big_int(0));
        assert_eq!(1 - big_int(1), big_int(0));
        assert_eq!(2 as i32 * big_int(2), big_int(4));
        assert_eq!(2 as i64 * big_int(2), big_int(4));
        assert_eq!(2 as u32 * big_int(2), big_int(4));
        assert_eq!(2 as u64 * big_int(2), big_int(4));
        assert_eq!(2 as isize * big_int(2), big_int(4));
        assert_eq!(2 as usize * big_int(2), big_int(4));
        assert_eq!(2 * big_int(2), big_int(4));
        assert_eq!(4 as i32 / big_int(2), big_int(2));
        assert_eq!(4 as i64 / big_int(2), big_int(2));
        assert_eq!(4 as u32 / big_int(2), big_int(2));
        assert_eq!(4 as u64 / big_int(2), big_int(2));
        assert_eq!(4 as isize / big_int(2), big_int(2));
        assert_eq!(4 as usize / big_int(2), big_int(2));
        assert_eq!(4 / big_int(2), big_int(2));
        assert_eq!(3 as i32 / big_int(2), big_int(1));
        assert_eq!(3 as i64 / big_int(2), big_int(1));
        assert_eq!(3 as u32 / big_int(2), big_int(1));
        assert_eq!(3 as u64 / big_int(2), big_int(1));
        assert_eq!(3 as isize / big_int(2), big_int(1));
        assert_eq!(3 as usize / big_int(2), big_int(1));
        assert_eq!(3 / big_int(2), big_int(1));
    }

    #[test]
    fn bigint_op_float() {
        assert_eq!(big_int(1) + 1.0 as f64, big_decimal(2.0));
        assert_eq!(big_int(1) + 1.0 as f32, big_decimal(2.0));
        assert_eq!(big_int(1) + 1.0, big_decimal(2.0));
        assert_eq!(big_int(1) - 1.0 as f64, big_decimal(0.0));
        assert_eq!(big_int(1) - 1.0 as f32, big_decimal(0.0));
        assert_eq!(big_int(1) - 1.0, big_decimal(0.0));
        assert_eq!(big_int(2) * 2.0 as f64, big_decimal(4.0));
        assert_eq!(big_int(2) * 2.0 as f32, big_decimal(4.0));
        assert_eq!(big_int(2) * 2.0, big_decimal(4.0));
        assert_eq!(big_int(4) / 2.0 as f64, big_decimal(2.0));
        assert_eq!(big_int(4) / 2.0 as f32, big_decimal(2.0));
        assert_eq!(big_int(4) / 2.0, big_decimal(2.0));
    }

    #[test]
    fn float_op_bigint() {
        assert_eq!(1.0 as f64 + big_int(1), big_decimal(2.0));
        assert_eq!(1.0 as f32 + big_int(1), big_decimal(2.0));
        assert_eq!(1.0 + big_int(1), big_decimal(2.0));
        assert_eq!(1.0 as f64 - big_int(1), big_decimal(0.0));
        assert_eq!(1.0 as f32 - big_int(1), big_decimal(0.0));
        assert_eq!(1.0 - big_int(1), big_decimal(0.0));
        assert_eq!(2.0 as f64 * big_int(2), big_decimal(4.0));
        assert_eq!(2.0 as f32 * big_int(2), big_decimal(4.0));
        assert_eq!(2.0 * big_int(2), big_decimal(4.0));
        assert_eq!(4.0 as f64 / big_int(2), big_decimal(2.0));
        assert_eq!(4.0 as f32 / big_int(2), big_decimal(2.0));
        assert_eq!(4.0 / big_int(2), big_decimal(2.0));
    }

    #[test]
    fn bigint_op_bigdecimal() {
        assert_eq!(big_int(1) + big_decimal(1.0), big_decimal(2.0));
        assert_eq!(big_int(1) - big_decimal(1.0), big_decimal(0.0));
        assert_eq!(big_int(2) * big_decimal(2.0), big_decimal(4.0));
        assert_eq!(big_int(4) / big_decimal(2.0), big_decimal(2.0));
    }

    #[test]
    fn bigdecimal_op_bigint() {
        assert_eq!(big_decimal(1.0) + big_int(1), big_decimal(2.0));
        assert_eq!(big_decimal(1.0) - big_int(1), big_decimal(0.0));
        assert_eq!(big_decimal(2.0) * big_int(2), big_decimal(4.0));
        assert_eq!(big_decimal(4.0) / big_int(2), big_decimal(2.0));
    }

    #[test]
    fn bigint_divide_by_decimals() {
        assert_eq!(big_uint(50000).to_decimal(3), big_decimal(50.0));

        assert_eq!(big_uint(112000000).to_decimal(5), big_decimal(1120.0));

        assert_eq!(
            big_uint(11205450180000000000).to_decimal(18),
            big_decimal(11.20545018)
        );

        assert_eq!(
            big_uint(112054501800000000).to_decimal(18),
            big_decimal(0.1120545018)
        );

        assert_eq!(
            big_uint(11205450180000000000).to_decimal(20),
            big_decimal(0.1120545018)
        );
    }

    #[test]
    fn bigdecimal_divide_by_decimals() {
        assert_eq!(
            BigDecimal::divide_by_decimals(big_decimal(50000.0), 3),
            big_decimal(50.0)
        );

        assert_eq!(
            BigDecimal::divide_by_decimals(big_decimal(112000000.5), 5),
            big_decimal(1120.000005)
        );

        assert_eq!(
            BigDecimal::divide_by_decimals(big_decimal(11205450180000000000.51), 18),
            big_decimal(11.20545018)
        );

        assert_eq!(
            BigDecimal::divide_by_decimals(big_decimal(112054501800000000.51), 18),
            big_decimal(0.1120545018)
        );

        assert_eq!(
            BigDecimal::divide_by_decimals(big_decimal(11205450180000000000.51), 20),
            big_decimal(0.1120545018)
        );
    }
}

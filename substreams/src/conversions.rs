/// This module contains conversion traits between different BigInt implementations.
/// 
/// Currently, it provides conversions between:
/// - `substreams::scalar::BigInt` (aka `scalar::BigInt`)
/// - `substreams_ethereum::pb::eth::v2::BigInt` (aka `pb::BigInt`)
/// 
/// The conversions are implemented using the `From` and `Into` traits, which allows
/// for seamless conversion between the different types.
/// 
/// Note: This module is only available when the `ethereum` feature is enabled.
/// 
/// # Examples
/// 
/// ```rust
/// # #[cfg(feature = "ethereum")]
/// # {
/// use substreams::scalar::BigInt as ScalarBigInt;
/// use substreams_ethereum::pb::eth::v2::BigInt as PbBigInt;
/// 
/// // Convert from scalar::BigInt to pb::eth::v2::BigInt
/// let scalar_bigint = ScalarBigInt::from(42);
/// let pb_bigint: PbBigInt = scalar_bigint.into();
/// 
/// // Convert from pb::eth::v2::BigInt to scalar::BigInt
/// let pb_bigint = PbBigInt { bytes: vec![42] };
/// let scalar_bigint: ScalarBigInt = pb_bigint.into();
/// # }
/// ```

/// This trait is implemented for the Ethereum BigInt type to convert from scalar::BigInt.
/// 
/// Example:
/// ```rust
/// # #[cfg(feature = "ethereum")]
/// # {
/// use substreams::scalar::BigInt as ScalarBigInt;
/// use substreams_ethereum::pb::eth::v2::BigInt as PbBigInt;
/// 
/// let scalar_bigint = ScalarBigInt::from(42);
/// let pb_bigint: PbBigInt = scalar_bigint.into();
/// # }
/// ```
#[cfg(feature = "ethereum")]
impl From<crate::scalar::BigInt> for substreams_ethereum::pb::eth::v2::BigInt {
    fn from(value: crate::scalar::BigInt) -> Self {
        // Convert scalar::BigInt to pb::eth::v2::BigInt
        // The Ethereum BigInt uses bytes representation
        let bytes = value.to_signed_bytes_be();
        substreams_ethereum::pb::eth::v2::BigInt { bytes }
    }
}

/// This trait is implemented for the scalar::BigInt type to convert from Ethereum BigInt.
/// 
/// Example:
/// ```rust
/// # #[cfg(feature = "ethereum")]
/// # {
/// use substreams::scalar::BigInt as ScalarBigInt;
/// use substreams_ethereum::pb::eth::v2::BigInt as PbBigInt;
/// 
/// let pb_bigint = PbBigInt { bytes: vec![0x01] };
/// let scalar_bigint: ScalarBigInt = pb_bigint.into();
/// # }
/// ```
#[cfg(feature = "ethereum")]
impl From<substreams_ethereum::pb::eth::v2::BigInt> for crate::scalar::BigInt {
    fn from(value: substreams_ethereum::pb::eth::v2::BigInt) -> Self {
        // Convert pb::eth::v2::BigInt to scalar::BigInt
        // The scalar::BigInt can be created from signed big-endian bytes
        crate::scalar::BigInt::from_signed_bytes_be(&value.bytes)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[cfg(feature = "ethereum")]
mod tests {
    use crate::scalar::BigInt as ScalarBigInt;
    use substreams_ethereum::pb::eth::v2::BigInt as PbBigInt;

    #[test]
    fn test_scalar_bigint_to_pb_bigint() {
        // Test conversion from scalar::BigInt to pb::eth::v2::BigInt
        let scalar_bigint = ScalarBigInt::from(42);
        let pb_bigint: PbBigInt = scalar_bigint.into();
        
        // The Ethereum BigInt uses big-endian signed bytes
        assert_eq!(pb_bigint.bytes, vec![42]);
        
        // Test with a larger number
        let scalar_bigint = ScalarBigInt::from(256);
        let pb_bigint: PbBigInt = scalar_bigint.into();
        assert_eq!(pb_bigint.bytes, vec![1, 0]);
        
        // Test with a negative number
        let scalar_bigint = ScalarBigInt::from(-42);
        let pb_bigint: PbBigInt = scalar_bigint.into();
        // In two's complement, -42 in a single byte would be 214 (256 - 42)
        assert_eq!(pb_bigint.bytes, vec![214]);
    }

    #[test]
    fn test_pb_bigint_to_scalar_bigint() {
        // Test conversion from pb::eth::v2::BigInt to scalar::BigInt
        let pb_bigint = PbBigInt { bytes: vec![42] };
        let scalar_bigint: ScalarBigInt = pb_bigint.into();
        
        // Convert back to a value we can check
        assert_eq!(scalar_bigint.to_string(), "42");
        
        // Test with a larger number
        let pb_bigint = PbBigInt { bytes: vec![1, 0] };
        let scalar_bigint: ScalarBigInt = pb_bigint.into();
        assert_eq!(scalar_bigint.to_string(), "256");
        
        // Test with a negative number (in two's complement)
        let pb_bigint = PbBigInt { bytes: vec![214] }; // -42 in two's complement
        let scalar_bigint: ScalarBigInt = pb_bigint.into();
        assert_eq!(scalar_bigint.to_string(), "-42");
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test roundtrip conversion: scalar::BigInt -> pb::eth::v2::BigInt -> scalar::BigInt
        let original = ScalarBigInt::from(12345);
        let pb_bigint: PbBigInt = original.clone().into();
        let roundtrip: ScalarBigInt = pb_bigint.into();
        
        assert_eq!(original.to_string(), roundtrip.to_string());
        
        // Test with a negative number
        let original = ScalarBigInt::from(-12345);
        let pb_bigint: PbBigInt = original.clone().into();
        let roundtrip: ScalarBigInt = pb_bigint.into();
        
        assert_eq!(original.to_string(), roundtrip.to_string());
    }
}


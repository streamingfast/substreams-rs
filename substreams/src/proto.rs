//! Protobuf helpers for Substreams.
//!
//! This crate offers a few protobuf helper functions which
//! are used across Substreams
//!

use core::slice;

use prost::{DecodeError, EncodeError};

/// Given an array of bytes, it will decode data in a Protobuf Message
pub fn decode<T: Default + prost::Message>(buf: &Vec<u8>) -> Result<T, DecodeError> {
    ::prost::Message::decode(&buf[..])
}

/// Given a pointer to a byte array, it will read and decode the data in a Protobuf message.
pub unsafe fn decode_ptr<T: Default + prost::Message>(
    ptr: *mut u8,
    size: usize,
) -> Result<T, DecodeError> {
    ::prost::Message::decode(slice::from_raw_parts(ptr, size))
}

/// Given a Protobuf message it will encode it and return the byte array.
pub fn encode<M: prost::Message>(msg: &M) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::new();

    let encoded_len = msg.encoded_len();
    buf.reserve(encoded_len);

    match msg.encode(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e),
    }
}

/// Given a Protobuf message it will encode it and return a pointer to the byte array
pub fn encode_to_ptr<M: prost::Message>(
    msg: &M,
) -> Result<(*const u8, usize, Vec<u8>), EncodeError> {
    match encode(msg) {
        Ok(buffer) => Ok((buffer.as_ptr(), buffer.len(), buffer)),
        Err(e) => Err(e),
    }
}

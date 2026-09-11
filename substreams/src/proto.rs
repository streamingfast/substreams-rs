//! Protobuf helpers for Substreams.
//!
//! This crate offers a few protobuf helper functions which
//! are used across Substreams
//!

use buffa::{DecodeError, Message};

/// Given an array of bytes, it will decode data in a Protobuf Message
pub fn decode<T: Default + Message>(buf: &Vec<u8>) -> Result<T, DecodeError> {
    T::decode_from_slice(&buf[..])
}

/// Given a pointer to a byte array, it will read and decode the data in a Protobuf message.
pub fn decode_ptr<T: Default + Message>(ptr: *mut u8, size: usize) -> Result<T, DecodeError> {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) };
    T::decode_from_slice(bytes)
}

/// Given a Protobuf message it will encode it and return the byte array.
pub fn encode<M: Message>(msg: &M) -> Vec<u8> {
    msg.encode_to_vec()
}

/// Given a Protobuf message it will encode it and return a pointer to the byte array
pub fn encode_to_ptr<M: Message>(msg: &M) -> (*const u8, usize, Vec<u8>) {
    let buffer = encode(msg);
    (buffer.as_ptr(), buffer.len(), buffer)
}

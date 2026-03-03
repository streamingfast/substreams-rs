//! Functional tests for substreams-macro.
//!
//! This crate contains integration tests that verify the substreams macros
//! generate correct, working code. Unlike unit tests that just compare AST,
//! these tests actually run the generated handlers with real data.
//!
//! # Test Architecture
//!
//! Tests are organized by feature/functionality:
//! - `quick_protobuf/` - Tests for quick-protobuf support
//! - `prost/` - Tests for prost support (default)
//!
//! Each test module uses the `test_capture` feature from the substreams crate
//! to capture output bytes, which can then be verified.

pub mod pb;

/// Test utilities for running handler functions with raw pointer inputs.
pub mod harness {
    use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Writer};

    /// Encode a quick-protobuf message to bytes.
    pub fn encode_quick<M: MessageWrite>(msg: &M) -> Vec<u8> {
        let size = msg.get_size();
        let mut buffer = Vec::with_capacity(size);
        let mut writer = Writer::new(&mut buffer);
        msg.write_message(&mut writer)
            .expect("Failed to encode message");
        buffer
    }

    /// Decode a quick-protobuf message from bytes.
    pub fn decode_quick<'a, T: MessageRead<'a>>(bytes: &'a [u8]) -> T {
        let mut reader = BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes).expect("Failed to decode message")
    }

    /// Call an extern "C" handler with a single protobuf input.
    ///
    /// This function:
    /// 1. Encodes the input message to bytes
    /// 2. Creates raw pointers
    /// 3. Calls the handler
    /// 4. Returns the captured output bytes (if any)
    ///
    /// # Safety
    /// This function deals with raw pointers but is safe because:
    /// - The input data lives for the duration of the handler call
    /// - The handler is expected to copy or process the data, not store pointers
    pub fn call_handler_1<M: MessageWrite>(
        handler: extern "C" fn(*mut u8, usize),
        input: &M,
    ) -> Option<Vec<u8>> {
        // Clear any previous output
        substreams::testing::clear();

        // Encode input
        let mut input_bytes = encode_quick(input);
        let ptr = input_bytes.as_mut_ptr();
        let len = input_bytes.len();

        // Call handler
        handler(ptr, len);

        // Return captured output
        substreams::testing::take_output()
    }

    /// Call an extern "C" handler with two protobuf inputs.
    pub fn call_handler_2<M1: MessageWrite, M2: MessageWrite>(
        handler: extern "C" fn(*mut u8, usize, *mut u8, usize),
        input1: &M1,
        input2: &M2,
    ) -> Option<Vec<u8>> {
        // Clear any previous output
        substreams::testing::clear();

        // Encode inputs
        let mut input1_bytes = encode_quick(input1);
        let ptr1 = input1_bytes.as_mut_ptr();
        let len1 = input1_bytes.len();

        let mut input2_bytes = encode_quick(input2);
        let ptr2 = input2_bytes.as_mut_ptr();
        let len2 = input2_bytes.len();

        // Call handler
        handler(ptr1, len1, ptr2, len2);

        // Return captured output
        substreams::testing::take_output()
    }
}

#[cfg(test)]
mod quick_protobuf_tests {
    use super::harness::{call_handler_1, decode_quick};
    use super::pb::{TestInput, TestOutput};

    // Define a handler using the quick_protobuf macro option
    #[substreams::handlers::map(quick_protobuf)]
    fn map_test_handler(input: TestInput) -> TestOutput {
        TestOutput {
            result: format!("processed: {}", input.name),
            count: input.value as u32 * 2,
        }
    }

    #[test]
    fn test_quick_protobuf_handler_basic() {
        // Create test input
        let input = TestInput {
            name: "hello".to_string(),
            value: 21,
        };

        // Call the generated extern "C" handler
        let output_bytes =
            call_handler_1(map_test_handler, &input).expect("Handler should produce output");

        // Decode and verify output
        let output: TestOutput = decode_quick(&output_bytes);
        assert_eq!(output.result, "processed: hello");
        assert_eq!(output.count, 42);
    }

    #[test]
    fn test_quick_protobuf_handler_empty_input() {
        // Test with default/empty input
        let input = TestInput::default();

        let output_bytes =
            call_handler_1(map_test_handler, &input).expect("Handler should produce output");

        let output: TestOutput = decode_quick(&output_bytes);
        assert_eq!(output.result, "processed: ");
        assert_eq!(output.count, 0);
    }

    // Test the testable __impl_ function directly
    #[test]
    fn test_quick_protobuf_impl_function() {
        let input = TestInput {
            name: "direct".to_string(),
            value: 50,
        };

        // Call the testable function directly (no pointer handling)
        let output = __impl_map_test_handler(input);
        assert_eq!(output.result, "processed: direct");
        assert_eq!(output.count, 100);
    }
}

//! Testing utilities for Substreams handlers.
//!
//! **Experimental API**: This module is under active development. While we aim to
//! minimize breaking changes, we reserve the right to modify the API as needed.
//! Please provide feedback on the API design.
//!
//! # Overview
//!
//! This module provides utilities for testing Substreams handlers without the
//! WASM runtime:
//!
//! - [`map!`] - Macro for calling testable handler functions
//! - [`clock`] - Helper function to create [`Clock`] instances for testing
//! - [`take_output`], [`clear`], [`has_output`] - Output capture utilities
//!   (available on non-wasm32 platforms)
//!
//! # Example
//!
//! ```ignore
//! use substreams::testing::{map, clock};
//!
//! #[substreams::handlers::map]
//! fn map_transfers(clock: Clock, blk: eth::Block) -> Result<pb::Transfers, Error> {
//!     // handler logic
//! }
//!
//! #[test]
//! fn test_handler() {
//!     let clock = clock("12345@1000");
//!     let blk = eth::Block::default();
//!     let result = map!(map_transfers(clock, blk));
//!     assert!(result.is_ok());
//! }
//! ```
//!
//! [`Clock`]: crate::pb::substreams::Clock

use crate::pb::substreams::Clock;
use std::cell::RefCell;

thread_local! {
    static CAPTURED_OUTPUT: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

/// Capture output bytes. Called by output functions on non-wasm32 platforms.
///
/// This function is used internally by the output functions to store the
/// serialized protobuf bytes for later verification in tests.
pub fn capture_output(data: Vec<u8>) {
    CAPTURED_OUTPUT.with(|cell| {
        *cell.borrow_mut() = Some(data);
    });
}

/// Take the last captured output, clearing the storage. The output is captured when:
/// - A handler produces output (e.g. via `output` function on non-wasm32 platforms)
/// - Code is run on non-wasm32 platforms, which is the case for unit tests
///
/// Returns `None` if no output was captured since the last call to [`clear`]
/// or [`take_output`].
///
/// # Example
///
/// ```ignore
/// use substreams::testing::take_output;
///
/// // After calling a handler that produces output...
/// let output_bytes = take_output().expect("Handler should produce output");
/// // Decode and verify output_bytes
/// ```
pub fn take_output() -> Option<Vec<u8>> {
    CAPTURED_OUTPUT.with(|cell| cell.borrow_mut().take())
}

/// Clear any captured output without returning it.
///
/// Call this before running a handler to ensure you're capturing fresh output.
pub fn clear() {
    CAPTURED_OUTPUT.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Check if output was captured.
///
/// Returns `true` if a handler has produced output since the last call to
/// [`clear`] or [`take_output`].
pub fn has_output() -> bool {
    CAPTURED_OUTPUT.with(|cell| cell.borrow().is_some())
}

// ============================================================================
// Test Helpers
// ============================================================================

/// Test helper macro that transforms a handler call to use the `__impl_` testable function.
///
/// By default, `#[substreams::handlers::map]` generates a testable `__impl_<name>` function
/// alongside the WASM export. This macro provides a convenient way to call the testable
/// function in tests.
///
/// # Example
///
/// ```ignore
/// use substreams::testing::map;
///
/// #[substreams::handlers::map]
/// fn map_transfers(blk: eth::Block) -> Result<Events, Error> {
///     // handler logic
/// }
///
/// #[test]
/// fn test_map_transfers() {
///     let blk = eth::Block::default();
///     let result = map!(map_transfers(blk));
///     assert!(result.is_ok());
/// }
/// ```
///
/// The macro transforms `map!(map_transfers(blk))` into `__impl_map_transfers(blk)`.
pub use substreams_macro::test_map as map;

/// Creates a [`Clock`] instance from a string representation for testing.
///
/// **Experimental API**: This function is under active development. While we aim to
/// minimize breaking changes, we reserve the right to modify the API as needed.
///
/// # Format
///
/// The input string follows these rules:
///
/// - **Block number**: If the string starts with digits, the leading numeric portion
///   is parsed as the block number. Examples: `"1"`, `"000001"`, `"10212"`
///
/// - **Block ID (hash)**: The full string (including leading digits) is used as the
///   block ID/hash.
///
/// - **Timestamp**: If the string contains `@<value>`, the value after `@` is parsed
///   as milliseconds since Unix epoch. If empty or not provided, the block number is
///   used as the timestamp (in seconds).
///
/// # Examples
///
/// ```
/// use substreams::testing::clock;
///
/// // Block number 12345, ID "12345", timestamp from block number (12345 seconds)
/// let c = clock("12345");
/// assert_eq!(c.number, 12345);
/// assert_eq!(c.id, "12345");
/// assert_eq!(c.timestamp.unwrap().seconds, 12345);
///
/// // Block number 100, ID "100abc", timestamp from block number
/// let c = clock("100abc");
/// assert_eq!(c.number, 100);
/// assert_eq!(c.id, "100abc");
///
/// // Block number 50, explicit timestamp of 1609459200000 ms (2021-01-01 00:00:00 UTC)
/// let c = clock("50@1609459200000");
/// assert_eq!(c.number, 50);
/// assert_eq!(c.id, "50");
/// assert_eq!(c.timestamp.unwrap().seconds, 1609459200);
/// assert_eq!(c.timestamp.unwrap().nanos, 0);
///
/// // Non-numeric ID, block number defaults to 0
/// let c = clock("genesis");
/// assert_eq!(c.number, 0);
/// assert_eq!(c.id, "genesis");
///
/// // Explicit timestamp with non-numeric start
/// let c = clock("blockhash@1609459200500");
/// assert_eq!(c.number, 0);
/// assert_eq!(c.id, "blockhash");
/// assert_eq!(c.timestamp.unwrap().seconds, 1609459200);
/// assert_eq!(c.timestamp.unwrap().nanos, 500_000_000);
/// ```
pub fn clock(input: impl AsRef<str>) -> Clock {
    let input = input.as_ref();

    // Split on '@' to separate the ID part from the timestamp part
    let (id_part, timestamp_str) = match input.find('@') {
        Some(pos) => (&input[..pos], Some(&input[pos + 1..])),
        None => (input, None),
    };

    // Parse block number from leading digits
    let leading_digits: String = id_part.chars().take_while(|c| c.is_ascii_digit()).collect();
    let number: u64 = if leading_digits.is_empty() {
        0
    } else {
        leading_digits.parse().unwrap_or(0)
    };

    // Use the id_part as the block ID (hash)
    let id = id_part.to_string();

    // Parse timestamp
    let timestamp = if let Some(ts_str) = timestamp_str {
        if ts_str.is_empty() {
            // Empty timestamp after @, use block number as seconds
            Some(prost_types::Timestamp {
                seconds: number as i64,
                nanos: 0,
            })
        } else {
            // Parse milliseconds since epoch
            match ts_str.parse::<i64>() {
                Ok(millis) => Some(prost_types::Timestamp {
                    seconds: millis / 1000,
                    nanos: ((millis % 1000) * 1_000_000) as i32,
                }),
                Err(_) => {
                    // Invalid timestamp, fall back to block number
                    Some(prost_types::Timestamp {
                        seconds: number as i64,
                        nanos: 0,
                    })
                }
            }
        }
    } else {
        // No @ in string, use block number as timestamp (seconds)
        Some(prost_types::Timestamp {
            seconds: number as i64,
            nanos: 0,
        })
    };

    Clock {
        id,
        number,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_simple_number() {
        let c = clock("12345");
        assert_eq!(c.number, 12345);
        assert_eq!(c.id, "12345");
        assert_eq!(c.timestamp.unwrap().seconds, 12345);
        assert_eq!(c.timestamp.unwrap().nanos, 0);
    }

    #[test]
    fn test_clock_padded_number() {
        let c = clock("000001");
        assert_eq!(c.number, 1);
        assert_eq!(c.id, "000001");
    }

    #[test]
    fn test_clock_number_with_suffix() {
        let c = clock("100abc");
        assert_eq!(c.number, 100);
        assert_eq!(c.id, "100abc");
    }

    #[test]
    fn test_clock_non_numeric() {
        let c = clock("genesis");
        assert_eq!(c.number, 0);
        assert_eq!(c.id, "genesis");
    }

    #[test]
    fn test_clock_with_explicit_timestamp() {
        let c = clock("50@1609459200000");
        assert_eq!(c.number, 50);
        assert_eq!(c.id, "50");
        assert_eq!(c.timestamp.unwrap().seconds, 1609459200);
        assert_eq!(c.timestamp.unwrap().nanos, 0);
    }

    #[test]
    fn test_clock_with_timestamp_millis() {
        let c = clock("1@1609459200500");
        assert_eq!(c.number, 1);
        assert_eq!(c.timestamp.unwrap().seconds, 1609459200);
        assert_eq!(c.timestamp.unwrap().nanos, 500_000_000);
    }

    #[test]
    fn test_clock_with_empty_timestamp() {
        let c = clock("100@");
        assert_eq!(c.number, 100);
        assert_eq!(c.id, "100");
        assert_eq!(c.timestamp.unwrap().seconds, 100);
    }

    #[test]
    fn test_clock_non_numeric_with_timestamp() {
        let c = clock("blockhash@1609459200500");
        assert_eq!(c.number, 0);
        assert_eq!(c.id, "blockhash");
        assert_eq!(c.timestamp.unwrap().seconds, 1609459200);
        assert_eq!(c.timestamp.unwrap().nanos, 500_000_000);
    }
}

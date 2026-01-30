//! **Experimental** SQE (Substreams Query Expression) - High-performance boolean expression parser and matcher.
//!
//! > **Warning**: This module is experimental and will replace the current [`crate::expr_parser`]
//! > implementation in a future release. The API may change before stabilization.
//!
//! SQE provides a fast, zero-allocation way to match sets of keys against boolean expressions.
//! It is significantly faster than the current Pest-based implementation:
//!
//! - **Parsing: 5-9x faster** across all expression types
//! - **Matching: 3-14x faster** with zero allocations
//! - **Repeated matching: 9-14x faster** - the main use case sees the biggest wins
//!
//! # Features
//!
//! - **Zero-copy parsing**: The AST stores byte ranges into the original input, not owned strings.
//! - **Zero-allocation matching**: Once parsed, expressions can be matched without any allocations.
//! - **Simple grammar**: Supports AND (`&&`), OR (`||`), implicit AND (whitespace), and grouping `()`.
//! - **Quoted keys**: Keys with spaces can be quoted with single or double quotes.
//!
//! # Example
//!
//! ```rust
//! use substreams::sqe::ExprMatcher;
//!
//! // Parse an expression once
//! let matcher = ExprMatcher::parse("(a || b) && c").unwrap();
//!
//! // Match against different key sets (zero allocations)
//! assert!(matcher.matches_keys(&["a", "c"]));
//! assert!(matcher.matches_keys(&["b", "c"]));
//! assert!(!matcher.matches_keys(&["a"]));
//! ```
//!
//! # Grammar
//!
//! ```text
//! expression := or EOF
//! or         := and (OR and)*
//! and        := value ((WHITESPACE value) | (AND value))*
//! value      := KEY | QUOTED_KEY | '(' or ')'
//! ```
//!
//! Operator precedence (lowest to highest):
//! 1. `||` (OR)
//! 2. `&&` (AND) / implicit AND (whitespace)
//! 3. `()` (grouping)

use anyhow::{Context, Error};

mod ast;
mod error;
mod lexer;
mod parser;

use ast::Expression;

/// An experimental expression matcher that can be used to match keys from a given expression.
///
/// > **Warning**: This is an experimental API that will replace [`crate::expr_parser::ExprMatcher`]
/// > in a future release.
///
/// Create a new `ExprMatcher` using [`ExprMatcher::parse`] with the input expression.
/// You can then re-use the matcher to match multiple keys against the same expression,
/// re-using the expression's parsed state with zero allocations.
///
/// # Example
///
/// ```rust
/// use substreams::sqe::ExprMatcher;
///
/// let matcher = ExprMatcher::parse("(a || b) && c").unwrap();
///
/// // Match against different key sets (zero allocations)
/// assert!(matcher.matches_keys(&["a", "c"]));
/// assert!(matcher.matches_keys(&["b", "c"]));
/// assert!(!matcher.matches_keys(&["a"]));
/// ```
pub struct ExprMatcher<'a> {
    expr: Expression<'a>,
}

impl<'a> ExprMatcher<'a> {
    /// Creates a new expression matcher from the given input.
    ///
    /// Returns an error if the input cannot be parsed as a valid expression.
    pub fn parse(input: &'a str) -> Result<Self, Error> {
        Ok(ExprMatcher {
            expr: parser::parse(input).context("parsing expression")?,
        })
    }

    /// Matches the given keys against the expression.
    ///
    /// Returns `true` if the keys match the expression, `false` otherwise.
    ///
    /// This method performs zero allocations.
    #[inline]
    pub fn matches_keys<K: AsRef<str>>(&self, keys: &[K]) -> bool {
        self.expr.matches(keys)
    }
}

/// Creates a new expression matcher from the given input.
///
/// > **Warning**: This is an experimental API that will replace [`crate::expr_matcher`]
/// > in a future release.
///
/// # Panics
///
/// Panics if the input cannot be parsed as a valid expression.
/// Use [`ExprMatcher::new`] if you want to handle parse errors.
pub fn expr_matcher(input: &'_ str) -> ExprMatcher<'_> {
    ExprMatcher::parse(input).expect("creating expression matcher failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_KEYS: &[&str] = &[
        "test",
        "test1",
        "test2",
        "test3",
        "test4",
        "test5",
        "test 6",
        "test.7",
        "test:8",
        "test_9",
        "test*19z_|",
        "type:wasm-MarketUpdated",
    ];

    #[test]
    fn test_expr_matcher_api() {
        let matcher = expr_matcher("test || test1");
        assert!(matcher.matches_keys(TEST_KEYS));
        assert!(matcher.matches_keys(&["test"]));
        assert!(!matcher.matches_keys(&["nonexistent"]));
    }

    #[test]
    fn test_expr_matcher_reuse() {
        let matcher = ExprMatcher::parse("(a || b) && c").unwrap();

        // Test multiple different key sets
        assert!(matcher.matches_keys(&["a", "c"]));
        assert!(matcher.matches_keys(&["b", "c"]));
        assert!(!matcher.matches_keys(&["a"]));
        assert!(!matcher.matches_keys(&["b"]));
        assert!(!matcher.matches_keys(&["c"]));
        assert!(matcher.matches_keys(&["a", "b", "c"]));
    }
}

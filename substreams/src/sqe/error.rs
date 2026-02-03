//! Error types for the SQE parser.

use std::fmt;

/// Position in the input string where an error occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Byte offset where the error starts.
    pub start: usize,
    /// Byte offset where the error ends (exclusive).
    pub end: usize,
}

impl Span {
    #[inline]
    #[allow(dead_code)]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn at(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos + 1,
        }
    }
}

/// Error type for parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The kind of error that occurred.
    pub kind: ParseErrorKind,
    /// The position in the input where the error occurred.
    pub span: Span,
}

/// The kind of parse error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// Unexpected character encountered.
    UnexpectedChar(char),
    /// Unexpected end of input.
    UnexpectedEof,
    /// Key cannot start with a dash.
    KeyStartsWithDash,
    /// Unclosed quoted string.
    UnclosedQuote(char),
    /// Unclosed parenthesis.
    UnclosedParen,
    /// Empty expression.
    EmptyExpression,
    /// Expected a value (key, quoted key, or parenthesized expression).
    ExpectedValue,
    /// Unmatched closing parenthesis.
    UnmatchedCloseParen,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::UnexpectedChar(c) => {
                write!(
                    f,
                    "unexpected character '{}' at position {}",
                    c, self.span.start
                )
            }
            ParseErrorKind::UnexpectedEof => {
                write!(f, "unexpected end of input at position {}", self.span.start)
            }
            ParseErrorKind::KeyStartsWithDash => {
                write!(
                    f,
                    "key cannot start with '-' at position {}",
                    self.span.start
                )
            }
            ParseErrorKind::UnclosedQuote(q) => {
                write!(
                    f,
                    "unclosed {} quote starting at position {}",
                    q, self.span.start
                )
            }
            ParseErrorKind::UnclosedParen => {
                write!(f, "unclosed parenthesis at position {}", self.span.start)
            }
            ParseErrorKind::EmptyExpression => {
                write!(f, "empty expression")
            }
            ParseErrorKind::ExpectedValue => {
                write!(f, "expected value at position {}", self.span.start)
            }
            ParseErrorKind::UnmatchedCloseParen => {
                write!(
                    f,
                    "unmatched closing parenthesis at position {}",
                    self.span.start
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn unexpected_char(c: char, pos: usize) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedChar(c),
            span: Span::at(pos),
        }
    }

    pub fn unexpected_eof(pos: usize) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedEof,
            span: Span::at(pos),
        }
    }

    pub fn key_starts_with_dash(pos: usize) -> Self {
        Self {
            kind: ParseErrorKind::KeyStartsWithDash,
            span: Span::at(pos),
        }
    }

    pub fn unclosed_quote(quote: char, start: usize) -> Self {
        Self {
            kind: ParseErrorKind::UnclosedQuote(quote),
            span: Span::at(start),
        }
    }

    pub fn unclosed_paren(start: usize) -> Self {
        Self {
            kind: ParseErrorKind::UnclosedParen,
            span: Span::at(start),
        }
    }

    pub fn empty_expression() -> Self {
        Self {
            kind: ParseErrorKind::EmptyExpression,
            span: Span::at(0),
        }
    }

    pub fn expected_value(pos: usize) -> Self {
        Self {
            kind: ParseErrorKind::ExpectedValue,
            span: Span::at(pos),
        }
    }

    pub fn unmatched_close_paren(pos: usize) -> Self {
        Self {
            kind: ParseErrorKind::UnmatchedCloseParen,
            span: Span::at(pos),
        }
    }
}

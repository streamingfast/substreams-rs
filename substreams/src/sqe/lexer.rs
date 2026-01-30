//! Zero-copy lexer for SQE expressions.
//!
//! The lexer tokenizes the input string without allocating any new strings.
//! All tokens contain byte ranges into the original input.

use super::error::{ParseError, Span};

/// A token produced by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    /// An unquoted key (e.g., "test", "type:wasm-Event").
    Key(&'a str),
    /// A quoted key with the quotes stripped (e.g., content of "'test 6'").
    QuotedKey(&'a str),
    /// Opening parenthesis.
    OpenParen,
    /// Closing parenthesis.
    CloseParen,
    /// Logical AND operator (&&).
    And,
    /// Logical OR operator (||).
    Or,
    /// End of input.
    Eof,
}

/// A lexer that tokenizes SQE expressions.
///
/// The lexer is zero-copy: it only produces references into the original input.
pub struct Lexer<'a> {
    /// The full input string.
    input: &'a str,
    /// Current byte position in the input.
    pos: usize,
    /// Whether we just consumed whitespace (for implicit AND detection).
    had_whitespace: bool,
    /// Whitespace state before the last peeked token.
    peeked_had_whitespace: Option<bool>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input.
    #[inline]
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            had_whitespace: false,
            peeked_had_whitespace: None,
        }
    }

    /// Returns the current byte position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns whether whitespace was consumed before the current position.
    #[inline]
    #[allow(dead_code)] // Used in tests
    pub fn had_whitespace(&self) -> bool {
        self.had_whitespace
    }

    /// Peeks at the current character without advancing.
    #[inline]
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Peeks at the next character (after current).
    #[inline]
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next();
        chars.next()
    }

    /// Advances by one character and returns it.
    #[inline]
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Skips whitespace and returns whether any was found.
    fn skip_whitespace(&mut self) -> bool {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' {
                self.advance();
            } else {
                break;
            }
        }
        self.pos > start
    }

    /// Returns the next token.
    pub fn next_token(&mut self) -> Result<Token<'a>, ParseError> {
        // Track if we had whitespace
        self.had_whitespace = self.skip_whitespace();

        let Some(c) = self.peek() else {
            return Ok(Token::Eof);
        };

        match c {
            '(' => {
                self.advance();
                Ok(Token::OpenParen)
            }
            ')' => {
                self.advance();
                Ok(Token::CloseParen)
            }
            '|' => {
                if self.peek_next() == Some('|') {
                    self.advance();
                    self.advance();
                    Ok(Token::Or)
                } else {
                    // Single '|' is part of a key
                    self.scan_key()
                }
            }
            '&' => {
                if self.peek_next() == Some('&') {
                    self.advance();
                    self.advance();
                    Ok(Token::And)
                } else {
                    // Single '&' is part of a key
                    self.scan_key()
                }
            }
            '\'' => self.scan_quoted_key('\''),
            '"' => self.scan_quoted_key('"'),
            '-' => Err(ParseError::key_starts_with_dash(self.pos)),
            _ => self.scan_key(),
        }
    }

    /// Scans an unquoted key.
    fn scan_key(&mut self) -> Result<Token<'a>, ParseError> {
        let start = self.pos;

        // Check if key starts with dash
        if self.peek() == Some('-') {
            return Err(ParseError::key_starts_with_dash(self.pos));
        }

        while let Some(c) = self.peek() {
            if self.is_key_char(c) {
                self.advance();
            } else {
                break;
            }
        }

        let end = self.pos;
        if end == start {
            return Err(ParseError::expected_value(self.pos));
        }

        Ok(Token::Key(&self.input[start..end]))
    }

    /// Checks if a character can be part of an unquoted key.
    #[inline]
    fn is_key_char(&self, c: char) -> bool {
        // Cannot be whitespace, quotes, parens, or operator characters
        !matches!(c, ' ' | '\t' | '\n' | '\'' | '"' | '(' | ')') && !self.is_operator_start(c)
    }

    /// Checks if the character at current position starts an operator.
    #[inline]
    fn is_operator_start(&self, c: char) -> bool {
        match c {
            '|' => self.peek_next() == Some('|'),
            '&' => self.peek_next() == Some('&'),
            _ => false,
        }
    }

    /// Scans a quoted key (single or double quotes).
    fn scan_quoted_key(&mut self, quote: char) -> Result<Token<'a>, ParseError> {
        let quote_start = self.pos;
        self.advance(); // consume opening quote

        let content_start = self.pos;

        // Check if content starts with dash
        if self.peek() == Some('-') {
            return Err(ParseError::key_starts_with_dash(self.pos));
        }

        // Scan until closing quote
        while let Some(c) = self.peek() {
            if c == quote {
                let content_end = self.pos;
                self.advance(); // consume closing quote

                if content_end == content_start {
                    // Empty quoted string - check next char for dash handling
                    return Ok(Token::QuotedKey(&self.input[content_start..content_end]));
                }

                return Ok(Token::QuotedKey(&self.input[content_start..content_end]));
            }
            self.advance();
        }

        // Reached end without closing quote
        Err(ParseError::unclosed_quote(quote, quote_start))
    }

    /// Peek at the next token without consuming it.
    /// Note: This does NOT preserve had_whitespace state - use had_whitespace_before_peek() after peeking
    /// to check if there was whitespace before the peeked token.
    pub fn peek_token(&mut self) -> Result<Token<'a>, ParseError> {
        let saved_pos = self.pos;
        let saved_ws = self.had_whitespace;
        let token = self.next_token()?;
        // Save whether there was whitespace before the peeked token
        let ws_before_token = self.had_whitespace;
        self.pos = saved_pos;
        self.had_whitespace = saved_ws;
        // Store for retrieval
        self.peeked_had_whitespace = Some(ws_before_token);
        Ok(token)
    }

    /// Returns whether whitespace was present before the last peeked token.
    pub fn had_whitespace_before_peek(&self) -> bool {
        self.peeked_had_whitespace.unwrap_or(false)
    }
}

/// Represents a token with its position in the source.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Reserved for future use
pub struct SpannedToken<'a> {
    pub token: Token<'a>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::error::ParseErrorKind;

    fn tokens(input: &str) -> Result<Vec<Token>, ParseError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token()?;
            if matches!(token, Token::Eof) {
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    #[test]
    fn test_simple_key() {
        assert_eq!(tokens("test").unwrap(), vec![Token::Key("test")]);
    }

    #[test]
    fn test_multiple_keys() {
        assert_eq!(
            tokens("test1 test2").unwrap(),
            vec![Token::Key("test1"), Token::Key("test2")]
        );
    }

    #[test]
    fn test_or_operator() {
        assert_eq!(
            tokens("a || b").unwrap(),
            vec![Token::Key("a"), Token::Or, Token::Key("b")]
        );
    }

    #[test]
    fn test_and_operator() {
        assert_eq!(
            tokens("a && b").unwrap(),
            vec![Token::Key("a"), Token::And, Token::Key("b")]
        );
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(
            tokens("(a || b)").unwrap(),
            vec![
                Token::OpenParen,
                Token::Key("a"),
                Token::Or,
                Token::Key("b"),
                Token::CloseParen
            ]
        );
    }

    #[test]
    fn test_single_quoted() {
        assert_eq!(
            tokens("'test key'").unwrap(),
            vec![Token::QuotedKey("test key")]
        );
    }

    #[test]
    fn test_double_quoted() {
        assert_eq!(
            tokens("\"test key\"").unwrap(),
            vec![Token::QuotedKey("test key")]
        );
    }

    #[test]
    fn test_special_chars_in_key() {
        assert_eq!(
            tokens("type:wasm-MarketUpdated").unwrap(),
            vec![Token::Key("type:wasm-MarketUpdated")]
        );
    }

    #[test]
    fn test_key_with_dots() {
        assert_eq!(tokens("test.7").unwrap(), vec![Token::Key("test.7")]);
    }

    #[test]
    fn test_key_with_asterisk() {
        assert_eq!(
            tokens("test*19z_|").unwrap(),
            vec![Token::Key("test*19z_|")]
        );
    }

    #[test]
    fn test_dash_at_start_error() {
        let err = tokens("-test").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::KeyStartsWithDash));
    }

    #[test]
    fn test_quoted_dash_at_start_error() {
        let err = tokens("'-test'").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::KeyStartsWithDash));
    }

    #[test]
    fn test_unclosed_quote() {
        let err = tokens("'test").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnclosedQuote('\'')));
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(
            tokens("(test1 || test2) && test3").unwrap(),
            vec![
                Token::OpenParen,
                Token::Key("test1"),
                Token::Or,
                Token::Key("test2"),
                Token::CloseParen,
                Token::And,
                Token::Key("test3")
            ]
        );
    }

    #[test]
    fn test_whitespace_variations() {
        assert_eq!(
            tokens("a     ||     b").unwrap(),
            vec![Token::Key("a"), Token::Or, Token::Key("b")]
        );
    }

    #[test]
    fn test_no_space_around_operators() {
        assert_eq!(
            tokens("a||b").unwrap(),
            vec![Token::Key("a"), Token::Or, Token::Key("b")]
        );
        assert_eq!(
            tokens("a&&b").unwrap(),
            vec![Token::Key("a"), Token::And, Token::Key("b")]
        );
    }

    #[test]
    fn test_had_whitespace() {
        let mut lexer = Lexer::new("a b");

        let _ = lexer.next_token().unwrap(); // "a"
        assert!(!lexer.had_whitespace()); // no whitespace before first token

        let _ = lexer.next_token().unwrap(); // "b"
        assert!(lexer.had_whitespace()); // whitespace before "b"
    }
}

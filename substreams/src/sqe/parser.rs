//! Recursive descent parser for SQE expressions.
//!
//! Grammar (precedence from lowest to highest):
//! ```text
//! expression := or EOF
//! or         := and (OR and)*
//! and        := value ((WHITESPACE value) | (AND value))*
//! value      := KEY | QUOTED_KEY | '(' or ')'
//! ```

use super::ast::{Expression, Node};
use super::error::ParseError;
use super::lexer::{Lexer, Token};

/// Parser for SQE expressions.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    input: &'a str,
    /// Flat storage for all nodes - we build the tree bottom-up.
    nodes: Vec<Node>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            input,
            nodes: Vec::new(),
        }
    }

    /// Parses the input and returns an Expression.
    pub fn parse(mut self) -> Result<Expression<'a>, ParseError> {
        // Handle empty input
        let token = self.lexer.peek_token()?;
        if matches!(token, Token::Eof) {
            return Err(ParseError::empty_expression());
        }

        let root = self.parse_or()?;

        // Ensure we consumed all input
        let token = self.lexer.next_token()?;
        if !matches!(token, Token::Eof) {
            return Err(ParseError::unexpected_char('?', self.lexer.position()));
        }

        Ok(Expression::new(self.input, self.nodes, root))
    }

    /// Parses an OR expression: and (OR and)*
    fn parse_or(&mut self) -> Result<u32, ParseError> {
        let mut children = Vec::new();
        children.push(self.parse_and()?);

        loop {
            let token = self.lexer.peek_token()?;
            if matches!(token, Token::Or) {
                self.lexer.next_token()?; // consume ||
                children.push(self.parse_and()?);
            } else {
                break;
            }
        }

        // Optimize: if only one child, return it directly
        if children.len() == 1 {
            return Ok(children[0]);
        }

        // Store children contiguously and create OR node
        let children_start = self.nodes.len() as u32;
        for child_idx in &children {
            // Copy the child node to the contiguous section
            let child_node = self.nodes[*child_idx as usize];
            self.nodes.push(child_node);
        }

        let or_node = Node::Or {
            children_start,
            children_count: children.len() as u16,
        };
        let or_idx = self.nodes.len() as u32;
        self.nodes.push(or_node);

        Ok(or_idx)
    }

    /// Parses an AND expression: value ((WHITESPACE value) | (AND value))*
    fn parse_and(&mut self) -> Result<u32, ParseError> {
        let mut children = Vec::new();
        children.push(self.parse_value()?);

        loop {
            let token = self.lexer.peek_token()?;
            let had_ws = self.lexer.had_whitespace_before_peek();

            match token {
                Token::And => {
                    self.lexer.next_token()?; // consume &&
                    children.push(self.parse_value()?);
                }
                Token::Key(_) | Token::QuotedKey(_) | Token::OpenParen => {
                    // Implicit AND only if we had whitespace before
                    if had_ws {
                        children.push(self.parse_value()?);
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        // Optimize: if only one child, return it directly
        if children.len() == 1 {
            return Ok(children[0]);
        }

        // Store children contiguously and create AND node
        let children_start = self.nodes.len() as u32;
        for child_idx in &children {
            let child_node = self.nodes[*child_idx as usize];
            self.nodes.push(child_node);
        }

        let and_node = Node::And {
            children_start,
            children_count: children.len() as u16,
        };
        let and_idx = self.nodes.len() as u32;
        self.nodes.push(and_node);

        Ok(and_idx)
    }

    /// Parses a value: KEY | QUOTED_KEY | '(' or ')'
    fn parse_value(&mut self) -> Result<u32, ParseError> {
        let token = self.lexer.next_token()?;

        match token {
            Token::Key(key) => {
                let start = key.as_ptr() as usize - self.input.as_ptr() as usize;
                let end = start + key.len();
                let node = Node::Key {
                    start: start as u32,
                    end: end as u32,
                };
                let idx = self.nodes.len() as u32;
                self.nodes.push(node);
                Ok(idx)
            }
            Token::QuotedKey(key) => {
                let start = key.as_ptr() as usize - self.input.as_ptr() as usize;
                let end = start + key.len();
                let node = Node::QuotedKey {
                    start: start as u32,
                    end: end as u32,
                };
                let idx = self.nodes.len() as u32;
                self.nodes.push(node);
                Ok(idx)
            }
            Token::OpenParen => {
                let paren_pos = self.lexer.position() - 1;
                let inner = self.parse_or()?;

                let close = self.lexer.next_token()?;
                if !matches!(close, Token::CloseParen) {
                    return Err(ParseError::unclosed_paren(paren_pos));
                }

                Ok(inner)
            }
            Token::CloseParen => {
                Err(ParseError::unmatched_close_paren(self.lexer.position() - 1))
            }
            Token::Eof => Err(ParseError::unexpected_eof(self.lexer.position())),
            _ => Err(ParseError::expected_value(self.lexer.position())),
        }
    }
}

/// Parses an expression string into an Expression.
pub fn parse(input: &str) -> Result<Expression<'_>, ParseError> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(expr: &str, keys: &[&str]) -> bool {
        parse(expr).unwrap().matches(keys)
    }

    #[test]
    fn test_simple_key() {
        assert!(matches("test", &["test"]));
        assert!(!matches("test", &["other"]));
    }

    #[test]
    fn test_quoted_key() {
        assert!(matches("'test'", &["test"]));
        assert!(matches("\"test\"", &["test"]));
        assert!(matches("'test 6'", &["test 6"]));
    }

    #[test]
    fn test_or() {
        assert!(matches("a || b", &["a"]));
        assert!(matches("a || b", &["b"]));
        assert!(matches("a || b", &["a", "b"]));
        assert!(!matches("a || b", &["c"]));
    }

    #[test]
    fn test_and_explicit() {
        assert!(matches("a && b", &["a", "b"]));
        assert!(!matches("a && b", &["a"]));
        assert!(!matches("a && b", &["b"]));
    }

    #[test]
    fn test_and_implicit() {
        assert!(matches("a b", &["a", "b"]));
        assert!(!matches("a b", &["a"]));
        assert!(!matches("a b", &["b"]));
    }

    #[test]
    fn test_parentheses() {
        assert!(matches("(a || b) && c", &["a", "c"]));
        assert!(matches("(a || b) && c", &["b", "c"]));
        assert!(!matches("(a || b) && c", &["a"]));
        assert!(!matches("(a || b) && c", &["c"]));
    }

    #[test]
    fn test_nested_complex() {
        let expr = "(test1 || test6 || test7) && (test4 || test5) && test3";
        assert!(matches(expr, &["test1", "test4", "test3"]));
        assert!(matches(expr, &["test1", "test5", "test3"]));
        assert!(!matches(expr, &["test1", "test3"]));
    }

    #[test]
    fn test_special_chars() {
        assert!(matches("type:wasm-MarketUpdated", &["type:wasm-MarketUpdated"]));
        assert!(matches("test.7", &["test.7"]));
        assert!(matches("test:8", &["test:8"]));
        assert!(matches("test_9", &["test_9"]));
        assert!(matches("test*19z_|", &["test*19z_|"]));
    }

    #[test]
    fn test_dash_error() {
        assert!(parse("-test").is_err());
        assert!(parse("'-test'").is_err());
    }

    #[test]
    fn test_key_with_dash_in_middle() {
        assert!(matches("test-8", &["test-8"]));
        assert!(matches("'test-8'", &["test-8"]));
    }

    #[test]
    fn test_empty_expression() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn test_whitespace_variations() {
        assert!(matches("test1     test2", &["test1", "test2"]));
        assert!(matches("test1    && test2", &["test1", "test2"]));
        assert!(matches("test1    ||     test2", &["test1"]));
    }

    // Comprehensive test cases from the original Pest implementation
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
    fn test_comprehensive_cases() {
        // All test cases from the original Pest implementation
        assert!(matches("test", TEST_KEYS));
        assert!(matches("'test'", TEST_KEYS));
        assert!(matches("'test 6' || test7", TEST_KEYS));
        assert!(!matches("'test_6' && test3", TEST_KEYS));
        assert!(matches("\"test 6\"|| test7", TEST_KEYS));
        assert!(matches("\"test 6\" && test3", TEST_KEYS));
        assert!(matches("test.7", TEST_KEYS));
        assert!(matches("type:wasm-MarketUpdated", TEST_KEYS));
        assert!(!matches("type:was-mMarketUpdated", TEST_KEYS));
        assert!(!matches("test.8", TEST_KEYS));
        assert!(matches("test:8", TEST_KEYS));
        assert!(matches("test*19z_|", TEST_KEYS));
        assert!(!matches("test:9", TEST_KEYS));
        assert!(matches("test_9", TEST_KEYS));
        assert!(!matches("test_10", TEST_KEYS));
        assert!(matches("test10 ||test.7", TEST_KEYS));
        assert!(!matches("test10 && test:8", TEST_KEYS));
        assert!(matches("(test10 && test_9) || (test.7 && test:8)", TEST_KEYS));
        assert!(matches("(test10 && test_9) || (test.7 && test*19z_|)", TEST_KEYS));
        assert!(matches(
            "(test10 && test_9) || test*19z || (test.7 && test*19z_|)",
            TEST_KEYS
        ));
        assert!(!matches(
            "(test10 && test_9) || test*19z && (test.7 && test*19z_|)",
            TEST_KEYS
        ));
        assert!(matches("test1 || test", TEST_KEYS));
        assert!(matches("test1 || test6", TEST_KEYS));
        assert!(!matches("test6 || test7", TEST_KEYS));
        assert!(matches("test1 || test || test2", TEST_KEYS));
        assert!(matches("test1 || test6 || test7", TEST_KEYS));
        assert!(!matches("test6 || test7 || test8", TEST_KEYS));
        assert!(matches("test1 && test", TEST_KEYS));
        assert!(!matches("test1 && test6", TEST_KEYS));
        assert!(!matches("test6 && test7", TEST_KEYS));
        assert!(matches("test1 && test && test2", TEST_KEYS));
        assert!(!matches("test1&& test2 &&test7", TEST_KEYS));
        assert!(!matches("test6 &&test7 && test8", TEST_KEYS));
        assert!(matches("test1 test", TEST_KEYS));
        assert!(!matches("test1 test6", TEST_KEYS));
        assert!(!matches("test6 test7", TEST_KEYS));
        assert!(matches("(test1)", TEST_KEYS));
        assert!(!matches("(test1 test6)", TEST_KEYS));
        assert!(matches("test1     test2 ", TEST_KEYS));
        assert!(matches("test1    && test2 ", TEST_KEYS));
        assert!(!matches("test1    &&     test6", TEST_KEYS));
        assert!(!matches("(test1   ||  test3)       &&  test6 ", TEST_KEYS));
        assert!(matches(
            "(test1  ||     test6 || test7  )     && (test4 || test5) && test3 ",
            TEST_KEYS
        ));
        assert!(matches(
            "(test1 || test6 || test7) && (test4 || test5) && test3 ",
            TEST_KEYS
        ));
        assert!(matches(
            "(test1 && test6 && test7) || (test4 && test5) || test3 ",
            TEST_KEYS
        ));
    }
}

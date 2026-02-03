//! AST representation for SQE expressions.
//!
//! The AST is designed for:
//! - Zero-copy parsing: stores byte ranges into the original input
//! - Flat storage: uses indices instead of pointers for cache efficiency
//! - Zero-allocation matching: can be evaluated without any allocations

/// A parsed expression that can be matched against a set of keys.
///
/// The expression stores the original input string and a flat array of nodes
/// that represent the parsed AST. This design allows for zero-allocation
/// matching by using indices and byte ranges instead of owned strings.
#[derive(Debug)]
pub struct Expression<'a> {
    /// The original input string (for zero-copy key access).
    pub(crate) input: &'a str,
    /// Flat storage of all AST nodes.
    pub(crate) nodes: Vec<Node>,
    /// Index of the root node in the nodes array.
    pub(crate) root: u32,
}

/// A single node in the AST.
///
/// Uses a compact representation with indices to enable flat storage
/// and cache-friendly traversal.
#[derive(Debug, Clone, Copy)]
pub enum Node {
    /// A key (unquoted) - stores byte range (start, end) into input.
    Key {
        /// Start byte offset in input string.
        start: u32,
        /// End byte offset in input string (exclusive).
        end: u32,
    },
    /// A quoted key - stores byte range (start, end) into input (without quotes).
    QuotedKey {
        /// Start byte offset in input string (after opening quote).
        start: u32,
        /// End byte offset in input string (before closing quote).
        end: u32,
    },
    /// OR expression - matches if any child matches.
    /// Children are stored contiguously starting at `children_start`.
    Or {
        /// Start index of children in the nodes array.
        children_start: u32,
        /// Number of children.
        children_count: u16,
    },
    /// AND expression - matches if all children match.
    /// Children are stored contiguously starting at `children_start`.
    And {
        /// Start index of children in the nodes array.
        children_start: u32,
        /// Number of children.
        children_count: u16,
    },
}

impl<'a> Expression<'a> {
    /// Creates a new expression from parsed components.
    #[inline]
    pub(crate) fn new(input: &'a str, nodes: Vec<Node>, root: u32) -> Self {
        Self { input, nodes, root }
    }

    /// Returns the original input string.
    #[inline]
    #[allow(dead_code)]
    pub fn input(&self) -> &'a str {
        self.input
    }

    /// Returns the number of nodes in the AST.
    #[inline]
    #[allow(dead_code)]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Matches the expression against a set of keys.
    ///
    /// Returns `true` if the keys satisfy the expression.
    ///
    /// This method performs zero allocations - it only borrows the keys.
    #[inline]
    pub fn matches<K: AsRef<str>>(&self, keys: &[K]) -> bool {
        self.eval_node(self.root as usize, keys)
    }

    /// Recursively evaluates a node against the given keys.
    #[inline]
    fn eval_node<K: AsRef<str>>(&self, node_idx: usize, keys: &[K]) -> bool {
        match self.nodes[node_idx] {
            Node::Key { start, end } => {
                let key_str = &self.input[start as usize..end as usize];
                keys.iter().any(|k| k.as_ref() == key_str)
            }
            Node::QuotedKey { start, end } => {
                let key_str = &self.input[start as usize..end as usize];
                keys.iter().any(|k| k.as_ref() == key_str)
            }
            Node::Or {
                children_start,
                children_count,
            } => {
                let start = children_start as usize;
                let count = children_count as usize;
                (0..count).any(|i| self.eval_node(start + i, keys))
            }
            Node::And {
                children_start,
                children_count,
            } => {
                let start = children_start as usize;
                let count = children_count as usize;
                (0..count).all(|i| self.eval_node(start + i, keys))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_simple_key() {
        // Manually construct a simple expression: "test"
        let input = "test";
        let nodes = vec![Node::Key { start: 0, end: 4 }];
        let expr = Expression::new(input, nodes, 0);

        assert!(expr.matches(&["test"]));
        assert!(!expr.matches(&["other"]));
        assert!(expr.matches(&["test", "other"]));
    }

    #[test]
    fn test_expression_or() {
        // Manually construct: "a || b"
        let input = "a || b";
        let nodes = vec![
            Node::Key { start: 0, end: 1 }, // "a" at index 0
            Node::Key { start: 5, end: 6 }, // "b" at index 1
            Node::Or {
                children_start: 0,
                children_count: 2,
            }, // at index 2
        ];
        let expr = Expression::new(input, nodes, 2);

        assert!(expr.matches(&["a"]));
        assert!(expr.matches(&["b"]));
        assert!(expr.matches(&["a", "b"]));
        assert!(!expr.matches(&["c"]));
    }

    #[test]
    fn test_expression_and() {
        // Manually construct: "a && b"
        let input = "a && b";
        let nodes = vec![
            Node::Key { start: 0, end: 1 }, // "a" at index 0
            Node::Key { start: 5, end: 6 }, // "b" at index 1
            Node::And {
                children_start: 0,
                children_count: 2,
            }, // at index 2
        ];
        let expr = Expression::new(input, nodes, 2);

        assert!(!expr.matches(&["a"]));
        assert!(!expr.matches(&["b"]));
        assert!(expr.matches(&["a", "b"]));
        assert!(!expr.matches(&["c"]));
    }

    #[test]
    fn test_expression_quoted_key() {
        // Manually construct: "'test key'"
        let input = "'test key'";
        let nodes = vec![Node::QuotedKey { start: 1, end: 9 }];
        let expr = Expression::new(input, nodes, 0);

        assert!(expr.matches(&["test key"]));
        assert!(!expr.matches(&["test"]));
        assert!(!expr.matches(&["'test key'"]));
    }
}

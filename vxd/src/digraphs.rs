//! Digraph system.
//!
//! Digraphs are two-character sequences that insert a single character.

use std::collections::HashMap;

/// Digraph table with built-in defaults plus user-defined entries.
#[derive(Debug, Clone)]
pub struct DigraphTable {
    map: HashMap<(char, char), char>,
}

impl DigraphTable {
    /// Create a new digraph table with a small built-in set.
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(('a', 'e'), '\u{00E6}'); // ae -> æ
        map.insert(('A', 'E'), '\u{00C6}'); // AE -> Æ
        map.insert(('o', 's'), '\u{00F8}'); // os -> ø
        map.insert(('O', 'S'), '\u{00D8}'); // OS -> Ø
        DigraphTable { map }
    }

    /// Look up a digraph by its two-character sequence.
    pub fn lookup(&self, first: char, second: char) -> Option<char> {
        self.map.get(&(first, second)).copied()
    }

    /// Add or override a digraph mapping.
    pub fn insert(&mut self, first: char, second: char, output: char) {
        self.map.insert((first, second), output);
    }
}

impl Default for DigraphTable {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digraph_defaults() {
        let table = DigraphTable::new();
        assert_eq!(table.lookup('a', 'e'), Some('\u{00E6}'));
        assert_eq!(table.lookup('O', 'S'), Some('\u{00D8}'));
        assert_eq!(table.lookup('x', 'y'), None);
    }

    #[test]
    fn test_digraph_insert_override() {
        let mut table = DigraphTable::new();
        table.insert('x', 'y', '\u{2192}');
        assert_eq!(table.lookup('x', 'y'), Some('\u{2192}'));
    }
}

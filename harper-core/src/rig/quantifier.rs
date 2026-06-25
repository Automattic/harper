use super::{MatchResult, RegexNode};

/// Quantifier modes for repetition.
#[derive(Clone, Copy, Debug)]
pub enum QuantifierMode {
    /// Greedy: match as much as possible (default, like `*`, `+` in regex)
    Greedy,
    /// Non-greedy: match as little as possible (like `*?`, `+?` in regex)
    NonGreedy,
}

/// Repeats a pattern a specified number of times.
///
/// This is analogous to regex quantifiers like `*`, `+`, `?`, `{n,m}`.
pub struct Quantifier {
    /// The pattern to repeat
    inner: Box<dyn RegexNode>,
    /// Minimum number of repetitions
    min: usize,
    /// Maximum number of repetitions (None for unbounded)
    max: Option<usize>,
    /// Whether to match greedily or non-greedily
    mode: QuantifierMode,
}

impl Quantifier {
    /// Create a new quantifier.
    pub fn new(
        inner: Box<dyn RegexNode>,
        min: usize,
        max: Option<usize>,
        mode: QuantifierMode,
    ) -> Self {
        Self {
            inner,
            min,
            max,
            mode,
        }
    }

    /// Zero or more repetitions (greedy, like `*`)
    pub fn zero_or_more(inner: Box<dyn RegexNode>) -> Self {
        Self::new(inner, 0, None, QuantifierMode::Greedy)
    }

    /// Zero or more repetitions (non-greedy, like `*?`)
    pub fn zero_or_more_non_greedy(inner: Box<dyn RegexNode>) -> Self {
        Self::new(inner, 0, None, QuantifierMode::NonGreedy)
    }

    /// One or more repetitions (greedy, like `+`)
    pub fn one_or_more(inner: Box<dyn RegexNode>) -> Self {
        Self::new(inner, 1, None, QuantifierMode::Greedy)
    }

    /// One or more repetitions (non-greedy, like `+?`)
    pub fn one_or_more_non_greedy(inner: Box<dyn RegexNode>) -> Self {
        Self::new(inner, 1, None, QuantifierMode::NonGreedy)
    }

    /// Zero or one repetition (like `?`)
    pub fn optional(inner: Box<dyn RegexNode>) -> Self {
        Self::new(inner, 0, Some(1), QuantifierMode::Greedy)
    }

    /// Exactly n repetitions (like `{n}`)
    pub fn exactly(inner: Box<dyn RegexNode>, n: usize) -> Self {
        Self::new(inner, n, Some(n), QuantifierMode::Greedy)
    }

    /// Between n and m repetitions (like `{n,m}`)
    pub fn between(inner: Box<dyn RegexNode>, min: usize, max: usize) -> Self {
        Self::new(inner, min, Some(max), QuantifierMode::Greedy)
    }
}

impl RegexNode for Quantifier {
    fn exec(
        &self,
        tokens: &[crate::Token],
        source: &[char],
        start_idx: usize,
    ) -> Option<MatchResult> {
        match self.mode {
            QuantifierMode::Greedy => self.exec_greedy(tokens, source, start_idx),
            QuantifierMode::NonGreedy => self.exec_non_greedy(tokens, source, start_idx),
        }
    }
}

impl Quantifier {
    fn exec_greedy(
        &self,
        tokens: &[crate::Token],
        source: &[char],
        start_idx: usize,
    ) -> Option<MatchResult> {
        let mut current_idx = start_idx;
        let mut aggregated_captures = hashbrown::HashMap::new();
        let mut count = 0;

        // Match as much as possible
        loop {
            // Check if we've hit the maximum
            if let Some(max) = self.max
                && count >= max
            {
                break;
            }

            // Try to match one more
            if let Some(result) = self.inner.exec(tokens, source, current_idx) {
                aggregated_captures.extend(result.captures);
                current_idx += result.tokens_consumed;
                count += 1;
            } else {
                break;
            }
        }

        // Check if we met the minimum
        if count < self.min {
            return None;
        }

        Some(MatchResult {
            captures: aggregated_captures,
            tokens_consumed: current_idx - start_idx,
        })
    }

    fn exec_non_greedy(
        &self,
        tokens: &[crate::Token],
        source: &[char],
        start_idx: usize,
    ) -> Option<MatchResult> {
        let mut current_idx = start_idx;
        let mut aggregated_captures = hashbrown::HashMap::new();
        let mut count = 0;

        // Match the minimum first
        while count < self.min {
            if let Some(result) = self.inner.exec(tokens, source, current_idx) {
                aggregated_captures.extend(result.captures);
                current_idx += result.tokens_consumed;
                count += 1;
            } else {
                return None; // Failed to meet minimum
            }
        }

        // Try to match more, but stop as soon as possible
        // For non-greedy, we only match more if it allows the overall pattern to succeed
        // This is a simplified version - a full implementation would need backtracking
        // when used in larger patterns
        if let Some(max) = self.max {
            while count < max {
                if let Some(result) = self.inner.exec(tokens, source, current_idx) {
                    aggregated_captures.extend(result.captures);
                    current_idx += result.tokens_consumed;
                    count += 1;
                } else {
                    break;
                }
            }
        }

        Some(MatchResult {
            captures: aggregated_captures,
            tokens_consumed: current_idx - start_idx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rig::Atom;

    #[test]
    fn test_zero_or_more() {
        let doc = Document::new_plain_english_curated("hello hello hello");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::zero_or_more(Box::new(Atom::word("hello")));
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_some());
        // Only matches first "hello" since next token is whitespace
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_one_or_more() {
        let doc = Document::new_plain_english_curated("hello hello hello");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::one_or_more(Box::new(Atom::word("hello")));
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_some());
        // Only matches first "hello" since next token is whitespace
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_one_or_more_fail() {
        let doc = Document::new_plain_english_curated("world world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::one_or_more(Box::new(Atom::word("hello")));
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_optional() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::optional(Box::new(Atom::word("hello")));
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_some());
        // Should match at least one (greedy)
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_optional_no_match() {
        let doc = Document::new_plain_english_curated("world world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::optional(Box::new(Atom::word("hello")));
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_some());
        // Should match zero (optional)
        assert_eq!(result.unwrap().tokens_consumed, 0);
    }

    #[test]
    fn test_exactly() {
        let doc = Document::new_plain_english_curated("hello");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::exactly(Box::new(Atom::word("hello")), 1);
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_exactly_fail() {
        let doc = Document::new_plain_english_curated("hello");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::exactly(Box::new(Atom::word("hello")), 2);
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_between() {
        let doc = Document::new_plain_english_curated("hello");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let quant = Quantifier::between(Box::new(Atom::word("hello")), 1, 3);
        let result = quant.exec(tokens, source, 0);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }
}

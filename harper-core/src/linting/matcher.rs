use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{CharString, Document, Punctuation, Span, Token, TokenKind, WordMetadata};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct PatternToken {
    /// The general variant of the token.
    /// The inner data of the [`TokenKind`] should be replaced with the default
    /// value.
    kind: TokenKind,
    content: Option<CharString>,
}

impl PatternToken {
    fn from_token(token: Token, document: &Document) -> Self {
        if token.kind.is_word() {
            Self {
                kind: token.kind.with_default_data(),
                content: Some(document.get_span_content(token.span).into()),
            }
        } else {
            Self {
                kind: token.kind,
                content: None,
            }
        }
    }
}

macro_rules! vecword {
    ($lit:literal) => {
        $lit.chars().collect()
    };
}

macro_rules! pt {
    ($str:literal) => {
        PatternToken {
            kind: TokenKind::Word(WordMetadata::default()),
            content: Some($str.chars().collect()),
        }
    };
    (Word) => {
        PatternToken {
            kind: TokenKind::Word,
            content: None,
        }
    };
    (Period) => {
        PatternToken {
            kind: TokenKind::Punctuation(Punctuation::Period),
            content: None,
        }
    };
    (Hyphen) => {
        PatternToken {
            kind: TokenKind::Punctuation(Punctuation::Hyphen),
            content: None,
        }
    };
    (Space) => {
        PatternToken {
            kind: TokenKind::Space(1),
            content: None,
        }
    };
    ( $($($str:literal),* => $repl:literal),*) => {
        vec![
            $(
                {
                    let mut rule = Rule {
                        pattern: vec![$(
                            pt!($str),
                            pt!(Space),
                        )*],
                        replace_with: $repl.chars().collect()
                    };

                    if rule.pattern.len() > 0{
                        rule.pattern.pop();
                    }

                    rule
                },
            )*
        ]
    };
}

struct Rule {
    pattern: Vec<PatternToken>,
    replace_with: Vec<char>,
}

/// A linter that uses a variety of curated pattern matches to find and fix
/// common grammatical issues.
pub struct Matcher {
    triggers: Vec<Rule>,
}

impl Matcher {
    pub fn new() -> Self {
        // This match list needs to be automatically expanded instead of explicitly
        // defined like it is now.
        let mut triggers = pt! {
            "spacial","attention" => "special attention",
            "wellbeing" => "well-being",
            "hashtable" => "hash table",
            "hashmap" => "hash map",
            "dep" => "dependency",
            "deps" => "dependencies",
            "off","the","cuff" => "off-the-cuff",
            "an","in" => "and in",
            "my","self" => "myself",
            "human","live" => "human life",
            "eight","grade" => "eighth grade",
            "and","also" => "and",
            "todo" => "to-do",
            "To-Do" => "To-do",
            "performing","this" => "perform this",
            "mins" => "minutes",
            "min" => "minute",
            "min" => "minimum",
            "secs" => "seconds",
            "sec" => "second",
            "hrs" => "hours",
            "hr" => "hour",
            "w/o" => "without",
            "w/" => "with",
            "wordlist" => "word list" ,
            "the","challenged" => "that challenged",
            "stdin" => "standard input",
            "stdout" => "standard output",
            "no","to" => "not to",
            "No","to" => "not to",
            "ngram" => "n-gram",
            "grammer" => "grammar",
            "There","fore" => "Therefore",
            "fatal","outcome" => "death",
            "geiger","counter" => "Geiger counter",
            "world","war","2" => "World War II",
            "World","war","ii" => "World War II",
            "world","War","ii" => "World War II",
            "World","War","Ii" => "World War II",
            "World","War","iI" => "World War II",
            "black","sea" => "Black Sea",
            "I","a","m" => "I am",
            "We","a","re" => "We are",
            "The","re" => "There",
            "my","french" => "my French",
            "It","cam" => "It can",
            "can","be","seem" => "can be seen",
            "mu","house" => "my house",
            "kid","regards" => "kind regards",
            "miss","understand" => "misunderstand",
            "miss","use" => "misuse",
            "miss","used" => "misused",
            "bee","there" => "been there",
            "want","be" => "won't be",
            "more","then" => "more than",
            "gong","to" => "going to",
            "then","others" => "than others",
            "Then","others" => "than others",
            "then","before" => "than before",
            "Then","before" => "than before",
            "then","last","week" => "than last week",
            "then","her" => "than her",
            "then","hers" => "than hers",
            "then","him" => "than him",
            "then","his" => "than his",
            "simply","grammatical" => "simple grammatical",
            "you","r" => "your",
            "you","re" => "you're",
            "that","s" => "that's",
            "That","s" => "That's",
            "that","s" => "that is",
            "That","s" => "that is",
            "ms" => "milliseconds",
            "the","hing" => "the thing",
            "The","hing" => "The thing",
            "need","helps" => "need help",
            "an","this" => "and this",
            "case", "sensitive" => "case-sensitive",
            "Tree", "sitter" => "Tree-sitter",
            "all", "of", "the" => "all the",
            "not", "longer" => "no longer",
            "to", "towards" => "towards",
            "though", "process" => "thought process",
            "the", "this" => "that this",
            "take", "a", "decision" => "make a decision",
            "same", "than" => "same as",
            "Same", "than" => "same as",
            "same", "then" => "same as",
            "Same", "then" => "same as"
        };

        triggers.push(Rule {
            pattern: vec![pt!("L"), pt!(Period), pt!("L"), pt!(Period), pt!("M")],
            replace_with: vecword!("large language model"),
        });

        triggers.push(Rule {
            pattern: vec![
                pt!("L"),
                pt!(Period),
                pt!("L"),
                pt!(Period),
                pt!("M"),
                pt!(Period),
            ],
            replace_with: vecword!("large language model"),
        });

        Self { triggers }
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for Matcher {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let mut match_tokens = Vec::new();

        for (index, _) in document.tokens().enumerate() {
            for trigger in &self.triggers {
                match_tokens.clear();

                for (p_index, pattern) in trigger.pattern.iter().enumerate() {
                    let Some(token) = document.get_token(index + p_index) else {
                        break;
                    };

                    let t_pattern = PatternToken::from_token(token, document);

                    if t_pattern != *pattern {
                        break;
                    }

                    match_tokens.push(token);
                }

                if match_tokens.len() == trigger.pattern.len() && !match_tokens.is_empty() {
                    let span = Span::new(
                        match_tokens.first().unwrap().span.start,
                        match_tokens.last().unwrap().span.end,
                    );

                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::ReplaceWith(trigger.replace_with.to_owned())],
                        message: format!(
                            "Did you mean “{}”?",
                            trigger.replace_with.iter().collect::<String>()
                        ),
                        priority: 15,
                    })
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "A collection of curated rules. A catch-all that will be removed in the future."
    }
}

#[cfg(test)]
mod tests {
    use super::{Linter, Matcher};
    use crate::Document;

    #[test]
    fn matches_therefore() {
        let document = Document::new_plain_english_curated("There fore.");
        let mut matcher = Matcher::new();
        let lints = matcher.lint(&document);
        assert_eq!(lints.len(), 1);
    }
}

use crate::{
    CharStringExt, Lint, Token, TokenKind,
    char_ext::CharExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, find_the_only_token_matching, followed_by_token},
    },
};

// Note: MUST be lowercased! also any conjunction??
const WORDS_NOT_USUAL_BEFORE_AND: &[&str] = &[
    "and", // conj.coord
    "as",  // conj.subord
    "by",  // prep
    // "for",   // prep - too many false positives
    "of",   // prep
    "on",   // prep
    "to",   // prep
    "with", // prep
    "is",   // copula
    "was",  // copula
];

const POS_NOT_USUAL_AFTER_AN: &[fn(&TokenKind) -> bool] = &[
    TokenKind::is_personal_pronoun, // ! "IT"
    TokenKind::is_determiner,       // ! "all", "every"
    TokenKind::is_preposition,      // ! "anti"
    TokenKind::is_conjunction,
    // TokenKind::is_adverb,        // ! adverbs of manner
];
// Note: MUST be lowercased! also any Pronoun, Determiner, Preposition, Conjunction, Adverb??
const WORDS_NOT_USUAL_AFTER_AN: &[&str] = &[
    "a",     // det.indef
    "his",   // det.poss
    "its",   // det.poss
    "that",  // det / pron.rel
    "the",   // det.def
    "their", // det.poss,
    "he",    // pron.subj
    "i",     // pron subj.
    // "it",    // pron.subj+obj - BUT NOT "IT"!
    "you", // pron.subj+obj
    // "in",    // prep - "in" is also a noun: "his parents got him an in with the company"
    "to",   // prep
    "if",   // conj.subord / special
    "so",   // conj.coord / special
    "then", // adverb / special
    "when", // conj/special
];

pub struct AnAnd {
    expr: FirstMatchOf,
}

impl Default for AnAnd {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(SequenceExpr::aco("an").t_ws().then_any_of([
                    Box::new(SequenceExpr::default().then_kind_any_except(
                        POS_NOT_USUAL_AFTER_AN,
                        &[
                            /* determiners */ "all", "every", /* prepositions */ "anti",
                            "in",
                        ],
                    )),
                    Box::new(SequenceExpr::word_set(WORDS_NOT_USUAL_AFTER_AN)),
                ])),
                Box::new(
                    SequenceExpr::word_set(WORDS_NOT_USUAL_BEFORE_AND)
                        .t_ws()
                        .t_aco("and"),
                ),
            ]),
        }
    }
}

impl ExprLinter for AnAnd {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let span = find_the_only_token_matching(toks, src, |t: &Token, s: &[char]| {
            t.get_ch(s)
                .eq_any_ignore_ascii_case_chars(&[&['a', 'n'], &['a', 'n', 'd']])
        })?
        .span;

        let ch = span.get_content(src);

        // if ch is 'an' we already have the next word in toks, so get the previous word from ctx
        // if ch is 'and' we already have the prev word in toks, so get the next word from ctx
        match ch.len() {
            2 => {
                // got 'an' - change to 'and'?

                let maybe_prev = if let Some((before, _)) = ctx
                    && let [ws, word, ..] = before
                    && ws.kind.is_whitespace()
                {
                    Some(word)
                } else {
                    None
                };

                let next_tok = &toks[2];
                let next_ch = next_tok.get_ch(src);

                // "an it" would be a mistake but "an IT worker" etc. are fine
                if next_ch == ['I', 'T'] || next_ch == ['O', 'R'] {
                    return None;
                }

                if next_ch.eq_any_ignore_ascii_case_chars(&[
                    &['i', 'n'],
                    &['o', 'v', 'e', 'r'],
                    &['u', 'p'],
                ]) && followed_by_token(ctx, |t| t.kind.is_hyphen())
                {
                    return None;
                }

                // don't change it to 'and' if the previous word is one that doesn't usually come before 'and'
                if maybe_prev.is_some_and(|t| {
                    t.get_ch(src)
                        .eq_any_ignore_ascii_case_str(WORDS_NOT_USUAL_BEFORE_AND)
                }) {
                    return None;
                }
            }
            3 => {
                // got 'and' - change to 'an'?

                let prev_tok = &toks[0];
                let prev_ch = prev_tok.get_ch(src);

                let maybe_next = if let Some((_, after)) = ctx
                    && let [ws, word, ..] = after
                    && ws.kind.is_whitespace()
                {
                    Some(word)
                } else {
                    None
                };

                // Check if "an" would not be appropriate based on the next (and previous) word context
                if maybe_next.is_some_and(|t| {
                    let next_ch = t.get_ch(src);

                    // 1. Next word doesn't start with a vowel sound
                    next_ch.first().is_some_and(|c| !c.is_vowel()) ||

                    // 2. Next word doesn't usually come after 'an'
                    next_ch.eq_any_ignore_ascii_case_str(WORDS_NOT_USUAL_AFTER_AN) ||

                    // 3. Idiom: 'on' + 'off' or 'on' + 'on'
                    (prev_ch.eq_ch(&['o', 'n']) && next_ch.eq_any_ignore_ascii_case_chars(&[&['o', 'f', 'f'], &['o', 'n']]))
                }) {
                    return None;
                }
            }
            _ => return None,
        }

        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        let mut the_word: Vec<char> = ch.to_vec();

        if the_word.len() == 3 {
            the_word.pop();
        } else {
            // make an 'n' the same case as the 'n' (not the 'a' as the first letter of a word may be capitalized)
            let d_to_n_delta = b'n' - b'd';
            let n = the_word[1] as u8;
            let d: char = (n - d_to_n_delta) as char;
            the_word.push(d);
        }

        let suggestions = vec![Suggestion::ReplaceWith(the_word.clone())];

        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions,
            message: format!("Is this a typo for `{}`?", the_word.to_string()),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects typos mixing up `an` with `and`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AnAnd;

    // Contrived examples

    #[test]
    fn fix_he_an_i() {
        assert_suggestion_result(
            "he an I are friends",
            AnAnd::default(),
            "he and I are friends",
        );
    }

    #[test]
    fn fix_and_ox() {
        assert_suggestion_result(
            "as hungry as and ox",
            AnAnd::default(),
            "as hungry as an ox",
        );
    }

    // Real-world examples

    #[test]
    fn dont_flag_on_and_off() {
        assert_no_lints(
            "Features that can be turned on and off in the conversation or in \"settings\"",
            AnAnd::default(),
        );
    }

    #[test]
    fn dashes_description() {
        assert_no_lints(
            "Writers often type `--` or `---` expecting their editor to convert them into proper dashes. Replace these sequences with the correct characters: use an en dash (–) for ranges or connections and an em dash (—) for a break in thought.",
            AnAnd::default(),
        );
    }

    #[test]
    fn an_we_should() {
        // I just made this typo myself a couple of minutes ago
        assert_suggestion_result(
            "This would've been done since some people consider it incorrect an we should support everybody's ideas of correctness.",
            AnAnd::default(),
            "This would've been done since some people consider it incorrect and we should support everybody's ideas of correctness.",
        );
    }
}

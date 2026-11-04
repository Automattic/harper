use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

static ACCESS_VERBS: &[&str] = &[
    "get",
    "gets",
    "got",
    "gotten",
    "getting",
    "gain",
    "gains",
    "gained",
    "gaining",
    "have",
    "has",
    "had",
    "having",
    "obtain",
    "obtains",
    "obtained",
    "obtaining",
    "grant",
    "grants",
    "granted",
    "granting",
    "give",
    "gives",
    "gave",
    "given",
    "giving",
    "request",
    "requests",
    "requested",
    "requesting",
    "provide",
    "provides",
    "provided",
    "providing",
    "seek",
    "seeks",
    "sought",
    "seeking",
    "secure",
    "secures",
    "secured",
    "securing",
    "deny",
    "denies",
    "denied",
    "denying",
    "allow",
    "allows",
    "allowed",
    "allowing",
    "lose",
    "loses",
    "lost",
    "losing",
];

static FALSE_POSITIVE_WORDS: &[&str] = &[
    "all",
    "least",
    "once",
    "present",
    "will",
    "night",
    "noon",
    "midnight",
    "dusk",
    "dawn",
    "runtime",
    "scale",
    "speed",
    "index",
    "indices",
    "offset",
    "offsets",
    "address",
    "addresses",
    "port",
    "ports",
    "line",
    "lines",
    "position",
    "positions",
    "branch",
    "branches",
    "store",
    "stores",
    "site",
    "sites",
    "station",
    "stations",
];

static TEMPORAL_DETERMINERS: &[&str] = &[
    "this", "that", "these", "those", "the", "a", "an", "any", "all", "one", "each", "every",
];

static TEMPORAL_NOUNS: &[&str] = &[
    "time", "times", "moment", "moments", "stage", "stages", "point", "points",
];

/// Corrects "get access at <resource>" to "get access to <resource>", while avoiding
/// false positives like "get access at all", "get access at least", URLs, or time expressions.
pub struct GetAccessAt {
    expr: SequenceExpr,
}

impl Default for GetAccessAt {
    fn default() -> Self {
        let access_verbs = SequenceExpr::word_set(ACCESS_VERBS);

        let optional_modifiers = SequenceExpr::default().then_determiner().t_ws();

        let optional_adjectives = SequenceExpr::default().then_one_or_more_adjectives().t_ws();

        Self {
            expr: access_verbs
                .t_ws()
                .then_optional(optional_modifiers)
                .then_optional(optional_adjectives)
                .then_word_seq(&["access", "at"]),
        }
    }
}

impl ExprLinter for GetAccessAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags `get access at` and suggests replacing `at` with `to`."
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((_, after)) = ctx {
            let mut following_words = [None; 4];
            for (slot, tok) in following_words
                .iter_mut()
                .zip(after.iter().filter(|t| !t.kind.is_whitespace()))
            {
                *slot = Some(tok);
            }
            let following_words_len = following_words.iter().flatten().count();

            if let Some(first_tok) = following_words[0] {
                // If followed by URL, hostname, email, or unlintable token
                if matches!(
                    first_tok.kind,
                    TokenKind::Url
                        | TokenKind::Hostname
                        | TokenKind::EmailAddress
                        | TokenKind::Unlintable
                ) {
                    return None;
                }

                // The lexer only produces `Hostname` for common TLDs, so `example.dev` arrives
                // as `example`, `.`, `dev`, and the chunk ends at that period. Read the source:
                // a period followed by whitespace ended a sentence, one followed by a letter
                // is part of a domain.
                if let [Some('.'), Some(next)] =
                    [src.get(first_tok.span.end), src.get(first_tok.span.end + 1)]
                    && next.is_alphanumeric()
                {
                    return None;
                }

                let first_chars = first_tok.span.get_content(src);

                // Check if followed by known false positive words (like "all", "least", etc.)
                if first_chars.eq_any_ignore_ascii_case_str(FALSE_POSITIVE_WORDS) {
                    return None;
                }

                // Check for temporal adverbials like "at this time", "at any point", "at that moment"
                if following_words_len >= 2 {
                    let second_chars = following_words[1].unwrap().span.get_content(src);
                    if first_chars.eq_any_ignore_ascii_case_str(TEMPORAL_DETERMINERS)
                        && second_chars.eq_any_ignore_ascii_case_str(TEMPORAL_NOUNS)
                    {
                        return None;
                    }
                }

                // Check for level expressions like "at the disk level", "at the system level"
                if first_chars.eq_any_ignore_ascii_case_str(TEMPORAL_DETERMINERS) {
                    for following_tok in following_words[1..].iter().flatten() {
                        let word_chars = following_tok.span.get_content(src);
                        if word_chars.eq_any_ignore_ascii_case_str(&["level", "levels"]) {
                            return None;
                        }
                    }
                }
            }
        }

        let at_tok = toks.last()?;
        let at_span = at_tok.span;

        Some(Lint {
            span: at_span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                at_span.get_content(src),
            )],
            message: "Did you mean `to` instead of `at` when specifying what is being accessed?"
                .to_owned(),
            priority: 45,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::GetAccessAt;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn test_get_access_at_cpu() {
        assert_suggestion_result(
            "They really need to get access at some CPU time.",
            GetAccessAt::default(),
            "They really need to get access to some CPU time.",
        );
    }

    #[test]
    fn test_get_access_at_models() {
        assert_suggestion_result(
            "In order to get access at models, send a request.",
            GetAccessAt::default(),
            "In order to get access to models, send a request.",
        );
    }

    #[test]
    fn test_got_access_at_database() {
        assert_suggestion_result(
            "I got access at the server database.",
            GetAccessAt::default(),
            "I got access to the server database.",
        );
    }

    #[test]
    fn test_obtain_access_at_repository() {
        assert_suggestion_result(
            "You must obtain access at the repository before cloning it.",
            GetAccessAt::default(),
            "You must obtain access to the repository before cloning it.",
        );
    }

    // --- False Positive Tests ---

    #[test]
    fn ignore_access_at_all() {
        assert_lint_count(
            "I didn't manage to get access at all.",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_least() {
        assert_lint_count(
            "Is it possible to get access at least to the file content?",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_this_time() {
        assert_lint_count(
            "The quickest method to get access at this time is to submit a form.",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_url() {
        assert_lint_count(
            "Get access at https://example.com/login",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_bare_hostname() {
        assert_lint_count("Get access at example.com today", GetAccessAt::default(), 0);
    }

    #[test]
    fn ignore_access_at_www_hostname() {
        assert_lint_count("Get access at www.example.com", GetAccessAt::default(), 0);
    }

    #[test]
    fn ignore_access_at_hostname_with_path() {
        assert_lint_count(
            "Get access at github.com/org/repo",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_hostname_with_uncommon_tld() {
        assert_lint_count("Get access at example.ai", GetAccessAt::default(), 0);
    }

    #[test]
    fn ignore_access_at_subdomain_with_uncommon_tld() {
        assert_lint_count("Get access at docs.example.dev", GetAccessAt::default(), 0);
    }

    #[test]
    fn flags_access_at_word_ending_a_sentence() {
        assert_lint_count(
            "You get access at docs. Then log in.",
            GetAccessAt::default(),
            1,
        );
    }

    #[test]
    fn test_get_access_at_the_original_request() {
        assert_suggestion_result(
            "To get access at the original request within the hooks, use http middleware.",
            GetAccessAt::default(),
            "To get access to the original request within the hooks, use http middleware.",
        );
    }

    #[test]
    fn test_get_access_at_host_system() {
        assert_suggestion_result(
            "Succeeded to get access at host system with login command.",
            GetAccessAt::default(),
            "Succeeded to get access to host system with login command.",
        );
    }

    #[test]
    fn test_gain_access_at() {
        assert_suggestion_result(
            "The attacker managed to gain access at the database.",
            GetAccessAt::default(),
            "The attacker managed to gain access to the database.",
        );
    }

    #[test]
    fn test_have_access_at() {
        assert_suggestion_result(
            "Users have access at the internal network.",
            GetAccessAt::default(),
            "Users have access to the internal network.",
        );
    }

    #[test]
    fn test_with_adjective() {
        assert_suggestion_result(
            "They need to get full access at the database.",
            GetAccessAt::default(),
            "They need to get full access to the database.",
        );
    }

    #[test]
    fn test_with_possessive_determiner() {
        assert_suggestion_result(
            "They need to get their access at the database.",
            GetAccessAt::default(),
            "They need to get their access to the database.",
        );
    }

    #[test]
    fn test_with_quantifier() {
        assert_suggestion_result(
            "They need to get some access at the database.",
            GetAccessAt::default(),
            "They need to get some access to the database.",
        );
    }

    #[test]
    fn test_ignore_get_access_at_the_level() {
        assert_lint_count(
            "Getting access at the disk level would not normally be called a mount.",
            GetAccessAt::default(),
            0,
        );
    }
}

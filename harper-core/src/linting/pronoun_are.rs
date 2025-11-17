use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Corrects the shorthand `r` after plural first- and second-person pronouns.
pub struct PronounAre {
    expr: Box<dyn Expr>,
}

impl Default for PronounAre {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then(|tok: &Token, _src: &[char]| {
                tok.kind.is_pronoun()
                    && tok.kind.is_subject_pronoun()
                    && (tok.kind.is_second_person_pronoun()
                        || tok.kind.is_first_person_plural_pronoun()
                        || tok.kind.is_third_person_plural_pronoun())
            })
            .t_ws()
            .t_aco("r");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for PronounAre {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = tokens.span()?;
        let pronoun = tokens.first()?;
        let gap = tokens.get(1)?;
        let letter = tokens.get(2)?;

        let pronoun_chars = pronoun.span.get_content(source);
        let gap_chars = gap.span.get_content(source);
        let letter_chars = letter.span.get_content(source);

        let all_pronoun_letters_uppercase = pronoun_chars
            .iter()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase());
        let letter_has_uppercase = letter_chars.iter().any(|c| c.is_uppercase());
        let uppercase_suffix = letter_has_uppercase || all_pronoun_letters_uppercase;

        let are_suffix: Vec<char> = if uppercase_suffix {
            vec!['A', 'R', 'E']
        } else {
            vec!['a', 'r', 'e']
        };

        let re_suffix: Vec<char> = if uppercase_suffix {
            vec!['R', 'E']
        } else {
            vec!['r', 'e']
        };

        let mut with_are = pronoun_chars.to_vec();
        with_are.extend_from_slice(gap_chars);
        with_are.extend(are_suffix);

        let mut with_contraction = pronoun_chars.to_vec();
        with_contraction.push('\'');
        with_contraction.extend(re_suffix);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::ReplaceWith(with_are),
                Suggestion::ReplaceWith(with_contraction),
            ],
            message: "Use the full verb or the contraction after this pronoun.".to_owned(),
            priority: 40,
        })
    }

    fn description(&self) -> &str {
        "Spots the letter `r` used in place of `are` or `you're` after plural first- or second-person pronouns."
    }
}

#[cfg(test)]
mod tests {
    use super::PronounAre;
    use crate::linting::tests::{
        assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn fixes_you_r() {
        assert_suggestion_result(
            "You r absolutely right.",
            PronounAre::default(),
            "You are absolutely right.",
        );
    }

    #[test]
    fn offers_contraction_option() {
        assert_nth_suggestion_result(
            "You r absolutely right.",
            PronounAre::default(),
            "You're absolutely right.",
            1,
        );
    }

    #[test]
    fn keeps_uppercase_pronoun() {
        assert_suggestion_result(
            "YOU r welcome here.",
            PronounAre::default(),
            "YOU ARE welcome here.",
        );
    }

    #[test]
    fn fixes_they_r_with_comma() {
        assert_suggestion_result(
            "They r, of course, arriving tomorrow.",
            PronounAre::default(),
            "They are, of course, arriving tomorrow.",
        );
    }

    #[test]
    fn fixes_we_r_lowercase() {
        assert_suggestion_result(
            "we r ready now.",
            PronounAre::default(),
            "we are ready now.",
        );
    }

    #[test]
    fn fixes_they_r_sentence_start() {
        assert_suggestion_result(
            "They r planning ahead.",
            PronounAre::default(),
            "They are planning ahead.",
        );
    }

    #[test]
    fn fixes_lowercase_sentence() {
        assert_suggestion_result(
            "they r late again.",
            PronounAre::default(),
            "they are late again.",
        );
    }

    #[test]
    fn handles_line_break() {
        assert_suggestion_result(
            "We r\nready to go.",
            PronounAre::default(),
            "We are\nready to go.",
        );
    }

    #[test]
    fn does_not_flag_contraction() {
        assert_lint_count("You're looking great.", PronounAre::default(), 0);
    }

    #[test]
    fn does_not_flag_full_form() {
        assert_lint_count("They are excited about it.", PronounAre::default(), 0);
    }

    #[test]
    fn ignores_similar_word() {
        assert_lint_count("Your results impressed everyone.", PronounAre::default(), 0);
    }
}

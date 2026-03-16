use crate::linting::expr_linter::Chunk;
use crate::{
    Dialect, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct HaveTakeALook {
    expr: Box<dyn Expr>,
    dialect: Dialect,
}

impl HaveTakeALook {
    pub fn new(dialect: Dialect) -> Self {
        let light_verb = match dialect {
            // Match the opposite of what is used in the dialect.
            Dialect::British | Dialect::Australian => &["take", "took", "taken", "takes", "taking"],
            _ => &["have", "had", "had", "has", "having"],
        };

        let expr = SequenceExpr::word_set(light_verb)
            .t_ws()
            .then_fixed_phrase("a look");

        Self {
            expr: Box::new(expr),
            dialect,
        }
    }
}

impl ExprLinter for HaveTakeALook {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let light_verb_tok = toks.first().unwrap();
        let light_verb_str = light_verb_tok.span.get_content_string(src);
        let light_verb = light_verb_str.to_ascii_lowercase();

        let translated_light_verb: &[&str] = match light_verb.as_str() {
            "have" => &["take"],
            "had" => &["took", "taken"],
            "has" => &["takes"],
            "having" => &["taking"],
            "take" => &["have"],
            "took" => &["had"],
            "taken" => &["had"],
            "takes" => &["has"],
            "taking" => &["having"],
            _ => return None,
        };

        let suggestions = translated_light_verb
            .iter()
            .map(|s| {
                Suggestion::replace_with_match_case(
                    s.chars().collect(),
                    light_verb_tok.span.get_content(src),
                )
            })
            .collect();

        let message = format!(
            "{} English prefers {} over `{} a look`.",
            self.dialect,
            translated_light_verb
                .iter()
                .map(|lv| format!("`{lv} a look`"))
                .collect::<Vec<_>>()
                .join(" or "),
            light_verb,
        );

        Some(Lint {
            span: light_verb_tok.span,
            lint_kind: LintKind::Regionalism,
            suggestions,
            message,
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Corrects either `have a look` or `take a look` to the other, depending on the dialect."
    }
}

#[cfg(test)]
mod tests {
    use super::HaveTakeALook;
    use crate::{Dialect, linting::tests::assert_suggestion_result};

    #[test]
    fn correct_taking_a_look() {
        assert_suggestion_result(
            "Consider taking a look at crossorigin attribute.",
            HaveTakeALook::new(Dialect::British),
            "Consider having a look at crossorigin attribute.",
        );
    }

    #[test]
    fn correct_take_a_look() {
        assert_suggestion_result(
            "Have time to help take a look at the jdk21 upgrade issue.",
            HaveTakeALook::new(Dialect::Australian),
            "Have time to help have a look at the jdk21 upgrade issue.",
        );
    }

    #[test]
    fn correct_have_a_look() {
        assert_suggestion_result(
            "Have a look at this question crashing histoire using init-state and ref.",
            HaveTakeALook::new(Dialect::American),
            "Take a look at this question crashing histoire using init-state and ref.",
        );
    }

    #[test]
    fn correct_taken_a_look() {
        assert_suggestion_result(
            "Have you taken a look at HQEMU?",
            HaveTakeALook::new(Dialect::British),
            "Have you had a look at HQEMU?",
        );
    }

    #[test]
    fn correct_had_a_look() {
        assert_suggestion_result(
            "I had a look at the podman.go and have some theories I could test.",
            HaveTakeALook::new(Dialect::Canadian),
            "I took a look at the podman.go and have some theories I could test.",
        );
    }

    #[test]
    fn correct_took_a_look() {
        assert_suggestion_result(
            "I though GitHub's “Dashboard” page might help with this, so I took a look.",
            HaveTakeALook::new(Dialect::Australian),
            "I though GitHub's “Dashboard” page might help with this, so I had a look.",
        );
    }

    #[test]
    fn correct_takes_a_look() {
        assert_suggestion_result(
            "I'm closing this one, but it would be nice if someone takes a look at the notes in the original issue.",
            HaveTakeALook::new(Dialect::British),
            "I'm closing this one, but it would be nice if someone has a look at the notes in the original issue.",
        );
    }

    #[test]
    fn correct_having_a_look() {
        assert_suggestion_result(
            "It only appeared after I was having a look through the files.",
            HaveTakeALook::new(Dialect::American),
            "It only appeared after I was taking a look through the files.",
        );
    }

    #[test]
    fn correct_has_a_look() {
        assert_suggestion_result(
            "When Serializing messages the code in SchemaRegistrySerde has a look into the registry using the topic name.",
            HaveTakeALook::new(Dialect::Canadian),
            "When Serializing messages the code in SchemaRegistrySerde takes a look into the registry using the topic name.",
        );
    }
}

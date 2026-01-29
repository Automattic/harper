use crate::{
    EnglishDialect, Token,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::english::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct HaveTakeALook {
    expr: Box<dyn Expr>,
    dialect: EnglishDialect,
}

impl HaveTakeALook {
    pub fn new(dialect: EnglishDialect) -> Self {
        let light_verb = match dialect {
            // Match the opposite of what is used in the dialect.
            EnglishDialect::British | EnglishDialect::Australian => {
                &["take", "took", "taken", "takes", "taking"]
            }
            _ => &["have", "had", "had", "has", "having"],
        };

        let expr = SequenceExpr::default()
            .then(WordSet::new(light_verb))
            .t_ws()
            .then(FixedPhrase::from_phrase("a look"));

        Self {
            expr: Box::new(expr),
            dialect,
        }
    }
}

impl ExprLinter for HaveTakeALook {
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
    use crate::{
        EnglishDialect,
        linting::english::{HaveTakeALook, tests::assert_suggestion_result},
    };

    #[test]
    fn correct_taking_a_look() {
        assert_suggestion_result(
            "Consider taking a look at crossorigin attribute.",
            HaveTakeALook::new(EnglishDialect::British),
            "Consider having a look at crossorigin attribute.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_take_a_look() {
        assert_suggestion_result(
            "Have time to help take a look at the jdk21 upgrade issue.",
            HaveTakeALook::new(EnglishDialect::Australian),
            "Have time to help have a look at the jdk21 upgrade issue.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_have_a_look() {
        assert_suggestion_result(
            "Have a look at this question crashing histoire using init-state and ref.",
            HaveTakeALook::new(EnglishDialect::American),
            "Take a look at this question crashing histoire using init-state and ref.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_taken_a_look() {
        assert_suggestion_result(
            "Have you taken a look at HQEMU?",
            HaveTakeALook::new(EnglishDialect::British),
            "Have you had a look at HQEMU?",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_had_a_look() {
        assert_suggestion_result(
            "I had a look at the podman.go and have some theories I could test.",
            HaveTakeALook::new(EnglishDialect::Canadian),
            "I took a look at the podman.go and have some theories I could test.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_took_a_look() {
        assert_suggestion_result(
            "I though GitHub's “Dashboard” page might help with this, so I took a look.",
            HaveTakeALook::new(EnglishDialect::Australian),
            "I though GitHub's “Dashboard” page might help with this, so I had a look.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_takes_a_look() {
        assert_suggestion_result(
            "I'm closing this one, but it would be nice if someone takes a look at the notes in the original issue.",
            HaveTakeALook::new(EnglishDialect::British),
            "I'm closing this one, but it would be nice if someone has a look at the notes in the original issue.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_having_a_look() {
        assert_suggestion_result(
            "It only appeared after I was having a look through the files.",
            HaveTakeALook::new(EnglishDialect::American),
            "It only appeared after I was taking a look through the files.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn correct_has_a_look() {
        assert_suggestion_result(
            "When Serializing messages the code in SchemaRegistrySerde has a look into the registry using the topic name.",
            HaveTakeALook::new(EnglishDialect::Canadian),
            "When Serializing messages the code in SchemaRegistrySerde takes a look into the registry using the topic name.",
            crate::languages::LanguageFamily::English,
        );
    }
}

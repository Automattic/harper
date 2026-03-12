//! Linter for correcting "despite" used with incorrect verb forms.
//!
//! Handles cases like "despite it is" -> "despite it being" or "despite its being"

use crate::{
    CharStringExt, Token, TokenStringExt,
    dict_word_metadata::Person,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

/// Linter that corrects incorrect verb forms after "despite".
///
/// For example:
/// - "despite it is" -> "despite it being" or "despite its being"
/// - "despite I am" -> "despite me being" or "despite my being"
pub struct DespiteItIs {
    expr: Box<dyn Expr>,
}

impl Default for DespiteItIs {
    fn default() -> Self {
        let subj = SequenceExpr::default().then_subject_pronoun();
        let be = WordSet::new(&["am", "are", "is", "was", "were"]);

        let expr = SequenceExpr::aco("despite")
            .t_ws()
            .then(subj)
            .t_ws()
            .then(be);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for DespiteItIs {
    type Unit = Chunk;

    fn description(&self) -> &'static str {
        "Corrects `despite` being used with the wrong form of `is`."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let next_is_ing = ctx.is_some_and(|(_, then)| {
            then.first().is_some_and(|t| t.kind.is_whitespace())
                && then
                    .get(1)
                    .is_some_and(|t| t.kind.is_verb_progressive_form())
        });

        let subj = toks.get(2)?;
        let be = toks.get(4)?;

        let subj_kind = &subj.kind;

        // Only handle personal subject pronouns
        if !(subj_kind.is_personal_pronoun() && subj_kind.is_subject_pronoun()) {
            return None;
        }

        let subj_chars = subj.span.get_content(src);
        let be_chars = be.span.get_content(src);
        let pron_be_toks = &toks[2..5];

        let subj_pers = subj_kind.get_pronoun_person()?;

        // BUT BUT BUT
        // despite I  am happy   -> me/my  being happy
        // despite I  am eating  -> me/my        eating
        // despite it is big     ->        being
        //                       -> it/its being big
        // despite it is running -> it           running
        //                       -> it/its       running

        let (obj, poss) = match (
            subj_pers,
            subj_kind.is_singular_pronoun(),
            subj_kind.is_plural_pronoun(),
        ) {
            (Person::First, true, false) => ("me", "my"),
            (Person::First, false, true) => ("us", "our"),
            (Person::Second, true, true) => ("you", "your"),
            (Person::Third, false, true) => ("them", "their"),
            (Person::Third, true, false) => match subj_chars {
                chs if chs.eq_ignore_ascii_case_chars(&['h', 'e']) => ("him", "his"),
                chs if chs.eq_ignore_ascii_case_chars(&['s', 'h', 'e']) => ("her", "her"),
                chs if chs.eq_ignore_ascii_case_chars(&['i', 't']) => ("it", "its"),
                _ => return None,
            },
            _ => return None,
        };

        let mut suggestions = Vec::with_capacity(3);

        // Special case for "it" which can also be omitted
        if subj_chars.eq_any_ignore_ascii_case_str(&["it", "they"]) {
            suggestions.push(Suggestion::replace_with_match_case_str("being", be_chars));
        }

        let [obj_vec, poss_vec] = [obj, poss].map(|pron| {
            if !next_is_ing {
                format!("{} being", pron).chars().collect()
            } else {
                pron.chars().collect()
            }
        });

        suggestions.push(Suggestion::replace_with_match_case(obj_vec, be_chars));
        suggestions.push(Suggestion::replace_with_match_case(poss_vec, be_chars));

        if suggestions.is_empty() {
            return None;
        }

        let span_to_replace = pron_be_toks.span()?;

        Some(Lint {
            span: span_to_replace,
            lint_kind: LintKind::Grammar,
            suggestions,
            message: "Use the gerund form of the verb after `despite`.".into(),
            ..Lint::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DespiteItIs;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};

    #[test]
    fn despite_i_am() {
        assert_good_and_bad_suggestions(
            "Cronicle shuts down randomly despite I am running simple Python scripts via \"Test Plugin\"",
            DespiteItIs::default(),
            &[
                "Cronicle shuts down randomly despite me running simple Python scripts via \"Test Plugin\"",
                "Cronicle shuts down randomly despite my running simple Python scripts via \"Test Plugin\"",
            ],
            &[],
        );
    }

    #[test]
    fn despite_it_is_available() {
        assert_good_and_bad_suggestions(
            "Actual behavior Extension not installed despite it is available in PECL",
            DespiteItIs::default(),
            &[
                "Actual behavior Extension not installed despite being available in PECL",
                "Actual behavior Extension not installed despite it being available in PECL",
                "Actual behavior Extension not installed despite its being available in PECL",
            ],
            &[],
        );
    }

    #[test]
    fn despite_it_is_detected() {
        assert_good_and_bad_suggestions(
            "FP2 not detected despite it is detected - split brain?",
            DespiteItIs::default(),
            &[
                "FP2 not detected despite being detected - split brain?",
                "FP2 not detected despite it being detected - split brain?",
                "FP2 not detected despite its being detected - split brain?",
            ],
            &[],
        );
    }

    #[test]
    fn despite_i_am_in() {
        assert_good_and_bad_suggestions(
            "My application was rejected due to location basis despite I am in the same city as my campus.",
            DespiteItIs::default(),
            &[
                "My application was rejected due to location basis despite me being in the same city as my campus.",
                "My application was rejected due to location basis despite my being in the same city as my campus.",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "negatives are not handled yet"]
    fn despite_it_was_not() {
        assert_good_and_bad_suggestions(
            "despite it was not able to fulfill desired ordering with these modules",
            DespiteItIs::default(),
            &[
                "despite it not being able to fulfill desired ordering with these modules",
                "despite its not being able to fulfill desired ordering with these modules",
                "despite not being able to fulfill desired ordering with these modules",
            ],
            &[],
        );
    }

    #[test]
    fn despite_we_are_using() {
        assert_good_and_bad_suggestions(
            "However, GFW still can decode the content despite we are using overlapped ip fragmentation.",
            DespiteItIs::default(),
            &[
                "However, GFW still can decode the content despite us using overlapped ip fragmentation.",
                "However, GFW still can decode the content despite our using overlapped ip fragmentation.",
            ],
            &[],
        );
    }

    #[test]
    fn despite_they_are_already() {
        assert_good_and_bad_suggestions(
            "v5.7.2 keeps adding temperature commands on start_gcode despite they are already present",
            DespiteItIs::default(),
            &[
                "v5.7.2 keeps adding temperature commands on start_gcode despite them being already present",
                "v5.7.2 keeps adding temperature commands on start_gcode despite their being already present",
            ],
            &[],
        );
    }

    #[test]
    fn despite_it_was_removed() {
        assert_good_and_bad_suggestions(
            "Freshwater Research Station is selectable as starting location despite it was removed by Dark Days of the Dead mod",
            DespiteItIs::default(),
            &[
                "Freshwater Research Station is selectable as starting location despite being removed by Dark Days of the Dead mod",
                // TODO: Freshwater Research Station is selectable as starting location despite having been removed by Dark Days of the Dead mod
                "Freshwater Research Station is selectable as starting location despite it being removed by Dark Days of the Dead mod",
                // TODO: Freshwater Research Station is selectable as starting location despite it having been removed by Dark Days of the Dead mod
                "Freshwater Research Station is selectable as starting location despite its being removed by Dark Days of the Dead mod",
                // TODO: Freshwater Research Station is selectable as starting location despite its having been removed by Dark Days of the Dead mod
            ],
            &[],
        );
    }

    #[test]
    fn ignore_despite_they_shouldnt() {
        assert_no_lints(
            "Some tools and gears have attack damage values despite they shouldn't",
            DespiteItIs::default(),
        );
    }

    #[test]
    fn ignore_despite_i_was_playing() {
        assert_good_and_bad_suggestions(
            "it showed me Maria despite I was playing someone else",
            DespiteItIs::default(),
            &[
                "it showed me Maria despite me playing someone else",
                "it showed me Maria despite my playing someone else",
            ],
            &[],
        );
    }

    #[test]
    fn ignore_despite_they_were_valid() {
        assert_good_and_bad_suggestions(
            "You'll get pages that becomes invalid with time despite they were valid before",
            DespiteItIs::default(),
            &[
                "You'll get pages that becomes invalid with time despite being valid before",
                "You'll get pages that becomes invalid with time despite them being valid before",
                "You'll get pages that becomes invalid with time despite their being valid before",
            ],
            &[],
        );
    }
}

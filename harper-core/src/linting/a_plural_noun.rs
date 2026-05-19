use harper_brill::UPOS;

use crate::{
    CharStringExt, Dialect, IrregularNouns, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    indefinite_article::{InitialSound, starts_with_vowel},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    regular_nouns,
    spell::Dictionary,
};

pub struct APluralNoun<D> {
    expr: SequenceExpr,
    dict: D,
}

impl<D: Dictionary> APluralNoun<D> {
    pub fn new(dict: D) -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_indefinite_article()
                .t_ws()
                .then_zero_or_more(SequenceExpr::default().then_non_verb_adjective().t_ws())
                .then_plural_noun(),
            dict,
        }
    }
}

impl<D: Dictionary> ExprLinter for APluralNoun<D> {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let span = toks.span()?;
        let noun = toks.last()?;

        if noun.kind.is_singular_noun() {
            return None;
        }

        if noun.kind.is_adverb()
            || is_directional_adverb(noun, src)
            || is_great_many_phrase(toks, src)
        {
            return None;
        }

        if has_modifier_before_noun(toks, noun)
            && noun.kind.is_upos(UPOS::VERB)
            && noun.kind.is_verb_third_person_singular_present_form()
        {
            return None;
        }

        if noun.kind.is_verb_third_person_singular_present_form()
            && looks_like_third_person_verb_use(noun, ctx)
        {
            return None;
        }

        let article = toks.first()?;
        let plural = noun.span.get_content(src);
        let suggestions = singular_noun_suggestions(&self.dict, plural)
            .into_iter()
            .map(|singular| {
                let article_target = article_target(toks, noun, &singular, src)?;
                let mut replacement = indefinite_article_for(article_target).to_vec();
                replacement.extend(&src[article.span.end..noun.span.start]);
                replacement.extend(singular);

                Some(Suggestion::replace_with_match_case(
                    replacement,
                    span.get_content(src).to_vec(),
                ))
            })
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .collect::<Vec<_>>();

        if suggestions.is_empty() {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Agreement,
            suggestions,
            message: "Use a singular noun after the indefinite article `a` or `an`.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects plural nouns after the indefinite article `a` or `an`."
    }
}

trait SequenceExprExt {
    fn then_non_verb_adjective(self) -> Self;
}

impl SequenceExprExt for SequenceExpr {
    fn then_non_verb_adjective(self) -> Self {
        self.then(|tok: &Token, _src: &[char]| tok.kind.is_adjective() && !tok.kind.is_verb())
    }
}

fn looks_like_third_person_verb_use(noun: &Token, ctx: Option<(&[Token], &[Token])>) -> bool {
    let Some((_, after)) = ctx else {
        return false;
    };

    let mut non_ws = after.iter().filter(|tok| !tok.kind.is_whitespace());
    let Some(next) = non_ws.next() else {
        return noun.kind.is_upos(UPOS::VERB);
    };

    (noun.kind.is_upos(UPOS::VERB) && next.kind.is_sentence_terminator())
        || next.kind.is_pronoun()
        || next.kind.is_upos(UPOS::PRON)
}

fn has_modifier_before_noun(toks: &[Token], noun: &Token) -> bool {
    toks.iter()
        .skip(1)
        .any(|tok| !tok.kind.is_whitespace() && tok.span != noun.span)
}

fn is_directional_adverb(tok: &Token, src: &[char]) -> bool {
    tok.span.get_content(src).eq_any_ignore_ascii_case_str(&[
        "backwards",
        "downwards",
        "forwards",
        "inwards",
        "onwards",
        "outwards",
        "upwards",
    ])
}

fn is_great_many_phrase(toks: &[Token], src: &[char]) -> bool {
    let mut non_ws = toks.iter().filter(|tok| !tok.kind.is_whitespace());

    let Some(article) = non_ws.next() else {
        return false;
    };
    let Some(great) = non_ws.next() else {
        return false;
    };
    let Some(many) = non_ws.next() else {
        return false;
    };

    article.kind.is_word()
        && great.kind.is_word()
        && many.kind.is_word()
        && article
            .span
            .get_content(src)
            .eq_any_ignore_ascii_case_str(&["a"])
        && great
            .span
            .get_content(src)
            .eq_any_ignore_ascii_case_str(&["great"])
        && many
            .span
            .get_content(src)
            .eq_any_ignore_ascii_case_str(&["many"])
}

fn article_target<'a>(
    toks: &'a [Token],
    noun: &Token,
    singular: &'a [char],
    src: &'a [char],
) -> Option<&'a [char]> {
    let first_after_article = toks.iter().skip(1).find(|tok| !tok.kind.is_whitespace())?;

    if first_after_article.span == noun.span {
        Some(singular)
    } else {
        Some(first_after_article.span.get_content(src))
    }
}

fn singular_noun_suggestions<D: Dictionary>(dict: &D, plural: &[char]) -> Vec<Vec<char>> {
    let mut suggestions = Vec::new();

    if let Some(irregular) = IrregularNouns::curated().get_singular_for_plural_chars(plural) {
        suggestions.push(irregular.chars().collect());
    }

    suggestions.extend(regular_nouns::get_singulars(dict, plural));
    suggestions.sort();
    suggestions.dedup();
    suggestions
}

fn indefinite_article_for(noun: &[char]) -> &'static [char] {
    match starts_with_vowel(noun, Dialect::American) {
        Some(InitialSound::Vowel) => &['a', 'n'],
        _ => &['a'],
    }
}

#[cfg(test)]
mod tests {
    use super::APluralNoun;
    use crate::{linting::tests::assert_suggestion_result, spell::FstDictionary};
    use std::sync::Arc;

    fn linter() -> APluralNoun<Arc<FstDictionary>> {
        APluralNoun::new(FstDictionary::curated())
    }

    #[test]
    fn corrects_issue_example() {
        assert_suggestion_result(
            "I have included a notices on my landing.",
            linter(),
            "I have included a notice on my landing.",
        );
    }

    #[test]
    fn corrects_an_errors() {
        assert_suggestion_result("I saw an errors.", linter(), "I saw an error.");
    }

    #[test]
    fn corrects_article_before_consonant() {
        assert_suggestion_result("I rented an cars.", linter(), "I rented a car.");
    }

    #[test]
    fn corrects_article_before_vowel() {
        assert_suggestion_result("I ate a apples.", linter(), "I ate an apple.");
    }

    #[test]
    fn corrects_ies_plural() {
        assert_suggestion_result("I visited a cities.", linter(), "I visited a city.");
    }

    #[test]
    fn corrects_es_plural() {
        assert_suggestion_result("I packed a boxes.", linter(), "I packed a box.");
    }

    #[test]
    fn corrects_irregular_plural() {
        assert_suggestion_result("I saw a children.", linter(), "I saw a child.");
    }

    #[test]
    fn corrects_plural_after_adjective() {
        assert_suggestion_result(
            "A beautiful girls is sitting in the chair now.",
            linter(),
            "A beautiful girl is sitting in the chair now.",
        );
    }

    #[test]
    fn capitalizes_plural_after_adjective_at_sentence_start() {
        assert_suggestion_result(
            "an beautiful girls is sitting in the chair now.",
            linter(),
            "a beautiful girl is sitting in the chair now.",
        );
    }

    #[test]
    fn corrects_article_before_adjective() {
        assert_suggestion_result("I saw an red cars.", linter(), "I saw a red car.");
    }

    #[test]
    fn corrects_article_before_vowel_sound_adjective() {
        assert_suggestion_result("I saw a old errors.", linter(), "I saw an old error.");
    }

    #[test]
    fn allows_third_person_verb_after_modified_singular_subject() {
        crate::linting::tests::assert_no_lints("A predicate adjective follows.", linter());
    }

    #[test]
    fn allows_third_person_verb_after_adjective_subject() {
        crate::linting::tests::assert_no_lints("An auxiliary precedes it.", linter());
    }

    #[test]
    fn allows_third_person_verb_after_noun_modifier_subject() {
        crate::linting::tests::assert_no_lints("A problem remains in the design.", linter());
    }

    #[test]
    fn allows_great_many_plural() {
        crate::linting::tests::assert_no_lints("It had a great many teeth.", linter());
    }

    #[test]
    fn allows_adverb_after_modified_singular_noun() {
        crate::linting::tests::assert_no_lints("It ran a very little way forwards.", linter());
    }

    #[test]
    fn preserves_sentence_case() {
        assert_suggestion_result("A notices arrived.", linter(), "A notice arrived.");
    }

    #[test]
    fn allows_singular_after_a() {
        crate::linting::tests::assert_no_lints("I have included a notice.", linter());
    }

    #[test]
    fn allows_singular_after_an() {
        crate::linting::tests::assert_no_lints("I saw an error.", linter());
    }

    #[test]
    fn allows_bare_plural() {
        crate::linting::tests::assert_no_lints("I have included notices.", linter());
    }

    #[test]
    fn allows_a_lot_of_plural() {
        crate::linting::tests::assert_no_lints("I found a lot of bugs.", linter());
    }

    #[test]
    fn allows_a_few_plural() {
        crate::linting::tests::assert_no_lints("I found a few bugs.", linter());
    }

    #[test]
    fn allows_a_series_of_plural() {
        crate::linting::tests::assert_no_lints("I sent a series of notices.", linter());
    }

    #[test]
    fn allows_singular_or_plural_word() {
        crate::linting::tests::assert_no_lints("I saw a species.", linter());
    }
}

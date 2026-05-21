use harper_brill::UPOS;

use crate::{
    CharStringExt, Dialect, IrregularNouns, Lint, Token, TokenStringExt, case,
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
                .then_zero_or_more_spaced(SequenceExpr::default().then_noun_phrase_member()),
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
        let head_index = head_noun_index(toks)?;
        let lint_toks = &toks[..=head_index];
        let span = lint_toks.span()?;
        let noun = lint_toks.last()?;

        if has_line_boundary(lint_toks) {
            return None;
        }

        if !noun.kind.is_plural_noun() || noun.kind.is_singular_noun() {
            return None;
        }

        if is_to_pieces_preposition_typo(lint_toks, src, ctx)
            || is_plural_location_determiner_typo(lint_toks, src, ctx)
        {
            return None;
        }

        if noun.kind.is_adverb()
            || is_directional_adverb(noun, src)
            || is_quantity_phrase(lint_toks, src)
            || is_great_many_phrase(lint_toks, src)
        {
            return None;
        }

        if has_modifier_before_noun(lint_toks, noun)
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
        let mut suggestions = if is_plural_location_noun(plural) {
            Vec::new()
        } else {
            singular_noun_suggestions(&self.dict, plural)
                .into_iter()
                .map(|singular| {
                    let article_target = article_target(lint_toks, noun, &singular, src)?;
                    let mut replacement = match_article_case(
                        indefinite_article_for(article_target),
                        article.span.get_content(src),
                    );
                    replacement.extend(&src[article.span.end..noun.span.start]);
                    replacement.extend(singular);

                    Some(Suggestion::ReplaceWith(replacement))
                })
                .collect::<Option<Vec<_>>>()?
        };

        if let Some(mut plural_intent_suggestions) =
            plural_intent_suggestions(lint_toks, src, span.get_content(src), noun)
        {
            suggestions.append(&mut plural_intent_suggestions);
        }

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

fn head_noun_index(toks: &[Token]) -> Option<usize> {
    let last_index = toks
        .iter()
        .enumerate()
        .rev()
        .find(|(_, tok)| !tok.kind.is_whitespace())?
        .0;
    let last = &toks[last_index];

    if last.kind.is_plural_noun() {
        return Some(last_index);
    }

    if last.kind.is_upos(UPOS::VERB) {
        return toks[..last_index]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, tok)| !tok.kind.is_whitespace())
            .and_then(|(index, tok)| tok.kind.is_plural_noun().then_some(index));
    }

    Some(last_index)
}

trait SequenceExprExt {
    fn then_noun_phrase_member(self) -> Self;
}

impl SequenceExprExt for SequenceExpr {
    fn then_noun_phrase_member(self) -> Self {
        self.then(|tok: &Token, src: &[char]| {
            (tok.kind.is_non_possessive_noun() || (tok.kind.is_adjective() && !tok.kind.is_verb()))
                && !tok.kind.is_preposition()
                && !tok.kind.is_pronoun()
                && !tok.kind.is_conjunction()
                && !tok.kind.is_quantifier()
                && !tok
                    .span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(&["ah", "few", "first", "said", "uh"])
        })
    }
}

fn is_to_pieces_preposition_typo(
    toks: &[Token],
    src: &[char],
    ctx: Option<(&[Token], &[Token])>,
) -> bool {
    let mut matched_words = toks.iter().filter(|tok| !tok.kind.is_whitespace());

    let Some(article) = matched_words.next() else {
        return false;
    };
    let Some(noun) = matched_words.next() else {
        return false;
    };

    if !article.span.get_content(src).eq_str("a") || !noun.span.get_content(src).eq_str("pieces") {
        return false;
    }

    let Some(previous_word) = previous_word_before_phrase(ctx) else {
        return false;
    };

    previous_word
        .span
        .get_content(src)
        .eq_any_ignore_ascii_case_str(&[
            "blew",
            "blown",
            "break",
            "breaking",
            "broke",
            "broken",
            "cut",
            "fall",
            "fallen",
            "fell",
            "rip",
            "ripped",
            "shatter",
            "shattered",
            "smash",
            "smashed",
            "tear",
            "tearing",
            "tore",
            "torn",
        ])
}

fn previous_word_before_phrase<'a>(ctx: Option<(&'a [Token], &'a [Token])>) -> Option<&'a Token> {
    ctx?.0
        .iter()
        .rev()
        .find(|tok| !tok.kind.is_whitespace())
        .filter(|tok| tok.kind.is_word())
}

const PLURAL_LOCATION_NOUNS: &[&str] = &[
    "grounds",
    "headquarters",
    "outskirts",
    "premises",
    "quarters",
    "stairs",
    "surroundings",
    "woods",
];

fn is_plural_location_noun(word: &[char]) -> bool {
    word.eq_any_ignore_ascii_case_str(PLURAL_LOCATION_NOUNS)
}

fn is_plural_location_determiner_typo(
    toks: &[Token],
    src: &[char],
    ctx: Option<(&[Token], &[Token])>,
) -> bool {
    let mut matched_words = toks.iter().filter(|tok| !tok.kind.is_whitespace());

    let Some(article) = matched_words.next() else {
        return false;
    };
    let Some(noun) = matched_words.next() else {
        return false;
    };

    if !article.span.get_content(src).eq_str("a")
        || !noun
            .span
            .get_content(src)
            .eq_any_ignore_ascii_case_str(PLURAL_LOCATION_NOUNS)
    {
        return false;
    }

    let Some(previous_word) = previous_word_before_phrase(ctx) else {
        return false;
    };

    previous_word
        .span
        .get_content(src)
        .eq_any_ignore_ascii_case_str(&["down", "from", "on", "to", "up"])
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

fn has_line_boundary(toks: &[Token]) -> bool {
    toks.iter()
        .any(|tok| tok.kind.is_newline() || tok.kind.is_paragraph_break())
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

fn is_quantity_phrase(toks: &[Token], src: &[char]) -> bool {
    let mut non_ws = toks.iter().filter(|tok| !tok.kind.is_whitespace());

    let Some(article) = non_ws.next() else {
        return false;
    };
    let Some(first_after_article) = non_ws.next() else {
        return false;
    };

    article
        .span
        .get_content(src)
        .eq_any_ignore_ascii_case_str(&["a", "an"])
        && first_after_article
            .span
            .get_content(src)
            .eq_any_ignore_ascii_case_str(&[
                "couple", "dozen", "hundred", "thousand", "million", "billion", "trillion", "one",
                "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
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
        && article.span.get_content(src).eq_str("a")
        && great.span.get_content(src).eq_str("great")
        && many.span.get_content(src).eq_str("many")
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

fn match_article_case(article: &[char], template: &[char]) -> Vec<char> {
    case::copy_casing(template, article).to_vec()
}

fn plural_intent_suggestions(
    toks: &[Token],
    src: &[char],
    span_template: &[char],
    noun: &Token,
) -> Option<Vec<Suggestion>> {
    let first_after_article = toks.iter().skip(1).find(|tok| !tok.kind.is_whitespace())?;
    let replacement_tail = &src[first_after_article.span.start..noun.span.end];

    let mut suggestions = Vec::with_capacity(2);
    suggestions.push(Suggestion::replace_with_match_case(
        replacement_tail.to_vec(),
        span_template,
    ));

    let mut some_replacement = "some".chars().collect::<Vec<_>>();
    some_replacement.push(' ');
    some_replacement.extend(replacement_tail);
    suggestions.push(Suggestion::replace_with_match_case(
        some_replacement,
        span_template,
    ));

    Some(suggestions)
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
    use crate::{
        linting::tests::{assert_good_and_bad_suggestions, assert_suggestion_result},
        spell::FstDictionary,
    };
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
    fn suggests_plural_intent_alternatives() {
        assert_good_and_bad_suggestions(
            "I saw a notices.",
            linter(),
            &["I saw a notice.", "I saw notices.", "I saw some notices."],
            &[],
        );
    }

    #[test]
    fn suggests_plural_intent_alternatives_with_modifier() {
        assert_good_and_bad_suggestions(
            "A beautiful girls is sitting in the chair now.",
            linter(),
            &[
                "A beautiful girl is sitting in the chair now.",
                "Beautiful girls is sitting in the chair now.",
                "Some beautiful girls is sitting in the chair now.",
            ],
            &[],
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
    fn corrects_article_and_plural_after_lowercase_sentence_start() {
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
    fn preserves_uppercase_article_case() {
        assert_suggestion_result("A notices arrived.", linter(), "A notice arrived.");
    }

    #[test]
    fn does_not_capitalize_lowercase_sentence_start() {
        assert_suggestion_result("a notices arrived.", linter(), "a notice arrived.");
    }

    #[test]
    fn allows_singular_after_a() {
        crate::linting::tests::assert_no_lints("I have included a notice.", linter());
    }

    #[test]
    fn allows_to_pieces_preposition_typo() {
        crate::linting::tests::assert_no_lints("With one attack, he was torn a pieces.", linter());
    }

    #[test]
    fn allows_plural_location_determiner_typo_after_preposition() {
        crate::linting::tests::assert_no_lints("Then I heard footsteps on a stairs.", linter());
        crate::linting::tests::assert_no_lints("They met on a premises nearby.", linter());
        crate::linting::tests::assert_no_lints("She waited on a grounds outside.", linter());
        crate::linting::tests::assert_no_lints("He lives on a outskirts of town.", linter());
    }

    #[test]
    fn does_not_cross_line_boundaries() {
        crate::linting::tests::assert_no_lints(
            "following it, in a way
contralto voices have",
            linter(),
        );
    }

    #[test]
    fn still_corrects_to_pieces_after_punctuation() {
        assert_suggestion_result(
            "The vase shattered. A pieces, sharp and white, lay on the floor.",
            linter(),
            "The vase shattered. Pieces, sharp and white, lay on the floor.",
        );
    }

    #[test]
    fn corrects_plural_head_before_trailing_verb() {
        assert_good_and_bad_suggestions(
            "He looked up. A stairs led to the attic.",
            linter(),
            &[
                "He looked up. Stairs led to the attic.",
                "He looked up. Some stairs led to the attic.",
            ],
            &["He looked up. A stair led to the attic."],
        );
    }

    #[test]
    fn still_corrects_plural_location_after_punctuation() {
        assert_good_and_bad_suggestions(
            "He looked up. A stairs, narrow and steep, led to the attic.",
            linter(),
            &[
                "He looked up. Stairs, narrow and steep, led to the attic.",
                "He looked up. Some stairs, narrow and steep, led to the attic.",
            ],
            &["He looked up. A stair, narrow and steep, led to the attic."],
        );
    }

    #[test]
    fn still_corrects_regular_plural_noun_after_preposition() {
        assert_suggestion_result("I put it on a tables.", linter(), "I put it on a table.");
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
    fn allows_quantity_phrase_with_plural_noun() {
        crate::linting::tests::assert_no_lints("I bought a dozen volumes.", linter());
        crate::linting::tests::assert_no_lints("It is worth a hundred pounds.", linter());
        crate::linting::tests::assert_no_lints("She was a thousand times larger.", linter());
        crate::linting::tests::assert_no_lints("She started a three months trip.", linter());
    }

    #[test]
    fn allows_coordinated_singular_and_plural_nouns() {
        crate::linting::tests::assert_no_lints(
            "A quorum shall consist of a member or members.",
            linter(),
        );
    }

    #[test]
    fn allows_a_series_of_plural() {
        crate::linting::tests::assert_no_lints("I sent a series of notices.", linter());
    }

    #[test]
    fn allows_singular_or_plural_word() {
        crate::linting::tests::assert_no_lints("I saw a species.", linter());
    }

    #[test]
    fn allows_plural_looking_modifier_before_head_noun() {
        crate::linting::tests::assert_no_lints("She hired a systems engineer.", linter());
    }

    #[test]
    fn allows_plural_looking_modifier_after_adjective() {
        crate::linting::tests::assert_no_lints("She hired a senior systems engineer.", linter());
    }

    #[test]
    fn corrects_plural_head_after_plural_looking_modifier() {
        assert_suggestion_result(
            "She hired a senior systems engineers.",
            linter(),
            "She hired a senior systems engineer.",
        );
    }

    #[test]
    fn allows_plural_looking_modifier_before_head_noun_with_different_head() {
        crate::linting::tests::assert_no_lints("She hired a communications director.", linter());
    }
}

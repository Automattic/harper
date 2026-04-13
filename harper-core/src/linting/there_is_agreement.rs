use crate::{
    CharStringExt, Dialect, IrregularNouns, Lint, Lrc, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, OwnedExprExt, SequenceExpr, SpelledNumberExpr},
    indefinite_article::{InitialSound, starts_with_vowel},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Sentence, followed_by_hyphen, followed_by_word},
    },
    regular_nouns,
};

#[derive(PartialEq, Debug)]
enum Mismatch {
    SingularBePluralNoun,
    PluralBeSingularNoun,
}

impl Mismatch {
    fn from_be(be: &[char]) -> Self {
        if be.eq_any_ignore_ascii_case_str(&["are", "were"]) {
            PluralBeSingularNoun
        } else {
            SingularBePluralNoun
        }
    }
}
use Mismatch::*;

#[derive(PartialEq)]
enum WordOrder {
    Statement, // normal: there is, etc.
    Question,  // inverted: is there?, etc.
}
use WordOrder::*;

#[derive(PartialEq)]
enum Tense {
    Present,
    Past,
}
use Tense::*;

pub struct ThereIsAgreement {
    expr: FirstMatchOf,
}

impl Default for ThereIsAgreement {
    fn default() -> Self {
        let plural_noun = Lrc::new(|t: &Token, _: &[char]| t.kind.is_plural_noun());

        // reject singular nouns that are also: adjectives, "no", spelled numbers
        // TODO but this rejects "problem"
        // "two" etc. are sg. nouns even though they can also be plural quantifiers
        let singular_noun = Lrc::new(SequenceExpr::default().then_singular_noun().and_not(
            FirstMatchOf::new(vec![
                Box::new(|t: &Token, s: &[char]| {
                    t.kind.is_adjective()
                        || t.get_ch(s)
                            .eq_any_ignore_ascii_case_chars(&[&['n', 'o'], &['n', 'o', 't']])
                }),
                Box::new(SpelledNumberExpr),
            ]),
        ));

        let first_match_of: Vec<Box<dyn Expr>> = [
            (SingularBePluralNoun, Statement, Present),
            (SingularBePluralNoun, Statement, Past),
            (SingularBePluralNoun, Question, Present),
            (SingularBePluralNoun, Question, Past),
            (PluralBeSingularNoun, Statement, Present),
            (PluralBeSingularNoun, Statement, Past),
            (PluralBeSingularNoun, Question, Present),
            (PluralBeSingularNoun, Question, Past),
        ]
        .iter()
        .map(|(mismatch, word_order, tense)| {
            let be = match (mismatch, tense) {
                (SingularBePluralNoun, Present) => "is",
                (SingularBePluralNoun, Past) => "was",
                (PluralBeSingularNoun, Present) => "are",
                (PluralBeSingularNoun, Past) => "were",
            };
            let (first, second) = match word_order {
                Statement => ("there", be),
                Question => (be, "there"),
            };

            let seq = SequenceExpr::aco(first).t_ws().t_aco(second).t_ws();

            (match mismatch {
                SingularBePluralNoun => Box::new(seq.then(plural_noun.clone())),
                PluralBeSingularNoun => Box::new(seq.then(singular_noun.clone())),
            }) as Box<dyn Expr>
        })
        .chain(std::iter::once(Box::new(
            SequenceExpr::fixed_phrase("there's ").then(plural_noun.clone()),
        ) as Box<dyn Expr>))
        .collect();

        Self {
            expr: FirstMatchOf::new(first_match_of),
        }
    }
}

impl ExprLinter for ThereIsAgreement {
    type Unit = Sentence;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<super::Lint> {
        match_to_lint(toks, src, ctx)
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for `is there` and its variants agreeing with singular vs plural subjects"
    }
}

fn match_to_lint(
    toks: &[Token],
    src: &[char],
    ctx: Option<(&[Token], &[Token])>,
) -> Option<super::Lint> {
    let first = toks[0].span.get_content(src);
    let second = toks[2].span.get_content(src);

    const BE_FORMS: &[&str] = &["is", "are", "was", "were"];

    match (
        first.starts_with_ignore_ascii_case_str("there"),
        first.last(),
    ) {
        (false, _) if second.eq_str("there") => handle_question(toks, src, ctx),
        (true, Some(&'s')) => handle_theres(toks, src, ctx),
        (true, _) => handle_statement(toks, src, ctx),
        _ => None, // unreachable
    }
}

// No need to allocate.
// Only four input words are possible, but lettercase can vary.
// Only grammatical number changes.
fn get_new_be(orig_be: &[char]) -> &[char] {
    let [first, second, ..] = orig_be else {
        return orig_be;
    };

    match (first.to_ascii_lowercase(), second.to_ascii_lowercase()) {
        ('i', 's') => &['a', 'r', 'e'],
        ('a', 'r') => &['i', 's'],
        ('w', 'a') => &['w', 'e', 'r', 'e'],
        ('w', 'e') => &['w', 'a', 's'],
        _ => orig_be,
    }
}

type NounFormGetters = (
    fn(&[char]) -> Option<Vec<char>>,
    fn(&[char]) -> Vec<Vec<char>>,
);

// Returns zero or more noun forms.
// Needs to allocate a vec for each noun and for the array of nouns.
// Each noun will be accompanied by an appropriate indefinite article.
// TODO does not yet handle words which take either/both "a"/"an"
fn get_new_nouns<'a>(
    orig_noun: &'a [char],
    mismatch_type: &Mismatch,
) -> Vec<(Vec<char>, &'a [char])> {
    let (irreg_func, reg_func): NounFormGetters = match mismatch_type {
        PluralBeSingularNoun => (get_irregular_plural, regular_nouns::get_plurals),
        SingularBePluralNoun => (get_irregular_singular, regular_nouns::get_singulars),
    };
    let (irregular, regulars) = (irreg_func(orig_noun), reg_func(orig_noun));

    irregular
        .into_iter()
        .chain(regulars)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|n| {
            let art = match starts_with_vowel(&n, Dialect::American) {
                Some(InitialSound::Vowel) => &['a', 'n'][..],
                Some(InitialSound::Consonant) => &['a'][..],
                // Some(InitialSound::Either) => &[&['a'][..], &['a', 'n'][..]],
                _ => &['a'][..],
            };
            (n, art)
        })
        .collect()
}

fn get_irregular_plural(singular: &[char]) -> Option<Vec<char>> {
    IrregularNouns::curated()
        .get_plural_for_singular_chars(singular)
        .map(|s| s.chars().collect())
}

fn get_irregular_singular(plural: &[char]) -> Option<Vec<char>> {
    IrregularNouns::curated()
        .get_singular_for_plural_chars(plural)
        .map(|s| s.chars().collect())
}

fn handle_statement(
    toks: &[Token],
    src: &[char],
    ctx: Option<(&[Token], &[Token])>,
) -> Option<Lint> {
    // Don't proceed if the next token is a hyphen, indicating a compound.
    // e.g. "function-like macros"
    if followed_by_hyphen(ctx) {
        return None;
    }

    // Or if the next word in a noun, also indicating a compound:
    // e.g. "there are config errors"
    if followed_by_word(ctx, |t| {
        (t.kind.is_noun()
            && !t.kind.is_verb_progressive_form()
            && !t
                .get_ch(src)
                .eq_any_ignore_ascii_case_str(&["but", "in", "like"]))
            // to allow "there are bug in ..." to be corrected
            || t.get_ch(src).eq_any_ignore_ascii_case_str(&["and", "or"])
    }) {
        return None;
    }

    // NOTE - for a statement we only need to replace two words.
    // NOTE - (there) "are man" -> "there is (a) man"
    //                          -> "there are men"
    let replacement_template = toks[2..=4].get_ch(src)?;

    // tok 2 is form of "be": is are was were
    // tok 4 is noun that doesn't agree in number with the form of "be"
    let orig_be = toks[2].get_ch(src);
    let orig_noun = toks[4].get_ch(src);

    let mismatch_type = Mismatch::from_be(orig_be);

    // fix 1 = change form of "be" to match the noun
    //         - suggestion is "corrected be" + noun
    let new_be = get_new_be(orig_be);

    // When PluralBeSingularNoun we need to look up the right
    // indefinite article for the original noun
    // and insert it before the noun (with a space between)
    let mut replacement_value: Vec<char> = new_be.to_vec();

    // Only add article and space for singular nouns
    if matches!(mismatch_type, PluralBeSingularNoun) {
        // get the indefinite article for the orig_noun
        let article = match starts_with_vowel(orig_noun, Dialect::American) {
            Some(InitialSound::Vowel) => &['a', 'n'][..],
            Some(InitialSound::Consonant) => &['a'][..],
            // TODO InitialSound::Either needs to return an array of ["a", "an"]
            _ => &['\u{1F170}', '\u{fe0f}', '\u{1F170}', '\u{fe0f}'][..],
        };
        replacement_value.extend(&[' ']);
        replacement_value.extend(article.iter());
    }

    replacement_value.push(' ');
    replacement_value.extend(orig_noun.iter());

    let be_suggestion =
        Suggestion::replace_with_match_case(replacement_value, replacement_template);

    // fix 2 = change noun to match the form of "be"
    //         - could be multiple
    //         - don't need a/an here because nouns are plural
    let new_nouns = get_new_nouns(orig_noun, &mismatch_type);

    let article_noun_pairs = new_nouns
        .iter()
        .map(|(noun, article)| {
            let mut result: Vec<char> = orig_be.to_vec();

            if matches!(mismatch_type, SingularBePluralNoun) {
                result.extend(&[' ']);
                result.extend(article.iter());
            }

            result.extend(&[' ']);
            result.extend(noun.iter());

            result
        })
        .collect::<Vec<Vec<char>>>();

    let noun_suggestions: Vec<Suggestion> = article_noun_pairs
        .iter()
        .map(|sug_repl_value| {
            Suggestion::replace_with_match_case(sug_repl_value.to_vec(), replacement_template)
        })
        .collect();

    Some(Lint {
        span: toks[2..=4].span()?,
        message: "There is disagreement in number between the verb and the noun.".to_string(),
        suggestions: [vec![be_suggestion], noun_suggestions].concat(),
        lint_kind: LintKind::Agreement,
        ..Default::default()
    })
}

fn handle_theres(toks: &[Token], src: &[char], _ctx: Option<(&[Token], &[Token])>) -> Option<Lint> {
    // NOTE - for there's we only need to replace two words.
    // NOTE - "there's men" -> there's a man
    //                      -> there are men

    let replacement_template = toks[0..=2].get_ch(src)?;

    // tok 0 is "there's" (with various types of apostrophe character)
    // tok 2 is noun that doesn't agree in number with "is" (the form of "be" encoded in "there's")
    let orig_there_be = toks[0].span.get_content(src);
    let orig_noun = toks[2].span.get_content(src);

    let mismatch_type = SingularBePluralNoun;

    // fix 1 = change "there's" to "there are" to match the noun
    //         - suggestion is "there are" + noun
    let new_there_be = &['t', 'h', 'e', 'r', 'e', ' ', 'a', 'r', 'e'][..];

    let replacement_value: Vec<char> = new_there_be
        .iter()
        .chain(&[' '])
        .chain(orig_noun.iter())
        .copied()
        .collect();

    let there_be_suggestion =
        Suggestion::replace_with_match_case(replacement_value, replacement_template);

    // fix 2
    let new_nouns = get_new_nouns(orig_noun, &mismatch_type);

    let article_noun_pairs = new_nouns
        .iter()
        .map(|(noun, article)| {
            let mut result: Vec<char> = orig_there_be.to_vec();

            // there's is always SingularBePluralNoun
            result.extend(&[' ']);
            result.extend(article.iter());

            result.extend(&[' ']);
            result.extend(noun.iter());

            result
        })
        .collect::<Vec<Vec<char>>>();

    let noun_suggestions: Vec<Suggestion> = article_noun_pairs
        .iter()
        .map(|sug_repl_value| {
            Suggestion::replace_with_match_case(sug_repl_value.to_vec(), replacement_template)
        })
        .collect();

    Some(Lint {
        span: toks[0..=2].span()?,
        lint_kind: LintKind::Agreement,
        suggestions: [vec![there_be_suggestion], noun_suggestions].concat(),
        message: "`There's` means `there is`, which requires a singular noun.".to_string(),
        ..Default::default()
    })
}

fn handle_question(
    toks: &[Token],
    src: &[char],
    ctx: Option<(&[Token], &[Token])>,
) -> Option<Lint> {
    // Don't proceed if the next token is a noun, indicating a compound:
    // e.g. "there are config errors"
    if followed_by_word(ctx, |t| {
        t.kind.is_noun()
            && !t.kind.is_verb_progressive_form()
            && !t.get_ch(src).eq_any_ignore_ascii_case_str(&["in", "or"])
    }) {
        return None;
    }

    // NOTE - for a question we must replace three words!
    // NOTE - "are there man" -> "is there (a) man"
    //                        -> "are there men"
    let replacement_template = toks[0..=4].get_ch(src)?;

    // tok 0 is form of "be": is are was were
    // tok 2 is "there"
    // tok 4 is noun that doesn't agree in number with the form of "be"
    let orig_be = toks[0].get_ch(src);
    let orig_noun = toks[4].get_ch(src);

    let mismatch_type = Mismatch::from_be(orig_be);

    // fix 1 = change form of "be" to match the noun
    //         - suggestion is "corrected be" + "there" + noun
    let new_be = get_new_be(orig_be);

    let mut replacement_value: Vec<char> = new_be
        .iter()
        .chain(&[' ', 't', 'h', 'e', 'r', 'e'])
        .copied()
        .collect();

    // Only add article and space for singular nouns
    if matches!(mismatch_type, PluralBeSingularNoun) {
        let article = match starts_with_vowel(orig_noun, Dialect::American) {
            Some(InitialSound::Vowel) => &['a', 'n'][..],
            Some(InitialSound::Consonant) => &['a'][..],
            // TODO InitialSound::Either needs to return an array of ["a", "an"]
            _ => &['\u{1F170}', '\u{fe0f}', '\u{1F170}', '\u{fe0f}'][..],
        };
        replacement_value.extend(&[' ']);
        replacement_value.extend(article.iter());
    }

    replacement_value.extend(&[' ']);
    replacement_value.extend(orig_noun.iter());

    let be_suggestion =
        Suggestion::replace_with_match_case(replacement_value, replacement_template);

    // fix 2
    let new_nouns = get_new_nouns(orig_noun, &mismatch_type);
    let article_noun_pairs = new_nouns
        .iter()
        .map(|(noun, article)| {
            let mut result: Vec<char> = orig_be
                .iter()
                .chain(&[' ', 't', 'h', 'e', 'r', 'e'])
                .copied()
                .collect();

            // Only add article and space for singular nouns
            if matches!(mismatch_type, SingularBePluralNoun) {
                result.extend(&[' ']);
                result.extend(article.iter());
            }

            result.extend(&[' ']);
            result.extend(noun.iter());

            result
        })
        .collect::<Vec<_>>();

    let noun_suggestions = article_noun_pairs
        .iter()
        .map(|replacement_value| {
            Suggestion::replace_with_match_case(replacement_value.to_vec(), replacement_template)
        })
        .collect::<Vec<_>>();

    Some(Lint {
        span: toks[0..=4].span()?,
        lint_kind: LintKind::Agreement,
        suggestions: [vec![be_suggestion], noun_suggestions].concat(),
        message: "There is disagreement in number between the verb and the noun.".to_string(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::ThereIsAgreement;

    // basic functionality

    #[test]
    fn statement_present_pl_regular() {
        assert_good_and_bad_suggestions(
            "there is things",
            ThereIsAgreement::default(),
            &["there are things", "there is a thing"],
            &[],
        );
    }

    #[test]
    fn statement_present_sg_irregular() {
        assert_good_and_bad_suggestions(
            "there are person",
            ThereIsAgreement::default(),
            &["there are people", "there is a person"],
            &[],
        );
    }

    #[test]
    fn statement_present_theres_pl() {
        assert_good_and_bad_suggestions(
            "there's secrets",
            ThereIsAgreement::default(),
            &["there are secrets", "there's a secret"],
            &[],
        );
    }

    #[test]
    fn statement_past_pl_vowel() {
        assert_good_and_bad_suggestions(
            "there was ideas",
            ThereIsAgreement::default(),
            &["there were ideas", "there was an idea"],
            &[],
        );
    }

    #[test]
    fn statement_past_sg() {
        assert_good_and_bad_suggestions(
            "there were child",
            ThereIsAgreement::default(),
            &["there were children", "there was a child"],
            &[],
        );
    }

    #[test]
    // NOTE suggests "mans" and "manes" due to how the dictionary & annotations work
    fn question_pres_sg() {
        assert_good_and_bad_suggestions(
            "are there man",
            ThereIsAgreement::default(),
            &["are there men", "is there a man"],
            &[],
        );
    }

    #[test]
    fn question_pres_pl() {
        assert_good_and_bad_suggestions(
            "is there women",
            ThereIsAgreement::default(),
            &["are there women", "is there a woman"],
            &[],
        );
    }

    #[test]
    fn question_past_sg() {
        assert_good_and_bad_suggestions(
            "were there cow",
            ThereIsAgreement::default(),
            &["were there cows", "was there a cow"],
            &[],
        );
    }

    #[test]
    fn question_past_pl() {
        assert_good_and_bad_suggestions(
            "were there elephant",
            ThereIsAgreement::default(),
            &["were there elephants", "was there an elephant"],
            &[],
        );
    }

    #[test]
    fn dont_flag_there_are_hyphenated_compound_starts_singular() {
        assert_no_lints(
            "there are function-like macros",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_there_are_open_compound_starts_singular() {
        assert_no_lints("there are config errors", ThereIsAgreement::default());
    }

    #[test]
    fn a_or_an_depends_on_dialect_herb() {
        assert_good_and_bad_suggestions(
            "there's herbs.",
            ThereIsAgreement::default(),
            &[
                "there are herbs.",
                // "there'ss a herb.", // TODO: This module does not yet accept a `Dialect`, `American` is hard-coded for now.
                "there's an herb.",
            ],
            &[],
        )
    }

    #[test]
    fn a_or_an_depends_on_dialect_hotel() {
        assert_good_and_bad_suggestions(
            "There are hotel.",
            ThereIsAgreement::default(),
            &[
                "There are hotels.",
                "There is a hotel.",
                // "There is an hotel.", // TODO: Dialect, preference, or both?
            ],
            &[],
        )
    }

    // Real-world tests

    // there is plural: fix

    #[test]
    fn fix_there_is_errors() {
        assert_good_and_bad_suggestions(
            "Hi， when I make the code, there is errors",
            ThereIsAgreement::default(),
            &[
                "Hi， when I make the code, there are errors",
                "Hi， when I make the code, there is an error",
            ],
            &["Hi， when I make the code, there is a error"],
        );
    }

    #[test]
    fn fix_there_is_warnings() {
        assert_good_and_bad_suggestions(
            "There is warnings from kotlin and dart, as reference: Elvis operator (?:) always returns the left operand of non-nullable type String.",
            ThereIsAgreement::default(),
            &[
                "There are warnings from kotlin and dart, as reference: Elvis operator (?:) always returns the left operand of non-nullable type String.",
                "There is a warning from kotlin and dart, as reference: Elvis operator (?:) always returns the left operand of non-nullable type String.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_is_problems() {
        assert_good_and_bad_suggestions(
            "Problem is that if there is a project that has a csproj file, then there is problems with the history folder.",
            ThereIsAgreement::default(),
            &[
                "Problem is that if there is a project that has a csproj file, then there are problems with the history folder.",
                "Problem is that if there is a project that has a csproj file, then there is a problem with the history folder.",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "`replace_with_match_case` matching case char-by-char causes the wrong letters to be uppercased"]
    fn fix_there_is_commands() {
        assert_good_and_bad_suggestions(
            "Additionally if there is Commands that can be used on multiple Resources at the same time.",
            ThereIsAgreement::default(),
            &[
                "Additionally if there are Commands that can be used on multiple Resources at the same time.",
                "Additionally if there is a Command that can be used on multiple Resources at the same time.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_is_values() {
        assert_suggestion_result(
            "This mean there would not be a single cache, but as many caches as there is values for the second argument.",
            ThereIsAgreement::default(),
            "This mean there would not be a single cache, but as many caches as there are values for the second argument.",
        );
    }

    #[test]
    fn fix_there_is_strings() {
        assert_good_and_bad_suggestions(
            "I can image other cases (tools different from SPSS) in which there is strings in both sides of the dictionary",
            ThereIsAgreement::default(),
            &[
                "I can image other cases (tools different from SPSS) in which there are strings in both sides of the dictionary",
                "I can image other cases (tools different from SPSS) in which there is a string in both sides of the dictionary",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_is_things() {
        assert_good_and_bad_suggestions(
            "even though we can check whether there is things running in Node, we can not do it for Chromium's message loops",
            ThereIsAgreement::default(),
            &[
                "even though we can check whether there are things running in Node, we can not do it for Chromium's message loops",
                "even though we can check whether there is a thing running in Node, we can not do it for Chromium's message loops",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_is_people() {
        // TODO `assert_good_and_bad_suggestions`` fails with more than two correct answers because
        // TODO it hasn't been converted to the new logic in `assert_suggestion_result`
        // assert_good_and_bad_suggestions(
        //     "there is people making projects, there is people doing tutorials",
        //     ThereIsAgreement::default(),
        //     &[
        //         "there are people making projects, there are people doing tutorials",
        //         "there is a person making projects, there is a person doing tutorials",
        //     ],
        //     &[],
        // );
        assert_suggestion_result(
            "there is people making projects, there is people doing tutorials",
            ThereIsAgreement::default(),
            "there are people making projects, there are people doing tutorials",
        );
        assert_suggestion_result(
            "there is people making projects, there is people doing tutorials",
            ThereIsAgreement::default(),
            "there is a person making projects, there is a person doing tutorials",
        );
    }

    #[test]
    fn fix_there_is_instructions() {
        assert_good_and_bad_suggestions(
            "I am just wondering if there is instructions somewhere for handling deep linking while using Redux.",
            ThereIsAgreement::default(),
            &[
                "I am just wondering if there are instructions somewhere for handling deep linking while using Redux.",
                "I am just wondering if there is an instruction somewhere for handling deep linking while using Redux.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_is_packages() {
        assert_suggestion_result(
            "if there is packages that handle such protocols installed",
            ThereIsAgreement::default(),
            "if there are packages that handle such protocols installed",
        );
    }

    // there is plural: don't flag

    #[test]
    // NOTE: This actually does pass because "who" is marked as a noun
    // NOTE:   because the dictionary is case-folded and "WHO" is a proper noun.
    #[ignore = "TODO: abort if next word is a relative pronoun like `who`"]
    fn dont_flag_there_is_people_who_have_done_xyz() {
        assert_no_lints(
            "The main expectation there is people who have a deprecated app installed will then get an error if it's disabled",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    #[ignore = "TODO: abort if next token is `/`"]
    fn dont_flag_there_is_packages_part_of_dir() {
        assert_no_lints(
            "For example there is packages/vite/src/node , but no packages/vite/src/deno",
            ThereIsAgreement::default(),
        );
    }

    // there are singular: fix

    #[test]
    fn fix_there_are_bug() {
        assert_good_and_bad_suggestions(
            "there are bug in svelte 3.0 that axios from 0.22.0 version undefined",
            ThereIsAgreement::default(),
            &[
                "there is a bug in svelte 3.0 that axios from 0.22.0 version undefined",
                "there are bugs in svelte 3.0 that axios from 0.22.0 version undefined",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_are_description() {
        assert_good_and_bad_suggestions(
            "there are description regarding thread safety in zmq document.",
            ThereIsAgreement::default(),
            &[
                "there is a description regarding thread safety in zmq document.",
                "there are descriptions regarding thread safety in zmq document.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_are_issue() {
        assert_good_and_bad_suggestions(
            "Seems like if there are issue with OpenAI, it is still trying to call chat completion and giving type error.",
            ThereIsAgreement::default(),
            &[
                "Seems like if there are issues with OpenAI, it is still trying to call chat completion and giving type error.",
                "Seems like if there is an issue with OpenAI, it is still trying to call chat completion and giving type error.",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "TODO: 'problem' is being rejected as an adjective as well as a singular noun"]
    fn fix_there_are_problem() {
        assert_good_and_bad_suggestions(
            "There are problem with official serving docker image gpu version",
            ThereIsAgreement::default(),
            &[
                "There is a problem with official serving docker image gpu version",
                "There are problems with official serving docker image gpu version",
            ],
            &[],
        );
    }

    // there are singular: don't flag

    #[test]
    fn dont_flag_there_are_dep_conflicts() {
        assert_no_lints(
            "Wrong advice when there are dependency conflicts",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_there_are_dep_errs() {
        assert_no_lints(
            "If there are dependency errors they will be immediately logged out to you.",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    #[ignore = "'definition syntax' looks singular but the full np is the plural 'definition syntax errors'"]
    fn dont_flag_there_are_def_syntax_errs() {
        assert_no_lints(
            "New Campaign Properties dialog loses changes if there are definition syntax errors",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_there_are_description_and_instruction_keys() {
        assert_no_lints(
            "There are description and instruction keys for the classes.",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_there_are_function_like() {
        assert_no_lints(
            "clang-format indents class member functions oddly if there are function-like macro invocations",
            ThereIsAgreement::default(),
        );
    }

    // there was pl: fix

    #[test]
    fn fix_there_was_configs() {
        assert_good_and_bad_suggestions(
            "I did see that there was configs for it before but it isn't in the configs anymore.",
            ThereIsAgreement::default(),
            &[
                "I did see that there were configs for it before but it isn't in the configs anymore.",
                "I did see that there was a config for it before but it isn't in the configs anymore.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_was_examples() {
        assert_good_and_bad_suggestions(
            "It would be awesome if there was examples on how to include inline citations",
            ThereIsAgreement::default(),
            &[
                "It would be awesome if there were examples on how to include inline citations",
                "It would be awesome if there was an example on how to include inline citations",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_was_functions() {
        assert_good_and_bad_suggestions(
            "I noticed in the AXP2101_Class for the unified library that there was functions like \"isPekeyShortPressIrq()\"",
            ThereIsAgreement::default(),
            &[
                "I noticed in the AXP2101_Class for the unified library that there were functions like \"isPekeyShortPressIrq()\"",
                "I noticed in the AXP2101_Class for the unified library that there was a function like \"isPekeyShortPressIrq()\"",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_was_issues() {
        assert_good_and_bad_suggestions(
            "Restored to a Snapshot, but there was issues with Hyprland.",
            ThereIsAgreement::default(),
            &[
                "Restored to a Snapshot, but there were issues with Hyprland.",
                "Restored to a Snapshot, but there was an issue with Hyprland.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_was_settings() {
        assert_good_and_bad_suggestions(
            "I also tried creating a fresh page on the same site, and there was settings but only 2 pages of settings",
            ThereIsAgreement::default(),
            &[
                "I also tried creating a fresh page on the same site, and there were settings but only 2 pages of settings",
                "I also tried creating a fresh page on the same site, and there was a setting but only 2 pages of settings",
            ],
            &[],
        );
    }

    // there were sg: fix

    #[test]
    fn fix_there_were_function() {
        assert_good_and_bad_suggestions(
            "Instead, it would be helpful if there were function in the autoloader capable of taking the a list of names of packages to enable.",
            ThereIsAgreement::default(),
            &[
                "Instead, it would be helpful if there were functions in the autoloader capable of taking the a list of names of packages to enable.",
                "Instead, it would be helpful if there was a function in the autoloader capable of taking the a list of names of packages to enable.",
                // TODO we don't support subjunctive "were" in contexts after "if"
                // "Instead, it would be helpful if there were a function in the autoloader capable of taking the a list of names of packages to enable.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_were_hint() {
        assert_good_and_bad_suggestions(
            "there were hint that this could break the build",
            ThereIsAgreement::default(),
            &[
                "there were hints that this could break the build",
                "there was a hint that this could break the build",
            ],
            &[],
        );
    }

    #[test]
    fn fix_there_were_issue() {
        assert_good_and_bad_suggestions(
            "there were issue with using venv and it was pulled",
            ThereIsAgreement::default(),
            &[
                "there were issues with using venv and it was pulled",
                "there was an issue with using venv and it was pulled",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "TODO: 'problem' is being rejected as an adjective as well as a singular noun"]
    fn fix_there_were_problem_about() {
        assert_good_and_bad_suggestions(
            "there were problem about version but this error is solved by correcting version in these files",
            ThereIsAgreement::default(),
            &[
                "there were problems about version but this error is solved by correcting version in these files",
                "there was a problem about version but this error is solved by correcting version in these files",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "TODO: 'problem' is being rejected as an adjective as well as a singular noun"]
    fn fix_there_were_problem_with() {
        assert_good_and_bad_suggestions(
            "there were problem with page alignment crossing",
            ThereIsAgreement::default(),
            &[
                "there were problems with page alignment crossing",
                "there was a problem with page alignment crossing",
            ],
            &[],
        );
    }

    // there were sg: don't flag

    #[test]
    // "two" is a technically a singular noun, but we avoid flagging numbers
    fn dont_flag_alice_there_were_two() {
        assert_no_lints(
            "This time there were two little shrieks, and more sounds of broken glass.",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_gatsby_there_were_twinkle_bells() {
        assert_no_lints(
            "When he realized what I was talking about, that there were twinkle-bells of sunshine in the room, he smiled like a weather man",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    #[ignore = "We can't detect first name + surname, especially for rare surnames like 'Waize'"]
    fn dont_flag_gatsby_there_were_a_and_b() {
        assert_no_lints(
            "Of theatrical people there were Gus Waize and Horace O’Donavan and Lester Myer and George Duckweed and Francis",
            ThereIsAgreement::default(),
        );
    }

    // is there plural

    #[test]
    fn fix_is_there_apps() {
        assert_good_and_bad_suggestions(
            "Ok, but is there apps that actually do that?",
            ThereIsAgreement::default(),
            &[
                "Ok, but are there apps that actually do that?",
                "Ok, but is there an app that actually do that?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_is_there_ideas() {
        assert_suggestion_result(
            "Is there ideas to make regular page to listen for api (POST/PUT/.. etc requests)",
            ThereIsAgreement::default(),
            "Are there ideas to make regular page to listen for api (POST/PUT/.. etc requests)",
        );
    }

    #[test]
    fn fix_is_there_people() {
        assert_suggestion_result(
            "please guys tell me, is there people really making money with bot?",
            ThereIsAgreement::default(),
            "please guys tell me, are there people really making money with bot?",
        );
    }

    #[test]
    fn fix_is_there_solutions() {
        assert_good_and_bad_suggestions(
            "Run-as binary without the suid bit set, is there solutions?",
            ThereIsAgreement::default(),
            &[
                "Run-as binary without the suid bit set, are there solutions?",
                "Run-as binary without the suid bit set, is there a solution?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_is_there_things() {
        assert_suggestion_result(
            "is there things you could change to make it a more general product",
            ThereIsAgreement::default(),
            "are there things you could change to make it a more general product",
        );
    }

    #[test]
    fn fix_is_there_tools() {
        assert_good_and_bad_suggestions(
            "Is there tools or documentation how to recover / rebuild /run fsck on the failed replicas.",
            ThereIsAgreement::default(),
            &[
                "Are there tools or documentation how to recover / rebuild /run fsck on the failed replicas.",
                "Is there a tool or documentation how to recover / rebuild /run fsck on the failed replicas.",
            ],
            &[],
        );
    }

    // are there singular: fix

    #[test]
    #[ignore = "TODO: 'problem' is being rejected as an adjective as well as a singular noun"]
    fn fix_are_there_problem() {
        assert_good_and_bad_suggestions(
            "Is it just the namespace or are there problem in the use statements as well?",
            ThereIsAgreement::default(),
            &[
                "Is it just the namespace or are there problems in the use statements as well?",
                "Is it just the namespace or is there a problem in the use statements as well?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_are_there_solution() {
        assert_good_and_bad_suggestions(
            "Are there solution for making lsws using h2 protocol in client browsers?",
            ThereIsAgreement::default(),
            &[
                "Are there solutions for making lsws using h2 protocol in client browsers?",
                "Is there a solution for making lsws using h2 protocol in client browsers?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_are_there_solution_slash_workaround() {
        assert_good_and_bad_suggestions(
            "are there solution/workaround ?",
            ThereIsAgreement::default(),
            &[
                // TODO the ideal fix pluralizes each word separated by `/`
                // "are there solutions/workarounds ?",
                "are there solutions/workaround ?",
                "is there a solution/workaround ?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_are_there_concept() {
        assert_good_and_bad_suggestions(
            "So, what is a external interface in C++? are there concept of interface in C++?",
            ThereIsAgreement::default(),
            &[
                "So, what is a external interface in C++? are there concepts of interface in C++?",
                "So, what is a external interface in C++? is there a concept of interface in C++?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_are_there_object() {
        assert_good_and_bad_suggestions(
            "He check are there object with same id, if there is no object with same id he creates new and add to array",
            ThereIsAgreement::default(),
            &[
                "He check are there objects with same id, if there is no object with same id he creates new and add to array",
                "He check is there an object with same id, if there is no object with same id he creates new and add to array",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "variable is being rejected as an adjective as well as a singular noun"]
    fn fix_are_there_variable() {
        assert_good_and_bad_suggestions(
            "Are there variable in side it or is it some sort of dataset?",
            ThereIsAgreement::default(),
            &[
                "Are there variables in side it or is it some sort of dataset?",
                "Is there a variable in side it or is it some sort of dataset?",
            ],
            &[],
        );
    }

    // are there singular: don't flag

    #[test]
    fn ignore_are_there_answer_generation_errors() {
        assert_no_lints(
            "Are there Answer Generation Errors?",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn ignore_are_there_application_objects() {
        assert_no_lints(
            "Are there application objects assigned to non-existent sites",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn ignore_are_there_code_files() {
        assert_no_lints(
            "Why are there code files more than 10k lines long?",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn ignore_are_there_error_logs() {
        assert_no_lints(
            "Are there error logs in your worker when flows fail?",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn ignore_are_there_tool_specific() {
        assert_no_lints(
            "Are there tool specific constraints for RM tool exchange for EA?",
            ThereIsAgreement::default(),
        );
    }

    // was there plural: fix

    #[test]
    fn fix_was_there_bugs() {
        assert_good_and_bad_suggestions(
            "Was there bugs, and goofed-up quests?",
            ThereIsAgreement::default(),
            &[
                "Were there bugs, and goofed-up quests?",
                "Was there a bug, and goofed-up quests?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_was_there_hints() {
        assert_good_and_bad_suggestions(
            "Was there hints in the game that points you to it?",
            ThereIsAgreement::default(),
            &[
                "Were there hints in the game that points you to it?",
                "Was there a hint in the game that points you to it?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_was_there_issues() {
        assert_good_and_bad_suggestions(
            "I saw you closed that PR, was there issues getting it merged upstream?",
            ThereIsAgreement::default(),
            &[
                "I saw you closed that PR, were there issues getting it merged upstream?",
                "I saw you closed that PR, was there an issue getting it merged upstream?",
            ],
            &[],
        );
    }

    #[test]
    fn fix_was_there_problems() {
        assert_good_and_bad_suggestions(
            "Was there problems with other files that you had written to the SD card (PRG/CRT)?",
            ThereIsAgreement::default(),
            &[
                "Were there problems with other files that you had written to the SD card (PRG/CRT)?",
                "Was there a problem with other files that you had written to the SD card (PRG/CRT)?",
            ],
            &[],
        );
    }

    // was there plural: don't flag

    #[test]
    fn dont_flag_last_time_i_was_there() {
        assert_no_lints(
            "Last time I was there flags were on almost every house or fence.",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    #[ignore = "Harper cannot detect this construction"]
    fn dont_flag_the_grate_was_there() {
        assert_no_lints(
            "Saying the grate was there helps him avoid a lawsuit.",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_when_he_was_there() {
        assert_no_lints(
            "When he was there tips were going way, way up.",
            ThereIsAgreement::default(),
        );
    }

    // were there sg: fix

    #[test]
    fn were_there_description() {
        assert_good_and_bad_suggestions(
            "Were there pictures of the Primarchs around before then? Were there description?",
            ThereIsAgreement::default(),
            &[
                "Were there pictures of the Primarchs around before then? Were there descriptions?",
                "Were there pictures of the Primarchs around before then? Was there a description?",
            ],
            &[],
        );
    }

    // were there sg: don't flag

    #[test]
    fn font_flag_were_there_bomb_drills() {
        assert_no_lints(
            "Were there bomb drills in school?",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_were_there_set_rules() {
        assert_no_lints(
            "For instance, were there set rules as to the funds that candidates would have to have acquired",
            ThereIsAgreement::default(),
        );
    }

    #[test]
    fn dont_flag_were_there_tour_buses() {
        assert_no_lints(
            "Were there tour buses and different hotels and parties each night? Yep.",
            ThereIsAgreement::default(),
        );
    }

    // there's plural

    #[test]
    fn fix_theres_children() {
        assert_good_and_bad_suggestions(
            "now, check whether there's children",
            ThereIsAgreement::default(),
            &[
                "now, check whether there's a child",
                "now, check whether there are children",
            ],
            &[],
        );
    }

    #[test]
    fn fix_theres_ideas() {
        assert_good_and_bad_suggestions(
            "there's ideas how it should behave for some media etc.",
            ThereIsAgreement::default(),
            &[
                "there's an idea how it should behave for some media etc.",
                "there are ideas how it should behave for some media etc.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_theres_people() {
        assert_good_and_bad_suggestions(
            "Currently there's people helping out, and it's fairly easy to find someone if you need something.",
            ThereIsAgreement::default(),
            &[
                "Currently there are people helping out, and it's fairly easy to find someone if you need something.",
                "Currently there's a person helping out, and it's fairly easy to find someone if you need something.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_theres_problems() {
        assert_good_and_bad_suggestions(
            "there's problems when using WSL::Ubuntu",
            ThereIsAgreement::default(),
            &[
                "there are problems when using WSL::Ubuntu",
                "there's a problem when using WSL::Ubuntu",
            ],
            &[],
        );
    }

    #[test]
    fn fix_theres_things() {
        assert_good_and_bad_suggestions(
            "If there's things you love/hate about Vidstack please let us know",
            ThereIsAgreement::default(),
            &[
                "If there are things you love/hate about Vidstack please let us know",
                "If there's a thing you love/hate about Vidstack please let us know",
            ],
            &[],
        );
    }

    #[test]
    fn fix_theres_urls() {
        assert_good_and_bad_suggestions(
            "so you're not suprised if there's urls missing or your data isn't being refreshed daily",
            ThereIsAgreement::default(),
            &[
                "so you're not suprised if there are urls missing or your data isn't being refreshed daily",
                "so you're not suprised if there's a url missing or your data isn't being refreshed daily",
                // "so you're not suprised if there's an url missing or your data isn't being refreshed daily",
            ],
            &[],
        );
    }
}

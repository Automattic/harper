use harper_brill::UPOS;

use crate::{
    CharStringExt, Dialect, Document, Token, TokenStringExt,
    indefinite_article::{InitialSound, starts_with_vowel},
};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct MissingIndefiniteArticle {
    dialect: Dialect,
}

impl MissingIndefiniteArticle {
    pub fn new(dialect: Dialect) -> Self {
        Self { dialect }
    }

    fn is_head(token: &Token, source: &[char]) -> bool {
        token.kind.is_upos(UPOS::NOUN)
            && token.kind.is_singular_noun()
            && !token.kind.is_plural_noun()
            && token.kind.is_countable_noun()
            && !token.kind.is_comparative_adjective()
            // Mixed count/mass nouns ("work", "water") also permit bare uses.
            && !token.kind.is_mass_noun()
            && !token.kind.is_proper_noun()
            && !token.kind.is_possessive_noun()
            // Tagging without an article can misread a predicate participle as
            // a noun ("is likely intended", "has finished").
            && !token.kind.is_verb_past_form()
            && !token.kind.is_verb_simple_past_form()
            && !token.kind.is_verb_past_participle_form()
            && !token.kind.is_verb_progressive_form()
            // Roles and certain idioms permit a bare singular noun: "became
            // president", "is home", "had lunch", "have trouble".
            && !token.get_ch(source).eq_any_ignore_ascii_case_str(&[
                "president", "chair", "chairman", "captain", "king", "queen", "head",
                "chief", "mayor", "director", "manager", "treasurer", "secretary", "owner",
                "home", "breakfast", "lunch", "dinner", "supper", "luncheon", "trouble", "part",
                "judge", "jury", "nephew", "niece", "mother", "father", "brother", "sister",
                "son", "daughter", "cousin",
                // These words have bare or predicate uses that are not
                // reliably distinguished by the dictionary and POS tagger.
                "plenty", "practice", "silence", "property", "alert", "operative", "subject",
                "kind", "permission", "flu", "rest", "courage", "exchange",
            ])
    }
}

impl Linter for MissingIndefiniteArticle {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let source = document.get_source();
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (index, verb) in chunk.iter().enumerate() {
                if !(verb.kind.is_upos(UPOS::VERB) || verb.kind.is_upos(UPOS::AUX))
                    || !verb.get_ch(source).eq_any_ignore_ascii_case_str(&[
                        "am", "is", "are", "was", "were", "be", "been", "being", "become",
                        "becomes", "became", "have", "has", "had", "need", "needs", "needed",
                        "want", "wants", "wanted", "buy", "buys", "bought",
                    ])
                {
                    continue;
                }

                let Some(space) = chunk.get(index + 1).filter(|t| t.kind.is_whitespace()) else {
                    continue;
                };
                let Some(first) = chunk.get(index + 2) else {
                    continue;
                };
                // Titles and names can have ordinary noun metadata ("Mr.").
                // Quantifier-like "such" needs a different insertion point.
                if first
                    .get_ch(source)
                    .first()
                    .is_some_and(|c| c.is_uppercase())
                    || first.kind.is_quantifier()
                    || matches!(
                        first.get_ch(source),
                        ['s', 'u', 'c', 'h'] | ['r', 'i', 'g', 'h', 't']
                    )
                {
                    continue;
                }
                let mut cursor = index + 2;
                let mut head = None;
                // Read the whole contiguous nominal phrase before deciding its
                // number: "book reviews" must not be treated as singular "book".
                while let Some(token) = chunk.get(cursor) {
                    if token.kind.is_determiner()
                        || token.kind.is_possessive_noun()
                        || (token.kind.is_proper_noun() && !token.kind.is_upos(UPOS::ADJ))
                        || !(token.kind.is_upos(UPOS::ADJ)
                            || token.kind.is_upos(UPOS::NOUN)
                            || (token.kind.is_upos(UPOS::PROPN) && token.kind.is_noun()))
                    {
                        head = None;
                        break;
                    }
                    if token.kind.is_comparative_adjective() {
                        head = None;
                        break;
                    }
                    head = Some(token);
                    if !chunk
                        .get(cursor + 1)
                        .is_some_and(|t| t.kind.is_whitespace())
                        || !chunk.get(cursor + 2).is_some_and(|t| {
                            t.kind.is_upos(UPOS::NOUN)
                                || t.kind.is_upos(UPOS::PROPN)
                                || t.kind.is_upos(UPOS::ADJ)
                        })
                    {
                        break;
                    }
                    cursor += 2;
                }
                if !head.is_some_and(|head| Self::is_head(head, source))
                    || chunk
                        .get(cursor + 2)
                        .is_some_and(|t| t.get_ch(source).eq_any_ignore_ascii_case_str(&["enough"]))
                {
                    continue;
                }

                let articles: &[&str] = match starts_with_vowel(first.get_ch(source), self.dialect)
                {
                    Some(InitialSound::Vowel) => &["an "],
                    Some(InitialSound::Consonant) => &["a "],
                    Some(InitialSound::Either) => &["a ", "an "],
                    None => continue,
                };
                lints.push(Lint {
                    span: space.span,
                    lint_kind: LintKind::Grammar,
                    suggestions: articles
                        .iter()
                        .map(|article| Suggestion::InsertAfter(article.chars().collect()))
                        .collect(),
                    message: "This singular count noun may need `a` or `an`.".to_owned(),
                    priority: 30,
                });
            }
        }
        lints
    }

    fn description(&self) -> &'static str {
        "Suggests missing `a` or `an` before singular count noun phrases after common verbs. Skips mass nouns, plurals, proper nouns, and common bare roles."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn teacher() {
        assert_suggestion_result(
            "She is teacher.",
            MissingIndefiniteArticle::new(Dialect::American),
            "She is a teacher.",
        );
    }

    #[test]
    fn engineer() {
        assert_suggestion_result(
            "He is engineer.",
            MissingIndefiniteArticle::new(Dialect::American),
            "He is an engineer.",
        );
    }

    #[test]
    fn idea() {
        assert_suggestion_result(
            "This is good idea.",
            MissingIndefiniteArticle::new(Dialect::American),
            "This is a good idea.",
        );
    }

    #[test]
    fn book() {
        assert_suggestion_result(
            "I bought book.",
            MissingIndefiniteArticle::new(Dialect::American),
            "I bought a book.",
        );
    }

    #[test]
    fn umbrella() {
        assert_suggestion_result(
            "We need umbrella.",
            MissingIndefiniteArticle::new(Dialect::American),
            "We need an umbrella.",
        );
    }

    #[test]
    fn doctor() {
        assert_suggestion_result(
            "She became doctor.",
            MissingIndefiniteArticle::new(Dialect::American),
            "She became a doctor.",
        );
    }

    #[test]
    fn compound() {
        assert_suggestion_result(
            "He is university student.",
            MissingIndefiniteArticle::new(Dialect::American),
            "He is a university student.",
        );
    }

    #[test]
    fn silent_h() {
        assert_suggestion_result(
            "It is honest mistake.",
            MissingIndefiniteArticle::new(Dialect::American),
            "It is an honest mistake.",
        );
    }

    #[test]
    fn car() {
        assert_suggestion_result(
            "I have car.",
            MissingIndefiniteArticle::new(Dialect::American),
            "I have a car.",
        );
    }

    #[test]
    fn accident() {
        assert_suggestion_result(
            "It was accident.",
            MissingIndefiniteArticle::new(Dialect::American),
            "It was an accident.",
        );
    }

    #[test]
    fn hour() {
        assert_suggestion_result(
            "We need hour.",
            MissingIndefiniteArticle::new(Dialect::American),
            "We need an hour.",
        );
    }

    #[test]
    fn adjectives() {
        assert_suggestion_result(
            "This is interesting new idea.",
            MissingIndefiniteArticle::new(Dialect::American),
            "This is an interesting new idea.",
        );
    }

    #[test]
    fn whitespace() {
        assert_suggestion_result(
            "I need\tcar.",
            MissingIndefiniteArticle::new(Dialect::American),
            "I need\ta car.",
        );
    }

    #[test]
    fn capitalization() {
        assert_suggestion_result(
            "She IS teacher.",
            MissingIndefiniteArticle::new(Dialect::American),
            "She IS a teacher.",
        );
    }

    #[test]
    fn unicode_prefix() {
        assert_suggestion_result(
            "Élodie has car.",
            MissingIndefiniteArticle::new(Dialect::American),
            "Élodie has a car.",
        );
    }

    #[test]
    fn supports_consonant_pronunciation() {
        assert_suggestion_result(
            "This is ubiquitous problem.",
            MissingIndefiniteArticle::new(Dialect::American),
            "This is a ubiquitous problem.",
        );
    }

    #[test]
    fn supports_vowel_pronunciation() {
        assert_suggestion_result(
            "This is ubiquitous problem.",
            MissingIndefiniteArticle::new(Dialect::American),
            "This is an ubiquitous problem.",
        );
    }

    #[test]
    fn curated_rule_inserts_once() {
        use crate::{linting::LintGroup, spell::FstDictionary};
        assert_suggestion_result(
            "He is engineer.",
            LintGroup::new_curated(FstDictionary::curated(), Dialect::American),
            "He is an engineer.",
        );
    }

    #[test]
    fn allows_indefinite_article_a() {
        assert_no_lints(
            "She is a teacher.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_indefinite_article_an() {
        assert_no_lints(
            "He is an engineer.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_definite_article() {
        assert_no_lints(
            "This is the idea.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_possessive_determiner() {
        assert_no_lints(
            "I have my car.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_quantified_plural() {
        assert_no_lints(
            "We need some books.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_plural_predicate() {
        assert_no_lints(
            "They are teachers.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_plural_compound() {
        assert_no_lints(
            "I bought book reviews.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_happy() {
        assert_no_lints(
            "He is happy.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_advice() {
        assert_no_lints(
            "They need advice.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_water() {
        assert_no_lints(
            "This is water.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_work() {
        assert_no_lints(
            "We have work.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_president() {
        assert_no_lints(
            "She became president.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_home() {
        assert_no_lints(
            "He is home.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_lunch() {
        assert_no_lints(
            "We had lunch.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_proper_noun() {
        assert_no_lints(
            "This is London.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_possessive_noun() {
        assert_no_lints(
            "I have John's book.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_perfect_progressive() {
        assert_no_lints(
            "She has been working.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_infinitive() {
        assert_no_lints(
            "I need to go.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_likely_intended() {
        assert_no_lints(
            "It is likely intended.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_finished() {
        assert_no_lints(
            "She has finished.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_running() {
        assert_no_lints(
            "He is running.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_part_of() {
        assert_no_lints(
            "It is part of a compound.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_mayor() {
        assert_no_lints(
            "She became mayor.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_plenty_of() {
        assert_no_lints(
            "She had plenty of time.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_good_practice() {
        assert_no_lints(
            "It was good practice to say it.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_judge_and_jury() {
        assert_no_lints(
            "I'll be judge, I'll be jury.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_silence() {
        assert_no_lints(
            "There was silence.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_postpositive_enough() {
        assert_no_lints(
            "There was mouth enough for it to speak.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_title_in_question() {
        assert_no_lints(
            "Is Mr. Smith in?",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_alert() {
        assert_no_lints(
            "He were alert for something.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_operative() {
        assert_no_lints(
            "It becomes operative from today.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_party() {
        assert_no_lints(
            "They shall be Party.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_subject_for() {
        assert_no_lints(
            "No person shall be subject for the offense.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_comparative() {
        assert_no_lints(
            "Laughter is easier minute by minute.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_nephew() {
        assert_no_lints(
            "He was nephew to the king.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_right_next_door() {
        assert_no_lints(
            "The house is right next door.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_common_property() {
        assert_no_lints(
            "They were common property.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_luncheon() {
        assert_no_lints(
            "We had luncheon.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_such_majority() {
        assert_no_lints(
            "They have such majority.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_unchecked_as_phrase() {
        assert_no_lints(
            "She works as nurse.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_institutional_school() {
        assert_no_lints(
            "Go to school.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_empty_input() {
        assert_no_lints("", MissingIndefiniteArticle::new(Dialect::American));
    }

    #[test]
    fn allows_standalone_noun() {
        assert_no_lints("teacher", MissingIndefiniteArticle::new(Dialect::American));
    }

    #[test]
    fn allows_good() {
        assert_no_lints(
            "This is good.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_software() {
        assert_no_lints(
            "We need software.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_trouble() {
        assert_no_lints(
            "I have trouble.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_intervening_punctuation() {
        assert_no_lints(
            "She is, teacher.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_inline_code() {
        assert_no_lints(
            "I need `car`.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_fine() {
        assert_no_lints(
            "I am fine.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_well() {
        assert_no_lints(
            "He is well.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_kind() {
        assert_no_lints(
            "She is kind.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_great() {
        assert_no_lints(
            "This is great.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_nice() {
        assert_no_lints(
            "It is nice.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_tall() {
        assert_no_lints(
            "He is tall.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_short() {
        assert_no_lints(
            "She is short.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_ready() {
        assert_no_lints(
            "It is ready.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_smart() {
        assert_no_lints(
            "She is smart.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_safe() {
        assert_no_lints(
            "It is safe.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_calm() {
        assert_no_lints(
            "He is calm.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_quiet() {
        assert_no_lints(
            "She is quiet.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_right() {
        assert_no_lints(
            "He is right.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_wrong() {
        assert_no_lints(
            "She is wrong.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_mine() {
        assert_no_lints(
            "It is mine.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_yours() {
        assert_no_lints(
            "This is yours.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_permission() {
        assert_no_lints(
            "We have permission.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_access() {
        assert_no_lints(
            "They have access.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_cancer() {
        assert_no_lints(
            "He has cancer.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_flu() {
        assert_no_lints(
            "She has flu.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_hope() {
        assert_no_lints(
            "I have hope.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_sleep() {
        assert_no_lints(
            "We need sleep.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_help() {
        assert_no_lints(
            "We need help.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_peace() {
        assert_no_lints(
            "I want peace.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_rest() {
        assert_no_lints(
            "They need rest.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_in_bed() {
        assert_no_lints(
            "He is in bed.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_at_school() {
        assert_no_lints(
            "He is at school.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_milk() {
        assert_no_lints(
            "This is milk.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_gold() {
        assert_no_lints(
            "This is gold.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_wood() {
        assert_no_lints(
            "It is wood.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_courage() {
        assert_no_lints(
            "She has courage.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_famous() {
        assert_no_lints(
            "He became famous.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_breakfast() {
        assert_no_lints(
            "They have breakfast.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_faith() {
        assert_no_lints(
            "I have faith.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_father() {
        assert_no_lints(
            "He is father to three children.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_mother() {
        assert_no_lints(
            "She is mother of twins.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_proof() {
        assert_no_lints(
            "This is proof of success.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_winter() {
        assert_no_lints(
            "It is winter.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_spring() {
        assert_no_lints(
            "It was spring.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_night() {
        assert_no_lints(
            "It is night.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_nonsense() {
        assert_no_lints(
            "This is nonsense.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_business() {
        assert_no_lints(
            "This is business.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_news() {
        assert_no_lints(
            "It is good news.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_company() {
        assert_no_lints(
            "We have company.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_life() {
        assert_no_lints(
            "This is real life.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_in_love() {
        assert_no_lints(
            "They are in love.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_laughter() {
        assert_no_lints(
            "There was laughter.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_joy() {
        assert_no_lints(
            "There was joy.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_experience() {
        assert_no_lints(
            "He has experience.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_second_in_command() {
        assert_no_lints(
            "He is second in command.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_professor() {
        assert_no_lints(
            "She is professor of physics.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_impossible() {
        assert_no_lints(
            "It is impossible.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_available() {
        assert_no_lints(
            "He is available.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_gone() {
        assert_no_lints(
            "She has gone.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_happened() {
        assert_no_lints(
            "It has happened.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_left() {
        assert_no_lints(
            "They have left.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_arrived() {
        assert_no_lints(
            "We have arrived.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_asleep() {
        assert_no_lints(
            "He is asleep.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_awake() {
        assert_no_lints(
            "She is awake.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_alive() {
        assert_no_lints(
            "We are alive.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_alone() {
        assert_no_lints(
            "He is alone.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_worth() {
        assert_no_lints(
            "It is worth the effort.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_done() {
        assert_no_lints(
            "This is done.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }

    #[test]
    fn allows_exchange() {
        assert_no_lints(
            "There has been exchange of ideas.",
            MissingIndefiniteArticle::new(Dialect::American),
        );
    }
}

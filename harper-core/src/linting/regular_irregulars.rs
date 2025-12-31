use crate::Token;
use crate::char_string::CharStringExt;
use crate::expr::Expr;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk};
use crate::spell::Dictionary;

use hashbrown::HashSet;
use phf::phf_map;

static IRREG_VERBS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "arise" => &["arose", "arisen"],
    "awake" => &["awoke", "awoken"],
    "become" => &["became", "become"],
    "begin" => &["began", "begun"],
    "bid" => &["bade", "bidden"],
    "bite" => &["bit", "bitten"],
    "blow" => &["blew", "blown"],
    "break" => &["broke", "broken"],
    "bring" => &["brought"],
    "build" => &["built"],
    "buy" => &["bought"],
    "catch" => &["caught"],
    "choose" => &["chose", "chosen"],
    "come" => &["came", "come"],
    "cost" => &["cost"],
    "cut" => &["cut"],
    "do" => &["did", "done"],
    "drink" => &["drank", "drunk"],
    "drive" => &["drove", "driven"],
    "eat" => &["ate", "eaten"],
    "fall" => &["fell", "fallen"],
    "feed" => &["fed"],
    "feel" => &["felt"],
    "give" => &["gave", "given"],
    "go" => &["went", "gone"],
    "grow" => &["grew", "grown"],
    "have" => &["had"],
    "hear" => &["heard"],
    "hit" => &["hit"],
    "know" => &["knew", "known"],
    "lead" => &["led"],
    "mistake" => &["mistook", "mistaken"],
    "output" => &["output"],
    "overtake" => &["overtook", "overtaken"],
    "partake" => &["partook", "partaken"],
    "pay" => &["paid"],
    "prove" => &["proved", "proven"],
    "put" => &["put"],
    "read" => &["read"],
    "reset" => &["reset"],
    "ride" => &["rode", "ridden"],
    "ring" => &["rang", "rung"],
    "rise" => &["rose", "risen"],
    "run" => &["ran", "run"],
    "see" => &["saw", "seen"],
    "set" => &["set"],
    "sew" => &["sewed", "sewn"],
    "sing" => &["sang", "sung"],
    "slay" => &["slayed", "slain"],
    "sleep" => &["slept"],
    "slide" => &["slid"],
    "speak" => &["spoke", "spoken"],
    "spring" => &["sprang", "sprung"],
    "stand" => &["stood"],
    "steal" => &["stole", "stolen"],
    "stink" => &["stank", "stunk"],
    "swear" => &["swore", "sworn"],
    "swim" => &["swam", "swum"],
    "take" => &["took", "taken"],
    "think" => &["thought"],
    "throw" => &["threw", "thrown"],
    "tread" => &["trod", "trodden"],
    "wake" => &["woke", "woken"],
    "weave" => &["wove", "woven"],
    "write" => &["wrote", "written"],
};

static IRREG_NOUNS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "axis" => &["axes"],
    "biceps" => &["biceps"],
    "child" => &["children"],
    "deer" => &["deer"],
    "foot" => &["feet"],
    "goose" => &["geese"],
    "man" => &["men"],
    "mouse" => &["mice"],
    "ox" => &["oxen"],
    "person" => &["people"],
    "sheep" => &["sheep"],
    "species" => &["species"],
    "thesis" => &["theses"],
    "tooth" => &["teeth"],
    "woman" => &["women"],
};

static IRREG_COMPAR: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "good" => &["better"],
    "far" => &["farther", "further"],
    "well" => &["better"],
    "bad" => &["worse"],
};

static IRREG_SUPER: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "good" => &["best"],
    "far" => &["farthest", "furthest"],
    "well" => &["best"],
    "bad" => &["worst"],
};

pub struct RegularIrregulars<D> {
    exp: Box<dyn Expr>,
    dict: D,
}

impl<D> RegularIrregulars<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        Self {
            exp: Box::new(|tok: &Token, src: &[char]| {
                let w = tok.span.get_content(src);
                (w.ends_with_ignore_ascii_case_str("s") && !tok.kind.is_plural_noun())
                    || (w.ends_with_ignore_ascii_case_str("ed")
                        && !tok.kind.is_verb_simple_past_form()
                        && !tok.kind.is_verb_past_participle_form())
                    || (w.ends_with_ignore_ascii_case_str("er")
                        && !tok.kind.is_comparative_adjective())
                    || (w.ends_with_ignore_ascii_case_str("est")
                        && !tok.kind.is_superlative_adjective())
            }),
            dict,
        }
    }
}

impl<D> ExprLinter for RegularIrregulars<D>
where
    D: Dictionary,
{
    type Unit = Chunk;

    fn description(&self) -> &'static str {
        "Replaces wrong regular inflections of words with their correct irregular forms."
    }

    fn expr(&self) -> &dyn Expr {
        self.exp.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 1 {
            return None;
        }
        let bad_tok = &toks[0];
        let bad_span = bad_tok.span;
        let bad_chars = bad_span.get_content(src);
        let bad_word = bad_span.get_content_string(src);

        let mut suggs: HashSet<String> = HashSet::new();

        if bad_chars.ends_with_ignore_ascii_case_str("s") {
            // Irregular plurals in -s: childs -> children etc.
            let key = &bad_word[..bad_word.len() - 1];
            if let Some(&forms) = IRREG_NOUNS.get(key) {
                suggs.extend(forms.iter().map(|s| s.to_string()));
            }
            // Irregular plurals in -es: oxes -> oxen etc.
            if bad_chars.ends_with_ignore_ascii_case_str("es") {
                let key = &bad_word[..bad_word.len() - 2];
                if let Some(&forms) = IRREG_NOUNS.get(key) {
                    suggs.extend(forms.iter().map(|s| s.to_string()));
                }
            }

            // Regular plurals with -fe to -ves stem change: knife -> knives etc.
            if bad_chars.ends_with_ignore_ascii_case_str("fes") {
                let w = format!("{}ves", &bad_word[..bad_word.len() - 3]);
                if self.dict.contains_word_str(&w) {
                    suggs.insert(w);
                }
            }
            // Regular plurals with -f to -ves stem change: calf -> calves etc.
            if bad_chars.ends_with_ignore_ascii_case_str("fs") {
                let w = format!("{}ves", &bad_word[..bad_word.len() - 2]);
                if self.dict.contains_word_str(&w) {
                    suggs.insert(w);
                }
            }
            // Regular plurals with -o to -oes stem change: tomato -> tomatoes etc.
            if bad_chars.ends_with_ignore_ascii_case_str("os") {
                let w = format!("{}oes", &bad_word[..bad_word.len() - 2]);
                if self.dict.contains_word_str(&w) {
                    suggs.insert(w);
                }
            }
            // Regular plurals with -y to -ies stem change: sky -> skies etc.
            if bad_chars.ends_with_ignore_ascii_case_str("ys") {
                let w = format!("{}ies", &bad_word[..bad_word.len() - 2]);
                if self.dict.contains_word_str(&w) {
                    suggs.insert(w);
                }
            }
        } else if bad_chars.ends_with_ignore_ascii_case_str("ed") {
            // Irregular verbs in -ed: childs -> children etc.
            let key = &bad_word[..bad_chars.len() - 2];
            if let Some(&forms) = IRREG_VERBS.get(key) {
                suggs.extend(forms.iter().map(|s| s.to_string()));
            }
            // Irregular verbs in -ed after double letter: resetted -> reset etc.
            let stem_chars = &bad_chars[..bad_chars.len() - 2];
            if stem_chars.len() >= 2
                && stem_chars[stem_chars.len() - 2] == stem_chars[stem_chars.len() - 1]
            {
                let key = &bad_word[..bad_word.len() - 3];
                if let Some(&forms) = IRREG_VERBS.get(key) {
                    suggs.extend(forms.iter().map(|s| s.to_string()));
                }
            }
        } else if bad_chars.ends_with_ignore_ascii_case_str("er") {
            // Irregular comparatives: gooder -> better etc.
            let key = &bad_word[..bad_chars.len() - 2];
            if let Some(&forms) = IRREG_COMPAR.get(key) {
                suggs.extend(forms.iter().map(|s| s.to_string()));
            }
        } else if bad_chars.ends_with_ignore_ascii_case_str("est") {
            // Irregular superlatives: goodest -> best etc.
            let key = &bad_word[..bad_chars.len() - 3];
            if let Some(&forms) = IRREG_SUPER.get(key) {
                suggs.extend(forms.iter().map(|s| s.to_string()));
            }
        }

        let suggestions: Vec<_> = suggs
            .iter()
            .map(|good_str| {
                Suggestion::replace_with_match_case(good_str.chars().collect(), bad_chars)
            })
            .collect();

        if suggestions.is_empty() {
            return None;
        }
        let irregulars = suggs
            .iter()
            .map(|s| format!("'{}'", s))
            .collect::<Vec<_>>()
            .join(" or ");

        Some(Lint {
            span: bad_span,
            lint_kind: LintKind::Grammar,
            message: format!(
                "Use the irregular form {} instead of '{}'",
                irregulars, bad_word
            ),
            suggestions,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RegularIrregulars;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_top3_suggestion_result};
    use crate::spell::FstDictionary;

    #[test]
    #[ignore = "correct combo not in the top3 - `assert_any_suggestion_result` needed!"]
    fn fix_irregular_past_verb() {
        assert_top3_suggestion_result(
            "I eated the banana.",
            RegularIrregulars::new(FstDictionary::curated()),
            "I ate the banana.",
        );
    }

    #[test]
    #[ignore = "correct combo not in the top3 - `assert_any_suggestion_result` needed!"]
    fn fix_irregular_plural_nouns() {
        assert_top3_suggestion_result(
            "All mans, womans, and childs are equal.",
            RegularIrregulars::new(FstDictionary::curated()),
            "All men, women, and children are equal.",
        );
    }

    #[test]
    fn fix_noun_plurals_with_stem_changes() {
        assert_top3_suggestion_result(
            "The puppys and kittys are playing in the leafs. The babys of oxes are called calfs.",
            RegularIrregulars::new(FstDictionary::curated()),
            "The puppies and kitties are playing in the leaves. The babies of oxen are called calves.",
        );
    }

    #[test]
    fn fix_noun_plurals_with_plurals_in_oes() {
        assert_top3_suggestion_result(
            "I can hear the echos of the heros as I pass the volcanos",
            RegularIrregulars::new(FstDictionary::curated()),
            "I can hear the echos of the heroes as I pass the volcanoes",
        );
    }

    #[test]
    fn fix_reset_and_knives() {
        assert_top3_suggestion_result(
            "I resetted the electronic knifes.",
            RegularIrregulars::new(FstDictionary::curated()),
            "I reset the electronic knives.",
        );
    }

    #[test]
    fn fix_adjectives() {
        assert_good_and_bad_suggestions(
            "This way is farer.",
            RegularIrregulars::new(FstDictionary::curated()),
            &["This way is farther.", "This way is further."],
            &[],
        );
    }
}

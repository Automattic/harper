use crate::{
    Token,
    char_string::CharStringExt,
    expr::Expr,
    irregular_nouns::IrregularNouns,
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};
use hashbrown::HashSet;

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
                tok.kind.is_oov()
                    && tok
                        .span
                        .get_content(src)
                        .ends_with_any_ignore_ascii_case_chars(&[
                            &['s'],
                            &['e', 'd'],
                            &['e', 'r'],
                            &['e', 's', 't'],
                        ])
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
        let tok = &toks[0];
        let span = tok.span;
        let chars = span.get_content(src);
        let word = span.get_content_string(src);

        let mut suggs: HashSet<String> = HashSet::new();

        handle_plural_nouns(&self.dict, &mut suggs, chars);
        handle_past_verbs(&self.dict, &mut suggs, chars);
        handle_adjectives(&mut suggs, chars);

        let suggestions: Vec<_> = suggs
            .iter()
            .map(|good_str| Suggestion::replace_with_match_case(good_str.chars().collect(), chars))
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
            span,
            lint_kind: LintKind::Grammar,
            message: format!(
                "Use the irregular form {} instead of '{}'",
                irregulars, word
            ),
            suggestions,
            ..Default::default()
        })
    }
}

fn get_irreg_compar(word: &str) -> Option<&'static [&'static str]> {
    match word {
        "good" => Some(&["better"]),
        "far" => Some(&["farther", "further"]),
        "well" => Some(&["better"]),
        "bad" => Some(&["worse"]),
        _ => None,
    }
}

fn get_irreg_super(word: &str) -> Option<&'static [&'static str]> {
    match word {
        "good" => Some(&["best"]),
        "far" => Some(&["farthest", "furthest"]),
        "well" => Some(&["best"]),
        "bad" => Some(&["worst"]),
        _ => None,
    }
}

fn handle_adjectives(suggs: &mut HashSet<String>, chars: &[char]) {
    if chars.ends_with_ignore_ascii_case_str("er") {
        // Irregular comparatives: gooder -> better etc.
        let key = chars[..chars.len() - 2].iter().collect::<String>();
        if let Some(forms) = get_irreg_compar(&key) {
            suggs.extend(forms.iter().map(|s| s.to_string()));
        }
    }

    if chars.ends_with_ignore_ascii_case_str("est") {
        // Irregular superlatives: goodest -> best etc.
        let key = chars[..chars.len() - 2].iter().collect::<String>();
        if let Some(forms) = get_irreg_super(&key) {
            suggs.extend(forms.iter().map(|s| s.to_string()));
        }
    }
}

fn handle_plural_nouns(dict: &dyn Dictionary, suggs: &mut HashSet<String>, chars: &[char]) {
    let mut plurals = vec![];
    let mut sg_candidates = vec![];

    if let Some(drop_s) = chars.strip_suffix(&['s']) {
        sg_candidates.push(drop_s);

        if let Some(drop_es) = drop_s.strip_suffix(&['e']) {
            sg_candidates.push(drop_es);
        }

        let singulars = sg_candidates.iter().filter(|sg| {
            dict.get_word_metadata(sg)
                .is_some_and(|m| m.is_singular_noun())
        });

        singulars.into_iter().for_each(|sg| {
            // irregular plurals
            if let Some(pl) =
                IrregularNouns::curated().get_plural_for_singular(&sg.iter().collect::<String>())
            {
                plurals.push(pl.chars().collect());
            }

            // skys -> sky -> skies etc.
            if let Some(drop_y) = sg.strip_suffix(&['y']) {
                let mut add_ies = drop_y.to_vec();
                add_ies.extend(['i', 'e', 's']);
                if dict
                    .get_word_metadata(&add_ies)
                    .is_some_and(|m| m.is_plural_noun())
                {
                    plurals.push(add_ies);
                }
            }

            // knifes -> knife -> knives etc. (No because "knifes" is a legit verb)
            if let Some(drop_fe) = sg.strip_suffix(&['f', 'e']) {
                let mut add_ves = drop_fe.to_vec();
                add_ves.extend(['v', 'e', 's']);
                if dict
                    .get_word_metadata(&add_ves)
                    .is_some_and(|m| m.is_plural_noun())
                {
                    plurals.push(add_ves);
                }
            }
            // calfs -> calf -> calves etc.
            if let Some(drop_f) = sg.strip_suffix(&['f']) {
                let mut add_ves = drop_f.to_vec();
                add_ves.extend(['v', 'e', 's']);
                if dict
                    .get_word_metadata(&add_ves)
                    .is_some_and(|m| m.is_plural_noun())
                {
                    plurals.push(add_ves);
                }
            }
            // tomatos -> tomato -> tomatoes etc.
            // TODO: boxs -> box -> boxes ?
            if sg.ends_with_ignore_ascii_case_chars(&['o']) {
                let mut add_es = sg.to_vec();
                add_es.extend(['e', 's']);
                if dict
                    .get_word_metadata(&add_es)
                    .is_some_and(|m| m.is_plural_noun())
                {
                    plurals.push(add_es);
                }
            }

            // TODO are there words which double the last consonant to pluralize? gas? whiz?
        });

        plurals.iter().for_each(|pl| {
            suggs.insert(pl.iter().collect());
        });
    }
}

fn handle_past_verbs(dict: &dyn Dictionary, suggs: &mut HashSet<String>, chars: &[char]) {
    let mut pasts: HashSet<Vec<char>> = HashSet::new();
    let mut vp_candidates = vec![];

    if chars.ends_with_ignore_ascii_case_chars(&['e', 'd']) {
        vp_candidates.push(&chars[..chars.len() - 2]); // drop_ed
        vp_candidates.push(&chars[..chars.len() - 1]); // drop_d

        // Handle doubled consonant before -ed (e.g., "resetted" -> "reset")
        // Check if position len-3 and len-4 are same char (the doubled consonant)
        if chars.len() >= 5 && chars[chars.len() - 3] == chars[chars.len() - 4] {
            vp_candidates.push(&chars[..chars.len() - 3]); // drop one of the doubled consonants + ed
        }
    }

    let lemmata = vp_candidates.into_iter().filter(|vp| {
        dict.get_word_metadata(vp)
            .is_some_and(|m| m.is_verb_lemma())
    });

    lemmata.into_iter().for_each(|lem| {
        let lem_str = lem.iter().collect::<String>();

        // Irregular verbs
        if let Some((pt, pp)) = IrregularVerbs::curated().get_pasts_for_lemma(&lem_str) {
            pasts.insert(pt.chars().collect());
            pasts.insert(pp.chars().collect());
        }
    });

    pasts.iter().for_each(|p| {
        suggs.insert(p.iter().collect());
    });
}

#[cfg(test)]
mod tests {
    mod nouns {
        use super::super::RegularIrregulars;
        use crate::linting::tests::assert_suggestion_result;
        use crate::spell::FstDictionary;

        #[test]
        fn fix_irregulars() {
            assert_suggestion_result(
                "Womans and childs first",
                RegularIrregulars::new(FstDictionary::curated()),
                "Women and children first",
            );
        }

        #[test]
        fn fix_ys_and_fs() {
            assert_suggestion_result(
                "Kittys playing on the shelfs.",
                RegularIrregulars::new(FstDictionary::curated()),
                "Kitties playing on the shelves.",
            );
        }

        #[test]
        fn fix_os_and_oes() {
            assert_suggestion_result(
                "The heros climb the volcanos",
                RegularIrregulars::new(FstDictionary::curated()),
                "The heroes climb the volcanoes",
            );
        }

        #[test]
        fn fix_oxen_and_meatloaves() {
            assert_suggestion_result(
                "These meatloafs are made out of oxes.",
                RegularIrregulars::new(FstDictionary::curated()),
                "These meatloaves are made out of oxen.",
            );
        }
    }

    mod verbs {
        use super::super::RegularIrregulars;
        use crate::linting::tests::assert_suggestion_result;
        use crate::spell::FstDictionary;

        #[test]
        fn fix_irregular_past_verb() {
            assert_suggestion_result(
                "I eated the banana.",
                RegularIrregulars::new(FstDictionary::curated()),
                "I ate the banana.",
            );
        }

        #[test]
        fn fix_readed() {
            assert_suggestion_result(
                "He readed the newspaper",
                RegularIrregulars::new(FstDictionary::curated()),
                "He read the newspaper",
            );
        }

        #[test]
        fn fix_writed() {
            assert_suggestion_result(
                "She writed many lines of code.",
                RegularIrregulars::new(FstDictionary::curated()),
                "She wrote many lines of code.",
            );
        }

        #[test]
        fn fix_runned() {
            assert_suggestion_result(
                "I runned faster than ever!",
                RegularIrregulars::new(FstDictionary::curated()),
                "I ran faster than ever!",
            );
        }

        #[test]
        fn fix_resetted() {
            assert_suggestion_result(
                "I resetted the phone to factory settings.",
                RegularIrregulars::new(FstDictionary::curated()),
                "I reset the phone to factory settings.",
            );
        }

        #[test]
        fn fix_eat_drink_sleep() {
            assert_suggestion_result(
                "I eated and drinked too much but I sleeped good.",
                RegularIrregulars::new(FstDictionary::curated()),
                "I ate and drank too much but I slept good.",
            );
        }
    }

    mod adjectives {
        use super::super::RegularIrregulars;
        use crate::linting::tests::assert_good_and_bad_suggestions;
        use crate::spell::FstDictionary;

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
}

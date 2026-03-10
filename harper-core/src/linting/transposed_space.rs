use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

pub struct TransposedSpace<D: Dictionary + 'static> {
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D: Dictionary + 'static> TransposedSpace<D> {
    pub fn new(dict: D) -> Self {
        Self {
            expr: Box::new(FirstMatchOf::new(vec![Box::new(
                SequenceExpr::default().then_oov().t_ws().then_oov(),
            )])),
            dict,
        }
    }

    pub fn sensitive(dict: D) -> Self {
        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(SequenceExpr::default().then_oov().t_ws().then_any_word()),
                Box::new(SequenceExpr::any_word().t_ws().then_oov()),
                Box::new(SequenceExpr::default().then_oov().t_ws().then_oov()),
            ])),
            dict,
        }
    }
}

fn keep_unique(values: &mut Vec<String>, word1: &[char], word2: &[char]) {
    let value = format!(
        "{} {}",
        word1.iter().collect::<String>(),
        word2.iter().collect::<String>()
    );
    if !values.contains(&value) {
        values.push(value);
    }
}

impl<D: Dictionary + 'static> ExprLinter for TransposedSpace<D> {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let toks_span = toks.span()?;

        // "thec" "at" / "th ecat"
        let word1 = toks.first()?.span.get_content(src);
        let word2 = toks.last()?.span.get_content(src);

        // "thec" -> "the c"
        let w1_start = &word1[..word1.len() - 1];
        let w1_last = word1.iter().last()?;

        // "ecat" -> "e cat"
        let w2_first = word2.first()?;
        let w2_end = &word2[1..];

        // "c" + "at" -> "cat"
        let mut w1_last_plus_w2 = word2.to_vec();
        w1_last_plus_w2.insert(0, *w1_last);

        // "th" + "e" -> "the"
        let mut w1_plus_w2_first = word1.to_vec();
        w1_plus_w2_first.push(*w2_first);

        let mut values = vec![];

        // "thec" "at" -> "the cat"
        if self.dict.contains_word(w1_start) && self.dict.contains_word(&w1_last_plus_w2) {
            let maybe_canon_w2 = self.dict.get_correct_capitalization_of(&w1_last_plus_w2);
            if let Some(canon_w1) = self.dict.get_correct_capitalization_of(w1_start) {
                if let Some(canon_w2) = maybe_canon_w2 {
                    keep_unique(&mut values, canon_w1, canon_w2);
                } else {
                    keep_unique(&mut values, canon_w1, &w1_last_plus_w2);
                }
            } else if let Some(canon_w2) = maybe_canon_w2 {
                keep_unique(&mut values, w1_start, canon_w2);
            }

            keep_unique(&mut values, w1_start, &w1_last_plus_w2);
        }

        // "th" "ecat" -> "the cat"
        if self.dict.contains_word(&w1_plus_w2_first) && self.dict.contains_word(w2_end) {
            let maybe_canon_w2 = self.dict.get_correct_capitalization_of(w2_end);
            if let Some(canon_w1) = self.dict.get_correct_capitalization_of(&w1_plus_w2_first) {
                if let Some(canon_w2) = maybe_canon_w2 {
                    keep_unique(&mut values, canon_w1, canon_w2);
                } else {
                    keep_unique(&mut values, canon_w1, w2_end);
                }
            } else if let Some(canon_w2) = maybe_canon_w2 {
                keep_unique(&mut values, &w1_plus_w2_first, canon_w2);
            }

            keep_unique(&mut values, &w1_plus_w2_first, w2_end);
        }

        if values.is_empty() {
            return None;
        }

        let suggestions = values
            .iter()
            .map(|value| {
                Suggestion::replace_with_match_case(
                    value.chars().collect(),
                    toks_span.get_content(src),
                )
            })
            .collect();

        Some(Lint {
            span: toks_span,
            lint_kind: LintKind::Typo,
            suggestions,
            message: format!(
                "Is the space between `{}` and `{}` one character out of place?",
                word1.iter().collect::<String>(),
                word2.iter().collect::<String>()
            ),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for a space one character too early or too late between words."
    }
}

#[cfg(test)]
mod tests {
    use super::TransposedSpace;
    use crate::{linting::tests::assert_suggestion_result, spell::FstDictionary};

    #[test]
    fn space_too_early() {
        assert_suggestion_result(
            "Th ecat sat on the mat.",
            TransposedSpace::sensitive(FstDictionary::curated()),
            "The cat sat on the mat.",
        );
    }

    #[test]
    fn space_too_late() {
        assert_suggestion_result(
            "Thec at sat on the mat.",
            TransposedSpace::sensitive(FstDictionary::curated()),
            "The cat sat on the mat.",
        );
    }

    #[test]
    fn test_early() {
        assert_suggestion_result(
            "Sometimes the spac eis one character early.",
            TransposedSpace::new(FstDictionary::curated()),
            "Sometimes the space is one character early.",
        );
    }
    #[test]
    fn test_late() {
        assert_suggestion_result(
            "Ands ometimes the space is a character late.",
            TransposedSpace::new(FstDictionary::curated()),
            "And sometimes the space is a character late.",
        );
    }
}

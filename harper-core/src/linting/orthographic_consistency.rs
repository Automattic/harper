use crate::dict_word_metadata_orthography::Orthography;
use crate::linting::{LintKind, Suggestion};
use std::sync::Arc;

use crate::expr::Expr;
use crate::spell::FstDictionary;
use crate::{OrthFlags, Token};

use super::{ExprLinter, Lint};
pub struct OrthographicConsistency {
    dict: Arc<FstDictionary>,
    expr: Box<dyn Expr>,
}

impl OrthographicConsistency {
    pub fn new() -> Self {
        Self {
            dict: FstDictionary::curated(),
            expr: Box::new(|tok: &Token, _: &[char]| tok.kind.is_word()),
        }
    }
}

impl Default for OrthographicConsistency {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for OrthographicConsistency {
    fn description(&self) -> &str {
        todo!()
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let word = &matched_tokens[0];

        let Some(Some(metadata)) = word.kind.as_word() else {
            return None;
        };

        let chars = word.span.get_content(source);

        let cur_flags = OrthFlags::from_letters(chars);

        if metadata.is_allcaps()
            && !metadata.is_lowercase()
            && !cur_flags.contains(OrthFlags::ALLCAPS)
        {
            return Some(Lint {
                span: word.span,
                lint_kind: LintKind::Capitalization,
                suggestions: vec![Suggestion::ReplaceWith(
                    chars.iter().map(|c| c.to_ascii_uppercase()).collect(),
                )],
                message: "This word's canonical spelling is all-caps.".to_owned(),
                ..Default::default()
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::OrthographicConsistency;

    #[test]
    fn nasa_should_be_all_caps() {
        assert_suggestion_result(
            "Nasa is a governmental institution.",
            OrthographicConsistency::default(),
            "NASA is a governmental institution.",
        );
    }

    #[test]
    fn ikea_should_be_all_caps() {
        assert_suggestion_result(
            "Ikea operates a vast retail network.",
            OrthographicConsistency::default(),
            "IKEA operates a vast retail network.",
        );
    }

    #[test]
    fn lego_should_be_all_caps() {
        assert_suggestion_result(
            "Lego bricks encourage creativity.",
            OrthographicConsistency::default(),
            "LEGO bricks encourage creativity.",
        );
    }

    #[test]
    fn nato_should_be_all_caps() {
        assert_suggestion_result(
            "Nato is a military alliance.",
            OrthographicConsistency::default(),
            "NATO is a military alliance.",
        );
    }

    #[test]
    fn fbi_should_be_all_caps() {
        assert_suggestion_result(
            "Fbi investigates federal crimes.",
            OrthographicConsistency::default(),
            "FBI investigates federal crimes.",
        );
    }

    #[test]
    fn cia_should_be_all_caps() {
        assert_suggestion_result(
            "Cia gathers intelligence.",
            OrthographicConsistency::default(),
            "CIA gathers intelligence.",
        );
    }

    #[test]
    fn hiv_should_be_all_caps() {
        assert_suggestion_result(
            "Hiv is a virus.",
            OrthographicConsistency::default(),
            "HIV is a virus.",
        );
    }

    #[test]
    fn dna_should_be_all_caps() {
        assert_suggestion_result(
            "Dna carries genetic information.",
            OrthographicConsistency::default(),
            "DNA carries genetic information.",
        );
    }

    #[test]
    fn rna_should_be_all_caps() {
        assert_suggestion_result(
            "Rna participates in protein synthesis.",
            OrthographicConsistency::default(),
            "RNA participates in protein synthesis.",
        );
    }

    #[test]
    fn cpu_should_be_all_caps() {
        assert_suggestion_result(
            "Cpu executes instructions.",
            OrthographicConsistency::default(),
            "CPU executes instructions.",
        );
    }

    #[test]
    fn gpu_should_be_all_caps() {
        assert_suggestion_result(
            "Gpu accelerates graphics.",
            OrthographicConsistency::default(),
            "GPU accelerates graphics.",
        );
    }

    #[test]
    fn html_should_be_all_caps() {
        assert_suggestion_result(
            "Html structures web documents.",
            OrthographicConsistency::default(),
            "HTML structures web documents.",
        );
    }

    #[test]
    fn url_should_be_all_caps() {
        assert_suggestion_result(
            "Url identifies a resource.",
            OrthographicConsistency::default(),
            "URL identifies a resource.",
        );
    }

    #[test]
    fn faq_should_be_all_caps() {
        assert_suggestion_result(
            "Faq answers common questions.",
            OrthographicConsistency::default(),
            "FAQ answers common questions.",
        );
    }

    #[test]
    fn pdf_should_be_all_caps() {
        assert_suggestion_result(
            "Pdf preserves formatting.",
            OrthographicConsistency::default(),
            "PDF preserves formatting.",
        );
    }

    #[test]
    fn ceo_should_be_all_caps() {
        assert_suggestion_result(
            "Our Ceo approved the budget.",
            OrthographicConsistency::default(),
            "Our CEO approved the budget.",
        );
    }

    #[test]
    fn cfo_should_be_all_caps() {
        assert_suggestion_result(
            "The Cfo presented the report.",
            OrthographicConsistency::default(),
            "The CFO presented the report.",
        );
    }

    #[test]
    fn hr_should_be_all_caps() {
        assert_suggestion_result(
            "The Hr team scheduled interviews.",
            OrthographicConsistency::default(),
            "The HR team scheduled interviews.",
        );
    }

    #[test]
    fn ai_should_be_all_caps() {
        assert_suggestion_result(
            "Ai enables new capabilities.",
            OrthographicConsistency::default(),
            "AI enables new capabilities.",
        );
    }

    #[test]
    fn ufo_should_be_all_caps() {
        assert_suggestion_result(
            "Ufo sightings provoke debate.",
            OrthographicConsistency::default(),
            "UFO sightings provoke debate.",
        );
    }
}

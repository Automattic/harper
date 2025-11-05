use crate::linting::{LintKind, Suggestion};
use std::sync::Arc;

use crate::expr::Expr;
use crate::spell::{Dictionary, FstDictionary};
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
        "Ensures word casing matches the dictionary's canonical orthography."
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
                priority: 127,
            });
        }

        let canonical_flags = metadata.orth_info;
        let flags_to_check = [
            OrthFlags::LOWER_CAMEL,
            OrthFlags::UPPER_CAMEL,
            OrthFlags::APOSTROPHE,
            OrthFlags::HYPHENATED,
        ];

        if flags_to_check
            .iter()
            .any(|flag| canonical_flags.contains(*flag) != cur_flags.contains(*flag))
            && let Some(canonical) = self.dict.get_correct_capitalization_of(chars)
        {
            return Some(Lint {
                span: word.span,
                lint_kind: LintKind::Capitalization,
                suggestions: vec![Suggestion::ReplaceWith(canonical.to_vec())],
                message: format!(
                    "The canonical dictionary spelling is `{}`.",
                    canonical.iter().collect::<String>()
                ),
                priority: 127,
            });
        }

        if metadata.is_titlecase()
            && cur_flags.contains(OrthFlags::LOWERCASE)
            && let Some(canonical) = self.dict.get_correct_capitalization_of(chars)
            && canonical != chars
        {
            return Some(Lint {
                span: word.span,
                lint_kind: LintKind::Capitalization,
                suggestions: vec![Suggestion::ReplaceWith(canonical.to_vec())],
                message: format!(
                    "The canonical dictionary spelling is `{}`.",
                    canonical.iter().collect::<String>()
                ),
                priority: 127,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

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
    fn linkedin_should_use_canonical_case() {
        assert_suggestion_result(
            "I updated my linkedin profile yesterday.",
            OrthographicConsistency::default(),
            "I updated my LinkedIn profile yesterday.",
        );
    }

    #[test]
    fn wordpress_should_use_canonical_case() {
        assert_suggestion_result(
            "She writes daily on her wordpress blog.",
            OrthographicConsistency::default(),
            "She writes daily on her WordPress blog.",
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

    #[test]
    fn markdown_should_be_caps() {
        assert_suggestion_result(
            "I adore markdown.",
            OrthographicConsistency::default(),
            "I adore Markdown.",
        );
    }

    #[test]
    fn canonical_forms_should_not_be_flagged() {
        let sentences = [
            "NASA is a governmental institution.",
            "IKEA operates a vast retail network.",
            "LEGO bricks encourage creativity.",
            "NATO is a military alliance.",
            "FBI investigates federal crimes.",
            "CIA gathers intelligence.",
            "HIV is a virus.",
            "DNA carries genetic information.",
            "RNA participates in protein synthesis.",
            "CPU executes instructions.",
            "GPU accelerates graphics.",
            "HTML structures web documents.",
            "URL identifies a resource.",
            "FAQ answers common questions.",
            "I updated my LinkedIn profile yesterday.",
            "She writes daily on her WordPress blog.",
            "PDF preserves formatting.",
            "Our CEO approved the budget.",
            "The CFO presented the report.",
            "The HR team scheduled interviews.",
            "AI enables new capabilities.",
            "UFO sightings provoke debate.",
            "I adore Markdown.",
        ];

        for sentence in sentences {
            assert_no_lints(sentence, OrthographicConsistency::default());
        }
    }
}

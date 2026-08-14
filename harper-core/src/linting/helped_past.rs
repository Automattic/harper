/// A linter that catches double-tensing errors when a past-tense verb
/// incorrectly follows "helped" (e.g., `helped built` instead of `helped build`).
use crate::{
    Lint, Token,
    char_ext::CharExt,
    char_string::CharStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

pub struct HelpedPast<D> {
    expr: All,
    dict: D,
}

impl<D: Dictionary> HelpedPast<D> {
    pub fn new(dict: D) -> Self {
        Self {
            expr: SequenceExpr::any_capitalization_of("helped")
                .t_ws()
                // Note that 'simple past forms' may apply only to irregular verbs
                // Note and that 'past forms' applies to regular verbs where preterite and participle share a form
                .then_kind_where(|k| {
                    (k.is_verb_simple_past_form() || k.is_verb_past_form()) && !k.is_verb_lemma()
                })
                .but_not(SequenceExpr::anything().t_any().t_set(&["did", "got"])),
            dict,
        }
    }

    fn keep_suggestion_if_lemma(&self, suggs: &mut Vec<Vec<char>>, candidate: &[char]) {
        if self
            .dict
            .get_word_metadata(candidate)
            .is_some_and(|md| md.is_verb_lemma())
        {
            suggs.push(candidate.to_vec());
        }
    }
}

impl<D: Dictionary> ExprLinter for HelpedPast<D> {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let vspan = toks.last()?.span;
        let vchars = vspan.get_content(src);
        let vstr = vspan.get_content_string(src);

        let mut suggs = vec![];

        // Chop -d/-ed off regular verbs

        if vchars.ends_with_ignore_ascii_case_chars(&['d']) {
            let without_d = &vchars[..vchars.len() - 1];

            if without_d.ends_with_ignore_ascii_case_chars(&['e']) {
                let without_ed = &without_d[..without_d.len() - 1];

                self.keep_suggestion_if_lemma(&mut suggs, without_ed);

                // If the stem without -ed now ends in -i, try changing that to -y to find the lemma
                if without_ed.ends_with_ignore_ascii_case_chars(&['i']) {
                    let mut with_final_y = without_ed[..without_ed.len() - 1].to_vec();
                    with_final_y.push('y');
                    self.keep_suggestion_if_lemma(&mut suggs, &with_final_y);
                }

                // If the stem without -ed ends in a doubled consonant, try with just a single one
                if without_ed.last().is_some_and(|c| !c.is_vowel()) {
                    let without_doubled_consonant = without_ed[..without_ed.len() - 1].to_vec();
                    self.keep_suggestion_if_lemma(&mut suggs, &without_doubled_consonant);
                }
            }
            self.keep_suggestion_if_lemma(&mut suggs, without_d);
        }

        // Look up irregular verbs

        if let Some(lemma) = IrregularVerbs::curated().get_lemma_for_preterite(&vstr) {
            suggs.push(lemma.chars().collect());
        }

        if !suggs.is_empty() {
            Some(Lint {
                span: vspan,
                lint_kind: LintKind::Grammar,
                suggestions: suggs
                    .into_iter()
                    .map(|s| Suggestion::replace_with_match_case(s, vchars))
                    .collect(),
                message: "Use the base form of the verb after \"helped\".".to_owned(),
                ..Default::default()
            })
        } else {
            None
        }
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects past forms of verbs to their base form, when used after \"helped\"."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        linting::tests::{assert_no_lints, assert_suggestion_result},
        spell::FstDictionary,
    };

    use super::HelpedPast;

    // The really common ones

    #[test]
    fn that_i_helped_built() {
        assert_suggestion_result(
            "... as I have moved from CraftCMS into a startup that I helped built",
            HelpedPast::new(FstDictionary::curated()),
            "... as I have moved from CraftCMS into a startup that I helped build",
        );
    }

    #[test]
    fn i_helped_built_some() {
        assert_suggestion_result(
            "Strangely, thru some form of \"magic\" I helped built some hundreds of commercial T2 targets",
            HelpedPast::new(FstDictionary::curated()),
            "Strangely, thru some form of \"magic\" I helped build some hundreds of commercial T2 targets",
        );
    }

    #[test]
    fn has_helped_built() {
        assert_suggestion_result(
            "Dify is an LLM application development platform that has helped built over 100,000 applications.",
            HelpedPast::new(FstDictionary::curated()),
            "Dify is an LLM application development platform that has helped build over 100,000 applications.",
        );
    }

    #[test]
    fn who_helped_made() {
        assert_suggestion_result(
            "our many amazing contributors who helped made this possible",
            HelpedPast::new(FstDictionary::curated()),
            "our many amazing contributors who helped make this possible",
        );
    }

    #[test]
    fn has_helped_made() {
        assert_suggestion_result(
            "everyone who says that something I've worked on has helped made their lives easier",
            HelpedPast::new(FstDictionary::curated()),
            "everyone who says that something I've worked on has helped make their lives easier",
        );
    }

    #[test]
    fn i_helped_made() {
        assert_suggestion_result(
            "This is a a video game I helped made for a national competition in high school.",
            HelpedPast::new(FstDictionary::curated()),
            "This is a a video game I helped make for a national competition in high school.",
        );
    }

    #[test]
    fn helped_made_at_start() {
        assert_suggestion_result(
            "Spinner Tutorial - Helped made the roulette wheel come to life.",
            HelpedPast::new(FstDictionary::curated()),
            "Spinner Tutorial - Helped make the roulette wheel come to life.",
        );
    }

    // The rest

    #[test]
    fn helped_chose() {
        assert_suggestion_result(
            "The 2018 analysis helped chose tires (as well as validated prior choices)",
            HelpedPast::new(FstDictionary::curated()),
            "The 2018 analysis helped choose tires (as well as validated prior choices)",
        );
    }

    #[test]
    fn helped_created() {
        assert_suggestion_result(
            "React helped created a light weight dynamic webpage that can be implmented in various different applications.",
            HelpedPast::new(FstDictionary::curated()),
            "React helped create a light weight dynamic webpage that can be implmented in various different applications.",
        );
    }

    #[test]
    fn helped_debugged() {
        assert_suggestion_result(
            "Someone helped debugged this issue.",
            HelpedPast::new(FstDictionary::curated()),
            "Someone helped debug this issue.",
        );
    }

    #[test]
    fn helped_designed() {
        assert_suggestion_result(
            "when speaking with Ka Ming who has helped designed many of the UX/UI elements for the Security Plugin",
            HelpedPast::new(FstDictionary::curated()),
            "when speaking with Ka Ming who has helped design many of the UX/UI elements for the Security Plugin",
        );
    }

    #[test]
    fn helped_implemented() {
        assert_suggestion_result(
            "The way to do that, that I have helped implemented in some other addons already",
            HelpedPast::new(FstDictionary::curated()),
            "The way to do that, that I have helped implement in some other addons already",
        );
    }

    #[test]
    fn also_helped_improved() {
        assert_suggestion_result(
            "Also helped improved and scale billing infrastructure as customer growth accelerated especially in the enterprise space.",
            HelpedPast::new(FstDictionary::curated()),
            "Also helped improve and scale billing infrastructure as customer growth accelerated especially in the enterprise space.",
        );
    }

    #[test]
    fn has_helped_improved() {
        assert_suggestion_result(
            "Thanks for making Psalm, it's a fantastic tool and has helped improved our code quality massively.",
            HelpedPast::new(FstDictionary::curated()),
            "Thanks for making Psalm, it's a fantastic tool and has helped improve our code quality massively.",
        );
    }

    #[test]
    fn helped_picked() {
        assert_suggestion_result(
            "you helped picked the default",
            HelpedPast::new(FstDictionary::curated()),
            "you helped pick the default",
        );
    }

    #[test]
    fn helped_wrote() {
        assert_suggestion_result(
            "Disclosure: I didn't find the bugs. I helped wrote the blog post.",
            HelpedPast::new(FstDictionary::curated()),
            "Disclosure: I didn't find the bugs. I helped write the blog post.",
        );
    }

    // Potential false positive

    #[test]
    fn dont_flag_shouldve_helped_given_that() {
        assert_no_lints(
            "However, it seems like 1 minute timeout should've helped given that default idle timeout is 10 minutes.",
            HelpedPast::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_be_helped_given_the() {
        assert_no_lints(
            "Major API breakages can't be helped given the nature of code generation",
            HelpedPast::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_helped_got() {
        assert_no_lints(
            "but along the way everyone who helped got +1 from me",
            HelpedPast::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_helped_did() {
        assert_no_lints(
            "So, that helped did it?",
            HelpedPast::new(FstDictionary::curated()),
        );
    }
}

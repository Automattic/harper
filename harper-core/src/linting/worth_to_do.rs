use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    char_ext::CharExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::{CommonDictFuncs, Dictionary},
};

pub struct WorthToDo<D>
where
    D: Dictionary,
{
    expr: Box<dyn Expr>,
    dict: D,
}

impl<D> WorthToDo<D>
where
    D: Dictionary,
{
    pub fn new(dict: D) -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::aco("worth")
                    .t_ws()
                    .t_aco("to")
                    .t_ws()
                    .then_verb_lemma(),
            ),
            dict,
        }
    }
}

impl<D> ExprLinter for WorthToDo<D>
where
    D: Dictionary,
{
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tolemtoks = &toks[toks.len() - 3..];
        let lemtok = toks.last()?;
        let tolemspan = tolemtoks.span()?;
        let lemspan = lemtok.span;
        let tolemchars = tolemspan.get_content(src);
        let lemchars = lemspan.get_content(src);
        let lemstr = lemspan.get_content_string(src);

        let mut gerunds = Vec::new();

        let glom_ing = format!("{}ing", lemstr);
        if self.dict.contains_word_str(&glom_ing) {
            gerunds.push(glom_ing);
        }

        if lemchars.ends_with_ignore_ascii_case_chars(&['e']) {
            let replace_e_with_ing = format!("{}ing", &lemstr[..lemstr.len() - 1]);
            if self.dict.contains_word_str(&replace_e_with_ing) {
                gerunds.push(replace_e_with_ing);
            }
        }

        if let Some(last_letter) = lemstr.chars().last()
            && !last_letter.is_vowel()
        {
            let double_consonant = format!("{}{}ing", lemstr, last_letter);
            if self.dict.contains_word_str(&double_consonant) {
                gerunds.push(double_consonant);
            }
        }

        let suggestions = gerunds
            .into_iter()
            .map(|gerund| {
                Suggestion::replace_with_match_case(
                    gerund.chars().collect::<Vec<char>>(),
                    tolemchars,
                )
            })
            .collect();

        Some(Lint {
            span: tolemspan,
            lint_kind: LintKind::Grammar,
            suggestions,
            message: "Use the `gerund` of the verb, the form that ends in `-ing`".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `worth to` + a verb to `worth` + the gerund of the verb."
    }
}

#[cfg(test)]
mod tests {
    use super::WorthToDo;
    use crate::{linting::tests::assert_suggestion_result, spell::FstDictionary};

    #[test]
    fn worth_to_add() {
        assert_suggestion_result(
            "Is it worth to add those files?",
            WorthToDo::new(FstDictionary::curated()),
            "Is it worth adding those files?",
        );
    }

    #[test]
    fn worth_to_adjust() {
        assert_suggestion_result(
            "If yes, it would be worth to adjust the description to make it easier to understand",
            WorthToDo::new(FstDictionary::curated()),
            "If yes, it would be worth adjusting the description to make it easier to understand",
        );
    }

    #[test]
    fn worth_to_ask() {
        assert_suggestion_result(
            "So it is worth to ask for this there or take a look at their wiki pages. ",
            WorthToDo::new(FstDictionary::curated()),
            "So it is worth asking for this there or take a look at their wiki pages. ",
        );
    }

    #[test]
    fn worth_to_buy() {
        assert_suggestion_result(
            "and it makes it really worth to buy the software",
            WorthToDo::new(FstDictionary::curated()),
            "and it makes it really worth buying the software",
        );
    }

    #[test]
    fn worth_to_deal() {
        assert_suggestion_result(
            "CC2531 is considered as crap in 2024 and its not worth to deal with it.",
            WorthToDo::new(FstDictionary::curated()),
            "CC2531 is considered as crap in 2024 and its not worth dealing with it.",
        );
    }

    #[test]
    fn worth_to_do() {
        assert_suggestion_result(
            "Is it worth to do the credit-card-balance-transfer?",
            WorthToDo::new(FstDictionary::curated()),
            "Is it worth doing the credit-card-balance-transfer?",
        );
    }

    #[test]
    fn worth_to_experiment() {
        assert_suggestion_result(
            "Hello @tkchia, thanks for the hint, I agree it is worth to experiment.",
            WorthToDo::new(FstDictionary::curated()),
            "Hello @tkchia, thanks for the hint, I agree it is worth experimenting.",
        );
    }

    #[test]
    fn worth_to_fix() {
        assert_suggestion_result(
            "i dont know if this is worth to fix, i just wanted to point this and start a discussion about this.",
            WorthToDo::new(FstDictionary::curated()),
            "i dont know if this is worth fixing, i just wanted to point this and start a discussion about this.",
        );
    }

    #[test]
    fn worth_to_get_published() {
        assert_suggestion_result(
            "think that would be worth to get published in my Thesis.",
            WorthToDo::new(FstDictionary::curated()),
            "think that would be worth getting published in my Thesis.",
        );
    }

    #[test]
    fn worth_to_imagine() {
        assert_suggestion_result(
            "Might be worth to imagine how the current Nu style would look like in other programming languages.",
            WorthToDo::new(FstDictionary::curated()),
            "Might be worth imagining how the current Nu style would look like in other programming languages.",
        );
    }

    #[test]
    fn worth_to_invest() {
        assert_suggestion_result(
            "It doesn't seem worth to invest much effort in this though...",
            WorthToDo::new(FstDictionary::curated()),
            "It doesn't seem worth investing much effort in this though...",
        );
    }

    #[test]
    fn worth_to_investigate() {
        assert_suggestion_result(
            "to get a feeling how CP-SAT works and what are directions worth to investigate i wanted to ask",
            WorthToDo::new(FstDictionary::curated()),
            "to get a feeling how CP-SAT works and what are directions worth investigating i wanted to ask",
        );
    }

    #[test]
    fn worth_to_play() {
        assert_suggestion_result(
            "Is worth to play with thread count if there are no issues?",
            WorthToDo::new(FstDictionary::curated()),
            "Is worth playing with thread count if there are no issues?",
        );
    }

    #[test]
    fn worth_to_put() {
        assert_suggestion_result(
            "Do you think it would be worth to put a suggestion to remove the kind network if cluster creation fails",
            WorthToDo::new(FstDictionary::curated()),
            "Do you think it would be worth putting a suggestion to remove the kind network if cluster creation fails",
        );
    }

    #[test]
    fn worth_to_read() {
        assert_suggestion_result(
            "Stored books worth to read.",
            WorthToDo::new(FstDictionary::curated()),
            "Stored books worth reading.",
        );
    }

    #[test]
    fn worth_to_revisit() {
        assert_suggestion_result(
            "we've had discussions before #260 and it maybe worth to revisit again",
            WorthToDo::new(FstDictionary::curated()),
            "we've had discussions before #260 and it maybe worth revisiting again",
        );
    }

    #[test]
    fn worth_to_rewrite() {
        assert_suggestion_result(
            "is puppet so bad that it is worth to rewrite everything?",
            WorthToDo::new(FstDictionary::curated()),
            "is puppet so bad that it is worth rewriting everything?",
        );
    }

    #[test]
    fn worth_to_try() {
        assert_suggestion_result(
            "is it really worth to try and what are facebook long-term plans about this engine.",
            WorthToDo::new(FstDictionary::curated()),
            "is it really worth trying and what are facebook long-term plans about this engine.",
        );
    }

    #[test]
    fn worth_to_update() {
        assert_suggestion_result(
            "Hi, maybe it's worth to update doc with the script for Bullseye given by @frenchfaso ?",
            WorthToDo::new(FstDictionary::curated()),
            "Hi, maybe it's worth updating doc with the script for Bullseye given by @frenchfaso ?",
        );
    }

    #[test]
    fn worth_to_upgrade() {
        assert_suggestion_result(
            "Your PR should've fixed that issue so I think it's worth to upgrade to 10.33 and see if that brings the delta down.",
            WorthToDo::new(FstDictionary::curated()),
            "Your PR should've fixed that issue so I think it's worth upgrading to 10.33 and see if that brings the delta down.",
        );
    }

    #[test]
    fn worth_to_use_and_develop() {
        assert_suggestion_result(
            "I think It worth to use and worth to develop further",
            WorthToDo::new(FstDictionary::curated()),
            "I think It worth using and worth developing further",
        );
    }

    #[test]
    fn works_with_uppercase_glom() {
        assert_suggestion_result(
            " YES IT IS WORTH TO DO",
            WorthToDo::new(FstDictionary::curated()),
            " YES IT IS WORTH DOING",
        );
    }

    #[test]
    fn works_with_uppercase_final_e() {
        assert_suggestion_result(
            "THIS LINTER WAS WORTH TO MAKE",
            WorthToDo::new(FstDictionary::curated()),
            "THIS LINTER WAS WORTH MAKING",
        );
    }

    #[test]
    fn works_with_uppercase_double_consonant() {
        assert_suggestion_result(
            "SO YEAH IT WAS WORTH TO GET THIS DONE",
            WorthToDo::new(FstDictionary::curated()),
            "SO YEAH IT WAS WORTH GETTING THIS DONE",
        );
    }
}

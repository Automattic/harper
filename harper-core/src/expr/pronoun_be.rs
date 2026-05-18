use crate::{Span, Token, patterns::WordSet};

use super::{Expr, SequenceExpr};

pub struct PronounBe {
    expr: SequenceExpr,
}

impl Default for PronounBe {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default().then_any_of(vec![
                Box::new(
                    SequenceExpr::default()
                        .then_subject_pronoun()
                        .t_ws()
                        .t_set(&["am", "are", "is", "was", "were"]),
                ),
                Box::new(WordSet::new(&[
                    "i'm", "we're", "you're", "he's", "she's", "it's", "they're",
                ])),
            ]),
        }
    }
}

impl Expr for PronounBe {
    fn run(&self, cursor: usize, toks: &[Token], src: &[char]) -> Option<Span<Token>> {
        if toks.is_empty() {
            return None;
        }

        self.expr.run(cursor, toks, src)
    }
    fn run_rev(&self, _cursor: usize, _toks: &[Token], _src: &[char]) -> Option<Span<Token>> {
        eprintln!("🛑 PRONOUN_BE");
        None // Not implemented
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document,
        expr::{ExprExt, PronounBe},
    };

    fn assert_count(text: &str, expected_count: usize) {
        assert_eq!(
            PronounBe::default()
                .iter_matches_in_doc(&Document::new_plain_english_curated(text))
                .count(),
            expected_count
        );
    }

    #[test]
    fn ok_i_am() {
        assert_count("I am an auditor of software in M&A.", 1);
    }

    #[test]
    fn ok_i_m() {
        assert_count(
            "I'm using Beads in every new project and adding it to every old project I visit with an agent.",
            1,
        );
    }

    #[test]
    fn ok_we_are() {
        assert_count(
            "Error: we are enable to complete your request at this time.",
            1,
        );
    }

    #[test]
    fn ok_we_re() {
        assert_count(
            "We're currently experiencing high demand, which may cause temporary errors.",
            1,
        );
    }

    #[test]
    fn ok_you_are() {
        assert_count(
            "You are Dolphin, an uncensored and unbiased AI assistant.",
            1,
        );
    }

    #[test]
    fn ok_you_re() {
        assert_count(
            "You're trying to use a SD1 LoRA model with a SDXL Stable Diffusion model.",
            1,
        );
    }

    #[test]
    fn ok_he_is() {
        assert_count("He is just a modulary guy for checking service health.", 1);
    }

    #[test]
    fn ok_he_s_and_it_is() {
        assert_count(
            "He's Dead, Jim is a link checking program, specifically it is a command-line tool for finding and reporting dead links",
            2,
        );
    }

    #[test]
    fn ok_she_is() {
        assert_count(
            "SHE is designed by following the SHE functional specification.",
            1,
        );
    }

    #[test]
    fn ok_she_s() {
        assert_count(
            "She's Coding is an open-source website project currently under development in cooperation with the documentary film CODE: Debugging the Gender Gap.",
            1,
        );
    }

    #[test]
    fn ok_it_is() {
        assert_count("It is same as ms-dos command where.exe.", 1);
    }

    #[test]
    fn ok_it_s_and_you_are() {
        assert_count(
            "It's not working · Microcontrollers have only a limited amount of RAM: Verify that you are not running out of available RAM!",
            2,
        );
    }

    #[test]
    fn ok_they_are() {
        assert_count(
            "Multidist binaries ignore the way they are called and check only the running binary name, making them hard to invoke from a single one.",
            1,
        );
    }

    #[test]
    fn ok_they_re() {
        assert_count("Cannot load dataset, they're greyed out.", 1);
    }

    #[test]
    fn various_apostrophes() {
        assert_count("it's/it’s", 2);
        assert_count("it;s/it´s", 0);
    }
}

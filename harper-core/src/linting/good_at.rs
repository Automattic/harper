use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::{InflectionOfBe, WordSet},
};

pub struct GoodAt {
    expr: Box<dyn Expr>,
}

impl Default for GoodAt {
    fn default() -> Self {
        let we_re_not_always_very_good_in_sth = SequenceExpr::any_of(vec![
            Box::new(InflectionOfBe::default()),
            Box::new(WordSet::new(&[
                "I'm", "we're", "you're", "he's", "she's", "it's", "they're", "Im", "were",
                "youre", "your", "hes", "shes", "its", "theyre",
            ])),
        ])
        .t_ws()
        .then_optional(SequenceExpr::aco("not").t_ws())
        .then_optional(SequenceExpr::default().then_frequency_adverb().t_ws())
        .then_optional(SequenceExpr::default().then_degree_adverb().t_ws())
        .then_word_set(&["good", "bad", "great", "okay", "OK"])
        .t_ws()
        .t_aco("in")
        .t_ws()
        .then_any_word();

        let good_in_skill_or_subject =
            SequenceExpr::word_set(&["good", "bad", "great", "okay", "OK"])
                .t_ws()
                .t_aco("in")
                .t_ws()
                .then_word_set(&[
                    // sciences
                    "biology",
                    "chemistry",
                    "math",
                    "mathematics",
                    "physics",
                    // programming
                    "programming",
                    "coding",
                    "c", // Note: C++ would be multiple tokens
                    "debugging",
                    "go",
                    "java",
                    "javascript",
                    "laravel",
                    "python",
                    "ruby",
                    // languages
                    "english",
                    "chinese",
                    "french",
                    "german",
                    "japanese",
                    "spanish",
                ]);

        let expr = FirstMatchOf::new(vec![
            Box::new(we_re_not_always_very_good_in_sth),
            Box::new(good_in_skill_or_subject),
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for GoodAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prep_span = toks.get_rel(-3)?.span;

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                "at".chars().collect(),
                prep_span.get_content(src),
            )],
            message: "Use 'good at' to describe proficiency with a skill.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Checks for `good in` used instead of `good at` to describe proficiency with a skill."
    }
}

#[cfg(test)]
mod tests {
    use super::GoodAt;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_good_in_being_frugal() {
        assert_suggestion_result(
            "but we found that Claude is not always very good in being frugal ( Gemini seemed better at it ) .",
            GoodAt::default(),
            "but we found that Claude is not always very good at being frugal ( Gemini seemed better at it ) .",
        );
    }

    #[test]
    fn fix_im_not_good_in_python() {
        assert_suggestion_result(
            "can't run it i'm not good in python",
            GoodAt::default(),
            "can't run it i'm not good at python",
        );
    }

    #[test]
    fn fix_not_good_in_go() {
        assert_suggestion_result(
            "Hi, I have very similar problem and I'm not good in go either.",
            GoodAt::default(),
            "Hi, I have very similar problem and I'm not good at go either.",
        );
    }

    #[test]
    fn fix_not_good_in_coding_stuff() {
        assert_suggestion_result(
            "Unfortunately I can't help in anyway but testing, because I'm not good in coding stuff.",
            GoodAt::default(),
            "Unfortunately I can't help in anyway but testing, because I'm not good at coding stuff.",
        );
    }

    #[test]
    fn fix_very_good_in_mathematics() {
        assert_suggestion_result(
            "They were very good in Mathematics and were the pets of Ranjani Ma'am.",
            GoodAt::default(),
            "They were very good at Mathematics and were the pets of Ranjani Ma'am.",
        );
    }

    #[test]
    fn fix_not_good_in_coding() {
        assert_suggestion_result(
            "Didnt know these things.. and since im not good in coding, maybe one day someone will work on this.",
            GoodAt::default(),
            "Didnt know these things.. and since im not good at coding, maybe one day someone will work on this.",
        );
    }

    #[test]
    fn fix_very_good_in_most_things() {
        assert_suggestion_result(
            "But I do want to continue using riverpod because its very good in most things I need from my app.",
            GoodAt::default(),
            "But I do want to continue using riverpod because its very good at most things I need from my app.",
        );
    }

    #[test]
    fn fix_not_good_in_laravel() {
        assert_suggestion_result(
            "im not good in laravel.",
            GoodAt::default(),
            "im not good at laravel.",
        );
    }

    #[test]
    fn fixim_not_good_in_english() {
        assert_suggestion_result(
            "Sorry about my grammar im not good in English.",
            GoodAt::default(),
            "Sorry about my grammar im not good at English.",
        );
    }

    #[test]
    fn fix_not_all_good_in_english() {
        assert_suggestion_result(
            "I think if there is translation support will be great, our school society not all good in english.",
            GoodAt::default(),
            "I think if there is translation support will be great, our school society not all good at english.",
        );
    }

    #[test]
    fn fix_i_am_not_good_in_english() {
        assert_suggestion_result(
            "I am in Togo, i am not good in english but i will ask you to apologize me for my speaking.",
            GoodAt::default(),
            "I am in Togo, i am not good at english but i will ask you to apologize me for my speaking.",
        );
    }

    #[test]
    fn fix_not_good_in_programming() {
        assert_suggestion_result(
            "Is it My Drive,Chess or \"My Drive\",\"Chess\" or what? Because I'm not good in programming.",
            GoodAt::default(),
            "Is it My Drive,Chess or \"My Drive\",\"Chess\" or what? Because I'm not good at programming.",
        );
    }

    #[test]
    fn fix_not_so_good_in_coding() {
        assert_suggestion_result(
            "I'm not so good in coding to understand how to handle all these bytes...",
            GoodAt::default(),
            "I'm not so good at coding to understand how to handle all these bytes...",
        );
    }

    #[test]
    fn fix_im_not_good_in_programming() {
        assert_suggestion_result(
            "Im not good in programming , but can i ask what is the password of your codes?",
            GoodAt::default(),
            "Im not good at programming , but can i ask what is the password of your codes?",
        );
    }
}

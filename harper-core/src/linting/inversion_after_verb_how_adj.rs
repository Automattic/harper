use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, debug::format_lint_match, expr_linter::Chunk},
};

const HOW_VERBS: [&str; 5] = [
    "see",
    "seeing",
    "wonder",
    "wondered",
    "wondering",
    // "wonders",
];

pub struct InversionAfterVerbhowAdj {
    expr: SequenceExpr,
}

impl Default for InversionAfterVerbhowAdj {
    fn default() -> Self {
        let how_seq = SequenceExpr::word_set(HOW_VERBS)
            .t_ws()
            .t_aco("how")
            .t_ws()
            .then_adjective();

        Self {
            expr: SequenceExpr::any_of([
                Box::new(how_seq),
                // TODO: what, when, where, which, who, why
            ])
            .t_ws()
            .t_set(["am", "are", "is", "was", "were", "do", "does", "did"])
            .t_ws()
            .then_longest_of([
                Box::new(SequenceExpr::default().then_kind_any(&[
                    TokenKind::is_personal_pronoun,
                    TokenKind::is_noun,
                    TokenKind::is_determiner,
                ])),
                Box::new(SequenceExpr::aco("to").t_ws().then_verb_lemma()),
            ]),
        }
    }
}

impl ExprLinter for InversionAfterVerbhowAdj {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordOrder,
            suggestions: vec![],
            message: "After [verb] `how` [adjective], the next verb should come before the subject. For example `see how big is it` → `see how big it is`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects lack of inversion after `[verb] how` [adjective]."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_lint_count;

    use super::InversionAfterVerbhowAdj;

    #[test]
    fn see_how_big_are() {
        assert_lint_count(
            "One simply needs to to see how big are input *.csv files, *.pickle files or many other.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "One simply needs to to see how big input *.csv files, *.pickle files or many other are.",
        );
    }

    #[test]
    fn see_how_big_is_det_art() {
        assert_lint_count(
            "you still have all thumbnails in the UI, so you clearly see how big is the conference",
            InversionAfterVerbhowAdj::default(),
            1,
            // "you still have all thumbnails in the UI, so you clearly see how big the conference is",
        );
    }

    #[test]
    fn see_how_easy_is_to_verb_inf() {
        assert_lint_count(
            "See how easy is to implement crud functionality with firebase, reactjs, tailwindcss.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "See how easy to implement crud functionality is with firebase, reactjs, tailwindcss.",
        );
    }

    #[test]
    fn see_how_fast_is_verb_prog() {
        assert_lint_count(
            "See how fast is rendering in your Ruby on Rails app.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "See how fast rendering in your Ruby on Rails app is.",
        );
    }

    #[test]
    fn see_how_good_is_det_art() {
        assert_lint_count(
            "I would love to read more about it and see how good is the fit of such protocol with nimble_parsec.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I would love to read more about it and see how good the fit of such protocol is with nimble_parsec.",
            // "I would love to read more about it and see how good the fit of such protocol with nimble_parsec is.",
        );
    }

    #[test]
    fn wonder_how_difficult_is_to_inf() {
        assert_lint_count(
            "I wonder how difficult is to use it in pure JS.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I wonder how difficult itis to use it in pure JS.",
        );
    }

    #[test]
    fn wonder_how_good_is_det_poss() {
        assert_lint_count(
            "Now with Colossal-AI framework, I wonder how good is their solution.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "Now with Colossal-AI framework, I wonder how good their solution is.",
        );
    }

    #[test]
    fn wonder_how_hard_is_pron_demons() {
        assert_lint_count(
            "I wonder how hard is this ?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I wonder how hard this is ?",
        );
    }

    #[test]
    fn wonder_how_reliable_are_npl() {
        assert_lint_count(
            "I really wonder how reliable are rewrites like that, a 65k lines you didn't actually read.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I really wonder how reliable rewrites like that are, a 65k lines you didn't actually read.",
        );
    }

    #[test]
    fn wonder_how_slow_is_adj() {
        assert_lint_count(
            "But, every time I hear that, I wonder how slow is okay?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "But, every time I hear that, I wonder how slow okay is?",
        );
    }

    #[test]
    fn wonder_how_small_is_small() {
        assert_lint_count(
            "If that is to be considered \"large\" I wonder how small is small supposed to be.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "If that is to be considered \"large\" I wonder how small small is supposed to be.",
        );
    }

    #[test]
    fn wonder_how_smart_are_det_art() {
        assert_lint_count(
            "Nowadays, I started to wonder how smart are the smart devices?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "Nowadays, I started to wonder how smart the smart devices are?",
        );
    }

    #[test]
    fn wondered_how_hard_was_to_verb_inf() {
        assert_lint_count(
            "Every time I saw those pictures or zooming GIFs I've always wondered how hard was to build that wonderful image and all its infinite patterns.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "Every time I saw those pictures or zooming GIFs I've always wondered how hard to build that wonderful image and all its infinite patterns was.",
        );
    }

    #[test]
    fn wondering_how_big_is_det_demons() {
        assert_lint_count(
            "I am wondering how big is this file supposed to be?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I am wondering how big this file is supposed to be?",
        );
    }

    #[test]
    fn wondering_how_easy_is_pron() {
        assert_lint_count(
            "and I am wondering how easy is it to run DART headless?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "and I am wondering how easy it is to run DART headless?",
        );
    }

    #[test]
    fn wondering_how_expensive_is_det_demons() {
        assert_lint_count(
            "I am wondering how expensive is this type conversion that is happening on the SQL side?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I am wondering how expensive this type conversion is that is happening on the SQL side?",
        );
    }

    #[test]
    fn wondering_how_fast_is_det_art() {
        assert_lint_count(
            "I was wondering how fast is the first copy with geo-replication on glusterfs.",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I was wondering how fast the first copy with geo-replication is on glusterfs.",
        );
    }

    #[test]
    fn wondering_how_secure_det_demons() {
        assert_lint_count(
            "I am wondering how secure is this connection?",
            InversionAfterVerbhowAdj::default(),
            1,
            // "I am wondering how secure this connection is?",
        );
    }
}

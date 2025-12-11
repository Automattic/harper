use crate::{
    Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct BrandBrandish {
    expr: Box<dyn Expr>,
}

impl Default for BrandBrandish {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["brandish", "brandished", "brandishes", "brandishing"])
                    .t_ws()
                    // "her" is also a possessive determiner as in "she brandished her sword"
                    // "it" and "them" can refer to objects as in "draw your sword(s) and brandish it/them"
                    .then_kind_except(TokenKind::is_object_pronoun, &["her", "it", "them"]),
            ),
        }
    }
}

impl ExprLinter for BrandBrandish {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let verb_span = toks.first()?.span;
        let verb_chars = verb_span.get_content(src);

        enum Form {
            Base,
            Past,
            ThirdPerson,
            Ing,
        }

        let infl = match verb_chars.last().map(|c| c.to_ascii_lowercase()) {
            Some('h') => Form::Base,
            Some('d') => Form::Past,
            Some('s') => Form::ThirdPerson,
            Some('g') => Form::Ing,
            _ => return None,
        };

        Some(Lint {
            span: verb_span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                match infl {
                    Form::Base => "brand",
                    Form::Past => "branded",
                    Form::ThirdPerson => "brands",
                    Form::Ing => "branding",
                },
                verb_chars,
            )],
            message: "`Brandish` means to wield a weapon. You probably mean `brand`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Looks for `brandish` wrongly used when `brand` is intended."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{brand_brandish::BrandBrandish, tests::assert_suggestion_result};

    #[test]
    fn correct_brandish_a_traitor() {
        assert_suggestion_result(
            "Unretire Gretzky's sweater . Brandish him a traitor.",
            BrandBrandish::default(),
            "Unretire Gretzky's sweater . Brand him a traitor.",
        );
    }

    #[test]
    fn correct_brandish_a_criminal() {
        assert_suggestion_result(
            "lied to stop kuma's ideology from taking root and to brandish him a criminal that they could arrest",
            BrandBrandish::default(),
            "lied to stop kuma's ideology from taking root and to brand him a criminal that they could arrest",
        );
    }

    #[test]
    fn correct_brandish_as_a() {
        assert_suggestion_result(
            "he was so afraid his thoughts could brandish him as a paedophile",
            BrandBrandish::default(),
            "he was so afraid his thoughts could brand him as a paedophile",
        );
    }

    #[test]
    fn correct_brandish_an_offender() {
        assert_suggestion_result(
            "Chanel Oberlin's reason for purposely leading on Pete Martinez in order to humiliate him and brandish him a registered sex offender",
            BrandBrandish::default(),
            "Chanel Oberlin's reason for purposely leading on Pete Martinez in order to humiliate him and brand him a registered sex offender",
        );
    }

    #[test]
    fn correct_brandish_with_nicknames() {
        assert_suggestion_result(
            "?? spoke out over the move by Kenyans to continuously brandish him with nicknames even after ...",
            BrandBrandish::default(),
            "?? spoke out over the move by Kenyans to continuously brand him with nicknames even after ...",
        );
    }

    #[test]
    fn correct_brandish_as_a_aymbol() {
        assert_suggestion_result(
            "brandish him as an acclaimed symbol of humility, integrity and incorruptibility in the face of today's corrupt economic and political elite1",
            BrandBrandish::default(),
            "brand him as an acclaimed symbol of humility, integrity and incorruptibility in the face of today's corrupt economic and political elite1",
        );
    }

    #[test]
    fn correct_brandish_as_illegal() {
        assert_suggestion_result(
            "To attempt to brandish him as an “illegal immigrant” is absolutely ridiculous and warrants an immediate retraction and apology.",
            BrandBrandish::default(),
            "To attempt to brand him as an “illegal immigrant” is absolutely ridiculous and warrants an immediate retraction and apology.",
        );
    }

    #[test]
    fn correct_brandish_with_nickname() {
        assert_suggestion_result(
            "The small minded townsfolk brandish him with the nickname \"Genepool\" due to his physical and cognitive shortcomings.",
            BrandBrandish::default(),
            "The small minded townsfolk brand him with the nickname \"Genepool\" due to his physical and cognitive shortcomings.",
        );
    }

    #[test]
    fn correct_brandish_with_label() {
        assert_suggestion_result(
            "One such reason that critics brandish him with this label is due to Peterson's opposition to Canada's Bill C-16",
            BrandBrandish::default(),
            "One such reason that critics brand him with this label is due to Peterson's opposition to Canada's Bill C-16",
        );
    }

    #[test]
    fn correct_brandished_us() {
        assert_suggestion_result(
            "The mark they brandished us with will fade to dust when we finally meet our end.",
            BrandBrandish::default(),
            "The mark they branded us with will fade to dust when we finally meet our end.",
        )
    }

    #[test]
    fn correct_brandishing_him() {
        assert_suggestion_result(
            "he said some words trying to hit back at the center for brandishing him as a Pakistani at an NRC rally",
            BrandBrandish::default(),
            "he said some words trying to hit back at the center for branding him as a Pakistani at an NRC rally",
        )
    }

    #[test]
    fn correct_brandish_us() {
        assert_suggestion_result(
            "Our resolute determination for the ultimate quality and all-inclusive directory of food commodities brandish us as a flawless associate in B2B",
            BrandBrandish::default(),
            "Our resolute determination for the ultimate quality and all-inclusive directory of food commodities brand us as a flawless associate in B2B",
        )
    }

    #[test]
    fn correct_brandished_him() {
        assert_suggestion_result(
            "Frank discovers Myra brandished him with the letter 'R', for rapist.",
            BrandBrandish::default(),
            "Frank discovers Myra branded him with the letter 'R', for rapist.",
        )
    }

    #[test]
    fn correct_brandishes_him() {
        assert_suggestion_result(
            "Whether one turns a blind eye to Tim's wrongs or brandishes him a traitor will plant audiences in their own personal line in the sand.",
            BrandBrandish::default(),
            "Whether one turns a blind eye to Tim's wrongs or brands him a traitor will plant audiences in their own personal line in the sand.",
        )
    }
}

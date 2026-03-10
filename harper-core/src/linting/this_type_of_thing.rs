use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct ThisTypeOfThing {
    expr: Box<dyn Expr>,
}

impl Default for ThisTypeOfThing {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["this", "these", "that", "those"])
                    .t_ws()
                    .then(
                        SequenceExpr::word_set(&[
                            "kind", "kinds", "sort", "sorts", "type", "types",
                        ])
                        .t_ws(),
                    )
                    .t_aco("of")
                    .t_ws()
                    .then_any_of(vec![
                        // "thing" is common in this construction and won't be part of a compound noun.
                        Box::new(WordSet::new(&["thing", "things"])),
                        // Other singular nouns may be part of hard-to-determine compound nouns, but plural nouns won't.
                        Box::new(
                            SequenceExpr::default()
                                .then_kind_where(|k| k.is_plural_noun() && !k.is_singular_noun()),
                        ),
                    ]),
            ),
        }
    }
}

impl ExprLinter for ThisTypeOfThing {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn description(&self) -> &str {
        "Checks that the parts of `this/these type(s) of thing(s)` agree in grammatical number"
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        #[derive(PartialEq)]
        enum Num {
            Sg,
            Pl,
        }

        let (det_tok, type_tok, thing_tok) = (toks.first()?, toks.get(2)?, toks.last()?);
        let (type_kind, thing_kind) = (&type_tok.kind, &thing_tok.kind);
        let (det_span, type_span) = (det_tok.span, type_tok.span);
        let (det_chars, type_chars) = (det_span.get_content(src), type_span.get_content(src));
        let (det_num, type_num, thing_num) = (
            if det_chars.eq_any_ignore_ascii_case_str(&["this", "that"]) {
                Num::Sg
            } else {
                Num::Pl
            },
            if type_kind.is_plural_noun() {
                Num::Pl
            } else {
                Num::Sg
            },
            if thing_kind.is_plural_noun() {
                Num::Pl
            } else {
                Num::Sg
            },
        );
        if det_num == type_num && type_num == thing_num {
            return None;
        };

        enum Deixis {
            Proximal,
            Distal,
        }
        let deixis = if det_chars.eq_any_ignore_ascii_case_str(&["this", "these"]) {
            Deixis::Proximal
        } else {
            Deixis::Distal
        };

        enum Specie {
            Kind,
            Sort,
            Type,
        }
        let specie = match type_chars.first()? {
            'k' | 'K' => Specie::Kind,
            's' | 'S' => Specie::Sort,
            't' | 'T' => Specie::Type,
            _ => return None,
        };

        // Due to the logic above, when we get here we either have 1 singular and 2 plurals or 2 plurals and 1 singular.
        // Meaning one of the three varying words does not agree in number with the other two.
        let bad_tok = match (&det_num, &type_num, &thing_num) {
            (Num::Sg, Num::Sg, Num::Pl) => thing_tok,
            (Num::Sg, Num::Pl, Num::Sg) => type_tok,
            (Num::Sg, Num::Pl, Num::Pl) => det_tok,
            (Num::Pl, Num::Sg, Num::Sg) => det_tok,
            (Num::Pl, Num::Sg, Num::Pl) => type_tok,
            (Num::Pl, Num::Pl, Num::Sg) => thing_tok,
            _ => return None,
        };

        Some(Lint {
            span: bad_tok.span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::replace_with_match_case(
                if bad_tok == det_tok {
                    match (det_num, deixis) {
                        (Num::Sg, Deixis::Proximal) => "these",
                        (Num::Sg, Deixis::Distal) => "those",
                        (Num::Pl, Deixis::Proximal) => "this",
                        (Num::Pl, Deixis::Distal) => "that",
                    }
                } else if bad_tok == type_tok {
                    match (type_num, specie) {
                        (Num::Sg, Specie::Kind) => "kinds",
                        (Num::Pl, Specie::Kind) => "kind",
                        (Num::Sg, Specie::Sort) => "sorts",
                        (Num::Pl, Specie::Sort) => "sort",
                        (Num::Sg, Specie::Type) => "types",
                        (Num::Pl, Specie::Type) => "type",
                    }
                } else if bad_tok == thing_tok {
                    match thing_num {
                        Num::Sg => "things",
                        Num::Pl => "thing",
                    }
                } else {
                    return None;
                }
                .chars()
                .collect(),
                bad_tok.span.get_content(src),
            )],
            message: "The grammatical number of the determiner and the two nouns must agree."
                .to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{tests::assert_suggestion_result, this_type_of_thing::ThisTypeOfThing};

    #[test]
    fn fix_that_kind_of_things() {
        assert_suggestion_result(
            "it's specific to TypeScript and not Go nor Python can do that kind of things",
            ThisTypeOfThing::default(),
            "it's specific to TypeScript and not Go nor Python can do that kind of thing",
        );
    }

    #[test]
    fn fix_that_sort_of_things() {
        assert_suggestion_result(
            "there isn't a trivial stb-like ready-to-use C++ library to do that sort of things",
            ThisTypeOfThing::default(),
            "there isn't a trivial stb-like ready-to-use C++ library to do that sort of thing",
        );
    }

    #[test]
    fn fix_these_kind_of_things() {
        assert_suggestion_result(
            "For these kind of things, I think it would be great to have a user-defined field which can be used to search for files.",
            ThisTypeOfThing::default(),
            "For these kinds of things, I think it would be great to have a user-defined field which can be used to search for files.",
        );
    }

    #[test]
    fn fix_these_sort_of_thing() {
        assert_suggestion_result(
            "People from npm actually get death threats for these sort of thing",
            ThisTypeOfThing::default(),
            "People from npm actually get death threats for this sort of thing",
        );
    }

    #[test]
    fn fix_these_sort_of_things() {
        assert_suggestion_result(
            "I suppose doing these sort of things should be legal",
            ThisTypeOfThing::default(),
            "I suppose doing these sorts of things should be legal",
        );
    }

    #[test]
    fn fix_these_sorts_of_thing() {
        assert_suggestion_result(
            "What I would like to understand is what the syntactic structure is for these sorts of things.",
            ThisTypeOfThing::default(),
            "What I would like to understand is what the syntactic structure is for these sorts of things.",
        );
    }

    #[test]
    fn fix_these_type_of_things() {
        assert_suggestion_result(
            "You can use the Symfony validator to validate these type of things easily.",
            ThisTypeOfThing::default(),
            "You can use the Symfony validator to validate these types of things easily.",
        );
    }

    #[test]
    fn fix_this_kind_of_things() {
        assert_suggestion_result(
            "this kind of things could exists in languages like Haskell which supports higher kinded types",
            ThisTypeOfThing::default(),
            "this kind of thing could exists in languages like Haskell which supports higher kinded types",
        );
    }

    #[test]
    fn fix_this_sort_of_things() {
        assert_suggestion_result(
            "I have heard this sort of things happening in the movie industry but it's appalling that it happens in the business world too",
            ThisTypeOfThing::default(),
            "I have heard this sort of thing happening in the movie industry but it's appalling that it happens in the business world too",
        );
    }

    #[test]
    fn fix_this_type_of_things() {
        assert_suggestion_result(
            "how to handle this type of things",
            ThisTypeOfThing::default(),
            "how to handle this type of thing",
        );
    }

    #[test]
    fn fix_those_kind_of_things() {
        assert_suggestion_result(
            "uh, so I was playing both of those kind of things",
            ThisTypeOfThing::default(),
            "uh, so I was playing both of those kinds of things",
        );
    }
}

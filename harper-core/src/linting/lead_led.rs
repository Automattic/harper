use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct LeadLed {
    expr: SequenceExpr,
}

impl Default for LeadLed {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_capitalization_of("lead"),
        }
    }
}

impl ExprLinter for LeadLed {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        // So far the pattern we match is only a single token, but this can prevent mistakes
        // if we expand the pattern in the future.
        let span = find_the_only_token_matching(toks, src, |t, s| t.get_ch(s).eq_str("lead"))?.span;

        // Forward-looking checks

        if let Some((_, after)) = ctx
            && let [ws_h, word, _after2 @ ..] = after
        {
            if ws_h.kind.is_whitespace() {
                eprintln!("⏩ 'lead' <ws>");
                if word.kind.is_noun() {
                    eprintln!(
                        "❌ 'lead' looks like attributive noun qualifying another noun: 'lead {}'",
                        word.get_str(src)
                    );
                    return None;
                }
            } else if ws_h.kind.is_hyphen() {
                eprintln!("⏩ 'lead' <hyphen>");
                if word.get_ch(src).eq_str("gen") {
                    eprintln!("❌ 'lead' is part of a hyphenated compound word: 'lead-gen'");
                    return None;
                }
            } else {
                eprintln!("⏩ 'lead' <not ws or hyphen>");
            }
        }

        // Backward-looking checks

        if let Some((before, _)) = ctx
            && let [before2 @ .., word, ws] = before
            && ws.kind.is_whitespace()
            && word.kind.is_word()
        {
            if word.kind.is_plural_noun() {
                eprintln!("⏩ 'lead' <plural noun>");
                eprintln!(
                    "❌ 'load' agrees with previous plural noun: '{} lead'",
                    word.get_str(src)
                );
                return None;
            }
            let ch = word.get_ch(src);
            if ch.eq_str("to") {
                eprintln!("❌ 'lead' follows 'to' so is a grammatical infinitive/present verb");
            }
            if ch.eq_str("which")
                && let [_before3 @ .., word2, ws] = before2
                && ws.kind.is_whitespace()
                && word2.kind.is_word()
                && word2.kind.is_plural_noun()
            {
                eprintln!(
                    "❌ 'lead' agrees with previous plural noun: '{} which lead'",
                    word2.get_str(src)
                );
                return None;
            }
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "led",
                span.get_content(src),
            )],
            message: "Is this supposed to be `led`, the past form of `to lead`?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::LeadLed;

    // Actual errors we should try hard to fix

    // sg. demonst. pron./det. + aux have + "lead" + "to" + pl. np.
    // NOTE: ambiguous - "this has lead to make it heavier" - CHECK for next word verb infin.
    #[test]
    fn fix_this_has_lead_to_quant_things() {
        assert_suggestion_result(
            "This has lead to many mistakes",
            LeadLed::default(),
            "This has led to many mistakes",
        );
    }

    #[test]
    fn fix_this_has_lead_to_adj_npl() {
        assert_suggestion_result(
            "This has lead to divergent implementations in the unspecified cases. Well-behaved compilers should not generate such undocumented combinations of instruction prefixes with base encodings.",
            LeadLed::default(),
            "This has led to divergent implementations in the unspecified cases. Well-behaved compilers should not generate such undocumented combinations of instruction prefixes with base encodings.",
        );
    }

    // sg. demonstrative pronoun/det. + "lead" + "to" + mass np.
    // NOTE: ambiguous - "use this lead to make it heavier" - CHECK for next word verb infin.
    #[test]
    fn fix_in_the_past_this_lead_to_stuff() {
        assert_suggestion_result(
            "In the past this lead to wasteful military spending",
            LeadLed::default(),
            "In the past this led to wasteful military spending",
        );
    }

    // sg. np. "lead" + "to" + mass noun
    #[test]
    fn fix_so_a_thing_lead_to_stuff() {
        assert_suggestion_result(
            "So a lack of competition lead to stagnation.",
            LeadLed::default(),
            "So a lack of competition led to stagnation.",
        )
    }

    // I was lead to incorrectly believe that Argentina had always been a protectionist country
    // sg. 1st pers. pron. + "was" + "lead" + "to" + (adv.) + verb infin.
    #[test]
    fn fix_i_was_lead_to_adv_believe_that_something_is_the_case() {
        assert_suggestion_result(
            "I was lead to incorrectly believe that Argentina had always been a protectionist country",
            LeadLed::default(),
            "I was led to incorrectly believe that Argentina had always been a protectionist country",
        )
    }

    #[test]
    fn fix_sg_noun_phrase_which_lead_to_an_event_is_hilarious() {
        assert_suggestion_result(
            "IBM dropping this after latest Anthropic announcement which lead to a drop in IBM stocks is hilarious",
            LeadLed::default(),
            "IBM dropping this after latest Anthropic announcement which led to a drop in IBM stocks is hilarious",
        )
    }

    #[test]
    fn fix_which_lead_to_the_thing_being_acted_upon() {
        assert_suggestion_result(
            "due to complications with the Cygnus developed DMX library (which lead to the Linux version being selected for release)",
            LeadLed::default(),
            "due to complications with the Cygnus developed DMX library (which led to the Linux version being selected for release)",
        )
    }

    #[test]
    fn fix_which_adv_lead_me_to_v_infin_the_thing() {
        assert_suggestion_result(
            "I ended up using perf in production, which indirectly lead me to understand the data race",
            LeadLed::default(),
            "I ended up using perf in production, which indirectly led me to understand the data race",
        )
    }

    #[test]
    fn fix_det_noun_has_a_thing_comma_which_lead_to_det_stuff_being_pp() {
        assert_suggestion_result(
            "Ulala's model has a low polygon count, which lead to her sex appeal being defined through her movement according to Mizuguchi",
            LeadLed::default(),
            "Ulala's model has a low polygon count, which led to her sex appeal being defined through her movement according to Mizuguchi",
        )
    }

    #[test]
    fn fix_sg_pron_lead_sg_pron() {
        assert_suggestion_result(
            "I don’t know what kind of hardware Apple tests on to notice this because it was never noticeable to me, but it lead me down a bit of a rabbit hole.",
            LeadLed::default(),
            "I don’t know what kind of hardware Apple tests on to notice this because it was never noticeable to me, but it led me down a bit of a rabbit hole.",
        );
    }

    // Potential false positives we should not flag

    #[test]
    fn dont_flag_let_stuff_lead_the_thing() {
        assert_no_lints(
            "We should let love lead the way in our actions.",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_things_modal_lead_to_things() {
        assert_no_lints(
            "missing or extra commas can lead to JSON syntax errors.",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_lead_research_assistant() {
        assert_no_lints(
            "Lead Research Assistant - Identifies and qualifies high-quality leads by analyzing your product",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_i_desire_to_lead() {
        assert_no_lints("I desire to lead a T-shaped existence.", LeadLed::default());
    }

    #[test]
    fn dont_flag_attr_noun_gen() {
        assert_no_lints(
            "Tiledesk - All-in-one customer engagement platform from lead-gen to post-sales",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_attr_noun_generation() {
        assert_no_lints("AI Lead Generation Agent", LeadLed::default());
    }

    #[test]
    fn dont_flag_things_that_lead_up_to_a_thing() {
        assert_no_lints(
            "And because you have portals that lead up to a stairwell or whatever",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_allowing_an_attr_agent_to_do_something() {
        assert_no_lints(
            "allowing a lead agent to coordinate a team of specialized agents in parallel",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_attr_writer_quit() {
        assert_no_lints("the lead writer quit reasonably so", LeadLed::default());
    }

    #[test]
    fn dont_flag_company_fired_their_attr_composer() {
        assert_no_lints(
            "Bungie also fired their lead composer and audio director over workplace drama",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_the_attr_composer_who_did_stuff() {
        assert_no_lints(
            "Like the lead composer who did all the work to complete the game's OST",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_this_modal_lead_to_stuff() {
        assert_no_lints(
            "This could lead to unauthorized access using your credentials and potentially compromise sensitive data or functionality.",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_imperative_lead_with_the_thing() {
        assert_no_lints("Lead with the next action.", LeadLed::default());
    }

    // NOTE: "might" is also a noun: "Their might led to their victory" is grammatical.
    #[test]
    fn dont_flag_using_other_things_modal_lead_to_things() {
        assert_no_lints(
            "Using other versions might lead to dependency errors.",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_note_that_things_modal_lead_to_things() {
        assert_no_lints(
            "Note that TensorRT optimizations may lead to slight variations or a small drop in output quality.",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_using_things_modal_lead_to_stuff() {
        assert_no_lints(
            "Using less capable models may lead to reduced performance.",
            LeadLed::default(),
        );
    }

    #[test]
    fn dont_flag_lead_as_head_of_compound_noun() {
        assert_no_lints(
            "@OfficialLoganKProduct Lead @ GeminiPosts about robotics vision",
            LeadLed::default(),
        );
        assert_no_lints(
            "@skalskip92Open Source Lead @ RoboflowBuilding tools for vision AI",
            LeadLed::default(),
        );
        assert_no_lints("a QA lead who opens a real browser", LeadLed::default());
        assert_no_lints("Developer Experience Lead", LeadLed::default());
        assert_no_lints("QA Lead", LeadLed::default());
        assert_no_lints("Engineering Lead at Shopify", LeadLed::default());
    }

    #[test]
    fn dont_flag_lead_as_attributive_part_of_compound_noun() {
        assert_no_lints("Lead Platform Developer at MindGeek", LeadLed::default());
    }

    // Ambiguous

    // we did use them as explanation points which lead to this piece of software.
    // NOTE: correct: explanation points lead to ...
    // NOTE: incorrect: (fact) which leads/led to ...
    #[test]
    fn dont_flag_ambiguous_which_lead_to() {
        assert_no_lints(
            "we did use them as explanation points which lead to this piece of software",
            LeadLed::default(),
        );
    }

    // Be Specific - Clear requirements lead to better results
    // NOTE things lead to things AND things led to things - both grammatical
    // NOTE but semantically, "be" is an imperative, which implies verb lemma: "Be this - foos lead to bars"
    #[test]
    fn dont_flag_be_specific() {
        assert_no_lints(
            "Be Specific - Clear requirements lead to better results",
            LeadLed::default(),
        );
    }

    // Know what the council can't answer — verdicts lead with Unresolved Questions and Recommended Next Steps,
    // NOTE should we "know that verdicts do lead" OR should we "know that verdicts once led"?
    #[test]
    fn dont_flag_verdicts_lead_with_unresolved_questions() {
        assert_no_lints(
            "Know what the council can't answer — verdicts lead with Unresolved Questions and Recommended Next Steps",
            LeadLed::default(),
        );
    }
}

use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Sentence},
};

pub struct PluralDecades {
    expr: SequenceExpr,
}

impl Default for PluralDecades {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_cardinal_number()
                .then_apostrophe()
                .t_aco("s"),
        }
    }
}

impl ExprLinter for PluralDecades {
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags plural decades erroneously using an apostrophe before the `s`"
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("📅 {}", format_lint_match(toks, ctx, src));
        if toks.len() != 3 {
            return None;
        }

        let (decade_chars, s_chars) =
            (toks[0].span.get_content(src), toks[2].span.get_content(src));

        // TODO does not yet support two-digit decades like 80's
        if decade_chars.len() != 4 || !decade_chars.ends_with(&['0']) {
            return None;
        }

        let (before, after): (Option<&[Token]>, Option<&[Token]>) = match ctx {
            Some((pw, nw)) => {
                if pw.is_empty() {
                    if nw.is_empty() {
                        (None, None)
                    } else {
                        (None, Some(nw))
                    }
                } else if nw.is_empty() {
                    (Some(pw), None)
                } else {
                    (Some(pw), Some(nw))
                }
            }
            None => (None, None),
        };

        enum UsageJudgment {
            NotMistake,
            IsMistake,
            Unsure,
        }

        struct Context<'a> {
            sep_is_hyphen: bool,
            word: &'a [char],
        }

        let before_context = if before.is_some_and(|b| b.len() >= 2)
            && let [.., pw, psphy] = before.unwrap()
            && (psphy.kind.is_whitespace() || psphy.kind.is_hyphen())
            && pw.kind.is_word()
        {
            Some(Context {
                sep_is_hyphen: psphy.kind.is_hyphen(),
                word: pw.span.get_content(src),
            })
        } else {
            None
        };

        let after_context = if after.is_some_and(|a| a.len() >= 2)
            && let [nsphy, nw, ..] = after.unwrap()
            && (nsphy.kind.is_whitespace() || nsphy.kind.is_hyphen())
            && nw.kind.is_word()
        {
            Some(Context {
                sep_is_hyphen: nsphy.kind.is_hyphen(),
                word: nw.span.get_content(src),
            })
        } else {
            None
        };

        let judgment = match (&before_context, &after_context) {
            // Temporal words before the decade suggest the apostrophe is a mistake
            (Some(before_ctx), _)
                if before_ctx
                    .word
                    .eq_any_ignore_ascii_case_str(&["early", "mid", "late"]) =>
            {
                UsageJudgment::IsMistake
            }
            // Hyphen before suggests username, not a mistake
            (Some(before_ctx), _) if before_ctx.sep_is_hyphen => UsageJudgment::NotMistake,
            // "style" after the decade suggests the apostrophe is a mistake
            (_, Some(after_ctx)) if after_ctx.word.eq_ignore_ascii_case_str("style") => {
                UsageJudgment::IsMistake
            }
            // Default case
            _ => UsageJudgment::Unsure,
        };

        if !matches!(judgment, UsageJudgment::IsMistake) {
            return None;
        }

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Usage,
            message: "Plural decades do not use an apostrophe before the `s`".to_string(),
            suggestions: vec![Suggestion::ReplaceWith([decade_chars, s_chars].concat())],
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod lints {
    use super::PluralDecades;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    // Made-up examples

    #[test]
    #[ignore = "wip"]
    fn eighties() {
        assert_lint_count("in the 1980's", PluralDecades::default(), 1);
    }

    #[test]
    #[ignore = "wip"]
    fn nineties() {
        assert_lint_count("the 1990’s were a bit grungy", PluralDecades::default(), 1);
    }

    #[test]
    fn dont_flag_three_digits() {
        assert_lint_count(
            "200's doesn't look like a decade",
            PluralDecades::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_five_digits() {
        assert_lint_count(
            "20000's doesn't look like a decade",
            PluralDecades::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_with_thousands_separator() {
        assert_lint_count(
            "Nobody says \"in the 1,950's\".",
            PluralDecades::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_not_ending_with_0() {
        assert_lint_count("1977's best month was October", PluralDecades::default(), 0);
    }

    // Real-world examples using sentences found on GitHub

    // 1900s (2 examples)

    #[test]
    #[ignore = "Too ambiguous to lint?"]
    // 1900 is probably a username, but otherwise it looks like a decade with the common apostrophe error
    fn dont_flag_ambiguous_1900s_nppl() {
        assert_no_lints(
            "star and fork 1900's gists by creating an account on GitHub.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_1900s_npsg() {
        assert_suggestion_result(
            "Children Aged 0-4 in 1900's Norway.",
            PluralDecades::default(),
            "Children Aged 0-4 in 1900s Norway.",
        );
    }

    // 1910s (1 example)

    #[test]
    #[ignore = "Looks like a product name, not a decade"]
    fn ignore_hp_1910s() {
        assert_no_lints("Add support for HP 1910's", PluralDecades::default());
    }

    // 1950s (5 examples)

    #[test]
    #[ignore = "Grammar would be correct but the computer is from 1951 so must be a mistake for 1950s"]
    fn dont_flag_ambiguous_1950s_npsg() {
        assert_no_lints(
            "Simulator for 1950's MIT Whirlwind Computer.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_1950s() {
        assert_suggestion_result(
            "Using the sandbox on the right, write and execute a query to return people born in the 1950's (1950 - 1959)",
            PluralDecades::default(),
            "Using the sandbox on the right, write and execute a query to return people born in the 1950s (1950 - 1959)",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_a_adj_1950s_npsg() {
        assert_suggestion_result(
            "Wave digital filter based emulation of a famous 1950's tube stereo limiter.",
            PluralDecades::default(),
            "Wave digital filter based emulation of a famous 1950s tube stereo limiter.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_1950s_npsg() {
        assert_suggestion_result(
            "1950's elevator randomly gets stuck",
            PluralDecades::default(),
            "1950s elevator randomly gets stuck",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_from_1950s_npsg() {
        assert_suggestion_result(
            "documenting my family's camera business, from 1950's England, run by my father",
            PluralDecades::default(),
            "documenting my family's camera business, from 1950s England, run by my father",
        );
    }

    // 1960s (4 examples)

    #[test]
    #[ignore = "wip"]
    fn fix_a_adj_1960s_npsg() {
        assert_suggestion_result(
            "Emulating a rare 1960's educational computer.",
            PluralDecades::default(),
            "Emulating a rare 1960s educational computer.",
        );
    }

    #[test]
    #[ignore = "Ambiguous - could be referring to the specific year 1960 or the decade 1960s"]
    fn ignore_ambiguous_1960s_npsg() {
        assert_no_lints(
            "MyTheil - 1960's IP Sprint Hillarity.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_1960s_npsg() {
        assert_suggestion_result(
            "Punchbag game inspired by 1960's TV Show Batman!",
            PluralDecades::default(),
            "Punchbag game inspired by 1960s TV Show Batman!",
        );
    }

    // Several "SP entrances" in 1960's Aperture have visible nodraw around entrance door
    #[test]
    #[ignore = "ambiguous, not sure what it means"]
    fn ignore_in_1960s_aperture() {
        assert_no_lints(
            "Several \"SP entrances\" in 1960's Aperture have visible nodraw around entrance door",
            PluralDecades::default(),
        );
    }

    // 1970s (7 examples)

    #[test]
    #[ignore = "wip"]
    fn fix_1970s_npsg() {
        assert_suggestion_result(
            "1970's chess engine CHEKMO-II + UCI adapter.",
            PluralDecades::default(),
            "1970s chess engine CHEKMO-II + UCI adapter.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_1970s_nppl() {
        assert_suggestion_result(
            "listsockets printing 1970's dates.",
            PluralDecades::default(),
            "listsockets printing 1970s dates.",
        );
    }

    #[test]
    fn fix_in_a_1970s_style_npsg() {
        assert_suggestion_result(
            "I tried to create some catwalk in a 1970's style level.",
            PluralDecades::default(),
            "I tried to create some catwalk in a 1970s style level.",
        );
    }

    #[test]
    #[ignore = "Grammar would be correct but Pong is from 1972 so must be a mistake for 1970s"]
    fn fix_1970s_pong_game() {
        assert_suggestion_result(
            "1970's Pong game rewritten in C++.",
            PluralDecades::default(),
            "1970s Pong game rewritten in C++.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_1970s_in_parens() {
        assert_suggestion_result(
            "Convert a MIDI file to a record compatible with vintage (1970's) Fisher Price music box record players",
            PluralDecades::default(),
            "Convert a MIDI file to a record compatible with vintage (1970s) Fisher Price music box record players",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_1970s() {
        assert_suggestion_result(
            "may have begun, depending on when you start counting, in the 1970's.",
            PluralDecades::default(),
            "may have begun, depending on when you start counting, in the 1970s.",
        );
    }

    // 1980s (12 examples)

    #[test]
    #[ignore = "wip"]
    fn fix_from_the_1980s_like() {
        assert_suggestion_result(
            "Old Stern tables from the 1980's like Flight 2000, Catacomb, etc. are playing audio samples twice, it seems.",
            PluralDecades::default(),
            "Old Stern tables from the 1980s like Flight 2000, Catacomb, etc. are playing audio samples twice, it seems.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_its_the_1980s() {
        assert_suggestion_result(
            "Declarative Rapid Application Development like it's the 1980's again.",
            PluralDecades::default(),
            "Declarative Rapid Application Development like it's the 1980s again.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_from_the_1980s_end() {
        assert_suggestion_result(
            "Former countries from the 1980's",
            PluralDecades::default(),
            "Former countries from the 1980s",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_a_1980s_npsg() {
        assert_suggestion_result(
            "A re-imiplementation of a classic 1980's DOS game, but in D.",
            PluralDecades::default(),
            "A re-imiplementation of a classic 1980s DOS game, but in D.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_of_the_1980s() {
        assert_suggestion_result(
            "The Pugputer is a little labor of love, made as a tribute to the early home computers of the 1980's.",
            PluralDecades::default(),
            "The Pugputer is a little labor of love, made as a tribute to the early home computers of the 1980s.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_of_the_1980s_npsg() {
        assert_suggestion_result(
            "FPGA implementation of the 1980's \"Music 5000\" wavetable synthesiser",
            PluralDecades::default(),
            "FPGA implementation of the 1980s \"Music 5000\" wavetable synthesiser",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_based_off_of_the_1980s_npsg() {
        assert_suggestion_result(
            "Space Fortress is based off of the 1980's vector-based arcade game by Cinematronics called Star Castle.",
            PluralDecades::default(),
            "Space Fortress is based off of the 1980s vector-based arcade game by Cinematronics called Star Castle.",
        );
    }

    #[test]
    #[ignore = "Ambiguous - could be referring to the specific year 1980 or the decade 1980s"]
    fn ignore_ambiguous_1980s() {
        assert_no_lints(
            "1980's Old aperture coop checkpoint uses the timer signage instead of checkmarks.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_1980s_nppl() {
        assert_suggestion_result(
            "A project resurrecting the classic 1980's Usborne Computer Guide books, for a new generation of programmers.",
            PluralDecades::default(),
            "A project resurrecting the classic 1980s Usborne Computer Guide books, for a new generation of programmers.",
        );
    }

    #[test]
    #[ignore = "Missing determiner is out of the scope of the current version of this linter"]
    fn fix_the_end_of_missing_determiner_1980s() {
        assert_suggestion_result(
            "System software for TIM011, a school computer from the end of 1980's made in former Yugoslavia",
            PluralDecades::default(),
            "System software for TIM011, a school computer from the end of the 1980s made in former Yugoslavia",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_1980s_for() {
        assert_suggestion_result(
            "HMSL was originally released in the 1980's for Mac Plus and Amiga",
            PluralDecades::default(),
            "HMSL was originally released in the 1980s for Mac Plus and Amiga",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_the_adj_1980s_npsg() {
        assert_suggestion_result(
            "Modern remake of Pole Position, the classic 1980's arcade racing game from Atari.",
            PluralDecades::default(),
            "Modern remake of Pole Position, the classic 1980s arcade racing game from Atari.",
        );
    }

    // 1990s (10 examples)

    #[test]
    #[ignore = "wip"]
    fn fix_the_adj_1990s_npsg() {
        assert_suggestion_result(
            "42 3d Graphics Project, recreating the classic 1990's game Wolfienstien 3d",
            PluralDecades::default(),
            "42 3d Graphics Project, recreating the classic 1990s game Wolfienstien 3d",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_a_1990s_npsg() {
        assert_suggestion_result(
            "A 1990's Retro linux-rice for Hyprland or Sway, based on Quickshell.",
            PluralDecades::default(),
            "A 1990s Retro linux-rice for Hyprland or Sway, based on Quickshell.",
        );
    }

    #[test]
    #[ignore = "Missing determiner is out of the scope of the current version of this linter"]
    fn lacks_determiner_stuck_in_1990s() {
        assert_suggestion_result(
            "Docs are stuck in 1990's - need AWS or Azure example",
            PluralDecades::default(),
            "Docs are stuck in the 1990s - need AWS or Azure example",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_the_1990s_npsg() {
        assert_suggestion_result(
            "This program recreates the 1990's arcade game \"Boulder Dash.\"",
            PluralDecades::default(),
            "This program recreates the 1990s arcade game \"Boulder Dash.\"",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_1990s_comma() {
        assert_suggestion_result(
            "In the 1990's, Innovative Computer Solutions released multiple programs for the Newton MessagePad as shareware",
            PluralDecades::default(),
            "In the 1990s, Innovative Computer Solutions released multiple programs for the Newton MessagePad as shareware",
        );
    }

    #[test]
    fn fix_a_mid_1990s_npsg() {
        assert_suggestion_result(
            "This repository is a modernization of a mid 1990's implementation of the ZMODEM protocol called 'zmtx-zmrx'.",
            PluralDecades::default(),
            "This repository is a modernization of a mid 1990s implementation of the ZMODEM protocol called 'zmtx-zmrx'.",
        );
    }

    #[test]
    #[ignore = "Missing determiner is out of the scope of the current version of this linter"]
    fn lacks_determiner_written_in_java_in_1990s() {
        assert_suggestion_result(
            "JMud, mud server written in Java in 1990's.",
            PluralDecades::default(),
            "JMud, mud server written in Java in the 1990s.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_a_adj_1990s_npsg() {
        assert_suggestion_result(
            "Port of a famous 1990's fighting game to MSX2.",
            PluralDecades::default(),
            "Port of a famous 1990s fighting game to MSX2.",
        );
    }

    #[test]
    #[ignore = "circa 1990s doesn't sound like natural English. changing to 'circa the 1990s' is out of scope for this linter"]
    fn fix_circa_1990s() {
        assert_suggestion_result(
            "Inspired by Digimon \"Digivices\" tamagotchis circa 1990's.",
            PluralDecades::default(),
            "Inspired by Digimon \"Digivices\" tamagotchis circa the 1990s.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_for_1990s_nppl() {
        assert_suggestion_result(
            "Daughter-board for reprogramming 1990's Toyota ECUs",
            PluralDecades::default(),
            "Daughter-board for reprogramming 1990s Toyota ECUs",
        );
    }

    // 2000s (5 example)

    #[test]
    fn fix_2000s_style() {
        assert_suggestion_result(
            "2000's-style media library for vintage cellphones (Nokia, etc.)",
            PluralDecades::default(),
            "2000s-style media library for vintage cellphones (Nokia, etc.)",
        );
    }

    #[test]
    // #[ignore = "Looks like a username, not a decade"]
    fn ignore_fork_username_hyphen_2000s_nppl() {
        assert_no_lints(
            "star and fork vishal-2000's gists by creating an account on GitHub.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_2000s() {
        assert_suggestion_result(
            "Simulator engine for reproducing LCD games made by McDonald's in the 2000's.",
            PluralDecades::default(),
            "Simulator engine for reproducing LCD games made by McDonald's in the 2000s.",
        );
    }

    #[test]
    #[ignore = "Missing determiner is out of the scope of the current version of this linter"]
    fn fix_in_the_early_2000s_missing_determiner() {
        assert_suggestion_result(
            "Silo was originally released in early 2000's using LLNL-home-grown license verbiage.",
            PluralDecades::default(),
            "Silo was originally released in the early 2000s using LLNL-home-grown license verbiage.",
        );
    }

    #[test]
    fn ignore_view_username_hyphen_2000s_npsg() {
        assert_no_lints(
            "View lxw-2000's full-sized avatar.",
            PluralDecades::default(),
        );
    }

    // 2010s (2 examples)

    #[test]
    #[ignore = "Sinnemäki 2010 here refers to the author's publication from 2010"]
    fn ignore_author_2010s_publication_reference() {
        assert_no_lints(
            "CLDF version of Sinnemäki 2010's dataset on zero marking and word order",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "Looks like a product name, not a decade"]
    fn ignore_bazel_calls_vs_2010s_cl() {
        assert_no_lints(
            "Bazel calls VS 2010's cl with /DEBUG:FASTLINK",
            PluralDecades::default(),
        );
    }

    // 2020s (5 examples)

    #[test]
    #[ignore = "Ambiguous. Looks like awkward wording for `the IEEE CEC's 2020 Strategy Card Game AI Competition"]
    fn ignore_ambiguous_2020s() {
        assert_no_lints(
            "This is a bot for Legends of Code and Magic submitted to the IEEE CEC 2020's Strategy Card Game AI Competition.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "Ambiguous. Maybe awkward wording for `CSSPI Fall 2020's frontend`??"]
    fn ignore_ambiguous_2020s_2() {
        assert_no_lints(
            "CSSPI Fall 2020's frontend mobile web application utilizing React Native.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "A human can tell from the comparison to `2024's` that `2020's` refers to a specific year/version/release. Harper has no way to tell."]
    fn ignore_ambiguous_2020s_2024s() {
        assert_no_lints(
            "App that converts MSFS 2020's DDS texture format to MSFS 2024's KTX2 format",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "eRum 2020 is probably an event name, not a decade"]
    fn ignore_erum_2020s() {
        assert_no_lints(
            "A repository for purposes of eRum 2020's workshop \"Image processing and computer vision with R\", held on Saturday, June 20, 2020.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "Ambiguous. Not sure what it means"]
    fn ignore_ambiguous_2020s_3() {
        assert_no_lints(
            "Crashing upon loading saved game in 2020's",
            PluralDecades::default(),
        );
    }

    // Multiple decades (6 examples)

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_1990s_and_the_2000s() {
        assert_suggestion_result(
            "NTXShape, a converter I developed in the 1990's and maintained through the 2000's",
            PluralDecades::default(),
            "NTXShape, a converter I developed in the 1990s and maintained through the 2000s",
        );
    }

    #[test]
    fn fix_early_2000s_early_1990s_early_1980s() {
        assert_suggestion_result(
            "CDISC since 2005, XML since the early 2000's, @SAS since the early 1990's, Programming since the early 1980's.",
            PluralDecades::default(),
            "CDISC since 2005, XML since the early 2000s, @SAS since the early 1990s, Programming since the early 1980s.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_dates_in_the_1960s_comma_1950s_early_1900s() {
        assert_suggestion_result(
            "OK for dates in the 1960's, 1950's... Now... I expect 1939-12-31T23 ... Was Belgium ever not in UTC+1 timezone in early 1900's ?",
            PluralDecades::default(),
            "OK for dates in the 1960s, 1950s... Now... I expect 1939-12-31T23 ... Was Belgium ever not in UTC+1 timezone in early 1900s ?",
        );
    }

    #[test]
    fn fix_late_1970s_early_1980s() {
        assert_suggestion_result(
            "Late 1970's/Early 1980's Text Adventure Game from the Mainframe era",
            PluralDecades::default(),
            "Late 1970s/Early 1980s Text Adventure Game from the Mainframe era",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_in_the_1970s_and_early_1980s() {
        assert_suggestion_result(
            "We modeled the gas mileage of 398 cars built in the 1970's and early 1980's",
            PluralDecades::default(),
            "We modeled the gas mileage of 398 cars built in the 1970s and early 1980s",
        );
    }

    #[test]
    fn fix_from_the_late_1970s_early_1980s() {
        assert_suggestion_result(
            "Europe Card Bus (ECB) is a Retro CPU Bus standard from the late 1970's / early 1980's.",
            PluralDecades::default(),
            "Europe Card Bus (ECB) is a Retro CPU Bus standard from the late 1970s / early 1980s.",
        );
    }
}

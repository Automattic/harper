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
            Legit,
            Mistake,
            Unsure,
        }

        let mut judgment = UsageJudgment::Unsure;

        // early-2000's / mid-1990's / late-1980's are mistakes we can fix
        if before.is_some_and(|b| b.len() >= 2)
            && let [.., pw, psphy] = before.unwrap()
            && (psphy.kind.is_whitespace() || psphy.kind.is_hyphen())
            && pw
                .span
                .get_content(src)
                .eq_any_ignore_ascii_case_str(&["early", "mid", "late"])
        {
            judgment = UsageJudgment::Mistake;
        }

        // 1970's style level / 2000's-style media library
        if after.is_some_and(|a| a.len() >= 2)
            && let [nsphy, nw, ..] = after.unwrap()
            && (nsphy.kind.is_whitespace() || nsphy.kind.is_hyphen())
            && nw.span.get_content(src).eq_ignore_ascii_case_str("style")
        {
            judgment = UsageJudgment::Mistake;
        }

        if !matches!(judgment, UsageJudgment::Mistake) {
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

    // 1900s (1 example)

    #[test]
    #[ignore = "Too ambiguous to lint?"]
    // 1900 is probably a username, but otherwise it looks like a decade with the common apostrophe error
    fn dont_flag_ambiguous_1900s_nppl() {
        assert_no_lints(
            "star and fork 1900's gists by creating an account on GitHub.",
            PluralDecades::default(),
        );
    }

    // 1950s (3 examples)

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

    // 1960s (2 examples)

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

    // 1970s (5 examples)

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

    // 1980s (7 examples)

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

    // 1990s (8 examples)

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

    // 2000s (1 example)

    #[test]
    fn fix_2000s_style() {
        assert_suggestion_result(
            "2000's-style media library for vintage cellphones (Nokia, etc.)",
            PluralDecades::default(),
            "2000s-style media library for vintage cellphones (Nokia, etc.)",
        );
    }

    // 2010s (1 example)

    #[test]
    #[ignore = "Sinnemäki 2010 here refers to the author's publication from 2010"]
    fn ignore_author_2010s_publication_reference() {
        assert_no_lints(
            "CLDF version of Sinnemäki 2010's dataset on zero marking and word order",
            PluralDecades::default(),
        );
    }

    // 2020s (3 examples)

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

    // Multiple decades (4 examples)

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
}

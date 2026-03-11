use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    linting::{LintKind, Suggestion},
};

pub fn match_to_lint_two_digits(
    toks: &[Token],
    src: &[char],
    decade: &[char],
    suffix: &[char],
    before: Option<&[Token]>,
    after: Option<&[Token]>,
) -> Option<Lint> {
    let empty_tokens: &[Token] = &[];
    let ctx_for_debug = match (before, after) {
        (Some(p), Some(n)) => Some((p, n)),
        (Some(p), None) => Some((p, empty_tokens)),
        (None, Some(n)) => Some((empty_tokens, n)),
        (None, None) => None,
    };

    eprintln!(
        "📅 {}",
        crate::linting::debug::format_lint_match(toks, ctx_for_debug, src)
    );

    let (mut preprepre, mut prepre, mut pre, mut next, mut nextnext): (
        Option<&Token>,
        Option<&Token>,
        Option<&Token>,
        Option<&Token>,
        Option<&Token>,
    ) = (None, None, None, None, None);
    if let Some(before) = before {
        if !before.is_empty() {
            pre = before.get_rel(-1);
            prepre = before.get_rel(-2);
            preprepre = before.get_rel(-3);
        }
    }
    if let Some(after) = after {
        if !after.is_empty() {
            next = after.get_rel(0);
            if after.len() > 1 {
                nextnext = after.get_rel(1);
            }
        }
    }

    #[derive(PartialEq)]
    enum UsageJudgment {
        NotMistake,
        IsMistakeForDecade,
        IsMistakeForAgeRange,
        Unsure,
    }

    let mut judgement = UsageJudgment::Unsure;

    // late 90's -> late '90s
    if pre.is_some_and(|p| p.kind.is_whitespace() || p.kind.is_hyphen())
        && prepre.is_some_and(|pp| {
            pp.span
                .get_content(src)
                .eq_any_ignore_ascii_case_str(&["early", "mid", "late"])
        })
    {
        judgement = UsageJudgment::IsMistakeForDecade;
    }
    // 80's style -> '80s style
    else if next.is_some_and(|n| n.kind.is_whitespace())
        && nextnext.is_some_and(|nn| nn.span.get_content(src).eq_ignore_ascii_case_str("style"))
    {
        judgement = UsageJudgment::IsMistakeForDecade;
    }
    // C++20's -> no mistake
    else if toks[0]
        .span
        .get_content(src)
        .eq_ignore_ascii_case_chars(&['2', '0'])
        && pre.is_some_and(|p| p.kind.is_plus())
        && prepre.is_some_and(|pp| pp.kind.is_plus())
        && preprepre.is_some_and(|ppp| ppp.span.get_content(src).eq_ignore_ascii_case_chars(&['c']))
    {
        judgement = UsageJudgment::NotMistake;
    }

    let with_apostrophe_before = [&['\''], decade, suffix].concat();
    let without_apostrophe = &with_apostrophe_before[1..];

    let mut suggestions = vec![];

    if judgement == UsageJudgment::NotMistake {
        return None;
    }
    if judgement == UsageJudgment::IsMistakeForDecade {
        suggestions.push(Suggestion::ReplaceWith(with_apostrophe_before.to_vec()));
    }
    if judgement == UsageJudgment::IsMistakeForAgeRange {
        suggestions.push(Suggestion::ReplaceWith(without_apostrophe.to_vec()));
    }

    Some(Lint {
        span: toks.span()?,
        lint_kind: LintKind::Usage,
        suggestions,
        message: "To refer to a decade the apostrophe must be before the decade. To refer to an age range, use no apostrophe.".to_string(),
        ..Default::default()
    })
}

#[cfg(test)]
mod lints {
    use super::super::PluralDecades;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    // Made-up examples

    #[test]
    #[ignore = "wip"]
    fn eighties() {
        assert_lint_count("in the 80's", PluralDecades::default(), 1);
    }

    #[test]
    #[ignore = "wip"]
    fn nineties() {
        assert_lint_count("the 90’s were a bit grungy", PluralDecades::default(), 1);
    }

    #[test]
    fn dont_flag_three_digits() {
        assert_no_lints("200's doesn't look like a decade", PluralDecades::default());
    }

    #[test]
    fn dont_flag_one_digit() {
        assert_no_lints("0's doesn't look like a decade", PluralDecades::default());
    }

    #[test]
    fn dont_flag_not_ending_with_0() {
        assert_no_lints("'77's best month was October", PluralDecades::default());
    }

    // Real-world examples using sentences found on GitHub

    // 10s

    #[test]
    #[ignore = "wip"]
    fn dont_flag_dot_version_numbers() {
        assert_no_lints(
            "A bug is apparently in FOG 1.5.10's normalize() function inside init.xz.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_showing_the_10s_of_hours() {
        assert_suggestion_result(
            "It took 10's of hours to debug this issue",
            PluralDecades::default(),
            "It took 10s of hours to debug this issue",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_windows_10() {
        assert_no_lints(
            "How about Windows 10's taskbar progress bar?",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_space_version_numbers_resharper_10() {
        assert_no_lints(
            "\"gd\" doesn't work correctly with ReSharper 10's \"Usage-aware Go to Declaration\"",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_space_version_numbers_mermaid_10() {
        assert_no_lints(
            "mermaid 10's ESM only support breaks compat with many apps",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_npm_10s_npsg() {
        assert_no_lints(
            "Align npm packages to npm 10's node engine range",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "well-known products could be checked for"]
    fn dont_flag_xcode_10s_version_number() {
        assert_no_lints(
            "Leverage Xcode 10's new \"File list\" feature for input/output files of Run Script build phases",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "non-well-known products couldn't really be checked for though"]
    fn dont_flag_modo_10s_version_number() {
        assert_no_lints(
            "Modo 10's Unreal editor plugin (for loading PBR materials / textures)",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_windows_10s_touch_keyboard() {
        assert_no_lints(
            "Arrow Key Command History Navigation Not Working Using Windows 10's Built-in 'Touch Keyboard'",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_android_10s_scoped_storage() {
        assert_no_lints(
            "Android 10's Scoped storage using Image picker (Gallery / Camera) with compression example.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_windows_10s_openssh() {
        assert_no_lints(
            "If I try to set Windows 10's OpenSSH ssh-agent.exe as the pageant executable, I get an error message",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "release/version number?"]
    fn dont_flag_node10s_resolution_algorithm() {
        assert_no_lints(
            "node10 encoded Node.js 10's resolution algorithm, which predates ESM support",
            PluralDecades::default(),
        );
    }

    // 20s

    #[test]
    fn dont_flag_cpp20s_std_span() {
        assert_no_lints(
            "This repository contains a single-header implementation of C++20's std::span, conforming to the C++20 committee draft.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_space_version_numbers_virtualenv_20() {
        assert_no_lints(
            "Clarifying virtualenv 20's -p behavior",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_hyphenated_version_numbers_soi_20() {
        assert_no_lints("View soi-20's full-sized avatar.", PluralDecades::default());
    }

    #[test]
    fn dont_flag_cpp20s_concepts() {
        assert_no_lints(
            "Replace SFINAE with C++20's Concepts and Constraints",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_cpp20s_std_latch() {
        assert_no_lints(
            "As part of an experiment I recently switched from ducc's latch class to C++20's std::latch, and to my surprise I noticed a significant speedup",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_team_20s_application() {
        assert_no_lints(
            "Team 20's application for the 2020 Teens In AI Global COVID Hackathon.",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "should we detect lesson numbers?"]
    fn dont_flag_lesson_20s_sql_query() {
        assert_no_lints(
            "Lesson 20's SQL Query is too inefficient",
            PluralDecades::default(),
        );
    }

    #[test]
    fn dont_flag_cpp20s_initialization_change() {
        assert_no_lints(
            "Potential issue with C++20's initialization change.",
            PluralDecades::default(),
        );
    }

    // 30s

    #[test]
    #[ignore = "wip"]
    fn dont_flag_sdk_versions() {
        assert_no_lints(
            "binder: We call SDK 30's bindServiceAsUser() and SDK 26's bindDeviceAdminServiceAsUser() methods without a runtime check",
            PluralDecades::default(),
        );
    }

    #[test]
    #[ignore = "wip"]
    fn dont_flag_jxn_hyphen_30s_username() {
        assert_no_lints(
            "GitHub Gist: star and fork jxn-30's gists by creating an account on GitHub",
            PluralDecades::default(),
        );
    }

    // 40s

    #[test]
    #[ignore = "might be too ambiguous to detect?"]
    fn dont_flag_group_40s() {
        assert_no_lints("Group 40's team maths game.", PluralDecades::default());
    }

    // 50s

    #[test]
    #[ignore = "here it's a username but Harper has no way to know"]
    fn dont_flag_50s_username() {
        assert_no_lints("View 50's full-sized avatar.", PluralDecades::default());
    }

    // 60s

    #[test]
    #[ignore = "here it means 60+ seconds"]
    fn dont_flag_60_seconds() {
        assert_no_lints("WSL cold startup 60's +", PluralDecades::default());
    }

    // 70s

    #[test]
    #[ignore = "ambiguous: version number?"]
    fn dont_flag_dotnet_runtime_70s() {
        assert_no_lints(
            "dotnet-runtime-70's release of 16th of May is causing \"version `GLIBC_2.34' not found\"",
            PluralDecades::default(),
        );
    }

    #[test]
    fn fix_late_hyphen_70s() {
        assert_suggestion_result(
            "Retrocomputer built from late-70's TTL logic chips",
            PluralDecades::default(),
            "Retrocomputer built from late-'70s TTL logic chips",
        );
    }

    // 80s

    #[test]
    #[ignore = "wip"]
    fn fix_of_the_80s_npsg() {
        assert_suggestion_result(
            "A reboot of the 80's Microwriter accessible chord keyboard done using an Arduino.",
            PluralDecades::default(),
            "A reboot of the '80s Microwriter accessible chord keyboard done using an Arduino.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_an_80s_npsg() {
        assert_suggestion_result(
            "A remake of an 80's card game classic \"Around the World\"",
            PluralDecades::default(),
            "A remake of an '80s card game classic \"Around the World\"",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_the_80s_npsg() {
        assert_suggestion_result(
            "Small remake of the 80's legendary paperboy arcade game",
            PluralDecades::default(),
            "Small remake of the '80s legendary paperboy arcade game",
        );
    }

    #[test]
    fn fix_the_80s_style_game_breakout() {
        assert_suggestion_result(
            "I called this pong but then was reminded that it more closely resembles the 80's style game Breakout.",
            PluralDecades::default(),
            "I called this pong but then was reminded that it more closely resembles the '80s style game Breakout.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_the_80s_microwriter() {
        assert_suggestion_result(
            "A reboot of the 80's Microwriter accessible chord keyboard done using an Arduino.",
            PluralDecades::default(),
            "A reboot of the '80s Microwriter accessible chord keyboard done using an Arduino.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_80s_neon_theme() {
        assert_suggestion_result(
            "A flat, 80's neon inspired theme for JupyterLab.",
            PluralDecades::default(),
            "A flat, '80s neon inspired theme for JupyterLab.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_80s_neon_theme_colors() {
        assert_suggestion_result(
            "Cool UI Theme for Atom based on 80's neon colors with big tabs for easy files Switch.",
            PluralDecades::default(),
            "Cool UI Theme for Atom based on '80s neon colors with big tabs for easy files Switch.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_80s_synthwave_theme() {
        assert_suggestion_result(
            "An clean 80's synthwave / outrun inspired theme for Vim.",
            PluralDecades::default(),
            "An clean '80s synthwave / outrun inspired theme for Vim.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_80s_3d_era() {
        assert_suggestion_result(
            "Experimenting with writing 80's era 3D code but in Javascript and with HTML5 Canvas acting as display buffer.",
            PluralDecades::default(),
            "Experimenting with writing '80s era 3D code but in Javascript and with HTML5 Canvas acting as display buffer.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_80s_theme_80s_aesthetics() {
        assert_suggestion_result(
            "Vibrant 80's Klipper Mainsail Theme, based around 80's Dark Neon Aesthetics.",
            PluralDecades::default(),
            "Vibrant '80s Klipper Mainsail Theme, based around '80s Dark Neon Aesthetics.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_80s_dark_retro_theme() {
        assert_suggestion_result(
            "80's dark retro theme for VS Code and Sublime Text",
            PluralDecades::default(),
            "'80s dark retro theme for VS Code and Sublime Text",
        );
    }

    #[test]
    fn fix_80s_chorus_effect() {
        assert_suggestion_result(
            "An 80's style chorus effect for your KORG 'logue synthesizers - hammondeggs/hera.",
            PluralDecades::default(),
            "An '80s style chorus effect for your KORG 'logue synthesizers - hammondeggs/hera.",
        );
    }

    #[test]
    #[ignore = "Does this mean a Chrome browser version? If so what's the possessive for?"]
    fn dont_flag_chrome_80s() {
        assert_no_lints(
            "Ready for Chrome 80's [Cookies default to SameSite=Lax] ?",
            PluralDecades::default(),
        );
    }

    #[test]
    fn fix_80s_style() {
        assert_suggestion_result(
            "Made your RStudio 80's style only after the sun goes down.",
            PluralDecades::default(),
            "Made your RStudio '80s style only after the sun goes down.",
        );
    }

    // 90s

    #[test]
    #[ignore = "wip"]
    fn fix_the_90s_were() {
        assert_suggestion_result(
            "Generate animated vector graphics for old-school 90's demos, like ST_NICCC",
            PluralDecades::default(),
            "Generate animated vector graphics for old-school '90s demos, like ST_NICCC",
        );
    }

    #[test]
    fn fix_late_90s() {
        assert_suggestion_result(
            "gmdrec is a USB interface between your late 90's Sony portable MiniDisc recorder and your PC.",
            PluralDecades::default(),
            "gmdrec is a USB interface between your late '90s Sony portable MiniDisc recorder and your PC.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_the_90s_npsg() {
        assert_suggestion_result(
            "A modern vision on the 90's game Log!cal.",
            PluralDecades::default(),
            "A modern vision on the '90s game Log!cal.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_from_the_90s() {
        assert_suggestion_result(
            "Digital Sound and Music Interface (from the 90's).",
            PluralDecades::default(),
            "Digital Sound and Music Interface (from the '90s).",
        );
    }

    #[test]
    fn fix_the_late_90s() {
        assert_suggestion_result(
            "A modified CircleMUD that ran in the late 90's.",
            PluralDecades::default(),
            "A modified CircleMUD that ran in the late '90s.",
        );
    }

    #[test]
    #[ignore = "wip"]
    fn fix_rad_90s_website() {
        assert_suggestion_result(
            "This is our rad 90's website.",
            PluralDecades::default(),
            "This is our rad '90s website.",
        );
    }

    #[test]
    #[ignore = "mixed 80 with no 's next to 90's with 's might be too oddball"]
    fn fix_mixed_80s_and_90s() {
        // We could 'half-fix' just the 90's -> '90s part ...
        assert_suggestion_result(
            "\"ワープロ明朝\" is a font that reproduced the smoothing algorithm used in the 80-90's Japanese word processors.",
            PluralDecades::default(),
            "\"ワープロ明朝\" is a font that reproduced the smoothing algorithm used in the 80s-'90s Japanese word processors.",
        );
    }

    #[test]
    #[ignore = "90 degrees"]
    fn dont_flag_all_90_degrees() {
        assert_no_lints(
            "get_map(\"Slope Degrees\") returns all 90's unless projected crs is specified",
            PluralDecades::default(),
        );
    }

    // Multiple decades

    #[test]
    #[ignore = "wip"]
    fn fix_multiple_ages() {
        assert_suggestion_result(
            "It generates 100,000 random \"people\" and randomly assigns them as being in their 20's, 30's, 40's, 50's, 60's, or 70's.",
            PluralDecades::default(),
            "It generates 100,000 random \"people\" and randomly assigns them as being in their 20s, 30s, 40s, 50s, 60s, or 70s.",
        );
    }

    #[test]
    #[ignore = "not sure if we should support missing 'the', especially when there's two decades"]
    fn fix_missing_the() {
        assert_suggestion_result(
            "A thoughtful full-stack reimplementation of gaming in 80's and 90's.",
            PluralDecades::default(),
            "A thoughtful full-stack reimplementation of gaming in the '80s and '90s.",
        );
    }
}

use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    linting::{LintKind, Suggestion},
};

pub fn match_to_lint_two_digits(
    toks: &[Token],
    src: &[char],
    decade: &[char],
    suffix: &[char],
    pre: Option<&[Token]>,
    post: Option<&[Token]>,
) -> Option<Lint> {
    None
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

    // mermaid 10's ESM only support breaks compat with many apps
    #[test]
    fn dont_flag_space_version_numbers_mermaid_10() {
        assert_no_lints(
            "mermaid 10's ESM only support breaks compat with many apps",
            PluralDecades::default(),
        );
    }

    // 20s

    #[test]
    #[ignore = "wip"]
    fn dont_flag_cpp_version() {
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

    // 30s

    #[test]
    #[ignore = "wip"]
    fn dont_flag_sdk_versions() {
        assert_no_lints(
            "binder: We call SDK 30's bindServiceAsUser() and SDK 26's bindDeviceAdminServiceAsUser() methods without a runtime check",
            PluralDecades::default(),
        );
    }

    // 40s

    #[test]
    #[ignore = "might be too ambiguous to detect?"]
    fn dont_flag_group_40s() {
        assert_no_lints("Group 40's team maths game.", PluralDecades::default());
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
    #[ignore = "wip"]
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
    #[ignore = "wip"]
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
    #[ignore = "wip"]
    fn fix_the_late_90s() {
        assert_suggestion_result(
            "A modified CircleMUD that ran in the late 90's.",
            PluralDecades::default(),
            "A modified CircleMUD that ran in the late '90s.",
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

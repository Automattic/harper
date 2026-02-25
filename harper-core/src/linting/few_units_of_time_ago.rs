use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::expr::TimeUnitExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token,
    linting::{ExprLinter, Lint, Suggestion},
};

pub struct FewUnitsOfTimeAgo {
    expr: Box<dyn Expr>,
}

impl Default for FewUnitsOfTimeAgo {
    fn default() -> Self {
        let units = TimeUnitExpr;

        let start = SequenceExpr::default().then_word_except(&["a"]).t_ws();

        let expr = Lrc::new(
            SequenceExpr::with(start)
                .t_aco("few")
                .then_whitespace()
                .then(units)
                .then_whitespace()
                .t_aco("ago"),
        );

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for FewUnitsOfTimeAgo {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let mut span = None;

        for tok in toks.iter().take(3) {
            if tok.span.get_content_string(src).eq_ignore_ascii_case("few") {
                span = Some(tok.span);
                break;
            }
        }

        span?;

        Some(Lint {
            span: span.unwrap(),
            message: "In this construction you need to use `a few` instead of just `few`."
                .to_string(),
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "a few",
                span.unwrap().get_content(src),
            )],
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Corrects some expressions using `few` where `a few` is correct."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::{
        assert_lint_count, assert_suggestion_result, assert_top3_suggestion_result,
    };

    // Basic unit tests

    #[test]
    #[ignore = "Needs ^ zero-width anchor that matches the start of a chunk"]
    fn fix_few_minutes_ago() {
        assert_suggestion_result(
            "Few minutes ago",
            FewUnitsOfTimeAgo::default(),
            "A few minutes ago",
        );
    }

    #[test]
    fn dont_flag_a_few_minutes_ago() {
        assert_lint_count("A few minutes ago", FewUnitsOfTimeAgo::default(), 0);
    }

    #[test]
    fn fix_done_few_minutes_ago() {
        assert_top3_suggestion_result(
            "Done few minutes ago",
            FewUnitsOfTimeAgo::default(),
            "Done a few minutes ago",
        );
    }

    #[test]
    fn dont_flag_done_a_few_minutes_ago() {
        assert_lint_count("Done a few minutes ago", FewUnitsOfTimeAgo::default(), 0);
    }

    #[test]
    #[ignore = "Needs ^ zero-width anchor that matches the start of a chunk"]
    fn fix_after_space() {
        assert_suggestion_result(
            " Few minutes ago.",
            FewUnitsOfTimeAgo::default(),
            " A few minutes ago.",
        );
    }

    #[test]
    #[ignore = "Needs ^ zero-width anchor that matches the start of a chunk"]
    fn fix_2nd_sentence() {
        assert_suggestion_result(
            "Hello World. Few minutes ago I bought your planet.",
            FewUnitsOfTimeAgo::default(),
            "Hello World. A few minutes ago I bought your planet.",
        );
    }

    // Real world examples from GitHub

    #[test]
    fn fix_days() {
        assert_suggestion_result(
            "My jupyter kernel always says restarting and never ever runs i ran into the problem few days ago before it was fine dont know what happened",
            FewUnitsOfTimeAgo::default(),
            "My jupyter kernel always says restarting and never ever runs i ran into the problem a few days ago before it was fine dont know what happened",
        );
    }

    #[test]
    fn fix_decades() {
        assert_suggestion_result(
            "This is very old piece of software I wrote few decades ago.",
            FewUnitsOfTimeAgo::default(),
            "This is very old piece of software I wrote a few decades ago.",
        );
    }

    #[test]
    fn fix_hours() {
        assert_suggestion_result(
            "I just updated my index file few hours ago and there's this error.",
            FewUnitsOfTimeAgo::default(),
            "I just updated my index file a few hours ago and there's this error.",
        );
    }

    #[test]
    fn fix_minutes() {
        assert_suggestion_result(
            "mysql installed few minutes ago somehow , ubuntu bash thinks its not installed.",
            FewUnitsOfTimeAgo::default(),
            "mysql installed a few minutes ago somehow , ubuntu bash thinks its not installed.",
        );
    }

    #[test]
    fn fix_months() {
        assert_suggestion_result(
            "Hello, I was working with D455 few months ago, and everything was working fine.",
            FewUnitsOfTimeAgo::default(),
            "Hello, I was working with D455 a few months ago, and everything was working fine.",
        );
    }

    #[test]
    fn fix_ms() {
        assert_suggestion_result(
            "So I not sure, by getting old signal (get from few ms ago), will it affected my result badly?",
            FewUnitsOfTimeAgo::default(),
            "So I not sure, by getting old signal (get from a few ms ago), will it affected my result badly?",
        );
    }

    #[test]
    fn fix_seconds() {
        assert_suggestion_result(
            "I have submitted the same issue few seconds ago.",
            FewUnitsOfTimeAgo::default(),
            "I have submitted the same issue a few seconds ago.",
        );
    }

    #[test]
    fn fix_weekends() {
        assert_suggestion_result(
            "This challenge is a Python jail escape and lucky for me our team had just done one few weekends ago so I was fairly familiar with the tricks to break out.",
            FewUnitsOfTimeAgo::default(),
            "This challenge is a Python jail escape and lucky for me our team had just done one a few weekends ago so I was fairly familiar with the tricks to break out.",
        );
    }

    #[test]
    fn fix_weeks() {
        assert_suggestion_result(
            "Terraform cloud crashes on plan (same configuration worked few weeks ago)",
            FewUnitsOfTimeAgo::default(),
            "Terraform cloud crashes on plan (same configuration worked a few weeks ago)",
        );
    }

    #[test]
    fn fix_years() {
        assert_suggestion_result(
            "sandbox-exec was deprecated on MacOS few years ago",
            FewUnitsOfTimeAgo::default(),
            "sandbox-exec was deprecated on MacOS a few years ago",
        );
    }

    // Real world non-errors from GitHub

    #[test]
    fn dont_flag_centuries() {
        assert_lint_count(
            "Would have been useful a few centuries ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_days() {
        assert_lint_count(
            "A few days ago, I upgraded ComfyUI to the latest version, then the prompt node can't upload prompt list text file in Ubuntu",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_decades() {
        assert_lint_count(
            "With your QA background you may have heard of the IBM black team of testers back a few decades ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_hours() {
        assert_lint_count(
            "It was working well and we could see the installation page a few hours ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_milliseconds() {
        assert_lint_count(
            "It is actually the true motor angle observed a few milliseconds ago (pd latency).",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_minutes() {
        assert_lint_count(
            "Example from DoD The following was circulated a few minutes ago on an IDESG/NSTIC list",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_moments() {
        assert_lint_count(
            "Our microservices started failing a few moments ago when creating new...",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_months() {
        assert_lint_count(
            "A few months ago there was an mixed reality project.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_nights() {
        assert_lint_count(
            "As an example, a few nights ago I was working on my laptop and stuff that had been working stopped working.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_seconds() {
        assert_lint_count(
            "0 - 45 seconds, a few seconds ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_weeks() {
        assert_lint_count(
            "It was all working perfectly till a few weeks ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_years() {
        assert_lint_count(
            "Hello, I've been an intensive user of your dada2 pipeline until a few years ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    // Real world non-errors from GitHub (but using singular forms)

    #[test]
    fn dont_flag_decade() {
        assert_lint_count(
            "With your QA background you may have heard of the IBM black team of testers back a few decade ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_hour() {
        assert_lint_count(
            "It was working well and we could see the installation page a few hour ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_millennia() {
        assert_lint_count(
            "A few millennia ago, there was a civilization here",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_minute() {
        assert_lint_count(
            "Example from DoD The following was circulated a few minute ago on an IDESG/NSTIC list",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_moment() {
        assert_lint_count(
            "No problem should be in the updated version pushed a few moment ago will be live in beta in about 10 min.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_month() {
        assert_lint_count(
            "I noticed the same thing a few month ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_second() {
        assert_lint_count(
            "Bug it doesnt even answer me rn, like a few second ago he did",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_week() {
        assert_lint_count(
            "A few week ago, when logging in the usual way",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_year() {
        assert_lint_count(
            "Hello, I've been an intensive user of your dada2 pipeline until a few year ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    // Real world non-errors from GitHub using apostrophes

    #[test]
    fn dont_flag_days_apos() {
        assert_lint_count(
            "And finally it got released a few day's ago.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_months_apos() {
        assert_lint_count(
            "I had thought that since I had done this process a few month's ago the database could just be updated.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_weeks_apos() {
        assert_lint_count(
            "A few week's ago, I was alerted in Webmin that Webmin was eligible to upgrade to 1.880 which I did through Webmin.",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_years_apos() {
        assert_lint_count(
            "A few year's ago a spammer registered an unused base url",
            FewUnitsOfTimeAgo::default(),
            0,
        );
    }

    // Real world mistakes from GitHub using singular forms

    #[test]
    fn fix_day() {
        assert_suggestion_result(
            "That worked few day ago with the same setting.",
            FewUnitsOfTimeAgo::default(),
            "That worked a few day ago with the same setting.",
        );
    }

    #[test]
    #[ignore = "Needs ^ zero-width anchor that matches the start of a chunk"]
    fn fix_decade() {
        assert_suggestion_result(
            "few decade ago, African Americans weren't allowed to swim in public",
            FewUnitsOfTimeAgo::default(),
            "a few decade ago, African Americans weren't allowed to swim in public",
        );
    }

    #[test]
    fn fix_minute() {
        assert_suggestion_result(
            "All works fine, but few minute ago the device stop responding from web",
            FewUnitsOfTimeAgo::default(),
            "All works fine, but a few minute ago the device stop responding from web",
        );
    }

    #[test]
    fn fix_weekend() {
        assert_suggestion_result(
            "I have done this few weekend ago.",
            FewUnitsOfTimeAgo::default(),
            "I have done this a few weekend ago.",
        );
    }
}

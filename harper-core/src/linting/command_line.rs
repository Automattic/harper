use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, following_word},
    },
    patterns::WordSet,
};

pub struct CommandLine {
    expr: FirstMatchOf,
}

impl Default for CommandLine {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(WordSet::new(&["commandsline", "commandlines"])) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::word_set(&["command", "commands"])
                        .t_ws_h()
                        .t_set(&["line", "lines"])
                        .but_not(SequenceExpr::aco("command").t_ws_h().t_aco("line")),
                ),
            ]),
        }
    }
}

impl ExprLinter for CommandLine {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let span = toks.span()?;

        let is_closed_compound = match toks.len() {
            1 => true,
            3 => false,
            _ => return None,
        };

        let (command, sep, line): (&[char], &[char], &[char]) = match is_closed_compound {
            true => {
                let chars = toks.first()?.get_ch(src);
                let i = chars.len()
                    - if chars.ends_with_ignore_ascii_case_chars(&['s']) {
                        5
                    } else {
                        4
                    };
                (&chars[0..i], &chars[i..i], &chars[i..])
            }
            false => (
                toks.first()?.get_ch(src),
                toks.get(1)?.get_ch(src),
                toks.get(2)?.get_ch(src),
            ),
        };

        let maybe_next_word = following_word(ctx).map(|t| t.get_ch(src));

        eprintln!(
            "🍑{}🍎{}🍐{}{}",
            command.to_string(),
            sep.to_string(),
            line.to_string(),
            maybe_next_word.map_or(String::new(), |w| format!(" 〖{}〗", w.to_string()))
        );

        let (pl_cmd, pl_line) = (
            command.ends_with_ignore_ascii_case_chars(&['s']),
            line.ends_with_ignore_ascii_case_chars(&['s']),
        );

        // abort for 'command lines at once'
        if !pl_cmd && pl_line && maybe_next_word.is_some_and(|c| c.eq_ch(&['a', 't'])) {
            return None;
        }
        // about for 'commands line by line'
        if pl_cmd && !pl_line && maybe_next_word.is_some_and(|c| c.eq_ch(&['b', 'y'])) {
            return None;
        }

        let (command, line) = if pl_cmd && pl_line && maybe_next_word.is_none() {
            // there's commands lines -> there's command lines
            (&command[0..7], line)
        } else {
            (&command[0..7], &line[0..4])
        };

        let message = "Fix this erorr".to_owned();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith([command, sep, line].concat())],
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of `command line`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CommandLine;

    #[test]
    fn fix_command_lines_interface() {
        assert_suggestion_result(
            "We use terminal commands are instructions typed into a command-lines interface(cli).",
            CommandLine::default(),
            "We use terminal commands are instructions typed into a command-line interface(cli).",
        );
    }

    #[test]
    fn dont_flag_command_lines_at_once() {
        assert_no_lints(
            "I want to execute multiple command lines at once",
            CommandLine::default(),
        );
    }

    #[test]
    fn dont_flag_commands_line_by_line() {
        assert_no_lints(
            "The shell executes commands line by line.",
            CommandLine::default(),
        )
    }

    #[test]
    fn fix_command_lines_utility() {
        assert_suggestion_result(
            "I will present you, briefly, some Django's command lines utility for administrative tasks",
            CommandLine::default(),
            "I will present you, briefly, some Django's command line utility for administrative tasks",
        );
    }

    #[test]
    fn fix_commands_line_hacks() {
        assert_suggestion_result(
            "Here is a link to a google sheet with these commands line hacks that you are welcome to add to your GDrive.",
            CommandLine::default(),
            "Here is a link to a google sheet with these command line hacks that you are welcome to add to your GDrive.",
        )
    }

    #[test]
    fn fix_with_the_commands_line() {
        assert_suggestion_result(
            "I am new to Linux but am getting better with the commands line and the Linux fundamentals",
            CommandLine::default(),
            "I am new to Linux but am getting better with the command line and the Linux fundamentals",
        );
    }

    #[test]
    fn fix_commands_line_organization() {
        assert_suggestion_result(
            "History of Linux, Fedora Project and Commands Line Organization",
            CommandLine::default(),
            "History of Linux, Fedora Project and Command Line Organization",
        );
    }

    #[test]
    fn fix_commandlines_output() {
        assert_suggestion_result(
            "Assets can be loaded from the filesystem, http servers, the clipboard, commandlines output or raw json strings.",
            CommandLine::default(),
            "Assets can be loaded from the filesystem, http servers, the clipboard, commandline output or raw json strings.",
        )
    }

    #[test]
    fn fix_commandlines_input() {
        assert_suggestion_result(
            "I am familiar with Linux and commandlines input but not an expert in it.",
            CommandLine::default(),
            "I am familiar with Linux and commandline input but not an expert in it.",
        );
    }

    #[test]
    fn fix_commands_lines_arguments() {
        assert_suggestion_result(
            "other executable console application that takes commands lines arguments as input",
            CommandLine::default(),
            "other executable console application that takes command line arguments as input",
        );
    }

    #[test]
    fn fix_certain_commands_lines_to_command_lines() {
        assert_suggestion_result(
            "there's also certain commands lines, tool offsets,spindle speeds, feed rates",
            CommandLine::default(),
            "there's also certain command lines, tool offsets,spindle speeds, feed rates",
        );
    }
}

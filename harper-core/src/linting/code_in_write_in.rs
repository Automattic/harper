use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct CodeInWriteIn {
    expr: SequenceExpr,
}

impl Default for CodeInWriteIn {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                // "code" produces too many false positives
                "coded",
                "codes",
                "coding",
                "write",
                "writes",
                "written",
                "wrote",
                "program",
                "programmed",
                "programming",
                // "programs", produces too many false positives
            ])
            .t_ws()
            .t_aco("on")
            .t_ws()
            .then_word_set(&[
                "ada",
                "asm",
                "assembler",
                "assembly", // could cause false positives "on assembly level"
                "bash",
                "basic",
                "c",
                "clisp",
                "cobol",
                "cpp",
                "csharp",
                "css",
                "dart",
                "ecmascript",
                "fortran",
                "haskell",
                "html",
                "go",
                "golang",
                "java",
                "javascript",
                "js",
                "julia",
                "kotlin",
                "lisp",
                "oberon",
                "objc",
                "odin",
                "pascal",
                "perl",
                "php",
                "python",
                "ruby",
                "rust",
                "sh",
                "shell",
                "swift",
                "typescript",
                "zig",
                "zsh",
            ]),
        }
    }
}

impl ExprLinter for CodeInWriteIn {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens.len() != 5 {
            return None;
        }
        let span = matched_tokens[2].span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "in",
                span.get_content(source),
            )],
            message: "For writing code, the preposition should be `in` rather than `on`."
                .to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the wrong preposition `on` to `in` when referring to writing code."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CodeInWriteIn;

    #[test]
    fn fix_coded_on_ada() {
        assert_suggestion_result(
            "Prompt generator based on Bernoulli. Coded on Ada language.",
            CodeInWriteIn::default(),
            "Prompt generator based on Bernoulli. Coded in Ada language.",
        );
    }

    #[test]
    fn fix_coded_on_cpp() {
        assert_suggestion_result(
            "A network function simulation project coded on cpp.",
            CodeInWriteIn::default(),
            "A network function simulation project coded in cpp.",
        );
    }

    #[test]
    fn fix_coded_on_php() {
        assert_suggestion_result(
            "A book keeping application coded on PHP to maintain an inventory of books and a history of users reading them.",
            CodeInWriteIn::default(),
            "A book keeping application coded in PHP to maintain an inventory of books and a history of users reading them.",
        );
    }

    #[test]
    fn fix_coding_on_objc_coding_on_swift() {
        assert_suggestion_result(
            "Import XibView.h and XibView.m if you're coding on objc or XibView.swift if you're coding on Swift.",
            CodeInWriteIn::default(),
            "Import XibView.h and XibView.m if you're coding in objc or XibView.swift if you're coding in Swift.",
        );
    }

    #[test]
    fn fix_coding_on_python() {
        assert_suggestion_result(
            "Was coding on Python via VS Code and started getting this error message",
            CodeInWriteIn::default(),
            "Was coding in Python via VS Code and started getting this error message",
        );
    }

    #[test]
    fn fix_program_on_assembler() {
        assert_suggestion_result(
            "Learn to program on assembler dividing numbers",
            CodeInWriteIn::default(),
            "Learn to program in assembler dividing numbers",
        );
    }

    #[test]
    #[ignore = "Known false positive - 'on assembly level' is a valid phrase"]
    fn dont_flag_program_on_assembly_level() {
        assert_no_lints(
            "This code is a tool which can be used to program on assembly level using natural language.",
            CodeInWriteIn::default(),
        );
    }

    #[test]
    fn fix_program_on_basic() {
        assert_suggestion_result(
            "Write this program on BASIC first:",
            CodeInWriteIn::default(),
            "Write this program in BASIC first:",
        );
    }

    #[test]
    fn fix_program_on_python() {
        assert_suggestion_result(
            "Performance Issue writing a long program on python",
            CodeInWriteIn::default(),
            "Performance Issue writing a long program in python",
        );
    }

    #[test]
    fn fix_programming_on_c() {
        assert_suggestion_result(
            "My complain about lark is that I cannot use it when programming on C or C++, Java, Javascript, etc.",
            CodeInWriteIn::default(),
            "My complain about lark is that I cannot use it when programming in C or C++, Java, Javascript, etc.",
        );
    }

    #[test]
    #[ignore = "'Platforms' is separated from 'ECMAScript' due to being part of a list."]
    fn dont_flag_programming_on_ecmascript() {
        assert_no_lints(
            "Secure Distributed Programming on ECMAScript 5 + HTML5 Platforms",
            CodeInWriteIn::default(),
        );
    }

    #[test]
    fn fix_programming_on_haskell() {
        assert_suggestion_result(
            "This is material for an introduction to functional programming on Haskell, with a focus on monads.",
            CodeInWriteIn::default(),
            "This is material for an introduction to functional programming in Haskell, with a focus on monads.",
        );
    }

    #[test]
    fn fix_programming_on_javascript() {
        assert_suggestion_result(
            "Massively parallel GPU programming on JavaScript, simple and clean.",
            CodeInWriteIn::default(),
            "Massively parallel GPU programming in JavaScript, simple and clean.",
        );
    }

    #[test]
    fn dont_flag_programs_on_java() {
        assert_no_lints("Notes and Programs on Java.", CodeInWriteIn::default());
    }

    #[test]
    fn fix_programming_on_ruby() {
        assert_suggestion_result(
            "Deep Learning Programming on Ruby by Kenta Murata & Yusaku Hatanaka",
            CodeInWriteIn::default(),
            "Deep Learning Programming in Ruby by Kenta Murata & Yusaku Hatanaka",
        );
    }

    #[test]
    fn fix_programming_on_rust() {
        assert_suggestion_result(
            "Programming on Rust is like solving riddles, and I love riddles.",
            CodeInWriteIn::default(),
            "Programming in Rust is like solving riddles, and I love riddles.",
        );
    }

    #[test]
    fn fix_written_on_asm() {
        assert_suggestion_result(
            "Contracts are written on ASM-like language.",
            CodeInWriteIn::default(),
            "Contracts are written in ASM-like language.",
        );
    }

    #[test]
    fn fix_written_on_assembler() {
        assert_suggestion_result(
            "Electronic design automation system written on Assembler (FASM).",
            CodeInWriteIn::default(),
            "Electronic design automation system written in Assembler (FASM).",
        );
    }

    #[test]
    fn fix_written_on_bash() {
        assert_suggestion_result(
            "A collection of simple and different tools written on Bash that simplify work.",
            CodeInWriteIn::default(),
            "A collection of simple and different tools written in Bash that simplify work.",
        );
    }

    #[test]
    fn fix_written_on_csharp() {
        assert_suggestion_result(
            "It is a small Windows application written on csharp and wpf that requires Stockfish to to assist in playing on chess.com and lichess.org.",
            CodeInWriteIn::default(),
            "It is a small Windows application written in csharp and wpf that requires Stockfish to to assist in playing on chess.com and lichess.org.",
        );
    }

    #[test]
    fn fix_written_on_dart() {
        assert_suggestion_result(
            "This is a command-line app written on dart language for flutter applications that will help you to generate some boilerplate code",
            CodeInWriteIn::default(),
            "This is a command-line app written in dart language for flutter applications that will help you to generate some boilerplate code",
        );
    }

    #[test]
    fn fix_written_on_fortran() {
        assert_suggestion_result(
            "Simple Telegram Bot written on FORTRAN for generating LaTeX pictures in private messages and inline mode",
            CodeInWriteIn::default(),
            "Simple Telegram Bot written in FORTRAN for generating LaTeX pictures in private messages and inline mode",
        );
    }

    #[test]
    fn fix_written_on_go() {
        assert_suggestion_result(
            "Template for a typical module written on Go.",
            CodeInWriteIn::default(),
            "Template for a typical module written in Go.",
        );
    }

    #[test]
    fn fix_written_on_golang() {
        assert_suggestion_result(
            "Discovery library for Tarantool 3.0 written on Golang",
            CodeInWriteIn::default(),
            "Discovery library for Tarantool 3.0 written in Golang",
        );
    }

    #[test]
    fn fix_written_on_html() {
        assert_suggestion_result(
            "Developer's Life is a simple but exciting web-game written on HTML, CSS and JS",
            CodeInWriteIn::default(),
            "Developer's Life is a simple but exciting web-game written in HTML, CSS and JS",
        );
    }

    #[test]
    fn fix_written_on_kotlin() {
        assert_suggestion_result(
            "As on the image above decompiled java bytecode of apps written on kotlin doesn't look so good.",
            CodeInWriteIn::default(),
            "As on the image above decompiled java bytecode of apps written in kotlin doesn't look so good.",
        );
    }

    #[test]
    fn fix_written_on_lisp() {
        assert_suggestion_result(
            "Use flexible configuration files written on Lisp, provided by hygienic user bindings",
            CodeInWriteIn::default(),
            "Use flexible configuration files written in Lisp, provided by hygienic user bindings",
        );
    }

    #[test]
    fn fix_written_on_pascal() {
        assert_suggestion_result(
            "Project allows converting program file written on Pascal into Java bytecode.",
            CodeInWriteIn::default(),
            "Project allows converting program file written in Pascal into Java bytecode.",
        );
    }

    #[test]
    fn fix_written_on_perl() {
        assert_suggestion_result(
            "exiftool is written on Perl!",
            CodeInWriteIn::default(),
            "exiftool is written in Perl!",
        );
    }

    #[test]
    fn fix_written_on_zig() {
        assert_suggestion_result(
            "XML parser written on Zig - by snektron.",
            CodeInWriteIn::default(),
            "XML parser written in Zig - by snektron.",
        );
    }

    #[test]
    fn fix_wrote_on_julia() {
        assert_suggestion_result(
            "I'm running into the same problem, and same illegal instruction, with some code that I wrote on Julia 1.0 (official binaries).",
            CodeInWriteIn::default(),
            "I'm running into the same problem, and same illegal instruction, with some code that I wrote in Julia 1.0 (official binaries).",
        );
    }
}

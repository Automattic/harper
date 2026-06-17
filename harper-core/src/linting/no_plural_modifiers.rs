use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct NoPluralModifiers {
    expr: SequenceExpr,
}

const MODIFIER_NOUNS: &[&str] = &[
    "agents",
    "apps",
    "books",
    "cars", // But "cars park" can be a legit noun+verb phrase
    "codes",
    "controls",
    "dishes",
    "examples",
    "files",
    "games",
    "keys",
    "modes",
    "modules",
    "scores",
    "shoes",
    "texts",
    "toys",
    "trains",
    "variables",
    "widgets",
];

const HEAD_NOUNS: &[&str] = &[
    "board",
    "collection",
    "detection",
    "editor",
    "groups",
    "list",
    "shop",
    "shops",
    "station",
    "store",
    "support",
    "system",
    "table",
    "wash",
    "washer",
];

impl Default for NoPluralModifiers {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(MODIFIER_NOUNS)
                .t_ws()
                .t_set(HEAD_NOUNS),
        }
    }
}

impl ExprLinter for NoPluralModifiers {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let modifier = &toks[0];
        let span = modifier.span;

        let plural_mod = modifier.get_ch(src);

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                plural_mod[..plural_mod.len() - 1].to_vec(),
                span.get_content(src),
            )],
            message: "In compound nouns generally the first word is singular.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects compounds with plural modifiers (`trains station`, `files system`, etc.) to use singular modifiers."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NoPluralModifiers;

    #[test]
    fn agents_list() {
        assert_suggestion_result(
            "Summary openclaw agents list (and --json) does not expose whether an agent has exec blocked",
            NoPluralModifiers::default(),
            "Summary openclaw agent list (and --json) does not expose whether an agent has exec blocked",
        );
    }

    #[test]
    fn apps_groups() {
        assert_suggestion_result(
            "Problem with creating and assigning apps groups.",
            NoPluralModifiers::default(),
            "Problem with creating and assigning app groups.",
        );
    }

    #[test]
    fn books_shop() {
        assert_suggestion_result(
            "The books shop owner asked you to create the online shop to buy book with delivery to user's home.",
            NoPluralModifiers::default(),
            "The book shop owner asked you to create the online shop to buy book with delivery to user's home.",
        );
    }

    #[test]
    fn books_store() {
        assert_suggestion_result(
            "A sample .NET application representing a fictional used books store.",
            NoPluralModifiers::default(),
            "A sample .NET application representing a fictional used book store.",
        );
    }

    #[test]
    fn codes_editor() {
        assert_suggestion_result(
            "I need to support codes editor (intellij idea) on Userland",
            NoPluralModifiers::default(),
            "I need to support code editor (intellij idea) on Userland",
        );
    }

    #[test]
    fn codes_editor_title_case() {
        assert_suggestion_result(
            "Examples of applications built with electron includes Visual Studio Codes Editor",
            NoPluralModifiers::default(),
            "Examples of applications built with electron includes Visual Studio Code Editor",
        );
    }

    #[test]
    fn codes_editor_with_json() {
        assert_suggestion_result(
            "Next, open the config file we created with a codes editor(I recommend Visual Studio Code) that supports json format files",
            NoPluralModifiers::default(),
            "Next, open the config file we created with a code editor(I recommend Visual Studio Code) that supports json format files",
        );
    }

    #[test]
    fn codes_table() {
        assert_suggestion_result(
            "laravel migration for country codes table",
            NoPluralModifiers::default(),
            "laravel migration for country code table",
        );
    }

    #[test]
    fn controls_editor() {
        assert_suggestion_result(
            "Controls editor for array of objects",
            NoPluralModifiers::default(),
            "Control editor for array of objects",
        );
    }

    #[test]
    fn examples_collection() {
        assert_suggestion_result(
            "You can see examples of GitHub Pages sites in the GitHub Pages examples collection.",
            NoPluralModifiers::default(),
            "You can see examples of GitHub Pages sites in the GitHub Pages example collection.",
        );
    }

    #[test]
    fn files_collection() {
        assert_suggestion_result(
            "The files collection is a bit different from other collections as it's built in but at the end, it sits in the same database structure as any collection.",
            NoPluralModifiers::default(),
            "The file collection is a bit different from other collections as it's built in but at the end, it sits in the same database structure as any collection.",
        );
    }

    #[test]
    fn files_editor() {
        assert_suggestion_result(
            "Obsidian Data Files editor plugin",
            NoPluralModifiers::default(),
            "Obsidian Data File editor plugin",
        );
    }

    #[test]
    fn files_system() {
        assert_suggestion_result(
            "For this I created an overlay files system that immich writes changes to.",
            NoPluralModifiers::default(),
            "For this I created an overlay file system that immich writes changes to.",
        );
    }

    #[test]
    fn files_system_title_case() {
        assert_suggestion_result(
            "Linux Files System breakdown.",
            NoPluralModifiers::default(),
            "Linux File System breakdown.",
        );
    }

    #[test]
    fn games_collection() {
        assert_suggestion_result(
            "Child Theme for GeneratePress and a starting point for my games collection",
            NoPluralModifiers::default(),
            "Child Theme for GeneratePress and a starting point for my game collection",
        );
    }

    #[test]
    fn games_and_examples_collection() {
        assert_suggestion_result(
            "raylib examples collection · raylib games collection",
            NoPluralModifiers::default(),
            "raylib example collection · raylib game collection",
        );
    }

    #[test]
    fn games_shops() {
        assert_suggestion_result(
            "As for games shops, we already have dice in the casino icon.",
            NoPluralModifiers::default(),
            "As for game shops, we already have dice in the casino icon.",
        );
    }

    #[test]
    fn games_store() {
        assert_suggestion_result(
            "This repository is a sample of Online Games Store upgraded to CodeIgniter4 using ci3-to-4-upgrade-helper.",
            NoPluralModifiers::default(),
            "This repository is a sample of Online Game Store upgraded to CodeIgniter4 using ci3-to-4-upgrade-helper.",
        );
    }

    #[test]
    fn keys_table() {
        assert_suggestion_result(
            "When I try to set this in lazy.nvim's keys table, I receive the \"E492: Not an editor command: ^UTmuxNavigateRight\" error",
            NoPluralModifiers::default(),
            "When I try to set this in lazy.nvim's key table, I receive the \"E492: Not an editor command: ^UTmuxNavigateRight\" error",
        );
    }

    #[test]
    fn modes_support() {
        assert_suggestion_result(
            "ECS screen modes support",
            NoPluralModifiers::default(),
            "ECS screen mode support",
        );
    }

    #[test]
    fn modules_collection() {
        assert_suggestion_result(
            "Ansible Modules Collection for using OpenStack.",
            NoPluralModifiers::default(),
            "Ansible Module Collection for using OpenStack.",
        );
    }

    #[test]
    fn modules_list() {
        assert_suggestion_result(
            "Add quill-paste-smart to modules list.",
            NoPluralModifiers::default(),
            "Add quill-paste-smart to module list.",
        );
    }

    #[test]
    fn scores_board() {
        assert_suggestion_result(
            "Edge cases where chance of playing in player scores table can be wrong",
            NoPluralModifiers::default(),
            "Edge cases where chance of playing in player score table can be wrong",
        );
    }

    #[test]
    fn shoes_shop() {
        assert_suggestion_result(
            "Welcome to our Shoes Shop web application!",
            NoPluralModifiers::default(),
            "Welcome to our Shoe Shop web application!",
        );
    }

    #[test]
    fn shoes_store() {
        assert_suggestion_result(
            "Shoes store web app project using HTML, CSS, and Javascript",
            NoPluralModifiers::default(),
            "Shoe store web app project using HTML, CSS, and Javascript",
        );
    }

    #[test]
    fn texts_editor() {
        assert_suggestion_result(
            "easy and accessible texts editor for blind",
            NoPluralModifiers::default(),
            "easy and accessible text editor for blind",
        );
    }

    #[test]
    fn widgets_editor() {
        assert_suggestion_result(
            "script should not be enqueued together with the new widgets editor",
            NoPluralModifiers::default(),
            "script should not be enqueued together with the new widget editor",
        );
    }

    #[test]
    fn widgets_editor_one_title_case() {
        assert_suggestion_result(
            "This is an overview of smaller Widgets editor enhancements to look into after WordPress 5.8.",
            NoPluralModifiers::default(),
            "This is an overview of smaller Widget editor enhancements to look into after WordPress 5.8.",
        );
    }

    #[test]
    fn widgets_editor_title_case() {
        assert_suggestion_result(
            "Block-based Widgets Editor",
            NoPluralModifiers::default(),
            "Block-based Widget Editor",
        );
    }
}

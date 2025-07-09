use super::{LintGroup, MapPhraseSetLinter};

#[cfg(test)]
mod tests;

/// Produce a [`LintGroup`] that looks for errors in sets of common phrases.
pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::default();

    macro_rules! add_exact_mappings {
        ($group:expr, {
            $($name:expr => ($input_correction_pairs:expr, $message:expr, $description:expr)),+ $(,)?
        }) => {
            $(
                $group.add_expr_linter(
                    $name,
                    Box::new(
                        MapPhraseSetLinter::new(
                            $input_correction_pairs,
                            $message,
                            $description
                        ),
                    ),
                );
            )+
        };
    }

    add_exact_mappings!(group, {
        // The name of the rule
        "DefiniteArticle" => (
            &[
                ("definitive article", "definite article"),
                ("definitive articles", "definite articles")
            ],
            "The correct term for `the` is `definite article`.",
            "The name of the word `the` is `definite article`."
        ),
        "Discuss" => (
            &[
                ("discuss about", "discuss"),
                ("discussed about", "discussed"),
                ("discusses about", "discusses"),
                ("discussing about", "discussing"),
            ],
            "`About` is redundant",
            "Removes unnecessary `about` after `discuss`."
        ),
        "ExpandArgument" => (
            &[
                ("arg", "argument"),
                ("args", "arguments"),
            ],
            "Use `argument` instead of `arg`",
            "Expands the abbreviation `arg` to the full word `argument` for clarity."
        ),
        "ExpandDependencies" => (
            &[
                ("deps", "dependencies"),
                ("dep", "dependency"),
            ],
            "Use `dependencies` instead of `deps`",
            "Expands the abbreviation `deps` to the full word `dependencies` for clarity."
        ),
        "ExplanationMark" => (
            &[
                ("explanation mark", "exclamation mark"),
                ("explanation marks", "exclamation marks"),
                ("explanation point", "exclamation point"),
            ],
            "The correct names for the `!` punctuation are `exclamation mark` and `exclamation point`.",
            "Corrects the eggcorn `explanation mark/point` to `exclamation mark/point`."
        ),
    });

    group.set_all_rules_to(Some(true));

    group
}

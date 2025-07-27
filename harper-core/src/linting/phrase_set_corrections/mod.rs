use crate::linting::LintKind;

use super::{LintGroup, MapPhraseSetLinter};

#[cfg(test)]
mod tests;

/// Produce a [`LintGroup`] that looks for errors in sets of common phrases.
pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::default();

    // Each correction pair has a single bad form and a single correct form.
    macro_rules! add_1_to_1_mappings {
        ($group:expr, {
            $($name:expr => ($input_correction_pairs:expr, $message:expr, $description:expr $(, $lint_kind:expr)?)),+ $(,)?
        }) => {
            $(
                $group.add_expr_linter(
                    $name,
                    Box::new(
                        MapPhraseSetLinter::one_to_one(
                            $input_correction_pairs,
                            $message,
                            $description,
                            None$(.or(Some($lint_kind)))?,
                        ),
                    ),
                );
            )+
        };
    }

    // Each correction pair has multiple bad forms and multiple correct forms.
    macro_rules! add_many_to_many_mappings {
        ($group:expr, {
            $($name:expr => ($input_correction_multi_pairs:expr, $message:expr, $description:expr $(, $lint_kind:expr)?)),+ $(,)?
        }) => {
            $(
                $group.add_expr_linter(
                    $name,
                    Box::new(
                        MapPhraseSetLinter::many_to_many(
                            $input_correction_multi_pairs,
                            $message,
                            $description,
                            None$(.or(Some($lint_kind)))?,
                        ),
                    ),
                );
            )+
        };
    }

    add_1_to_1_mappings!(group, {
        "Ado" => (
            &[
                ("further adieu", "further ado"),
                ("much adieu", "much ado"),
            ],
            "Don't confuse the French/German `adieu`, meaning `farewell`, with the English `ado`, meaning `fuss`.",
            "Corrects `adieu` to `ado`.",
            LintKind::Eggcorn
        ),
        "ChampAtTheBit" => (
            &[
                ("chomp at the bit", "champ at the bit"),
                ("chomped at the bit", "champed at the bit"),
                ("chomping at the bit", "champing at the bit"),
                ("chomps at the bit", "champs at the bit"),
            ],
            "The correct idiom is `champ at the bit`.",
            "Corrects `chomp at the bit` to the idiom `champ at the bit`, which has an equestrian origin referring to the way an anxious horse grinds its teeth against the metal part of the bridle.",
            LintKind::Eggcorn
        ),
        "ClientOrServerSide" => (
            &[
                ("client's side", "client-side"),
                ("server's side", "server-side"),
            ],
            "`Client-side` and `server-side` do not use an apostrophe.",
            "Corrects extraneous apostrophe in `client's side` and `server's side`.",
            LintKind::Punctuation
        ),
        "ConfirmThat" => (
            &[
                ("conform that", "confirm that"),
                ("conformed that", "confirmed that"),
                ("conforms that", "confirms that"),
                // Note: false positives in this inflection:
                // "is there any example of a case that isn't fully conforming that is supported today?"
                ("conforming that", "confirming that"),
            ],
            "Did you mean `confirm` rather than `conform`?",
            "Corrects `conform` typos to `confirm`.",
            LintKind::Typo
        ),
        "DefiniteArticle" => (
            &[
                ("definitive article", "definite article"),
                ("definitive articles", "definite articles")
            ],
            "The correct term for `the` is `definite article`.",
            "The name of the word `the` is `definite article`.",
            LintKind::Usage
        ),
        "Discuss" => (
            &[
                ("discuss about", "discuss"),
                ("discussed about", "discussed"),
                ("discusses about", "discusses"),
                ("discussing about", "discussing"),
            ],
            "`About` is redundant",
            "Removes unnecessary `about` after `discuss`.",
            // or maybe Redundancy?
            LintKind::Usage
        ),
        "DoesOrDose" => (
            &[
                // Negatives
                ("dose not", "does not"),
                // Pronouns
                ("he dose", "he does"),
                ("it dose", "it does"),
                ("she dose", "she does"),
                ("someone dose", "someone does"),
                // Interrogatives
                ("how dose", "how does"),
                ("when dose", "when does"),
                ("where dose", "where does"),
                ("who dose", "who does"),
                ("why dose", "why does"),
            ],
            "This may be a typo for `does`.",
            "Tries to correct typos of `dose` to `does`.",
            LintKind::Typo
        ),
        "ExpandArgument" => (
            &[
                ("arg", "argument"),
                ("args", "arguments"),
            ],
            "Use `argument` instead of `arg`",
            "Expands the abbreviation `arg` to the full word `argument` for clarity.",
            LintKind::Style
        ),
        "ExpandDependencies" => (
            &[
                ("deps", "dependencies"),
                ("dep", "dependency"),
            ],
            "Use `dependencies` instead of `deps`",
            "Expands the abbreviation `deps` to the full word `dependencies` for clarity.",
            LintKind::Style
        ),
        "ExpandStandardInputAndOutput" => (
            &[
                ("stdin", "standard input"),
                ("stdout", "standard output"),
                ("stderr", "standard error"),
            ],
            "Use `standard input`, `standard output`, and `standard error` instead of `stdin`, `stdout`, and `stderr`",
            "Expands the abbreviations `stdin`, `stdout`, and `stderr` to the full words `standard input`, etc. for clarity.",
            LintKind::Style
        ),
        "ExplanationMark" => (
            &[
                ("explanation mark", "exclamation mark"),
                ("explanation marks", "exclamation marks"),
                ("explanation point", "exclamation point"),
            ],
            "The correct names for the `!` punctuation are `exclamation mark` and `exclamation point`.",
            "Corrects the eggcorn `explanation mark/point` to `exclamation mark/point`.",
            LintKind::Usage
        ),
        "ExtendOrExtent" => (
            &[
                ("a certain extend", "a certain extent"),
                ("to an extend", "to an extent"),
                ("to some extend", "to some extent"),
                ("to the extend that", "to the extent that")
            ],
            "Use `extent` for the noun and `extend` for the verb.",
            "Corrects `extend` to `extent` when the context is a noun.",
            // ConfusedPair??
            LintKind::WordChoice
        ),
        "HaveGone" => (
            &[
                ("had went", "had gone"),
                ("has went", "has gone"),
                ("have went", "have gone"),
                ("having went", "having gone"),
            ],
            "`Have gone` is the correct form.",
            "Corrects `have went` to `have gone`.",
            LintKind::Grammar
        ),
        "HavePassed" => (
            &[
                ("had past", "had passed"),
                ("has past", "has passed"),
                ("have past", "have passed"),
                ("having past", "having passed"),
            ],
            "Did you mean the verb `passed`?",
            "Suggests `past` for `passed` in case a verb was intended.",
            // ConfusedPair?
            LintKind::WordChoice
        ),
        "HomeInOn" => (
            &[
                ("hone in on", "home in on"),
                ("honed in on", "homed in on"),
                ("hones in on", "homes in on"),
                ("honing in on", "homing in on"),
            ],
            "Use `home in on` rather than `hone in on`",
            "Corrects `hone in on` to `home in on`.",
            LintKind::Eggcorn
        ),
        "InDetail" => (
            &[
                ("in details", "in detail"),
                ("in more details", "in more detail"),
            ],
            "Use singular `in detail` for referring to a detailed description.",
            "Corrects unidiomatic plural `in details` to `in detail`.",
            LintKind::Usage
        ),
        "InvestIn" => (
            &[
                // Verb
                ("invest into", "invest in"),
                ("invested into", "invested in"),
                ("investing into", "investing in"),
                ("invests into", "invests in"),
                // Noun
                ("investment into", "investment in"),
                // Note "investments into" can be correct in some contexts
            ],
            "Traditionally `invest` uses the preposition `in`.",
            "`Invest` is traditionally followed by 'in,' not `into.`",
            LintKind::Usage
        ),
        "MakeDoWith" => (
            &[
                ("make due with", "make do with"),
                ("made due with", "made do with"),
                ("makes due with", "makes do with"),
                ("making due with", "making do with"),
            ],
            "Use `do` instead of `due` when referring to a resource scarcity.",
            "Corrects `make due` to `make do` when followed by `with`."
        ),
        "MootPoint" => (
            &[
                ("mute point", "moot point"),
                ("point is mute", "point is moot"),
            ],
            "Use `moot` instead of `mute` when referring to a debatable or irrelevant point.",
            "Corrects `mute` to `moot` in the phrase `moot point`.",
            LintKind::Eggcorn
        ),
        "OperatingSystem" => (
            &[
                ("operative system", "operating system"),
                ("operative systems", "operating systems"),
            ],
            "Did you mean `operating system`?",
            "Ensures `operating system` is used correctly instead of `operative system`.",
            LintKind::Usage
        ),
        "Piggyback" => (
            &[
                ("piggy bag", "piggyback"),
                ("piggy bagged", "piggybacked"),
                ("piggy bagging", "piggybacking"),
            ],
            "Did you mean `piggyback`?",
            "Corrects the eggcorn `piggy bag` to `piggyback`, which is the proper term for riding on someone’s back or using an existing system.",
            LintKind::Eggcorn
        ),
    });

    add_many_to_many_mappings!(group, {
        "ChangeTack" => (
            &[
                // verb
                (&["change tact", "change tacks", "change tacts"], &["change tack"]),
                (&["changed tact", "changed tacks", "changed tacts"], &["changed tack"]),
                (&["changes tact", "changes tacks", "changes tacts"], &["changes tack"]),
                (&["changing tact", "changing tacks", "changing tacts"], &["changing tack"]),
                // noun
                (&["change of tact", "change of tacks", "change of tacts"], &["change of tack"]),
                (&["changes of tact", "changes of tacks", "changes of tacts"], &["changes of tack"]),
                (&["changing of tact", "changing of tacks", "changing of tacts"], &["changing of tack"])
            ],
            "A change in direction or approach is a change of `tack`. Not `tact` (or `tacks` or `tacts`).",
            "Locates errors in the idioms `to change tack` and `change of tack` to convey the correct meaning of altering one's course or strategy.",
            LintKind::Eggcorn
        ),
        "GetRidOf" => (
            &[
                (&["get rid off", "get ride of", "get ride off"], &["get rid of"]),
                (&["gets rid off", "gets ride of", "gets ride off"], &["gets rid of"]),
                (&["getting rid off", "getting ride of", "getting ride off"], &["getting rid of"]),
                (&["got rid off", "got ride of", "got ride off"], &["got rid of"]),
                (&["gotten rid off", "gotten ride of", "gotten ride off"], &["gotten rid of"]),
            ],
            "The idiom is `to get rid of`, not `off` or `ride`.",
            "Corrects common misspellings of the idiom `get rid of`.",
            LintKind::Typo
        ),
        "HowItLooksLike" => (
            &[
                (&["how he looks like"], &["how he looks", "what he looks like"]),
                (&["how it looks like", "how it look like", "how it look's like"], &["how it looks", "what it looks like"]),
                (&["how she looks like"], &["how she looks", "what she looks like"]),
                (&["how they look like", "how they looks like"], &["how they look", "what they look like"]),
            ],
            "Don't use both `how` and `like` together to express similarity.",
            "Corrects `how ... looks like` to `how ... looks` or `what ... looks like`.",
            LintKind::Grammar
        ),
        "MakeItSeem" => (
            &[
                (&["make it seems"], &["make it seem"]),
                (&["made it seems", "made it seemed"], &["made it seem"]),
                (&["makes it seems"], &["makes it seem"]),
                (&["making it seems"], &["making it seem"]),
            ],
            "Don't inflect `seem` in `make it seem`.",
            "Corrects `make it seems` to `make it seem`."
        ),
        "RiseTheQuestion" => (
            &[
                (&["rise the question"], &["raise the question"]),
                (&["rises the question"], &["raises the question"]),
                (&["risen the question", "rose the question"], &["raised the question"]),
                (&["rising the question"], &["raising the question"])
            ],
            "Use `raise` instead of `rise` when referring to the act of asking a question.",
            "Corrects `rise the question` to `raise the question`.",
            LintKind::Grammar
        ),
        "WholeEntire" => (
            &[
                (&["whole entire"], &["whole", "entire"]),
                // Avoid suggestions resulting in "a entire ...."
                (&["a whole entire"], &["a whole", "an entire"]),
            ],
            "Avoid redundancy. Use either `whole` or `entire` for referring to the complete amount or extent.",
            "Corrects the redundancy in `whole entire` to `whole` or `entire`.",
            LintKind::Redundancy
        ),
        "WorseOrWorst" => (
            &[
                // worst -> worse
                (&["a lot worst", "alot worst"], &["a lot worse"]),
                (&["become worst"], &["become worse"]),
                (&["became worst"], &["became worse"]),
                (&["becomes worst"], &["becomes worse"]),
                (&["becoming worst"], &["becoming worse"]),
                (&["far worst"], &["far worse"]),
                (&["get worst"], &["get worse"]),
                (&["gets worst"], &["gets worse"]),
                (&["getting worst"], &["getting worse"]),
                (&["got worst"], &["got worse"]),
                (&["gotten worst"], &["gotten worse"]),
                (&["make it worst"], &["make it worse"]),
                (&["made it worst"], &["made it worse"]),
                (&["makes it worst"], &["makes it worse"]),
                (&["making it worst"], &["making it worse"]),
                (&["make them worst"], &["make them worse"]),
                (&["made them worst"], &["made them worse"]),
                (&["makes them worst"], &["makes them worse"]),
                (&["making them worst"], &["making them worse"]),
                (&["much worst"], &["much worse"]),
                (&["turn for the worst"], &["turn for the worse"]),
                (&["worst and worst", "worse and worst", "worst and worse"], &["worse and worse"]),
                (&["worst than"], &["worse than"]),
                // worse -> worst
                (&["worse case scenario", "worse-case scenario", "worse-case-scenario"], &["worst-case scenario"]),
                (&["worse ever"], &["worst ever"]),
            ],
            "`Worse` is for comparing and `worst` is for the extreme case.",
            "Corrects `worse` and `worst` used in contexts where the other belongs.",
            LintKind::Agreement
        )
    });

    group.set_all_rules_to(Some(true));

    group
}

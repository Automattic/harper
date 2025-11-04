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
        "DigestiveTract" => (
            &[
                ("digestive track", "digestive tract"),
                ("digestive tracks", "digestive tracts"),
            ],
            "The correct term is digestive `tract`.",
            "Corrects `digestive track` to `digestive tract`.",
            LintKind::WordChoice
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
        "FootTheBill" => (
            &[
                ("flip the bill", "foot the bill"),
                ("flipped the bill", "footed the bill"),
                ("flipping the bill", "footing the bill"),
                ("flips the bill", "foots the bill"),
            ],
            "The standard expression is `foot the bill`.",
            "Corrects `flip the bill` to `foot the bill`.",
            LintKind::Nonstandard
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
        "PassersBy" => (
            &[
                ("passerbys", "passersby"),
                ("passer-bys", "passers-by"),
            ],
            "The correct plural is `passersby` or `passers-by`.",
            "Corrects `passerbys` and `passer-bys` to `passersby` or `passers-by`.",
            LintKind::Grammar
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

        // General litotes (double negatives) → direct positive suggestions
        "LitotesDirectPositive" => (
            &[
                ("not uncommon", "common"),
                ("not unusual", "common"),
                ("not insignificant", "significant"),
                ("not unimportant", "important"),
                ("not unlikely", "likely"),
                ("not infrequent", "frequent"),
                ("not inaccurate", "accurate"),
                ("not unclear", "clear"),
                ("not irrelevant", "relevant"),
                ("not unpredictable", "predictable"),
                ("not inadequate", "adequate"),
                ("not unpleasant", "pleasant"),
                ("not unreasonable", "reasonable"),
                ("not impossible", "possible"),
                ("more preferable", "preferable"),
                ("not online", "offline"),
                ("not offline", "online"),
            ],
            "Consider the direct form.",
            "Offers direct-positive alternatives when double negatives might feel heavy.",
            LintKind::Style
        ),

        // Redundant degree modifiers on positives (double positives) → base form
        "RedundantSuperlatives" => (
            &[
                ("more optimal", "optimal"),
                ("most optimal", "optimal"),
                ("more ideal", "ideal"),
                ("most ideal", "ideal"),
            ],
            "Avoid redundant degree modifiers; prefer the base adjective.",
            "Simplifies redundant double positives like `most optimal` to the base form.",
            LintKind::Redundancy
        ),
    });

    add_many_to_many_mappings!(group, {
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
        "ToTooIdioms" => (
            &[
                (&["a bridge to far"], &["a bridge too far"]),
                (&["cake and eat it to"], &["cake and eat it too"]),
                // "a few to many" has many false positives

                (&["go to far"], &["go too far"]),
                (&["goes to far"], &["goes too far"]),
                (&["going to far"], &["going too far"]),
                (&["gone to far"], &["gone too far"]),
                (&["went to far"], &["went too far"]),

                // "in to deep" has many false positives
                (&["life's to short", "lifes to short"], &["life's too short"]),
                (&["life is to short"], &["life is too short"]),

                // "one to many" has many false positives
                (&["put to fine a point"], &["put too fine a point"], ),

                (&["speak to soon"], &["speak too soon"]),
                (&["speaking to soon"], &["speaking too soon"]),
                // "speaks to soon" is very rare
                (&["spoke to soon"], &["spoke too soon"]),
                (&["spoken to soon"], &["spoken too soon"]),

                (&["think to much"], &["think too much"]),
                (&["to big for"], &["too big for"]),
                (&["to big to fail"], &["too big to fail"]),
                (&["to good to be true", "too good too be true"], &["too good to be true"]),
                (&["to much information"], &["too much information"]),
            ],
            "Use `too` rather than `to` in this expression.",
            "Corrects `to` used instead of `too`.",
            LintKind::Grammar
        ),
        "TooTo" => (
            &[
                (&["too big too fail"], &["too big to fail"])
            ],
            "Use `to` rather than `too` in this expression.",
            "Corrects `too` used instead of `to`.",
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
                (&["at worse"], &["at worst"]),
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

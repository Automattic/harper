use super::{LintGroup, MapPhraseLinter};

#[cfg(test)]
mod tests;

/// Produce a [`LintGroup`] that looks for errors in common phrases.
/// Comes pre-configured with the recommended default settings.
pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::default();

    macro_rules! add_exact_mappings {
        ($group:expr, {
            $($name:expr => ($input:expr, $corrections:expr, $hint:expr, $description:expr)),+ $(,)?
        }) => {
            $(
                $group.add_expr_linter(
                    $name,
                    Box::new(
                        MapPhraseLinter::new_fixed_phrases(
                            $input,
                            $corrections,
                            $hint,
                            $description
                        ),
                    ),
                );
            )+
        };
    }

    add_exact_mappings!(group, {
        // The name of the rule
        "ACoupleMore" => (
            // The phrase(s) to look for.
            ["a couple of more"],
            // The corrections to provide.
            ["a couple more"],
            // The message to be shown with the error.
            "The correct wording is `a couple more`, without the `of`.",
            // A description of the rule.
            "Corrects `a couple of more` to `a couple more`."
        ),
        "AfterAWhile" => (
            ["after while"],
            ["after a while"],
            "When describafterg a timeframe, use `a while`.",
            "Corrects the missing article after `after while` or `after awhile`, forming `after a while`."
        ),
        "AllOfASudden" => (
            ["all of the sudden"],
            ["all of a sudden"],
            "The phrase is `all of a sudden`, meaning `unexpectedly`.",
            "Corrects `all of the sudden` to `all of a sudden`."
        ),
        "ALongTime" => (
            ["along time"],
            ["a long time"],
            "Use `a long time` for referring to a duration of time.",
            "Corrects `along time` to `a long time`."
        ),
        "ALotWorst" => (
            ["a lot worst", "alot worst"],
            ["a lot worse"],
            "Use `worse` for comparing. (`Worst` is for the extreme case)",
            "Corrects `a lot worst` to `a lot worse` for proper comparative usage."
        ),
        "AlzheimersDisease" => (
            ["old-timers' disease"],
            ["Alzheimer’s disease"],
            "Use the correct medical term.",
            "Fixes the common misnomer `old-timers' disease`, ensuring the correct medical term `Alzheimer’s disease` is used."
        ),
        "AnAnother" => (
            ["an another", "a another"],
            ["another"],
            "Use `another` on its own.",
            "Corrects `an another` and `a another`."
        ),
        "AndIn" => (
            ["an in"],
            ["and in"],
            "Did you mean `and in`?",
            "Fixes the incorrect phrase `an in` to `and in` for proper conjunction usage."
        ),
        "AndTheLike" => (
            ["an the like"],
            ["and the like"],
            "Did you mean `and the like`?",
            "Fixes the typo in `and the like`."
        ),
        "AnotherAn" => (
            ["another an"],
            ["another"],
            "Use `another` on its own.",
            "Corrects `another an` to `another`."
        ),
        "AnotherOnes" => (
            ["another ones"],
            ["another one", "another one's", "other ones"],
            "`another` is singular but `ones` is plural. Or maybe you meant the possessive `one's`.",
            "Corrects `another ones`."
        ),
        "AnotherThings" => (
            ["another things"],
            ["another thing", "other things"],
            "`another` is singular but `things` is plural.",
            "Corrects `another things`."
        ),
        "AsFarBackAs" => (
            ["as early back as"],
            ["as far back as"],
            "Use `as far back as` for referring to a time in the past.",
            "Corrects nonstandard `as early back as` to `as far back as`."
        ),
        "AsItHappens" => (
            ["as it so happens"],
            ["as it happens"],
            "Did you mean `as it happens`?",
            "Corrects `as it so happens` to `as it happens`."
        ),
        "AsOfLate" => (
            ["as of lately"],
            ["as of late"],
            "The standard form is `as of late`.",
            "Corrects `as of lately` to `as of late`."
        ),
        "AsWell" => (
            ["aswell"],
            ["as well"],
            "`as well` should be written as two words.",
            "Corrects `aswell` to `as well`."
        ),
        "AtFaceValue" => (
            ["on face value"],
            ["at face value"],
            "`at face value is more idiomatic and more common.",
            "Corrects `on face value` to the more usual `at face value`."
        ),
        "AtTheEndOfTheDay" => (
            ["in the end of the day"],
            ["at the end of the day"],
            "Did you mean `at the end of the day`?",
            "Corrects `in the end of the day` to `at the end of the day`."
        ),
        "AvoidAndAlso" => (
            ["and also"],
            ["and"],
            "Consider using just `and`.",
            "Reduces redundancy by replacing `and also` with `and`."
        ),
        // Avoid suggestions resulting in "a entire ...."
        "AWholeEntire" => (
            ["a whole entire"],
            ["a whole", "an entire"],
            "Avoid redundancy. Use either `whole` or `entire` for referring to the complete amount or extent.",
            "Corrects the redundancy in `whole entire` to `whole` or `entire`."
        ),
        "BadRap" => (
            ["bed rap", "bad rep"],
            ["bad rap"],
            "Did you mean `bad rap`?",
            "Changes `bed rap` to the proper idiom `bad rap`."
        ),
        "BanTogether" => (
            ["ban together"],
            ["band together"],
            "Did you mean `band together`?",
            "Detects and corrects the common error of using `ban together` instead of the idiom `band together`, which means to unite or join forces."
        ),
        "BareInMind" => (
            ["bare in mind"],
            ["bear in mind"],
            "Did you mean `bear in mind`?",
            "Ensures the phrase `bear in mind` is used correctly instead of `bare in mind`."
        ),
        "BatedBreath" => (
            ["baited breath"],
            ["bated breath"],
            "Did you mean `bated breath`?",
            "Changes `baited breath` to the correct `bated breath`."
        ),
        "BeckAndCall" => (
            ["back and call"],
            ["beck and call"],
            "Did you mean `beck and call`?",
            "Fixes `back and call` to `beck and call`."
        ),
        "BeenThere" => (
            ["bee there"],
            ["been there"],
            "Did you mean `been there`?",
            "Corrects the misspelling `bee there` to the proper phrase `been there`."
        ),
        "Beforehand" => (
            ["before hand", "before-hand"],
            ["beforehand"],
            "Prefer the single-word adverb `beforehand`.",
            "`Beforehand` functions as a fixed adverb meaning ‘in advance’; writing it as two words or with a hyphen is nonstandard and can jar readers."
        ),
        "BestRegards" => (
            ["beat regards"],
            ["best regards"],
            "Use `best regards` to convey sincere well wishes in a closing.",
            "In valedictions, `best` expresses your highest regard—avoid the typo `beat regards`."
        ),
        "BlanketStatement" => (
            ["blanketed statement"],
            ["blanket statement"],
            "Use the more idiomatic phrasing.",
            "Corrects common errors in the phrase `blanket statement`."
        ),
        "Brutality" => (
            ["brutalness"],
            ["brutality"],
            "This word has a more standard, more common synonym.",
            "Suggests the more standard and common synonym `brutality`."
        ),
        "BuiltIn" => (
            ["in built", "in-built", "built in"],
            ["built-in"],
            "Prefer the hyphenated compound `built-in`.",
            "English convention treats `built-in` as a single, attributive adjective—meaning something integrated from the outset—whereas other forms like `in built` are non-standard and can feel awkward to readers."
        ),
        "ByAccident" => (
            ["on accident"],
            ["by accident"],
            "Did you mean `by accident`?",
            "Incorrect preposition: `by accident` is the idiomatic expression."
        ),
        "CanBeSeen" => (
            ["can be seem"],
            ["can be seen"],
            "Did you mean `can be seen`?",
            "Corrects `can be seem` to the proper phrase `can be seen`."
        ),
        "CaseInPoint" => (
            ["case and point"],
            ["case in point"],
            "`Case in point` is the correct form of the phrase.",
            "Corrects `case and point` to `case in point`."
        ),
        "CaseSensitive" => (
            ["case sensitive"],
            ["case-sensitive"],
            "Use the hyphenated form for `case-sensitive`.",
            "Ensures `case-sensitive` is correctly hyphenated."
        ),
        "ChangedTack" => (
            ["changed tact", "changed tacks", "changed tacts"],
            ["changed tack"],
            "Did you mean `changed tack`? This idiom is commonly used to indicate a change in direction or approach.",
            "Locates errors in the idiom `to change tack` to convey the correct meaning of altering one's course or strategy."
        ),
        "ChangeOfTack" => (
            ["change of tact", "change of tacks", "change of tacts"],
            ["change of tack"],
            "Did you mean `change of tack`? This idiom is commonly used to indicate a change in direction or approach.",
            "Locates errors in the idiom `change of tack` to convey the correct meaning of an alternative course or strategy."
        ),
        "ChangesOfTack" => (
            ["changes of tact", "changes of tacks", "changes of tacts"],
            ["changes of tack"],
            "Did you mean `changes of tack`? This idiom is commonly used to indicate changes in direction or approach.",
            "Locates errors in the idiom `change of tack` to convey the correct meaning of an alternative course or strategy."
        ),
        "ChangesTack" => (
            ["changes tact", "changes tacks", "changes tacts"],
            ["changes tack"],
            "Did you mean `changes tack`? This idiom is commonly used to indicate a change in direction or approach.",
            "Locates errors in the idiom `to change tack` to convey the correct meaning of altering one's course or strategy."
        ),
        "ChangeTack" => (
            ["change tact", "change tacks", "change tacts"],
            ["change tack"],
            "Did you mean `change tack`? This idiom is commonly used to indicate a change in direction or approach.",
            "Locates errors in the idiom `to change tack` to convey the correct meaning of altering one's course or strategy."
        ),
        "ChangingOfTack" => (
            ["changing of tact", "changing of tacks", "changing of tacts"],
            ["changing of tack"],
            "Did you mean `changing of tack`? This idiom is commonly used to indicate a change in direction or approach.",
            "Locates errors in the idiom `to change of tack` to convey the correct meaning of altering one's course or strategy."
        ),
        "ChangingTack" => (
            ["changing tact", "changing tacks", "changing tacts"],
            ["changing tack"],
            "Did you mean `changing tack`? This idiom is commonly used to indicate a change in direction or approach.",
            "Locates errors in the idiom `to change tack` to convey the correct meaning of altering one's course or strategy."
        ),
        "ChockFull" => (
            ["chock full"],
            ["chock-full"],
            "Use the hyphenated form for `chock-full`.",
            "Ensures `chock-full` is correctly hyphenated."
        ),
        "ClientSide" => (
            ["client's side"],
            ["client-side"],
            "In client-server contexts, use `client-side` rather than `client's side`.",
            "Corrects `client's side` to `client-side`, which is usual in `client-server contexts`."
        ),
        "CondenseAllThe" => (
            ["all of the"],
            ["all the"],
            "Consider simplifying to `all the`.",
            "Suggests removing `of` in `all of the` for a more concise phrase."
        ),
        "CoursingThroughVeins" => (
            ["cursing through veins"],
            ["coursing through veins"],
            "In this idiom, blood “courses” (flows) through veins, not “curses”.",
            "In English idioms, “to course” means to flow rapidly—so avoid the eggcorn `cursing through veins.`"
        ),
        "DampSquib" => (
            ["damp squid"],
            ["damp squib"],
            "Use the correct phrase for a disappointing outcome.",
            "Corrects the eggcorn `damp squid` to `damp squib`, ensuring the intended meaning of a failed or underwhelming outcome."
        ),
        "DayAndAge" => (
            ["day in age"],
            ["day and age"],
            "Use `day and age` for referring to the present time.",
            "Corrects the eggcorn `day in age` to `day and age`, which properly means the current era or time period."
        ),
        "DefiniteArticle" => (
            ["definitive article"],
            ["definite article"],
            "The correct term for `the` is `definite article`.",
            "The name of the word `the` is `definite article`."
        ),
        "DefiniteArticles" => (
            ["definitive articles"],
            ["definite articles"],
            "The correct term for `the` is `definite article`.",
            "The name of the word `the` is `definite article`."
        ),
        "Discuss" => (
            ["discuss about"],
            ["discuss"],
            "`About` is redundant",
            "Removes unnecessary `about` after `discuss`."
        ),
        "Discussed" => (
            ["discussed about"],
            ["discussed"],
            "Use `discussed` without `about`.",
            "Removes unnecessary `about` after `discussed`."
        ),
        "Discusses" => (
            ["discusses about"],
            ["discusses"],
            "`About` is redundant",
            "Removes unnecessary `about` after `discusses`."
        ),
        "Discussing" => (
            ["discussing about"],
            ["discussing"],
            "`About` is redundant",
            "Removes unnecessary `about` after `discussing`."
        ),
        "DoNotWant" => (
            ["don't wan", "do not wan"],
            ["don't want", "do not want"],
            "Use the full verb “want” after negation: “don't want” or “do not want.”",
            "In English, negation still requires the complete verb form (“want”), so avoid truncating it to “wan.”"
        ),
        "EachAndEveryOne" => (
            ["each and everyone"],
            ["each and every one"],
            "Use `each and every one` for referring to a group of people or things.",
            "Corrects `each and everyone` to `each and every one`."
        ),
        "EludedTo" => (
            ["eluded to"],
            ["alluded to"],
            "Did you mean `alluded to`?",
            "Corrects `eluded to` to `alluded to` in contexts referring to indirect references."
        ),
        "EnMasse" => (
            ["on mass", "on masse", "in mass"],
            ["en masse"],
            "Did you mean `en masse`?",
            "Detects variants like `on mass` or `in mass` and suggests `en masse`."
        ),
        "EnRoute" => (
            ["on route to", "in route to", "on-route to", "in-route to"],
            ["en route to", "en-route to"],
            "Did you mean `en route`?",
            "Detects variants like `on route` or `in route` and suggests `en route`."
        ),
        "EverPresent" => (
            ["ever present"],
            ["ever-present"],
            "Hyphenate `ever-present` when it functions as a compound adjective.",
            "Corrects the missing hyphen in `ever present` to the compound adjective `ever-present`."
        ),
        "Excellent" => (
            ["very good"],
            ["excellent"],
            "Vocabulary enhancement: use `excellent` instead of `very good`",
            "Provides a stronger word choice by replacing `very good` with `excellent` for clarity and emphasis."
        ),
        "ExpandArgument" => (
            ["arg"],
            ["argument"],
            "Use `argument` instead of `arg`",
            "Expands the abbreviation `arg` to the full word `argument` for clarity."
        ),
        "ExpandArguments" => (
            ["args"],
            ["arguments"],
            "Use `arguments` instead of `args`",
            "Expands the abbreviation `args` to the full word `arguments` for clarity."
        ),
        "ExpandBecause" => (
            ["cuz"],
            ["because"],
            "Use `because` instead of informal `cuz`",
            "Expands the informal abbreviation `cuz` to the full word `because` for formality."
        ),
        "ExpandDependencies" => (
            ["deps"],
            ["dependencies"],
            "Use `dependencies` instead of `deps`",
            "Expands the abbreviation `deps` to the full word `dependencies` for clarity."
        ),
        "ExpandDependency" => (
            ["dep"],
            ["dependency"],
            "Use `dependency` instead of `dep`",
            "Expands the abbreviation `dep` to the full word `dependency` for clarity."
        ),
        "ExpandMinimum" => (
            ["min"],
            ["minimum"],
            "Use `minimum` instead of `min`",
            "Expands the abbreviation `min` to the full word `minimum` for clarity."
        ),
        "ExpandStandardInput" => (
            ["stdin"],
            ["standard input"],
            "Use `standard input` instead of `stdin`",
            "Expands the abbreviation `stdin` to `standard input` for clarity."
        ),
        "ExpandStandardOutput" => (
            ["stdout"],
            ["standard output"],
            "Use `standard output` instead of `stdout`",
            "Expands the abbreviation `stdout` to `standard output` for clarity."
        ),
        "ExpandWith" => (
            ["w/"],
            ["with"],
            "Use `with` instead of `w/`",
            "Expands the abbreviation `w/` to the full word `with` for clarity."
        ),
        "ExpandWithout" => (
            ["w/o"],
            ["without"],
            "Use `without` instead of `w/o`",
            "Expands the abbreviation `w/o` to the full word `without` for clarity."
        ),
        "Expatriate" => (
            ["ex-patriot"],
            ["expatriate"],
            "Use the correct term for someone living abroad.",
            "Fixes the misinterpretation of `expatriate`, ensuring the correct term is used for individuals residing abroad."
        ),
        "ExplanationMark" => (
            ["explanation mark"],
            ["exclamation mark"],
            "The correct name for the `!` punctuation is `exclamation mark`.",
            "Corrects the eggcorn `explanation mark` to `exclamation mark`."
        ),
        "ExplanationMarks" => (
            ["explanation marks"],
            ["exclamation marks"],
            "The correct name for the `!` punctuation is `exclamation mark`.",
            "Corrects the eggcorn `explanation mark` to `exclamation mark`."
        ),
        "ExplanationPoint" => (
            ["explanation point"],
            ["exclamation point"],
            "The correct name for the `!` punctuation is `exclamation point`.",
            "Corrects the eggcorn `explanation point` to `exclamation point`."
        ),
        "FaceFirst" => (
            ["face first into"],
            ["face-first into"],
            "Should this be `face-first`?",
            "Ensures `face first` is correctly hyphenated as `face-first` when used before `into`."
        ),
        "FairBit" => (
            ["fare bit"],
            ["fair bit"],
            "A `decent amount` is a `fair bit`. `Fare` is the price of a ticket.",
            "Corrects malapropisms of `a fair bit`."
        ),
        "FarWorse" => (
            ["far worst"],
            ["far worse"],
            "Use `worse` for comparing. (`Worst` is for the extreme case)",
            "Corrects `far worst` to `far worse` for proper comparative usage."
        ),
        "FastPaste" => (
            ["fast paste", "fast-paste"],
            ["fast-paced"],
            "Did you mean `fast-paced`?",
            "Detects incorrect usage of `fast paste` or `fast-paste` and suggests `fast-paced` as the correct phrase."
        ),
        "FatalOutcome" => (
            ["fatal outcome"],
            ["death"],
            "Consider using `death` for clarity.",
            "Replaces `fatal outcome` with the more direct term `death` for conciseness."
        ),
        "FetalPosition" => (
            ["the feeble position"],
            ["the fetal position"],
            "Use the correct term for a curled-up posture.",
            "Ensures the correct use of `fetal position`, avoiding confusion with `feeble position`, which is not a standard phrase."
        ),
        "ForAllIntentsAndPurposes" => (
            ["for all intensive purposes"],
            ["for all intents and purposes"],
            "Use the correct phrase meaning 'in every practical sense'.",
            "Corrects `for all intensive purposes` to `for all intents and purposes`, ensuring the phrase conveys its intended meaning."
        ),
        "ForALongTime" => (
            ["for along time"],
            ["for a long time"],
            "Use the standard phrase `for a long time` to indicate an extended duration.",
            "Eliminates the incorrect merging in `for along time`."
        ),
        "ForAWhile" => (
            ["for while"],
            ["for a while"],
            "When describing a timeframe, use `a while`.",
            "Corrects the missing article in `for while` or `for awhile`, forming `for a while`."
        ),
        "FreeRein" => (
            ["free reign"],
            ["free rein"],
            "Use the correct phrase for unrestricted control.",
            "Ensures the correct use of `free rein`, avoiding confusion with `free reign`, which incorrectly suggests authority rather than freedom of action."
        ),
        "Freezing" => (
            ["very cold", "really cold", "extremely cold"],
            ["freezing"],
            "A more vivid adjective would better capture extreme cold.",
            "Encourages vivid writing by suggesting `freezing` instead of weaker expressions like `very cold.`"
        ),
        "FurtherAdo" => (
            ["further adieu"],
            ["further ado"],
            "Don't confuse the French/German `adieu`, meaning `farewell`, with the English `ado`, meaning `fuss`.",
            "Corrects `adieu` to `ado`."
        ),
        "GetRidOff" => (
            ["get rid off", "get ride of", "get ride off"],
            ["get rid of"],
            "Did you mean `get rid of`?",
            "Ensures `get rid of` is used instead of `get rid off`."
        ),
        "GetsRidOff" => (
            ["gets rid off", "gets ride of", "gets ride off"],
            ["gets rid of"],
            "Did you mean `gets rid of`?",
            "Ensures `gets rid of` is used instead of `gets rid off`."
        ),
        "GettingRidOff" => (
            ["getting rid off", "getting ride of", "getting ride off"],
            ["getting rid of"],
            "Did you mean `getting rid of`?",
            "Ensures `getting rid of` is used instead of `getting rid off`."
        ),
        "GildedAge" => (
            ["guilded age"],
            ["Gilded Age"],
            "The period of economic prosperity is called the `Gilded Age`.",
            "If referring to the period of economic prosperity, the correct term is `Gilded Age`."
        ),
        "GoingTo" => (
            ["gong to"],
            ["going to"],
            "Did you mean `going to`?",
            "Corrects `gong to` to the intended phrase `going to`."
        ),
        "GotRidOff" => (
            ["got rid off", "got ride of", "got ride off"],
            ["got rid of"],
            "Did you mean `got rid of`?",
            "Ensures `got rid of` is used instead of `got rid off`."
        ),
        "GottenRidOff" => (
            ["gotten rid off", "gotten ride of", "gotten ride off"],
            ["gotten rid of"],
            "Did you mean `gotten rid of`?",
            "Ensures `gotten rid of` is used instead of `gotten rid off`."
        ),
        "GuineaBissau" => (
            // Note: this lint matches any case but cannot correct wrong case
            // Note: It can only correct the hyphenation
            // Note: See linting/matcher.rs for case corrections
            // Note: $input must already be the correct case
            // Note: do not add other case variants here
            ["Guinea Bissau"],
            ["Guinea-Bissau"],
            "The official spelling is hyphenated.",
            "Checks for the correct official name of the African country."
        ),
        "HadGone" => (
            ["had went"],
            ["had gone"],
            "`Had gone` is the correct form.",
            "Corrects `had went` to `had gone`."
        ),
        "HadOf" => (
            ["had of"],
            ["had have", "had've"],
            "Did you mean `had have` or `had've`?",
            "Flags the unnecessary use of `of` after `had` and suggests the correct forms."
        ),
        "HadPassed" => (
            ["had past"],
            ["had passed"],
            "Did you mean the verb `passed`?",
            "Suggests `past` for `passed` in case a verb was intended."
        ),
        "HalfAnHour" => (
            ["half an our"],
            ["half an hour"],
            "Remember the silent 'h' when writing `hour`: `half an hour`.",
            "Fixes the eggcorn `half an our` to the accepted `half an hour`."
        ),
        "Haphazard" => (
            ["half hazard", "half-hazard", "halfhazard"],
            ["haphazard"],
            "Use `haphazard` for randomness or lack of organization.",
            "Corrects the eggcorn `half hazard` to `haphazard`, which properly means lacking organization or being random."
        ),
        "HasGone" => (
            ["has went"],
            ["has gone"],
            "`Has gone` is the correct form.",
            "Corrects `has went` to `has gone`."
        ),
        "HasPassed" => (
            ["has past"],
            ["has passed"],
            "Did you mean the verb `passed`?",
            "Suggests `past` for `passed` in case a verb was intended."
        ),
        "HaveGone" => (
            ["have went"],
            ["have gone"],
            "`Have gone` is the correct form.",
            "Corrects `have went` to `have gone`."
        ),
        "HavePassed" => (
            ["have past"],
            ["have passed"],
            "Did you mean the verb `passed`?",
            "Suggests `past` for `passed` in case a verb was intended."
        ),
        "HavingGone" => (
            ["having went"],
            ["having gone"],
            "`Having gone` is the correct form.",
            "Corrects `having went` to `having gone`."
        ),
        "HavingPassed" => (
            ["having past"],
            ["having passed"],
            "Did you mean the verb `passed`?",
            "Suggests `past` for `passed` in case a verb was intended."
        ),
        "HomedInOn" => (
            ["honed in on"],
            ["homed in on"],
            "Use `home in on` rather than `hone in on`",
            "Corrects `hone in on` to `home in on`."
        ),
        "HomeInOn" => (
            ["hone in on"],
            ["home in on"],
            "Use `home in on` rather than `hone in on`",
            "Corrects `hone in on` to `home in on`."
        ),
        "HomesInOn" => (
            ["hones in on"],
            ["homes in on"],
            "Use `home in on` rather than `hone in on`",
            "Corrects `hone in on` to `home in on`."
        ),
        "HomingInOn" => (
            ["honing in on"],
            ["homing in on"],
            "Use `home in on` rather than `hone in on`",
            "Corrects `hone in on` to `home in on`."
        ),
        "HumanBeings" => (
            ["human's beings", "humans beings"],
            ["human beings"],
            "Use `human beings` to refer to people collectively.",
            "Eliminates the incorrect possessive/plural usage like `human's beings` or `humans beings`."
        ),
        "HumanLife" => (
            ["human live"],
            ["human life"],
            "Did you mean `human life`?",
            "Changes `human live` to `human life`."
        ),
        "HungerPang" => (
            ["hunger pain"],
            ["hunger pang"],
            "Did you mean `hunger pang`?",
            "Corrects `hunger pain` to `hunger pang`."
        ),
        "IAm" => (
            ["I a m"],
            ["I am"],
            "Did you mean `I am`?",
            "Fixes the incorrect spacing in `I a m` to properly form `I am`."
        ),
        "IAmAgreement" => (
            ["I are"],
            ["I am"],
            "The first-person singular pronoun `I` requires the verb form `am`; `are` belongs to second-person or plural contexts.",
            "Corrects `I are` to `I am`."
        ),
        "IDo" => (
            ["I does"],
            ["I do"],
            "`I` pairs with the bare verb `do`; the –s inflection `does` is reserved for third-person singular subjects.",
            "Corrects `I does` to `I do`."
        ),
        "InAndOfItself" => (
            ["in of itself"],
            ["in and of itself"],
            "Use `in and of itself` for referring to something's inherent or intrinsic quality.",
            "Corrects nonstandard `in of itself` to standard `in and of itself`."
        ),
        "InAnyWay" => (
            ["in anyway"],
            ["in any way"],
            "Use `in any way` for emphasizing a point.",
            "Corrects ungrammatical `in anyway` to `in any way`."
        ),
        "InAWhile" => (
            ["in while"],
            ["in a while"],
            "When describing a timeframe, use `a while`.",
            "Corrects the missing article in `in while` or `in awhile`, forming `in a while`."
        ),
        "InCase" => (
            ["incase"],
            ["in case"],
            "`In case` should be written as two words.",
            "Corrects `incase` to `in case`."
        ),
        "InDetail" => (
            ["in details"],
            ["in detail"],
            "Use singular `in detail` for referring to a detailed description.",
            "Correct unidiomatic plural `in details` to `in detail`."
        ),
        "InMoreDetail" => (
            ["in more details"],
            ["in more detail"],
            "Use singular `in more detail` for referring to a detailed description.",
            "Correct unidiomatic plural `in more details` to `in more detail`."
        ),
        "InNeedOf" => (
            ["in need for"],
            ["in need of"],
            "Use `in need of` for when something is required or necessary.",
            "Corrects `in need for` to `in need of`."
        ),
        "InOneFellSwoop" => (
            ["in one foul swoop"],
            ["in one fell swoop"],
            "Use the correct phrase for something happening suddenly.",
            "Corrects `in one foul swoop` to `in one fell swoop`, preserving the phrase’s original meaning of sudden and complete action."
        ),
        "InsteadOf" => (
            ["in stead of"],
            ["instead of"],
            "Use the modern single word `instead of` to indicate a replacement.",
            "Corrects the archaic or mistaken separation `in stead of` to `instead of` in everyday usage."
        ),
        "Insurmountable" => (
            ["unsurmountable"],
            ["insurmountable"],
            "This word has a more standard, more common synonym.",
            "Suggests the more standard and common synonym `insurmountable`."
        ),
        "Intact" => (
            ["in tact"],
            ["intact"],
            "Use `intact` to mean undamaged or whole.",
            "Prevents the erroneous spacing in `in tact`; `intact` is the single correct word."
        ),
        "InThe" => (
            ["int he"],
            ["in the"],
            "Did you mean `in the`?",
            "Detects and corrects a spacing error where `in the` is mistakenly written as `int he`. Proper spacing is essential for readability and grammatical correctness in common phrases."
        ),
        "InvestedIn" => (
            ["invested into"],
            ["invested in"],
            "Traditionally `invest` uses the preposition `in`.",
            "`Invest` is traditionally followed by 'in,' not `into.`"
        ),
        "InvestIn" => (
            ["invest into"],
            ["invest in"],
            "Traditionally `invest` uses the preposition `in`.",
            "`Invest` is traditionally followed by 'in,' not `into.`"
        ),
        "InvestingIn" => (
            ["investing into"],
            ["investing in"],
            "Traditionally `invest` uses the preposition `in`.",
            "`Invest` is traditionally followed by 'in,' not `into.`"
        ),
        "InvestsIn" => (
            ["invests into"],
            ["invests in"],
            "Traditionally `invest` uses the preposition `in`.",
            "`Invest` is traditionally followed by 'in,' not `into.`"
        ),
        "IsKnownFor" => (
            ["is know for"],
            ["is known for"],
            "Did you mean `is known for`?",
            "Typo: `known` is the correct past participle."
        ),
        "ItCan" => (
            ["It cam"],
            ["It can"],
            "Did you mean `It can`?",
            "Corrects the misspelling `It cam` to the proper phrase `It can`."
        ),
        "IveGotTo" => (
            ["I've go to"],
            ["I've got to"],
            "Use `I've got to` for necessity or obligation.",
            "Corrects the slip `I've go to` to the idiomatic `I've got to`."
        ),
        "JawDropping" => (
            ["jar-dropping"],
            ["jaw-dropping"],
            "Use the correct phrase for something astonishing.",
            "Corrects `jar-dropping` to `jaw-dropping`, ensuring the intended meaning of something that causes amazement."
        ),
        "JustDeserts" => (
            ["just desserts"],
            ["just deserts"],
            "Use the correct phrase for receiving what one deserves.",
            "Ensures `just deserts` is used correctly, preserving its meaning of receiving an appropriate outcome for one's actions."
        ),
        "KindOf" => (
            ["kinda of"],
            ["kind of", "kinda"],
            "`Kinda` already means `kind of`, so `kinda of` is redundant.",
            "Corrects `kinda of` to `kind of`."
        ),
        "KindRegards" => (
            ["kid regards"],
            ["kind regards"],
            "Did you mean `kind regards`?",
            "Changes `kid regards` to `kind regards`."
        ),
        "LastButNotLeast" => (
            ["last but not the least", "last, but not the least", "last but, not least", "last but not last"],
            ["last but not least"],
            "Use the more idiomatic phrasing.",
            "Corrects common errors in the phrase `last but not least`."
        ),
        "LastDitch" => (
            ["last ditch", "last ditched", "last-ditched"],
            ["last-ditch"],
            "In this idiom, `ditch` is a noun and a hyphen is needed.",
            "Corrects wrong variations of the idiomatic adjective `last-ditch`."
        ),
        "LetAlone" => (
            ["let along"],
            ["let alone"],
            "Did you mean `let alone`?",
            "Changes `let along` to `let alone`."
        ),
        "LikeThePlague" => (
            ["like a plague"],
            ["like the plague"],
            "`Things are avoided `like the plague` not `like a plague`.",
            "Corrects `like a plague` to `like the plague`."
        ),
        "LowHangingFruit" => (
            ["low hanging fruit", "low hanging fruits", "low-hanging fruits"],
            ["low-hanging fruit"],
            "The standard form is `low-hanging fruit` with a hyphen and singular form.",
            "Corrects non-standard variants of `low-hanging fruit`."
        ),
        "Monumentous" => (
            ["monumentous"],
            ["momentous", "monumental"],
            "Retain `monumentous` for jocular effect. Otherwise `momentous` indicates great signifcance while `monumental` indicates imposing size.",
            "Advises using `momentous` or `monumental` instead of `monumentous` for serious usage."
        ),
        "MuchAdo" => (
            ["much adieu"],
            ["much ado"],
            "Don't confuse the French/German `adieu`, meaning `farewell`, with the English `ado`, meaning `fuss`.",
            "Corrects `adieu` to `ado`."
        ),
        "MuchWorse" => (
            ["much worst"],
            ["much worse"],
            "Use `worse` for comparing. (`Worst` is for the extreme case)",
            "Corrects `much worst` to `much worse` for proper comparative usage."
        ),
        "MutePoint" => (
            ["mute point"],
            ["moot point"],
            "Did you mean `moot point`?",
            "Ensures `moot point` is used instead of `mute point`, as `moot` means debatable or irrelevant."
        ),
        "MyHouse" => (
            ["mu house"],
            ["my house"],
            "Did you mean `my house`?",
            "Fixes the typo `mu house` to `my house`."
        ),
        "NeedHelp" => (
            ["ned help"],
            ["need help"],
            "Did you mean `need help`?",
            "Changes `ned help` to the correct `need help`."
        ),
        "NerveRacking" => (
            ["nerve racking", "nerve wracking", "nerve wrecking", "nerve-wracking", "nerve-wrecking"],
            ["nerve-racking"],
            "Use `nerve-racking` for something that causes anxiety or tension.",
            "Corrects common misspellings and missing hyphen in `nerve-racking`."
        ),
        "NotIn" => (
            ["no in"],
            ["not in"],
            "Use `not in` for correct grammar.",
            "Replaces `no in` with `not in`."
        ),
        "NotTo" => (
            ["no to"],
            ["not to"],
            "Did you mean `not to`?",
            "Corrects `no to` to `not to`, ensuring proper negation."
        ),
        "OfCourse" => (
            ["off course", "o course"],
            ["Of course"],
            "Did you mean `of course`?",
            "Detects the non‐idiomatic phrase `off course` and suggests the correct form `of course`."
        ),
        "OffTheCuff" => (
            ["off the cuff"],
            ["off-the-cuff"],
            "Use the hyphenated form for `off-the-cuff`.",
            "Ensures `off-the-cuff` is correctly hyphenated."
        ),
        "OldWivesTale" => (
            ["old wise tale"],
            ["old wives' tale"],
            "Use the correct phrase for a superstition or myth.",
            "Corrects `old wise tale` to `old wives' tale`, preserving the phrase’s meaning as an unfounded traditional belief."
        ),
        "OnceInAWhile" => (
            ["once a while", "once and a while"],
            ["once in a while"],
            "The correct idiom is `once in a while`.",
            "Corrects two common malapropisms of `once in a while`."
        ),
        "OnSecondThought" => (
            ["on second though"],
            ["on second thought"],
            "Idiomatic expression: use `on second thought` instead of `on second though`",
            "Replaces the nonstandard `on second though` with the common idiom `on second thought` to indicate reconsideration."
        ),
        "OnTheSpurOfTheMoment" => (
            ["on the spurt of the moment", "at the spur of the moment", "in the spur of the moment"],
            ["on the spur of the moment"],
            "Use the correct phrase for acting spontaneously.",
            "Ensures the correct use of `on the spur of the moment`, avoiding nonstandard variations."
        ),
        "OperativeSystem" => (
            ["operative system"],
            ["operating system"],
            "Did you mean `operating system`?",
            "Ensures `operating system` is used correctly instead of `operative system`."
        ),
        "OperativeSystems" => (
            ["operative systems"],
            ["operating systems"],
            "Did you mean `operating systems`?",
            "Ensures `operating systems` is used correctly instead of `operative systems`."
        ),
        "PeaceOfMind" => (
            ["piece of mind"],
            ["peace of mind"],
            "The phrase is `peace of mind`, meaning `calm`. A `piece` is a `part` of something.",
            "Corrects `piece of mind` to `peace of mind`."
        ),
        "PiggyBag" => (
            ["piggy bag"],
            ["piggyback"],
            "Did you mean `piggyback`?",
            "Corrects the eggcorn `piggy bag` to `piggyback`, which is the proper term for riding on someone’s back or using an existing system."
        ),
        "PiggyBagged" => (
            ["piggy bagged"],
            ["piggybacked"],
            "Did you mean `piggybacked`?",
            "Corrects the eggcorn `piggy bagged` to `piggybacked`, the proper past tense form for riding on someone’s back or making use of an existing system."
        ),
        "PiggyBagging" => (
            ["piggy bagging"],
            ["piggybacking"],
            "Did you mean `piggybacking`?",
            "Corrects the eggcorn `piggy bagging` to `piggybacking`, the proper verb form for riding on someone’s back or leveraging an existing system."
        ),
        "PointIsMoot" => (
            ["your point is mute"],
            ["your point is moot"],
            "Did you mean `your point is moot`?",
            "Typo: `moot` (meaning debatable) is correct rather than `mute`."
        ),
        "PointsOfView" => (
            ["point of views"],
            ["points of view"],
            "The correct plural is `points of view`.",
            "Corrects pluralizing the wrong noun in `point of view`."
        ),
        "PortAuPrince" => (
            // Note: this lint matches any case but cannot correct wrong case
            // Note: It can only correct the hyphenation
            // Note: See linting/matcher.rs for case corrections
            // Note: $input must already be the correct case
            // Note: do not add other case variants here
            ["Port au Prince"],
            ["Port-au-Prince"],
            "The official spelling is hyphenated.",
            "Checks for the correct official name of the capital of Haiti."
        ),
        "PortoNovo" => (
            // Note: this lint matches any case but cannot correct wrong case
            // Note: It can only correct the hyphenation
            // Note: See linting/matcher.rs for case corrections
            // Note: $input must already be the correct case
            // Note: do not add other case variants here
            ["Porto Novo"],
            ["Porto-Novo"],
            "The official spelling is hyphenated.",
            "Checks for the correct official name of the capital of Benin."
        ),
        "PrayingMantis" => (
            ["preying mantis"],
            ["praying mantis"],
            "Use the insect's correct name.",
            "Corrects `preying mantis` to `praying mantis`, ensuring accurate reference to the insect’s characteristic pose."
        ),
        "RapidFire" => (
            ["rapid fire"],
            ["rapid-fire"],
            "It is more idiomatic to hypenate `rapid-fire`.",
            "Checks to ensure writers hyphenate `rapid-fire`."
        ),
        "RealTrouper" => (
            ["real trooper"],
            ["real trouper"],
            "Use the correct phrase for someone who perseveres.",
            "Ensures the correct use of `real trouper`, distinguishing it from `trooper`, which refers to a soldier or police officer."
        ),
        "RifeWith" => (
            ["ripe with"],
            ["rife with"],
            "Use the correct phrase for something abundant.",
            "Corrects `ripe with` to `rife with`, preserving the phrase’s meaning of being filled with something, often undesirable."
        ),
        "RoadMap" => (
            ["roadmap"],
            ["road map"],
            "Did you mean `road map`?",
            "Detects when `roadmap` is used instead of `road map`, prompting the correct spacing."
        ),
        "SameAs" => (
            ["same then"],
            ["same as"],
            "Did you mean `same as`?",
            "Corrects the incorrect phrase `same then` to the standard `same as`."
        ),
        "ScantilyClad" => (
            ["scandally clad"],
            ["scantily clad"],
            "Use the correct phrase for minimal attire.",
            "Fixes `scandally clad` to `scantily clad`, ensuring clarity in describing minimal attire."
        ),
        "ServerSide" => (
            ["server's side"],
            ["server-side"],
            "In client-server contexts, use `server-side` rather than `server's side`.",
            "Corrects `server's side` to `server-side`, which is usual in `client-server contexts`."
        ),
        "SimpleGrammatical" => (
            ["simply grammatical"],
            ["simple grammatical"],
            "Use `simple grammatical` for correct adjective usage.",
            "Corrects `simply grammatical` to `simple grammatical` for proper adjective usage."
        ),
        "SneakingSuspicion" => (
            ["sneaky suspicion"],
            ["sneaking suspicion"],
            "Did you mean `sneaking suspicion`?",
            "Changes `sneaky suspicion` to `sneaking suspicion`."
        ),
        "SomebodyElses" => (
            ["somebody's else", "somebody's else's"],
            ["somebody else's"],
            "This should be `somebody else's`?",
            "Corrects `somebody else's` when the `'s` is in the wrong place."
        ),
        "SomeOfThe" => (
            ["some the"],
            ["some of the"],
            "Add `of` to form the partitive phrase: `some of the`.",
            "Quantity words such as `some` normally take `of` before a definite article. Including `of` signals that you mean a subset of a larger set, preventing a momentary stumble in comprehension."
        ),
        "SoonerOrLater" => (
            ["sooner than later"],
            ["sooner rather than later", "sooner or later"],
            "Did you mean `sooner rather than later` or `sooner or later`?",
            "Fixes the improper phrase `sooner than later` by suggesting standard alternatives."
        ),
        "SpecialAttention" => (
            ["spacial attention"],
            ["special attention"],
            "Did you mean `special attention`?",
            "Changes `spacial attention` to `special attention`."
        ),
        "SpokeTooSoon" => (
            ["spoke to soon"],
            ["spoke too soon"],
            "Use the adverb `too` instead.",
            "Identifies common misuse of the preposition `to` in the phrase `spoke too soon`."
        ),
        "Starving" => (
            ["very hungry", "really hungry", "extremely hungry"],
            ["starving"],
            "A more vivid adjective would better convey intense hunger.",
            "Encourages vivid writing by suggesting `starving` instead of weaker expressions like `very hungry.`"
        ),
        "StateOfTheArt" => (
            ["state of art"],
            ["state of the art"],
            "Did you mean `state of the art`?",
            "Detects incorrect usage of `state of art` and suggests `state of the art` as the correct phrase."
        ),
        "StatuteOfLimitations" => (
            ["statue of limitations"],
            ["statute of limitations"],
            "A `statue` is a sculpture; in legal terms, the correct word is `statute`.",
            "Corrects `statue of limitations` to `statute of limitations`."
        ),
        "SufficeItToSay" => (
            ["suffice to say"],
            ["suffice it to say"],
            "`Suffice it to say` is the more standard and more common variant.",
            "Corrects `suffice to say` to `suffice it to say`."
        ),
        "SupposedTo" => (
            ["suppose to"],
            ["supposed to"],
            "Did you mean `supposed to`?",
            "Fixes `suppose to` to the correct `supposed to`."
        ),
        "TakeItPersonally" => (
            ["take it personal"],
            ["take it personally"],
            "The more standard, less colloquial form is `take it personally`.",
            "Corrects `take it personal` to `take it personally`."
        ),
        "ThanksALot" => (
            ["thanks lot", "thanks alot"],
            ["thanks a lot"],
            "Prefer the two-word expression `thanks a lot`.",
            "`Thanks a lot` is the fixed, widely accepted form, while variants like `thanks lot` or `thanks alot` are non-standard and can jar readers."
        ),
        "ThatChallenged" => (
            ["the challenged"],
            ["that challenged"],
            "Did you mean `that challenged`?",
            "Changes `the challenged` to `that challenged` to fix the misspelling."
        ),
        "ThatChallenged" => (
            ["the challenged"],
            ["that challenged"],
            "Use `that challenged` for correct relative clause.",
            "Corrects `the challenged` to `that challenged` for proper relative clause usage."
        ),
        "ThatThis" => (
            ["the this"],
            ["that this"],
            "Did you mean `that this`?",
            "Fixes `the this` to the correct phrase `that this`."
        ),
        "TheAnother" => (
            ["the another"],
            ["the other", "another"],
            "Use `the other` or `another`, not both.",
            "Corrects `the another`."
        ),
        "ThereIsAny" => (
            ["there any"],
            ["there is any"],
            "Insert `is` for correct grammar.",
            "Replaces `there any` with `there is any`."
        ),
        "ThoughtProcess" => (
            ["though process"],
            ["thought process"],
            "Did you mean `thought process`?",
            "Changes `though process` to `thought process`."
        ),
        "TickingTimeClock" => (
            ["ticking time clock"],
            ["ticking time bomb", "ticking clock"],
            "Use `ticking time bomb` for disastrous consequences, otherwise avoid redundancy with just `ticking clock`.",
            "Corrects `ticking time clock` to `ticking time bomb` for idiomatic urgency or `ticking clock` otherwise."
        ),
        "ToDoHyphen" => (
            ["todo"],
            ["to-do"],
            "Hyphenate `to-do`.",
            "Ensures `to-do` is correctly hyphenated."
        ),
        "ToTheMannerBorn" => (
            ["to the manor born"],
            ["to the manner born"],
            "Use the correct phrase for being naturally suited to something.",
            "Corrects `to the manor born` to `to the manner born`, ensuring the intended meaning of being naturally suited to a way of life."
        ),
        "Towards" => (
            ["to towards"],
            ["towards"],
            "Use `towards` without the preceding `to`.",
            "Removes redundant `to` before `towards`."
        ),
        "TrialAndError" => (
            ["trail and error"],
            ["trial and error"],
            "You misspelled `trial`.",
            "Corrects `trail` to `trial` in `trial and error`."
        ),
        "TurnForTheWorse" => (
            ["turn for the worst"],
            ["turn for the worse"],
            "Use `turn for the worse` for a negative change in circumstances. Avoid the incorrect `turn for the worst`.",
            "Corrects the nonstandard `turn for the worst` to the idiomatic `turn for the worse`, used to describe a situation that has deteriorated."
        ),
        "TurnItOff" => (
            ["turn it of", "turn i of"],
            ["turn it off"],
            "Did you mean `turn it off`?",
            "Fixes the mistake in the phrase `turn it off`."
        ),
        "Unless" => (
            ["unless if"],
            ["unless", "except if", "except when"],
            "`Unless if` is not idiomatic English. `Unless`, `except if`, and `except when` express a condition that is true in all cases except one.",
            "Corrects `unless if`."
        ),
        "WantBe" => (
            ["want be"],
            ["won't be", "want to be"],
            "Did you mean `won't be` or `want to be`?",
            "Detects incorrect usage of `want be` and suggests `won't be` or `want to be` based on context."
        ),
        "WaveFunction" => (
            ["wavefunction"],
            ["wave function"],
            "Did you mean `wave function`?",
            "Identifies the mistake of merging `wave` and `function` into one word. In quantum mechanics, a `wave function` (written as two words) describes the mathematical function that represents the quantum state of a particle or system. Correct usage is crucial for clear and accurate scientific communication."
        ),
        "WellBeing" => (
            ["wellbeing"],
            ["well-being"],
            "Use the hyphenated form for `well-being`.",
            "Ensures `well-being` is correctly hyphenated."
        ),
        "WellKept" => (
            ["highly-kept", "highly kept"],
            // There may be other good alternatives such as closely-guarded or tightly-held
            ["well-kept"],
            "`Highly-kept` is not standard. To describe secrets, `well-kept` is the most used phrase.",
            "Flags `highly-kept` and recommends `well-kept` as an alternative."
        ),
        "WhatHeLooksLike" => (
            ["how he looks like"],
            ["how he looks", "what he looks like"],
            "Don't use both `how` and `like` together to express similarity.",
            "Corrects `how ... looks like` to `how ... looks` or `what ... looks like`."
        ),
        "WhatItLooksLike" => (
            ["how it looks like", "how it look like", "how it look's like"],
            ["how it looks", "what it looks like"],
            "Don't use both `how` and `like` together to express similarity.",
            "Corrects `how ... looks like` to `how ... looks` or `what ... looks like`."
        ),
        "WhatSheLooksLike" => (
            ["how she looks like"],
            ["how she looks", "what she looks like"],
            "Don't use both `how` and `like` together to express similarity.",
            "Corrects `how ... looks like` to `how ... looks` or `what ... looks like`."
        ),
        "WhatTheyLookLike" => (
            ["how they look like", "how they looks like"],
            ["how they look", "what they look like"],
            "Don't use both `how` and `like` together to express similarity.",
            "Corrects `how ... look like` to `how ... look` or `what ... look like`."
        ),
        "WhetYourAppetite" => (
            ["wet your appetite"],
            ["whet your appetite"],
            "Use the correct phrase for stimulating desire.",
            "Ensures `whet your appetite` is used correctly, distinguishing it from the incorrect `wet` variation."
        ),
        "WholeEntire" => (
            ["whole entire"],
            ["whole", "entire"],
            "Avoid redundancy. Use either `whole` or `entire` for referring to the complete amount or extent.",
            "Corrects the redundancy in `whole entire` to `whole` or `entire`."
        ),
        "WillContain" => (
            ["will contains"],
            ["will contain"],
            "Did you mean `will contain`?",
            "Incorrect verb form: `will` should be followed by the base form `contain`."
        ),
        "WorseAndWorse" => (
            ["worst and worst", "worse and worst", "worst and worse"],
            ["worse and worse"],
            "Use `worse` for comparing. (`Worst` is for the extreme case)",
            "Corrects `worst and worst` to `worse and worse` for proper comparative usage."
        ),
        "WorseCaseScenario" => (
            ["worse case scenario", "worse-case scenario", "worse-case-scenario"],
            ["worst-case scenario"],
            "Use `worst` for referring to the worst possible scenario. (`Worse` is for comparing)",
            "Corrects `worst-case scenario` when the hyphen is missing or `worse` is used instead of `worst`."
        ),
        "WorseThan" => (
            ["worst than"],
            ["worse than"],
            "Use `worse` for comparing. (`Worst` is for the extreme case)",
            "Corrects `worst than` to `worse than` for proper comparative usage."
        ),
        "WorstCaseScenario" => (
            ["worst case scenario", "worst-case-scenario"],
            ["worst-case scenario"],
            "Hyphenate `worst-case`.",
            "Corrects `worst-case scenario` when the hyphen is missing or `worse` is used instead of `worst`."
        ),
        "WorstEver" => (
            ["worse ever"],
            ["worst ever"],
            "Use `worst` for the extreme case. (`Worse` is for comparing)",
            "Corrects `worse ever` to `worst ever` for proper comparative usage."
        ),
        "WroughtIron" => (
            ["rod iron", "rot iron", "rod-iron", "rot-iron"],
            ["wrought iron"],
            "Prefer the standard term `wrought iron`.",
            "`Wrought iron` is low-carbon, malleable iron used for decorative work; variants like `rod iron` or `rot iron` are phonetic misspellings that may confuse readers."
        ),
    });

    group.set_all_rules_to(Some(true));

    group
}

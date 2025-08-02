use crate::linting::LintKind;

use super::{LintGroup, MapPhraseLinter};

#[cfg(test)]
mod tests;

/// Produce a [`LintGroup`] that looks for errors in common phrases.
/// Comes pre-configured with the recommended default settings.
pub fn lint_group() -> LintGroup {
    let mut group = LintGroup::default();

    macro_rules! add_exact_mappings {
        ($group:expr, {
            $($name:expr => ($input:expr, $corrections:expr, $hint:expr, $description:expr $(, $lint_kind:expr)?)),+ $(,)?
        }) => {
            $(
                $group.add_expr_linter(
                    $name,
                    Box::new(
                        MapPhraseLinter::new_fixed_phrases(
                            $input,
                            $corrections,
                            $hint,
                            $description,
                            None$(.or(Some($lint_kind)))?,
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
            "Corrects `a couple of more` to `a couple more`.",
            LintKind::Redundancy
        ),
        "AfterAWhile" => (
            ["after while"],
            ["after a while"],
            "When describing a timeframe, use `a while`.",
            "Corrects the missing article in `after while`, forming `after a while`.",
            LintKind::Grammar
        ),
        "AheadAnd" => (
            ["ahead an"],
            ["ahead and"],
            "Did you make a typo? Shouldn't it be `and`?",
            "Corrects `an` to `and` after `ahead`."
        ),
        "AllOfASudden" => (
            ["all of the sudden"],
            ["all of a sudden"],
            "The phrase is `all of a sudden`, meaning `unexpectedly`.",
            "Corrects `all of the sudden` to `all of a sudden`.",
            LintKind::Nonstandard
        ),
        "ALongTime" => (
            ["along time"],
            ["a long time"],
            "Use `a long time` for referring to a duration of time.",
            "Corrects `along time` to `a long time`.",
            LintKind::Grammar
        ),
        "AlzheimersDisease" => (
            ["old-timers' disease"],
            ["Alzheimer’s disease"],
            "Use the correct medical term.",
            "Fixes the common misnomer `old-timers' disease`, ensuring the correct medical term `Alzheimer’s disease` is used.",
            LintKind::Eggcorn
        ),
        "AnAnother" => (
            ["an another", "a another"],
            ["another"],
            "Use `another` on its own.",
            "Corrects `an another` and `a another`.",
            LintKind::Redundancy
        ),
        "AndIn" => (
            ["an in"],
            ["and in"],
            "Did you mean `and in`?",
            "Fixes the incorrect phrase `an in` to `and in` for proper conjunction usage.",
            LintKind::Typo
        ),
        "AndTheLike" => (
            ["an the like"],
            ["and the like"],
            "Did you mean `and the like`?",
            "Fixes the typo in `and the like`.",
            LintKind::Typo
        ),
        "AnotherAn" => (
            ["another an"],
            ["another"],
            "Use `another` on its own.",
            "Corrects `another an` to `another`.",
            LintKind::Redundancy
        ),
        "AnotherOnes" => (
            ["another ones"],
            ["another one", "another one's", "other ones"],
            "`another` is singular but `ones` is plural. Or maybe you meant the possessive `one's`.",
            "Corrects `another ones`.",
            LintKind::Agreement
        ),
        "AnotherThings" => (
            ["another things"],
            ["another thing", "other things"],
            "`another` is singular but `things` is plural.",
            "Corrects `another things`.",
            LintKind::Agreement
        ),
        "AsFarBackAs" => (
            ["as early back as"],
            ["as far back as"],
            "Use `as far back as` for referring to a time in the past.",
            "Corrects nonstandard `as early back as` to `as far back as`.",
            LintKind::WordChoice
        ),
        "AsItHappens" => (
            ["as it so happens"],
            ["as it happens"],
            "Did you mean `as it happens`?",
            "Corrects `as it so happens` to `as it happens`.",
            LintKind::Usage
        ),
        "AsOfLate" => (
            ["as of lately"],
            ["as of late"],
            "The standard form is `as of late`.",
            "Corrects `as of lately` to `as of late`.",
            LintKind::WordChoice
        ),
        "AsWell" => (
            ["aswell"],
            ["as well"],
            "`as well` should be written as two words.",
            "Corrects `aswell` to `as well`.",
            LintKind::BoundaryError
        ),
        "AtFaceValue" => (
            ["on face value"],
            ["at face value"],
            "`at face value is more idiomatic and more common.",
            "Corrects `on face value` to the more usual `at face value`.",
            LintKind::WordChoice
        ),
        "AtTheEndOfTheDay" => (
            ["in the end of the day"],
            ["at the end of the day"],
            "Did you mean `at the end of the day`?",
            "Corrects `in the end of the day` to `at the end of the day`.",
            LintKind::WordChoice
        ),
        "AvoidAndAlso" => (
            ["and also"],
            ["and"],
            "Consider using just `and`.",
            "Reduces redundancy by replacing `and also` with `and`.",
            LintKind::Redundancy
        ),
        "BadRap" => (
            ["bed rap", "bad rep"],
            ["bad rap"],
            "Did you mean `bad rap`?",
            "Changes `bed rap` to the proper idiom `bad rap`.",
            LintKind::Eggcorn
        ),
        "BanTogether" => (
            ["ban together"],
            ["band together"],
            "Did you mean `band together`?",
            "Detects and corrects the common error of using `ban together` instead of the idiom `band together`, which means to unite or join forces.",
            LintKind::Eggcorn
        ),
        "BareInMind" => (
            ["bare in mind"],
            ["bear in mind"],
            "Did you mean `bear in mind`?",
            "Ensures the phrase `bear in mind` is used correctly instead of `bare in mind`.",
            LintKind::Eggcorn
        ),
        "BatedBreath" => (
            ["baited breath"],
            ["bated breath"],
            "Did you mean `bated breath`?",
            "Changes `baited breath` to the correct `bated breath`.",
            LintKind::Eggcorn
        ),
        "BeckAndCall" => (
            ["back and call"],
            ["beck and call"],
            "Did you mean `beck and call`?",
            "Fixes `back and call` to `beck and call`.",
            LintKind::Eggcorn
        ),
        "BeenThere" => (
            ["bee there"],
            ["been there"],
            "Did you mean `been there`?",
            "Corrects the misspelling `bee there` to the proper phrase `been there`.",
            LintKind::Typo
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
            "In valedictions, `best` expresses your highest regard—avoid the typo `beat regards`.",
            LintKind::Typo
        ),
        "BlanketStatement" => (
            ["blanketed statement"],
            ["blanket statement"],
            "Use the more idiomatic phrasing.",
            "Corrects common errors in the phrase `blanket statement`.",
            LintKind::Usage
        ),
        "Brutality" => (
            ["brutalness"],
            ["brutality"],
            "This word has a more standard, more common synonym.",
            "Suggests the more standard and common synonym `brutality`.",
            LintKind::WordChoice
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
            "Incorrect preposition: `by accident` is the idiomatic expression.",
            LintKind::Usage
        ),
        "CanBeSeen" => (
            ["can be seem"],
            ["can be seen"],
            "Did you mean `can be seen`?",
            "Corrects `can be seem` to the proper phrase `can be seen`.",
            LintKind::Typo
        ),
        "CaseInPoint" => (
            ["case and point"],
            ["case in point"],
            "`Case in point` is the correct form of the phrase.",
            "Corrects `case and point` to `case in point`.",
            LintKind::Malapropism
        ),
        "CaseSensitive" => (
            ["case sensitive"],
            ["case-sensitive"],
            "Use the hyphenated form for `case-sensitive`.",
            "Ensures `case-sensitive` is correctly hyphenated.",
            LintKind::Punctuation
        ),
        "CondenseAllThe" => (
            ["all of the"],
            ["all the"],
            "Consider simplifying to `all the`.",
            "Suggests removing `of` in `all of the` for a more concise phrase.",
            LintKind::Redundancy
        ),
        "CoursingThroughVeins" => (
            ["cursing through veins"],
            ["coursing through veins"],
            "In this idiom, blood “courses” (flows) through veins, not “curses”.",
            "In English idioms, “to course” means to flow rapidly—so avoid the eggcorn `cursing through veins.`",
            LintKind::Eggcorn
        ),
        "DampSquib" => (
            ["damp squid"],
            ["damp squib"],
            "Use the correct phrase for a disappointing outcome.",
            "Corrects the eggcorn `damp squid` to `damp squib`, ensuring the intended meaning of a failed or underwhelming outcome.",
            LintKind::Eggcorn
        ),
        "DayAndAge" => (
            ["day in age"],
            ["day and age"],
            "Use `day and age` for referring to the present time.",
            "Corrects the eggcorn `day in age` to `day and age`, which properly means the current era or time period.",
            LintKind::Eggcorn
        ),
        "DoNotWant" => (
            ["don't wan", "do not wan"],
            ["don't want", "do not want"],
            "Use the full verb “want” after negation: “don't want” or “do not want.”",
            "In English, negation still requires the complete verb form (“want”), so avoid truncating it to “wan.”",
            LintKind::Typo
        ),
        "EachAndEveryOne" => (
            ["each and everyone"],
            ["each and every one"],
            "Use `each and every one` for referring to a group of people or things.",
            "Corrects `each and everyone` to `each and every one`.",
            LintKind::BoundaryError
        ),
        "EludedTo" => (
            ["eluded to"],
            ["alluded to"],
            "Did you mean `alluded to`?",
            "Corrects `eluded to` to `alluded to` in contexts referring to indirect references.",
            LintKind::Malapropism
        ),
        "EnMasse" => (
            ["on mass", "on masse", "in mass"],
            ["en masse"],
            "Did you mean `en masse`?",
            "Detects variants like `on mass` or `in mass` and suggests `en masse`.",
            LintKind::Eggcorn
        ),
        "EnRoute" => (
            ["on route to", "in route to", "on-route to", "in-route to"],
            ["en route to", "en-route to"],
            "Did you mean `en route`?",
            "Detects variants like `on route` or `in route` and suggests `en route`.",
            LintKind::Eggcorn
        ),
        "EverPresent" => (
            ["ever present"],
            ["ever-present"],
            "Hyphenate `ever-present` when it functions as a compound adjective.",
            "Corrects the missing hyphen in `ever present` to the compound adjective `ever-present`.",
            LintKind::Punctuation
        ),
        "EveryTime" => (
            ["everytime"],
            ["every time"],
            "`Everytime` as a single word is proscribed. Use `every time` instead.",
            "Corrects `everytime` to `every time`.",
            LintKind::Usage
        ),
        "Excellent" => (
            ["very good"],
            ["excellent"],
            "Vocabulary enhancement: use `excellent` instead of `very good`",
            "Provides a stronger word choice by replacing `very good` with `excellent` for clarity and emphasis.",
            LintKind::Enhancement
        ),
        "ExpandBecause" => (
            ["cuz"],
            ["because"],
            "Use `because` instead of informal `cuz`",
            "Expands the informal abbreviation `cuz` to the full word `because` for formality.",
            LintKind::Style
        ),
        "ExpandMinimum" => (
            ["min"],
            ["minimum"],
            "Use `minimum` instead of `min`",
            "Expands the abbreviation `min` to the full word `minimum` for clarity.",
            LintKind::Style
        ),
        "ExpandWith" => (
            ["w/"],
            ["with"],
            "Use `with` instead of `w/`",
            "Expands the abbreviation `w/` to the full word `with` for clarity.",
            LintKind::Style
        ),
        "ExpandWithout" => (
            ["w/o"],
            ["without"],
            "Use `without` instead of `w/o`",
            "Expands the abbreviation `w/o` to the full word `without` for clarity.",
            LintKind::Style
        ),
        "Expatriate" => (
            ["ex-patriot"],
            ["expatriate"],
            "Use the correct term for someone living abroad.",
            "Fixes the misinterpretation of `expatriate`, ensuring the correct term is used for individuals residing abroad.",
            LintKind::Eggcorn
        ),
        "FaceFirst" => (
            ["face first into"],
            ["face-first into"],
            "Should this be `face-first`?",
            "Ensures `face first` is correctly hyphenated as `face-first` when used before `into`.",
            LintKind::Punctuation
        ),
        "FairBit" => (
            ["fare bit"],
            ["fair bit"],
            "A `decent amount` is a `fair bit`. `Fare` is the price of a ticket.",
            "Corrects malapropisms of `a fair bit`.",
            LintKind::Eggcorn
        ),
        "FastPaste" => (
            ["fast paste", "fast-paste"],
            ["fast-paced"],
            "Did you mean `fast-paced`?",
            "Detects incorrect usage of `fast paste` or `fast-paste` and suggests `fast-paced` as the correct phrase.",
            LintKind::Eggcorn
        ),
        "FatalOutcome" => (
            ["fatal outcome"],
            ["death"],
            "Consider using `death` for clarity.",
            "Replaces `fatal outcome` with the more direct term `death` for conciseness.",
            LintKind::Style
        ),
        "FetalPosition" => (
            ["the feeble position"],
            ["the fetal position"],
            "Use the correct term for a curled-up posture.",
            "Ensures the correct use of `fetal position`, avoiding confusion with `feeble position`, which is not a standard phrase.",
            LintKind::Malapropism
        ),
        "ForAllIntentsAndPurposes" => (
            ["for all intensive purposes"],
            ["for all intents and purposes"],
            "Use the correct phrase meaning 'in every practical sense'.",
            "Corrects `for all intensive purposes` to `for all intents and purposes`, ensuring the phrase conveys its intended meaning.",
            LintKind::Eggcorn
        ),
        "ForALongTime" => (
            ["for along time"],
            ["for a long time"],
            "Use the standard phrase `for a long time` to indicate an extended duration.",
            "Eliminates the incorrect merging in `for along time`.",
            LintKind::Grammar
        ),
        "ForAWhile" => (
            ["for while"],
            ["for a while"],
            "When describing a timeframe, use `a while`.",
            "Corrects the missing article in `for while`, forming `for a while`.",
            LintKind::Typo
        ),
        "FreeRein" => (
            ["free reign"],
            ["free rein"],
            "Use the correct phrase for unrestricted control.",
            "Ensures the correct use of `free rein`, avoiding confusion with `free reign`, which incorrectly suggests authority rather than freedom of action.",
            LintKind::Eggcorn
        ),
        "Freezing" => (
            ["very cold", "really cold", "extremely cold"],
            ["freezing"],
            "A more vivid adjective would better capture extreme cold.",
            "Encourages vivid writing by suggesting `freezing` instead of weaker expressions like `very cold.`",
            LintKind::Enhancement
        ),
        "FromTheGetGo" => (
            ["from the get go"],
            ["from the get-go"],
            "Use the hyphenated form: `from the get-go`.",
            "Ensures `from the get-go` is correctly hyphenated, preserving the idiom’s meaning of ‘from the very beginning’.",
            LintKind::Punctuation
        ),
        "GildedAge" => (
            ["guilded age"],
            ["Gilded Age"],
            "The period of economic prosperity is called the `Gilded Age`.",
            "If referring to the period of economic prosperity, the correct term is `Gilded Age`.",
            LintKind::Eggcorn
        ),
        "GoingTo" => (
            ["gong to"],
            ["going to"],
            "Did you mean `going to`?",
            "Corrects `gong to` to the intended phrase `going to`.",
            LintKind::Typo
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
            "Checks for the correct official name of the African country.",
            LintKind::Punctuation
        ),
        "HadOf" => (
            ["had of"],
            ["had have", "had've"],
            "Did you mean `had have` or `had've`?",
            "Flags the unnecessary use of `of` after `had` and suggests the correct forms.",
            LintKind::Grammar
        ),
        "HalfAnHour" => (
            ["half an our"],
            ["half an hour"],
            "Remember the silent 'h' when writing `hour`: `half an hour`.",
            "Fixes the eggcorn `half an our` to the accepted `half an hour`.",
            LintKind::Typo
        ),
        "Haphazard" => (
            ["half hazard", "half-hazard", "halfhazard"],
            ["haphazard"],
            "Use `haphazard` for randomness or lack of organization.",
            "Corrects the eggcorn `half hazard` to `haphazard`, which properly means lacking organization or being random.",
            LintKind::Eggcorn
        ),
        "HumanBeings" => (
            ["human's beings", "humans beings"],
            ["human beings"],
            "Use `human beings` to refer to people collectively.",
            "Eliminates the incorrect possessive/plural usage like `human's beings` or `humans beings`.",
            LintKind::Grammar
        ),
        "HumanLife" => (
            ["human live"],
            ["human life"],
            "Did you mean `human life`?",
            "Changes `human live` to `human life`.",
            LintKind::Typo
        ),
        "HungerPang" => (
            ["hunger pain"],
            ["hunger pang"],
            "Did you mean `hunger pang`?",
            "Corrects `hunger pain` to `hunger pang`.",
            LintKind::Eggcorn
        ),
        "IAm" => (
            ["I a m"],
            ["I am"],
            "Did you mean `I am`?",
            "Fixes the incorrect spacing in `I a m` to properly form `I am`.",
            LintKind::Typo
        ),
        "IDo" => (
            ["I does"],
            ["I do"],
            "`I` pairs with the bare verb `do`; the –s inflection `does` is reserved for third-person singular subjects.",
            "Corrects `I does` to `I do`.",
            LintKind::Agreement
        ),
        "InOfItself" => (
            ["in of itself"],
            ["in itself", "in and of itself"],
            "Use `in itself` (more common) or `in and of itself` (more formal) to mean 'intrinsically'.",
            "Corrects nonstandard `in of itself` to standard `in itself` or `in and of itself`.",
            LintKind::Usage
        ),
        "InAnyWay" => (
            ["in anyway"],
            ["in any way"],
            "Use `in any way` for emphasizing a point.",
            "Corrects ungrammatical `in anyway` to `in any way`.",
            LintKind::BoundaryError
        ),
        "InAWhile" => (
            ["in while"],
            ["in a while"],
            "When describing a timeframe, use `a while`.",
            "Corrects the missing article in `in while`, forming `in a while`.",
            LintKind::Grammar
        ),
        "InCase" => (
            ["incase"],
            ["in case"],
            "`In case` should be written as two words.",
            "Corrects `incase` to `in case`.",
            LintKind::BoundaryError
        ),
        "InNeedOf" => (
            ["in need for"],
            ["in need of"],
            "Use `in need of` for when something is required or necessary.",
            "Corrects `in need for` to `in need of`.",
            LintKind::Usage
        ),
        "InOneFellSwoop" => (
            ["in one foul swoop"],
            ["in one fell swoop"],
            "Use the correct phrase for something happening suddenly.",
            "Corrects `in one foul swoop` to `in one fell swoop`, preserving the phrase’s original meaning of sudden and complete action.",
            LintKind::Eggcorn
        ),
        "InsteadOf" => (
            ["in stead of"],
            ["instead of"],
            "Use the modern single word `instead of` to indicate a replacement.",
            "Corrects the archaic or mistaken separation `in stead of` to `instead of` in everyday usage.",
            LintKind::BoundaryError
        ),
        "Insensitive" => (
            ["unsensitive"],
            ["insensitive"],
            "This word has a more standard, more common synonym.",
            "Suggests the more standard and common synonym `insensitive`.",
            LintKind::WordChoice
        ),
        "Insurmountable" => (
            ["unsurmountable"],
            ["insurmountable"],
            "This word has a more standard, more common synonym.",
            "Suggests the more standard and common synonym `insurmountable`.",
            LintKind::WordChoice
        ),
        "Intact" => (
            ["in tact"],
            ["intact"],
            "Use `intact` to mean undamaged or whole.",
            "Prevents the erroneous spacing in `in tact`; `intact` is the single correct word.",
            LintKind::BoundaryError
        ),
        "InThe" => (
            ["int he"],
            ["in the"],
            "Did you mean `in the`?",
            "Detects and corrects a spacing error where `in the` is mistakenly written as `int he`. Proper spacing is essential for readability and grammatical correctness in common phrases.",
            LintKind::Typo
        ),
        "IsKnownFor" => (
            ["is know for"],
            ["is known for"],
            "Did you mean `is known for`?",
            "Typo: `known` is the correct past participle.",
            LintKind::Typo
        ),
        "ItCan" => (
            ["It cam"],
            ["It can"],
            "Did you mean `It can`?",
            "Corrects the misspelling `It cam` to the proper phrase `It can`.",
            LintKind::Typo
        ),
        "IveGotTo" => (
            ["I've go to"],
            ["I've got to"],
            "Use `I've got to` for necessity or obligation.",
            "Corrects the slip `I've go to` to the idiomatic `I've got to`.",
            LintKind::Typo
        ),
        "JawDropping" => (
            ["jar-dropping"],
            ["jaw-dropping"],
            "Use the correct phrase for something astonishing.",
            "Corrects `jar-dropping` to `jaw-dropping`, ensuring the intended meaning of something that causes amazement.",
            LintKind::Eggcorn
        ),
        "JustDeserts" => (
            ["just desserts"],
            ["just deserts"],
            "Use the correct phrase for receiving what one deserves.",
            "Ensures `just deserts` is used correctly, preserving its meaning of receiving an appropriate outcome for one's actions.",
            LintKind::Spelling
        ),
        "KindOf" => (
            ["kinda of"],
            ["kind of", "kinda"],
            "`Kinda` already means `kind of`, so `kinda of` is redundant.",
            "Corrects `kinda of` to `kind of`.",
            LintKind::Redundancy
        ),
        "KindRegards" => (
            ["kid regards"],
            ["kind regards"],
            "Did you mean `kind regards`?",
            "Changes `kid regards` to `kind regards`.",
            LintKind::Typo
        ),
        "LastButNotLeast" => (
            ["last but not the least", "last, but not the least", "last but, not least", "last but not last"],
            ["last but not least"],
            "Use the more idiomatic phrasing.",
            "Corrects common errors in the phrase `last but not least`.",
            LintKind::Usage
        ),
        "LastDitch" => (
            ["last ditch", "last ditched", "last-ditched"],
            ["last-ditch"],
            "In this idiom, `ditch` is a noun and a hyphen is needed.",
            "Corrects wrong variations of the idiomatic adjective `last-ditch`.",
            LintKind::Usage
        ),
        "LetAlone" => (
            ["let along"],
            ["let alone"],
            "Did you mean `let alone`?",
            "Changes `let along` to `let alone`.",
            LintKind::Typo
        ),
        "LikeAsIf" => (
            ["like as if"],
            ["like", "as if"],
            "Avoid redundancy. Use either `like` or `as if`.",
            "Corrects redundant `like as if` to `like` or `as if`."
        ),
        "LikeThePlague" => (
            ["like a plague"],
            ["like the plague"],
            "`Things are avoided `like the plague` not `like a plague`.",
            "Corrects `like a plague` to `like the plague`.",
            LintKind::Usage
        ),
        "LowHangingFruit" => (
            ["low hanging fruit", "low hanging fruits", "low-hanging fruits"],
            ["low-hanging fruit"],
            "The standard form is `low-hanging fruit` with a hyphen and singular form.",
            "Corrects non-standard variants of `low-hanging fruit`.",
            LintKind::Usage
        ),
        "Monumentous" => (
            ["monumentous"],
            ["momentous", "monumental"],
            "Retain `monumentous` for jocular effect. Otherwise `momentous` indicates great signifcance while `monumental` indicates imposing size.",
            "Advises using `momentous` or `monumental` instead of `monumentous` for serious usage.",
            LintKind::Nonstandard
        ),
        "MorePreferable" => (
            ["more preferable"],
            ["preferable"],
            "Use just `preferable` instead of `more preferable`.",
            "Corrects `more preferable` to `preferable`.",
            LintKind::Redundancy
        ),
        "MyHouse" => (
            ["mu house"],
            ["my house"],
            "Did you mean `my house`?",
            "Fixes the typo `mu house` to `my house`.",
            LintKind::Typo
        ),
        "NeedHelp" => (
            ["ned help"],
            ["need help"],
            "Did you mean `need help`?",
            "Changes `ned help` to the correct `need help`.",
            LintKind::Typo
        ),
        "NerveRacking" => (
            ["nerve racking", "nerve wracking", "nerve wrecking", "nerve-wracking", "nerve-wrecking"],
            ["nerve-racking"],
            "Use `nerve-racking` for something that causes anxiety or tension.",
            "Corrects common misspellings and missing hyphen in `nerve-racking`.",
            LintKind::Eggcorn
        ),
        "NotIn" => (
            ["no in"],
            ["not in"],
            "Use `not in` for correct grammar.",
            "Replaces `no in` with `not in`.",
            LintKind::Typo
        ),
        "NotTo" => (
            ["no to"],
            ["not to"],
            "Did you mean `not to`?",
            "Corrects `no to` to `not to`, ensuring proper negation.",
            LintKind::Typo
        ),
        "OfCourse" => (
            // See also: `of_course.rs` for "of curse/corse" → "of course" corrections
            ["off course", "o course", "ofcourse"],
            ["of course"],
            "Did you mean `of course`?",
            "Detects the common mistake `off course` and suggests the correct form `of course`.",
            LintKind::Eggcorn
        ),
        "OffTheCuff" => (
            ["off the cuff"],
            ["off-the-cuff"],
            "Use the hyphenated form for `off-the-cuff`.",
            "Ensures `off-the-cuff` is correctly hyphenated.",
            LintKind::Punctuation
        ),
        "OldWivesTale" => (
            ["old wise tale"],
            ["old wives' tale"],
            "Use the correct phrase for a superstition or myth.",
            "Corrects `old wise tale` to `old wives' tale`, preserving the phrase’s meaning as an unfounded traditional belief.",
            LintKind::Eggcorn
        ),
        "OnceInAWhile" => (
            ["once a while", "once and a while"],
            ["once in a while"],
            "The correct idiom is `once in a while`.",
            "Corrects two common malapropisms of `once in a while`.",
            LintKind::Usage
        ),
        "OnSecondThought" => (
            ["on second though"],
            ["on second thought"],
            "Idiomatic expression: use `on second thought` instead of `on second though`",
            "Replaces the nonstandard `on second though` with the common idiom `on second thought` to indicate reconsideration.",
            LintKind::Typo
        ),
        "OnTheSpurOfTheMoment" => (
            ["on the spurt of the moment", "at the spur of the moment", "in the spur of the moment"],
            ["on the spur of the moment"],
            "Use the correct phrase for acting spontaneously.",
            "Ensures the correct use of `on the spur of the moment`, avoiding nonstandard variations.",
            // "spurt" is Eggcorn, the others are probably Usage
            LintKind::Eggcorn
        ),
        "OnTopOf" => (
            ["ontop of"],
            ["on top of"],
            "Did you mean `on top of`?",
            "Corrects `ontop of` to `on top of`.",
            LintKind::BoundaryError
        ),
        "PeaceOfMind" => (
            ["piece of mind"],
            ["peace of mind"],
            "The phrase is `peace of mind`, meaning `calm`. A `piece` is a `part` of something.",
            "Corrects `piece of mind` to `peace of mind`.",
            LintKind::Eggcorn
        ),
        "PointsOfView" => (
            ["point of views"],
            ["points of view"],
            "The correct plural is `points of view`.",
            "Corrects pluralizing the wrong noun in `point of view`.",
            LintKind::Usage
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
            "Checks for the correct official name of the capital of Haiti.",
            LintKind::Punctuation
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
            "Checks for the correct official name of the capital of Benin.",
            LintKind::Punctuation
        ),
        "PrayingMantis" => (
            ["preying mantis"],
            ["praying mantis"],
            "Use the insect's correct name.",
            "Corrects `preying mantis` to `praying mantis`, ensuring accurate reference to the insect’s characteristic pose.",
            LintKind::Eggcorn
        ),
        "RapidFire" => (
            ["rapid fire"],
            ["rapid-fire"],
            "It is more idiomatic to hypenate `rapid-fire`.",
            "Checks to ensure writers hyphenate `rapid-fire`.",
            LintKind::Punctuation
        ),
        "RealTrouper" => (
            ["real trooper"],
            ["real trouper"],
            "Use the correct phrase for someone who perseveres.",
            "Ensures the correct use of `real trouper`, distinguishing it from `trooper`, which refers to a soldier or police officer.",
            LintKind::Eggcorn
        ),
        "RedundantIIRC" => (
            ["if IIRC", "IIRC correctly"], ["IIRC"],
            "`IIRC` already means 'if I recall correctly', so adding 'if' or 'correctly' is redundant.",
            "Flags redundant use of 'if' or 'correctly' with 'IIRC', since 'IIRC' already stands for 'if I recall correctly'.",
            LintKind::Redundancy
        ),
        "RifeWith" => (
            ["ripe with"],
            ["rife with"],
            "Use the correct phrase for something abundant.",
            "Corrects `ripe with` to `rife with`, preserving the phrase’s meaning of being filled with something, often undesirable.",
            LintKind::Eggcorn
        ),
        "RoadMap" => (
            ["roadmap"],
            ["road map"],
            "Did you mean `road map`?",
            "Detects when `roadmap` is used instead of `road map`, prompting the correct spacing.",
            LintKind::WordChoice
        ),
        "SameAs" => (
            ["same then"],
            ["same as"],
            "Did you mean `same as`?",
            "Corrects the incorrect phrase `same then` to the standard `same as`.",
            LintKind::Grammar
        ),
        "ScantilyClad" => (
            ["scandally clad"],
            ["scantily clad"],
            "Use the correct phrase for minimal attire.",
            "Fixes `scandally clad` to `scantily clad`, ensuring clarity in describing minimal attire.",
            LintKind::Eggcorn
        ),
        "SimpleGrammatical" => (
            ["simply grammatical"],
            ["simple grammatical"],
            "Use `simple grammatical` for correct adjective usage.",
            "Corrects `simply grammatical` to `simple grammatical` for proper adjective usage.",
            LintKind::Usage
        ),
        "SneakingSuspicion" => (
            ["sneaky suspicion"],
            ["sneaking suspicion"],
            "Did you mean `sneaking suspicion`?",
            "Changes `sneaky suspicion` to `sneaking suspicion`.",
            LintKind::Eggcorn
        ),
        "SomebodyElses" => (
            ["somebody's else", "somebody's else's"],
            ["somebody else's"],
            "This should be `somebody else's`?",
            "Corrects `somebody else's` when the `'s` is in the wrong place.",
            LintKind::Grammar
        ),
        "SomeOfThe" => (
            ["some the"],
            ["some of the"],
            "Add `of` to form the partitive phrase: `some of the`.",
            "Quantity words such as `some` normally take `of` before a definite article. Including `of` signals that you mean a subset of a larger set, preventing a momentary stumble in comprehension.",
            LintKind::Typo
        ),
        "SoonerOrLater" => (
            ["sooner than later"],
            ["sooner rather than later", "sooner or later"],
            "Did you mean `sooner rather than later` or `sooner or later`?",
            "Fixes the improper phrase `sooner than later` by suggesting standard alternatives.",
            LintKind::Usage
        ),
        "SpecialAttention" => (
            ["spacial attention"],
            ["special attention"],
            "Did you mean `special attention`?",
            "Changes `spacial attention` to `special attention`.",
            LintKind::Typo
        ),
        "SpokeTooSoon" => (
            ["spoke to soon"],
            ["spoke too soon"],
            "Use the adverb `too` instead.",
            "Identifies common misuse of the preposition `to` in the phrase `spoke too soon`.",
            LintKind::Grammar
        ),
        "Starving" => (
            ["very hungry", "really hungry", "extremely hungry"],
            ["starving"],
            "A more vivid adjective would better convey intense hunger.",
            "Encourages vivid writing by suggesting `starving` instead of weaker expressions like `very hungry.`",
            LintKind::Enhancement
        ),
        "StateOfTheArt" => (
            ["state of art"],
            ["state of the art"],
            "Did you mean `state of the art`?",
            "Detects incorrect usage of `state of art` and suggests `state of the art` as the correct phrase.",
            LintKind::Usage
        ),
        "StatuteOfLimitations" => (
            ["statue of limitations"],
            ["statute of limitations"],
            "A `statue` is a sculpture; in legal terms, the correct word is `statute`.",
            "Corrects `statue of limitations` to `statute of limitations`.",
            LintKind::Eggcorn
        ),
        "SufficeItToSay" => (
            ["suffice to say"],
            ["suffice it to say"],
            "`Suffice it to say` is the more standard and more common variant.",
            "Corrects `suffice to say` to `suffice it to say`.",
            LintKind::Usage
        ),
        "SupposedTo" => (
            ["suppose to"],
            ["supposed to"],
            "Did you mean `supposed to`?",
            "Fixes `suppose to` to the correct `supposed to`.",
            LintKind::Usage
        ),
        "TakeItPersonally" => (
            ["take it personal"],
            ["take it personally"],
            "The more standard, less colloquial form is `take it personally`.",
            "Corrects `take it personal` to `take it personally`.",
            LintKind::Usage
        ),
        "ThanksALot" => (
            ["thanks lot", "thanks alot"],
            ["thanks a lot"],
            "Prefer the two-word expression `thanks a lot`.",
            "`Thanks a lot` is the fixed, widely accepted form, while variants like `thanks lot` or `thanks alot` are non-standard and can jar readers.",
            LintKind::Usage
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
            "Fixes `the this` to the correct phrase `that this`.",
            LintKind::Typo
        ),
        "TheAnother" => (
            ["the another"],
            ["the other", "another"],
            "Use `the other` or `another`, not both.",
            "Corrects `the another`.",
            LintKind::Grammar
        ),
        "ThoughtProcess" => (
            ["though process"],
            ["thought process"],
            "Did you mean `thought process`?",
            "Changes `though process` to `thought process`.",
            LintKind::Typo
        ),
        "TickingTimeClock" => (
            ["ticking time clock"],
            ["ticking time bomb", "ticking clock"],
            "Use `ticking time bomb` for disastrous consequences, otherwise avoid redundancy with just `ticking clock`.",
            "Corrects `ticking time clock` to `ticking time bomb` for idiomatic urgency or `ticking clock` otherwise.",
            LintKind::Usage
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
            "Corrects `to the manor born` to `to the manner born`, ensuring the intended meaning of being naturally suited to a way of life.",
            LintKind::Eggcorn
        ),
        "Towards" => (
            ["to towards"],
            ["towards"],
            "Use `towards` without the preceding `to`.",
            "Removes redundant `to` before `towards`.",
            LintKind::Redundancy
        ),
        "TrialAndError" => (
            ["trail and error"],
            ["trial and error"],
            "You misspelled `trial`.",
            "Corrects `trail` to `trial` in `trial and error`.",
            LintKind::Typo
        ),
        "TurnItOff" => (
            ["turn it of", "turn i of"],
            ["turn it off"],
            "Did you mean `turn it off`?",
            "Fixes the mistake in the phrase `turn it off`.",
            LintKind::Typo
        ),
        "Unless" => (
            ["unless if"],
            ["unless", "except if", "except when"],
            "`Unless if` is not idiomatic English. `Unless`, `except if`, and `except when` express a condition that is true in all cases except one.",
            "Corrects `unless if`.",
            LintKind::Usage
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
            "Identifies the mistake of merging `wave` and `function` into one word. In quantum mechanics, a `wave function` (written as two words) describes the mathematical function that represents the quantum state of a particle or system. Correct usage is crucial for clear and accurate scientific communication.",
            LintKind::WordChoice
        ),
        "WellBeing" => (
            ["wellbeing"],
            ["well-being"],
            "Use the hyphenated form for `well-being`.",
            "Ensures `well-being` is correctly hyphenated.",
            LintKind::Punctuation
        ),
        "WellKept" => (
            ["highly-kept", "highly kept"],
            // There may be other good alternatives such as closely-guarded or tightly-held
            ["well-kept"],
            "`Highly-kept` is not standard. To describe secrets, `well-kept` is the most used phrase.",
            "Flags `highly-kept` and recommends `well-kept` as an alternative.",
            LintKind::Usage
        ),
        "WhetYourAppetite" => (
            ["wet your appetite"],
            ["whet your appetite"],
            "Use the correct phrase for stimulating desire.",
            "Ensures `whet your appetite` is used correctly, distinguishing it from the incorrect `wet` variation.",
            LintKind::Eggcorn
        ),
        "WillContain" => (
            ["will contains"],
            ["will contain"],
            "Did you mean `will contain`?",
            "Incorrect verb form: `will` should be followed by the base form `contain`.",
            LintKind::Agreement
        ),
        // WorseOrWorst corrects wrong words. This corrects hyphenation.
        // The descriptions and messages differ between the two linters.
        "WorstCaseScenario" => (
            ["worst case scenario", "worst-case-scenario"],
            ["worst-case scenario"],
            "Hyphenate `worst-case`.",
            "Corrects `worst-case scenario` when the hyphen is missing or `worse` is used instead of `worst`.",
            LintKind::Punctuation
        ),
        "WroughtIron" => (
            ["rod iron", "rot iron", "rod-iron", "rot-iron"],
            ["wrought iron"],
            "Prefer the standard term `wrought iron`.",
            "`Wrought iron` is low-carbon, malleable iron used for decorative work; variants like `rod iron` or `rot iron` are phonetic misspellings that may confuse readers.",
            LintKind::Eggcorn
        ),
        "The" => (
            ["teh"],
            ["the"],
            "Did you mean the definite article?",
            "Fixes especially common misspellings of the word `the`",
            LintKind::Typo
        )
    });

    group.set_all_rules_to(Some(true));

    group
}

use super::LintGroup;
use crate::linting::pooled_linter::for_tests::create_test_pool;
use crate::linting::tests::{
    assert_good_and_bad_suggestions, assert_lint_count, assert_no_lints, assert_suggestion_result,
};

use super::lint_group;

// Use a global pool of lint groups to amortize construction costs.
create_test_pool!(LintGroup, LintGroup, lint_group());

/// Helper function to create a lint group with only a single rule enabled.
fn single_lint(rule_name: &str) -> crate::linting::LintGroup {
    let mut group = lint_group();
    group.set_all_rules_to(None); // Disable all linters
    group.config.set_rule_enabled(rule_name, true); // Enable only the specified rule
    group
}

// AwaitFor

#[test]
fn correct_awaits_for() {
    assert_good_and_bad_suggestions(
        "Headless mode awaits for requested user feedback without showing any text for what that feedback should be",
        test_linter(),
        &[
            "Headless mode awaits requested user feedback without showing any text for what that feedback should be",
            "Headless mode waits for requested user feedback without showing any text for what that feedback should be",
        ],
        &[],
    );
}

#[test]
fn correct_awaiting_for() {
    assert_good_and_bad_suggestions(
        "gpg import fails awaiting for prompt answer",
        test_linter(),
        &[
            "gpg import fails waiting for prompt answer",
            "gpg import fails awaiting prompt answer",
        ],
        &[],
    );
}

#[test]
fn correct_await_for() {
    assert_good_and_bad_suggestions(
        "I still await for a college course on \"Followership 101\"",
        test_linter(),
        &[
            "I still wait for a college course on \"Followership 101\"",
            "I still await a college course on \"Followership 101\"",
        ],
        &[],
    );
}

#[test]
fn correct_awaited_for() {
    assert_good_and_bad_suggestions(
        "I have long awaited for the rise of the Dagoat agenda, and it is glorious.",
        test_linter(),
        &[
            "I have long awaited the rise of the Dagoat agenda, and it is glorious.",
            "I have long waited for the rise of the Dagoat agenda, and it is glorious.",
        ],
        &[],
    );
}

// BackhandedCompliment

#[test]
fn correct_backhand_compliment() {
    assert_suggestion_result(
        "What's the Most insulting /backhand compliment you have ever received?",
        test_linter(),
        "What's the Most insulting /backhanded compliment you have ever received?",
    );
}

#[test]
fn correct_back_hand_compliment_space() {
    assert_suggestion_result(
        "Thankfully, I'm a little young to receive that back hand compliment",
        test_linter(),
        "Thankfully, I'm a little young to receive that backhanded compliment",
    );
}

#[test]
fn correct_back_hand_compliment_hyphen() {
    assert_suggestion_result(
        "They \"have it all\", but still need to back-hand compliment, condescend, \"politely\" hurl passive-aggressive compliments/insults",
        test_linter(),
        "They \"have it all\", but still need to backhanded compliment, condescend, \"politely\" hurl passive-aggressive compliments/insults",
    );
}

#[test]
fn correct_backhand_compliments() {
    assert_suggestion_result(
        "If backhand compliments or flattery are frequent and come with ulterior motives, it's a red flag.",
        test_linter(),
        "If backhanded compliments or flattery are frequent and come with ulterior motives, it's a red flag.",
    );
}

#[test]
fn correct_back_hand_compliments() {
    assert_suggestion_result(
        "I am laughing so hard watching the \"commercial\" with her back hand compliments.",
        test_linter(),
        "I am laughing so hard watching the \"commercial\" with her backhanded compliments.",
    );
}

#[test]
fn correct_back_hand_compliments_caps() {
    assert_suggestion_result(
        "JUST SAY YOU DON'T LIKE THEM STOP GIVING FAKE BACK HAND COMPLIMENTS.",
        test_linter(),
        "JUST SAY YOU DON'T LIKE THEM STOP GIVING FAKE BACKHANDED COMPLIMENTS.",
    );
}

// no praises, just tons of unnecessary back-hand compliments and lack of support
#[test]
fn correct_back_hand_compliments_hyphen() {
    assert_suggestion_result(
        "no praises, just tons of unnecessary back-hand compliments and lack of support",
        test_linter(),
        "no praises, just tons of unnecessary backhanded compliments and lack of support",
    );
}

// CommitmentTo

#[test]
fn singular_towards() {
    assert_suggestion_result(
        "the platform's focus on multimedia projects and VideoLAN's long history of commitment towards free and open multimedia",
        test_linter(),
        "the platform's focus on multimedia projects and VideoLAN's long history of commitment to free and open multimedia",
    );
}

#[test]
fn plural_towards() {
    assert_suggestion_result(
        "the signer may express multiple commitments towards the data objects",
        test_linter(),
        "the signer may express multiple commitments to the data objects",
    );
}

#[test]
fn singular_toward() {
    assert_suggestion_result(
        "This document outlines the current level of commitment toward Linux distributions and packaging formats.",
        test_linter(),
        "This document outlines the current level of commitment to Linux distributions and packaging formats.",
    );
}

#[test]
fn plural_toward() {
    assert_suggestion_result(
        "... and are expected to inform parties in updating their commitments toward the Paris Agreement",
        test_linter(),
        "... and are expected to inform parties in updating their commitments to the Paris Agreement",
    );
}

// Copyright

#[test]
fn copywritten() {
    assert_suggestion_result(
        "Including digital copies of copywritten artwork with the project isn't advised.",
        test_linter(),
        "Including digital copies of copyrighted artwork with the project isn't advised.",
    );
}

#[test]
fn copywrites() {
    assert_suggestion_result(
        "Code is 99% copy/pasted from OpenSSH with an attempt to retain all copywrites",
        test_linter(),
        "Code is 99% copy/pasted from OpenSSH with an attempt to retain all copyrights",
    );
}

#[test]
fn copywrited() {
    assert_suggestion_result(
        "Proprietary copywrited code",
        test_linter(),
        "Proprietary copyrighted code",
    );
}

#[test]
fn copywrited_all_caps() {
    assert_suggestion_result(
        "URLS MAY CONTAIN COPYWRITED MATERIAL",
        test_linter(),
        "URLS MAY CONTAIN COPYRIGHTED MATERIAL",
    );
}

#[test]
fn copywrote() {
    assert_suggestion_result(
        "How do you find out if someone copywrote a movie",
        test_linter(),
        "How do you find out if someone copyrighted a movie",
    );
}

// DateBackFrom

#[test]
fn corrects_date_back_from() {
    assert_good_and_bad_suggestions(
        "There are too many open issues that date back from 4 years ago.",
        test_linter(),
        &[
            "There are too many open issues that date from 4 years ago.",
            "There are too many open issues that date back to 4 years ago.",
        ],
        &[],
    );
}

#[test]
fn corrects_dates_back_from() {
    assert_good_and_bad_suggestions(
        "This code dates back from 2014.",
        test_linter(),
        &[
            "This code dates from 2014.",
            "This code dates back to 2014.",
        ],
        &[],
    );
}

#[test]
fn allows_date_back_to() {
    assert_no_lints(
        "These scripts date back to when Perl was popular.",
        test_linter(),
    );
}

// Note: "the date back from" and "get dates back from" are known false
// positives where "date" is a noun (retrieving data). Phrase set matching
// cannot distinguish these from the verb form. See issue #2864.

// DoubleEdgedSword

#[test]
fn correct_double_edge_hyphen() {
    assert_suggestion_result(
        "I thought the global defaultTranslationValues was potentially a double-edge sword as it also obfuscates the full set of values",
        test_linter(),
        "I thought the global defaultTranslationValues was potentially a double-edged sword as it also obfuscates the full set of values",
    );
}

#[test]
fn correct_double_edge_space() {
    assert_suggestion_result(
        "It becomes a double edge sword when it should not be used in cases like this.",
        test_linter(),
        "It becomes a double-edged sword when it should not be used in cases like this.",
    );
}

#[test]
fn correct_double_edge_space_plural() {
    assert_suggestion_result(
        "Wake locks are really double edge swords.",
        test_linter(),
        "Wake locks are really double-edged swords.",
    );
}

#[test]
fn correct_double_edged_space() {
    assert_suggestion_result(
        "Use case. currently OPTIMIZE is a double edged sword and potentially a very dangerous tool to use.",
        test_linter(),
        "Use case. currently OPTIMIZE is a double-edged sword and potentially a very dangerous tool to use.",
    );
}

#[test]
fn correct_double_edged_space_plural() {
    assert_suggestion_result(
        "Change: Ambushers and Crusaders now protect their targets too, making them double edged swords",
        test_linter(),
        "Change: Ambushers and Crusaders now protect their targets too, making them double-edged swords",
    );
}

// ExpandAlloc

#[test]
fn corrects_allocs() {
    assert_suggestion_result(
        "cmd/compile: avoid allocs by better tracking of literals for interface conversions and make",
        test_linter(),
        "cmd/compile: avoid allocations by better tracking of literals for interface conversions and make",
    );
}

#[test]
fn expand_alloc() {
    assert_suggestion_result(
        "Used to find system libraries that alloc RWX regions on load.",
        test_linter(),
        "Used to find system libraries that allocate RWX regions on load.",
    );
}

// ExpandGovt

#[test]
fn corrects_govt_no_dot() {
    assert_suggestion_result(
        "Separation between privately issued credentials vs govt issued identity credentials",
        test_linter(),
        "Separation between privately issued credentials vs government issued identity credentials",
    );
}

#[test]
fn corrects_govt_do() {
    assert_suggestion_result(
        "Demystifying public comments on govt. regulations.",
        test_linter(),
        "Demystifying public comments on government regulations.",
    );
}

#[test]
fn corrects_govts() {
    assert_suggestion_result(
        "Those 'elite' economists have been advising govts for years.",
        test_linter(),
        "Those 'elite' economists have been advising governments for years.",
    );
}

// Expat

#[test]
fn correct_ex_pat_hyphen() {
    assert_suggestion_result(
        "It seems ex-pat means the person will be in a foreign country temporarily",
        test_linter(),
        "It seems expat means the person will be in a foreign country temporarily",
    );
}

#[test]
fn correct_ex_pats_hyphen() {
    assert_suggestion_result(
        "So, it might be correct to call most Brits ex-pats.",
        test_linter(),
        "So, it might be correct to call most Brits expats.",
    );
}

#[test]
fn correct_ex_pat_space() {
    assert_suggestion_result(
        "For me, the term ex pat embodies the exquisite hypocrisy of certain people feeling entitled",
        test_linter(),
        "For me, the term expat embodies the exquisite hypocrisy of certain people feeling entitled",
    );
}

#[test]
#[ignore = "replace_with_match_case results in ExPats"]
fn correct_ex_pats_space() {
    assert_suggestion_result(
        "Why are Brits who emigrate \"Ex Pats\" but people who come here \"immigrants\"?",
        test_linter(),
        "Why are Brits who emigrate \"Expats\" but people who come here \"immigrants\"?",
    );
}

// Expatriate

#[test]
fn correct_expatriot() {
    assert_suggestion_result(
        "Another expatriot of the era, James Joyce, also followed Papa's writing and drinking schedule.",
        test_linter(),
        "Another expatriate of the era, James Joyce, also followed Papa's writing and drinking schedule.",
    );
}

#[test]
fn correct_expatriots() {
    assert_suggestion_result(
        "Expatriots, upon discovering the delightful nuances of Dutch pronunciation, often find themselves in stitches.",
        test_linter(),
        "Expatriates, upon discovering the delightful nuances of Dutch pronunciation, often find themselves in stitches.",
    );
}

#[test]
fn correct_ex_patriot_hyphen() {
    assert_suggestion_result(
        "Then I added we should all be using the word 移民 immigrant, not ex-patriot, not 外国人 gaikokujin, and definitely not 外人 gaijin",
        test_linter(),
        "Then I added we should all be using the word 移民 immigrant, not expatriate, not 外国人 gaikokujin, and definitely not 外人 gaijin",
    );
}

#[test]
fn correct_ex_patriots_hyphen() {
    assert_suggestion_result(
        "Ex-patriots who move to Hong Kong to seek greener pastures and to experience a new culture seem to bring their own cultural baggage with them.",
        test_linter(),
        "Expatriates who move to Hong Kong to seek greener pastures and to experience a new culture seem to bring their own cultural baggage with them.",
    );
}

// GetRidOf

#[test]
fn get_rid_off() {
    assert_suggestion_result(
        "Please bump axios version to get rid off npm warning #624",
        test_linter(),
        "Please bump axios version to get rid of npm warning #624",
    );
}

#[test]
fn gets_rid_off() {
    assert_suggestion_result(
        "Adding at as a runtime dependency gets rid off that error",
        test_linter(),
        "Adding at as a runtime dependency gets rid of that error",
    );
}

#[test]
fn getting_rid_off() {
    assert_suggestion_result(
        "getting rid off of all the complexity of the different accesses method of API service providers",
        test_linter(),
        "getting rid of of all the complexity of the different accesses method of API service providers",
    );
}

#[test]
fn got_rid_off() {
    assert_suggestion_result(
        "For now we got rid off circular deps in model tree structure and it's API.",
        test_linter(),
        "For now we got rid of circular dependencies in model tree structure and it's API.",
    );
}

#[test]
fn gotten_rid_off() {
    assert_suggestion_result(
        "The baX variable thingy I have gotten rid off, that was due to a bad character in the encryption key.",
        test_linter(),
        "The baX variable thingy I have gotten rid of, that was due to a bad character in the encryption key.",
    );
}

#[test]
fn get_ride_of() {
    assert_suggestion_result(
        "Get ride of \"WARNING Deprecated: markdown_github. Use gfm\"",
        test_linter(),
        "Get rid of \"WARNING Deprecated: markdown_github. Use gfm\"",
    );
}

#[test]
fn get_ride_off() {
    assert_suggestion_result(
        "This exact hack was what I trying to get ride off. ",
        test_linter(),
        "This exact hack was what I trying to get rid of. ",
    );
}

#[test]
fn getting_ride_of() {
    assert_suggestion_result(
        "If you have any idea how to fix this without getting ride of bootstrap I would be thankfull.",
        test_linter(),
        "If you have any idea how to fix this without getting rid of bootstrap I would be thankfull.",
    );
}

#[test]
fn gets_ride_of() {
    assert_suggestion_result(
        ".. gets ride of a central back-end/server and eliminates all the risks associated to it.",
        test_linter(),
        ".. gets rid of a central back-end/server and eliminates all the risks associated to it.",
    );
}

#[test]
fn gotten_ride_of() {
    assert_suggestion_result(
        "I have gotten ride of the react-table and everything works just fine.",
        test_linter(),
        "I have gotten rid of the react-table and everything works just fine.",
    );
}

#[test]
fn got_ride_of() {
    assert_suggestion_result(
        "I had to adjust the labels on the free version because you guys got ride of ...",
        test_linter(),
        "I had to adjust the labels on the free version because you guys got rid of ...",
    );
}

// HolyWar

#[test]
#[ignore = "Known failure due to replace_with_match_case working by character index"]
fn correct_holy_war() {
    assert_suggestion_result(
        "I know it is Holly War about idempotent in HTTP and DELETE",
        test_linter(),
        "I know it is Holy War about idempotent in HTTP and DELETE",
    );
}

#[test]
fn correct_holly_wars() {
    assert_suggestion_result(
        "Anyway I'm not starting some holly wars about this point.",
        test_linter(),
        "Anyway I'm not starting some holy wars about this point.",
    );
}

// HowItLooksLike

#[test]
fn correct_how_it_looks_like_1() {
    assert_suggestion_result(
        "And here is how it looks like: As you can see, there is no real difference in the diagram itself.",
        test_linter(),
        "And here is how it looks: As you can see, there is no real difference in the diagram itself.",
    );
}

#[test]
fn correct_how_it_looks_like_2() {
    assert_suggestion_result(
        "This is how it looks like when run from Windows PowerShell or Cmd: image.",
        test_linter(),
        "This is what it looks like when run from Windows PowerShell or Cmd: image.",
    );
}

#[test]
fn correct_how_they_look_like_1() {
    assert_suggestion_result(
        "This is a sample project illustrating a demo of how to use the new Material 3 components and how they look like.",
        test_linter(),
        "This is a sample project illustrating a demo of how to use the new Material 3 components and how they look.",
    );
}

#[test]
fn correct_how_they_look_like_2() {
    assert_suggestion_result(
        "So for now I'll just leave this issue here of how they look like in the XLSX",
        test_linter(),
        "So for now I'll just leave this issue here of what they look like in the XLSX",
    );
}

#[test]
fn correct_how_they_looks_like_1() {
    assert_suggestion_result(
        "Here I demonstrate how disney works and how they looks like Don't miss to give me a star.",
        test_linter(),
        "Here I demonstrate how disney works and how they look Don't miss to give me a star.",
    );
}

#[test]
fn correct_how_they_looks_like_2() {
    assert_suggestion_result(
        "You can check how they looks like on Android app by this command:",
        test_linter(),
        "You can check what they look like on Android app by this command:",
    );
}

#[test]
fn correct_how_she_looks_like_1() {
    assert_suggestion_result(
        "You all know how she looks like.",
        test_linter(),
        "You all know how she looks.",
    );
}

#[test]
fn correct_how_he_looks_like_2() {
    assert_suggestion_result(
        "Here's how he looks like, when he's supposed to just look like his old fatui design.",
        test_linter(),
        "Here's what he looks like, when he's supposed to just look like his old fatui design.",
    );
}

#[test]
fn correct_how_it_look_like_1() {
    assert_suggestion_result(
        "And I don't mind how it look like, language code subpath or the last subpath as below.",
        test_linter(),
        "And I don't mind how it looks, language code subpath or the last subpath as below.",
    );
}

#[test]
fn correct_how_it_look_like_2() {
    assert_suggestion_result(
        "Here is how it look like in your browser:",
        test_linter(),
        "Here is what it looks like in your browser:",
    );
}

#[test]
fn correct_how_it_looks_like_with_apostrophe() {
    assert_suggestion_result(
        "In the picture we can see how It look's like on worker desktop.",
        test_linter(),
        "In the picture we can see how It looks on worker desktop.",
    );
}

// InRetaliationTo

#[test]
fn corrects_in_retaliation_to_to_for() {
    assert_suggestion_result(
        "Damage caused in retaliation to another attack by the Thorns enchantment.",
        test_linter(),
        "Damage caused in retaliation for another attack by the Thorns enchantment.",
    );
}

#[test]
fn corrects_in_retaliation_to_to_in_response_to() {
    assert_suggestion_result(
        "In retaliation to disagreeing with legal naming issues, a crucial (albeit rather small) section of code was removed from the NPM database",
        test_linter(),
        "In response to disagreeing with legal naming issues, a crucial (albeit rather small) section of code was removed from the NPM database",
    );
}

// LevelOfDetails

#[test]
fn corrects_level_of_details_singular_contrived() {
    assert_suggestion_result(
        "The model has a high level of details.",
        test_linter(),
        "The model has a high level of detail.",
    );
}

fn corrects_levels_of_details_plural_contrived() {
    assert_suggestion_result(
        "The game uses several level of details to save memory.",
        test_linter(),
        "The game uses several levels of detail to save memory.",
    );
}

#[test]
fn corrects_level_of_details_singular_real_world() {
    assert_suggestion_result(
        "How to implement a level of details visualizer for 3D meshes?",
        test_linter(),
        "How to implement a level of detail visualizer for 3D meshes?",
    );
}

#[test]
fn corrects_level_of_details_plural_real_world() {
    assert_suggestion_result(
        "LOD's (Level of details) are a set of lower models used for the purpose of optimisation",
        test_linter(),
        "LOD's (Levels of detail) are a set of lower models used for the purpose of optimisation",
    );
}

#[test]
fn corrects_levels_of_details_real_world() {
    assert_suggestion_result(
        "The file completion uses two levels of details to optimize performance.",
        test_linter(),
        "The file completion uses two levels of detail to optimize performance.",
    );
}

// Lookalike

#[test]
fn corrects_look_a_like() {
    assert_good_and_bad_suggestions(
        "Define the look-a-like of the cursor/mouse pointer:",
        test_linter(),
        &[
            "Define the lookalike of the cursor/mouse pointer:",
            "Define the look-alike of the cursor/mouse pointer:",
        ],
        &[],
    );
}

#[test]
fn corrects_look_a_likes() {
    assert_good_and_bad_suggestions(
        "Attempt at using AWS facial recognition to find look-a-likes in the Rijksmuseum's art collection.",
        test_linter(),
        &[
            "Attempt at using AWS facial recognition to find lookalikes in the Rijksmuseum's art collection.",
            "Attempt at using AWS facial recognition to find look-alikes in the Rijksmuseum's art collection.",
        ],
        &[],
    );
}

// MakeItSeem

#[test]
fn corrects_make_it_seems() {
    assert_suggestion_result(
        "but put it into unlisted list may make it seems like listed for GitHub",
        test_linter(),
        "but put it into unlisted list may make it seem like listed for GitHub",
    );
}

#[test]
fn corrects_made_it_seems() {
    assert_suggestion_result(
        "previous explanations made it seems like it would be n",
        test_linter(),
        "previous explanations made it seem like it would be n",
    );
}

#[test]
fn corrects_makes_it_seems() {
    assert_suggestion_result(
        "bundle gives an error that makes it seems like esbuild is trying to use lib/index.js from main",
        test_linter(),
        "bundle gives an error that makes it seem like esbuild is trying to use lib/index.js from main",
    );
}

#[test]
fn corrects_making_it_seems() {
    assert_suggestion_result(
        "Is it possible to teach the concept of assignment/reassignment at the very beginner stage instead of making it seems like constants?",
        test_linter(),
        "Is it possible to teach the concept of assignment/reassignment at the very beginner stage instead of making it seem like constants?",
    );
}

#[test]
fn corrects_made_it_seemed() {
    assert_suggestion_result(
        "The path made it seemed a bit \"internal\".",
        test_linter(),
        "The path made it seem a bit \"internal\".",
    );
}

// Monumentous

#[test]
fn corrects_monumentous() {
    assert_suggestion_result(
        "I think that would be a monumentous step in the right direction, and would DEFINATLY turn heads in not just the music industry, but every ...",
        test_linter(),
        "I think that would be a momentous step in the right direction, and would DEFINATLY turn heads in not just the music industry, but every ...",
    );
}

#[test]
fn corrects_monumentously() {
    assert_suggestion_result(
        "the most impressive thing out of all of this is that GitHub created such a monumentously good name",
        test_linter(),
        "the most impressive thing out of all of this is that GitHub created such a monumentally good name",
    );
}

// NervousWreck

#[test]
#[ignore = "Harper matches case by letter index as 'How Not to Be a Complete NervoUs wreck in an Interview'"]
fn correct_nerve_wreck_space_title_case() {
    assert_suggestion_result(
        "How Not to Be a Complete Nerve Wreck in an Interview",
        test_linter(),
        "How Not to Be a Complete Nervous Wreck in an Interview",
    );
}

#[test]
fn correct_nerve_wreck_space() {
    assert_suggestion_result(
        "The nerve wreck you are makes you seem anxious and agitated so your employer will believe the complaints.",
        test_linter(),
        "The nervous wreck you are makes you seem anxious and agitated so your employer will believe the complaints.",
    );
}

#[test]
fn correct_nerve_wreck_hyphen() {
    assert_suggestion_result(
        "the child receives little education and grows up to be a nerve-wreck",
        test_linter(),
        "the child receives little education and grows up to be a nervous wreck",
    );
}

#[test]
fn correct_nerve_wreck_hyphen_plural() {
    assert_suggestion_result(
        "This helps us not to become nerve wrecks while looking at the side mirrors",
        test_linter(),
        "This helps us not to become nervous wrecks while looking at the side mirrors",
    );
}

#[test]
#[ignore = "We can't detect when the altered form is used for an event rather than a person."]
fn dont_correct_it_was_a_nerve_wreck() {
    assert_no_lints(
        "It was a nerve-wreck, but I was also excited to see what would happen next.",
        test_linter(),
    );
}

#[test]
#[ignore = "We can't detect when the altered form is used for an event rather than a person."]
fn dont_correct_so_much_nerve_wreck() {
    assert_no_lints(
        "So much nerve wreck for such a simple game ...",
        test_linter(),
    );
}

// NotOnly

// -not only are-
#[test]
fn fix_no_only_are() {
    assert_suggestion_result(
        "No only are tests run on my pipeline but once successful, my app is deployed differently",
        test_linter(),
        "Not only are tests run on my pipeline but once successful, my app is deployed differently",
    );
}

// -not only is-
#[test]
fn fix_no_only_is() {
    assert_suggestion_result(
        "No only is it simple, it's efficient!",
        test_linter(),
        "Not only is it simple, it's efficient!",
    );
}

// -not only was-
#[test]
fn fix_no_only_was() {
    assert_suggestion_result(
        "No only was he happily creating shapes, but he was actively using distances and angles to do so.",
        test_linter(),
        "Not only was he happily creating shapes, but he was actively using distances and angles to do so.",
    );
}

// -not only were-
#[test]
fn fix_no_only_were() {
    assert_suggestion_result(
        "No only were there UI inconsistencies, but Safari lags behind chrome with things like the Popover API",
        test_linter(),
        "Not only were there UI inconsistencies, but Safari lags behind chrome with things like the Popover API",
    );
}

// Nowadays

#[test]
fn fix_now_a_days_spaces() {
    assert_suggestion_result(
        "Now a days, movie recommendation systems are well developed and are user focused.",
        test_linter(),
        "Nowadays, movie recommendation systems are well developed and are user focused.",
    );
}

#[test]
fn fix_now_a_days_apostrophe() {
    assert_suggestion_result(
        "Now a day's recognizing the activity from the surveillance video is a challenging task.",
        test_linter(),
        "Nowadays recognizing the activity from the surveillance video is a challenging task.",
    );
}

#[test]
fn fix_now_a_days_hyphen() {
    assert_suggestion_result(
        "Recommendation engines are now a one of the most common Machine Learning project that can be seen now-a-days.",
        test_linter(),
        "Recommendation engines are now a one of the most common Machine Learning project that can be seen nowadays.",
    );
}

#[test]
fn fix_now_a_day() {
    assert_suggestion_result(
        "Now a day a calendar is a daily essential things.",
        test_linter(),
        "Nowadays a calendar is a daily essential things.",
    );
}

#[test]
fn fix_now_a_day_hyphen() {
    assert_suggestion_result(
        "Now-a-day, lots of people prefer ordering food online to save their time and effort.",
        test_linter(),
        "Nowadays, lots of people prefer ordering food online to save their time and effort.",
    );
}

#[test]
fn fix_now_adays_hyphen() {
    assert_suggestion_result(
        "@andyp1per knows most about those Python scripts now-adays.",
        test_linter(),
        "@andyp1per knows most about those Python scripts nowadays.",
    );
}

#[test]
fn fix_now_adays_space() {
    assert_suggestion_result(
        "Coding is one of my fav thing to do now adays.!",
        test_linter(),
        "Coding is one of my fav thing to do nowadays.!",
    );
}

#[test]
fn fix_nowaday() {
    assert_suggestion_result(
        "nowaday, I have to capitalize the first letter of the name gets from @babel/types.",
        test_linter(),
        "nowadays, I have to capitalize the first letter of the name gets from @babel/types.",
    );
}

#[test]
fn fix_now_adays_apostrophe() {
    assert_suggestion_result(
        "I believe CSS clamp has great browser support now aday's as well.",
        test_linter(),
        "I believe CSS clamp has great browser support nowadays as well.",
    );
}

#[test]
fn fix_nowa_days() {
    assert_suggestion_result(
        "But discord would be great cause discord is universally used for all games and companies and schools nowa days.",
        test_linter(),
        "But discord would be great cause discord is universally used for all games and companies and schools nowadays.",
    );
}

#[test]
fn fix_now_aday() {
    assert_suggestion_result(
        "OF all Occupations that now aday is used,I would not be a butcher",
        test_linter(),
        "OF all Occupations that nowadays is used,I would not be a butcher",
    );
}

// Payed

#[test]
fn correct_payed() {
    assert_suggestion_result(
        "He payed the bill yesterday.",
        test_linter(),
        "He paid the bill yesterday.",
    );
}

#[test]
fn correct_overpayed() {
    assert_suggestion_result(
        "He overpayed in part to have the specification met.",
        test_linter(),
        "He overpaid in part to have the specification met.",
    );
}

// PlayAFactor

#[test]
fn play_a_factor() {
    assert_good_and_bad_suggestions(
        "I thought scaling might play a factor in this so I made sure it was 100% on all three desktops.",
        test_linter(),
        &[
            "I thought scaling might play a part in this so I made sure it was 100% on all three desktops.",
            "I thought scaling might be a factor in this so I made sure it was 100% on all three desktops.",
        ],
        &[],
    );
}

#[test]
fn played_a_factor_sg() {
    assert_good_and_bad_suggestions(
        "I want you to look past them, because none of them played a factor in composing a dream team.",
        test_linter(),
        &[
            "I want you to look past them, because none of them played a part in composing a dream team.",
            "I want you to look past them, because none of them was a factor in composing a dream team.",
        ],
        &[],
    );
}

#[test]
fn played_a_factor_pl() {
    assert_good_and_bad_suggestions(
        "we have no idea why they do what they do and what past external influences played factors in those decisions",
        test_linter(),
        &[
            "we have no idea why they do what they do and what past external influences played roles in those decisions",
            "we have no idea why they do what they do and what past external influences were a factor in those decisions",
        ],
        &[],
    );
}

#[test]
fn playing_a_factor() {
    assert_good_and_bad_suggestions(
        "I think my development 'forceIP' was playing a factor here.",
        test_linter(),
        &[
            "I think my development 'forceIP' was playing a part here.",
            "I think my development 'forceIP' was a factor here.",
        ],
        &[],
    );
}

#[test]
fn plays_a_factor() {
    assert_good_and_bad_suggestions(
        "The amount of time since the change certainly plays a factor since we upgraded to eslint v6 3 years ago.",
        test_linter(),
        &[
            "The amount of time since the change certainly plays a part since we upgraded to eslint v6 3 years ago.",
            "The amount of time since the change certainly is a factor since we upgraded to eslint v6 3 years ago.",
        ],
        &[],
    );
}

// - -PlayAFactor- known false positives -
// so more thought might go into bringing into play factors seldom addressed
// how much team play factors such as team cohesion, and skills coverage can affect the result
// It appears his role was to explore game playing factors, which are hard to ...
// Age, literacy, learning disabilities, as well as physical disabilities all play factors here.
// timing of defaults, salvage value, etc. all play factors

// RaiseTheQuestion

// -raise the question-
#[test]
fn detect_rise_the_question() {
    assert_suggestion_result(
        "That would rise the question how to deal with syntax errors etc.",
        test_linter(),
        "That would raise the question how to deal with syntax errors etc.",
    );
}

#[test]
fn detect_arise_the_question() {
    assert_suggestion_result(
        "As e.g. UTC+1, might arise the question whether it includes summer and winter time",
        test_linter(),
        "As e.g. UTC+1, might raise the question whether it includes summer and winter time",
    );
}

// -raises the question-
#[test]
fn detect_rises_the_question() {
    assert_suggestion_result(
        "However, this rises the question as to whether this test is conceptually sound.",
        test_linter(),
        "However, this raises the question as to whether this test is conceptually sound.",
    );
}

#[test]
fn detect_arises_the_question() {
    assert_suggestion_result(
        "And it arises the question, why?",
        test_linter(),
        "And it raises the question, why?",
    );
}

// -raising the question-
#[test]
fn detect_rising_the_question() {
    assert_suggestion_result(
        "as soon as a infoHash query is performed, a Torrent file is retried, rising the question of:",
        test_linter(),
        "as soon as a infoHash query is performed, a Torrent file is retried, raising the question of:",
    );
}

#[test]
fn detect_arising_the_question() {
    assert_suggestion_result(
        "arising the question whether the requirement of wgpu::Features::DEPTH24PLUS_STENCIL8 is precise",
        test_linter(),
        "raising the question whether the requirement of wgpu::Features::DEPTH24PLUS_STENCIL8 is precise",
    );
}

// -raised the question-
#[test]
fn detect_rose_the_question() {
    assert_suggestion_result(
        "Here is an example that rose the question at first: What works.",
        test_linter(),
        "Here is an example that raised the question at first: What works.",
    );
}

#[test]
fn detect_risen_the_question() {
    assert_suggestion_result(
        "That has risen the question in my mind if it is still possible to embed your own Flash player on Facebook today?",
        test_linter(),
        "That has raised the question in my mind if it is still possible to embed your own Flash player on Facebook today?",
    );
}

#[test]
fn detect_rised_the_question() {
    assert_suggestion_result(
        "I rised the question to Emax Support and they just came back to me inmediately with the below response.",
        test_linter(),
        "I raised the question to Emax Support and they just came back to me inmediately with the below response.",
    );
}

#[test]
#[ignore = "Not actually an error after when it's 'there arose'"]
fn dont_fag_there_arose_the_question() {
    assert_suggestion_result(
        "Hello, while I have been using modals manager there arose the question related to customizing of modal header.",
        test_linter(),
        "Hello, while I have been using modals manager there arose the question related to customizing of modal header.",
    );
}

#[test]
fn detect_arised_the_question() {
    assert_suggestion_result(
        "and that fact arised the question in my mind, what does exactly is happening",
        test_linter(),
        "and that fact raised the question in my mind, what does exactly is happening",
    );
}

#[test]
fn detect_arose_the_question() {
    assert_suggestion_result(
        "This arose the question, could I store 32 digits on the stack?",
        test_linter(),
        "This raised the question, could I store 32 digits on the stack?",
    );
}

#[test]
fn detect_arisen_the_question() {
    assert_suggestion_result(
        "Some have arisen the question like how to use this wireless HD mini camera",
        test_linter(),
        "Some have raised the question like how to use this wireless HD mini camera",
    );
}

// SideTangent

#[test]
fn fix_side_tangent_start_of_sentence() {
    assert_suggestion_result(
        "Side tangent: I personally wouldn't worry about using ; for removing the selection unless you need to.",
        test_linter(),
        "Tangent: I personally wouldn't worry about using ; for removing the selection unless you need to.",
    );
}

#[test]
fn fix_side_tangent_aside() {
    assert_suggestion_result(
        "As a side tangent, in addition to not solving the gradual code repair problem",
        test_linter(),
        "As an aside, in addition to not solving the gradual code repair problem",
    );
}

#[test]
fn fix_side_tangents() {
    assert_suggestion_result(
        "so we don't get bogged down by tiny formatting bikeshedding side tangents",
        test_linter(),
        "so we don't get bogged down by tiny formatting bikeshedding tangents",
    );
}

// ToToo

// -a bridge too far-
#[test]
fn fix_a_bridge_too_far() {
    assert_suggestion_result(
        "If Winforms can ever be conquered by the Mono developers may be a bridge to far.",
        test_linter(),
        "If Winforms can ever be conquered by the Mono developers may be a bridge too far.",
    );
}

// -cake and eat it too-
#[test]
fn fix_cake_and_eat_it_too() {
    assert_suggestion_result(
        "The solution: wouldn't it be great if I could have my cake and eat it to?",
        test_linter(),
        "The solution: wouldn't it be great if I could have my cake and eat it too?",
    );
}

// -go to far-
#[test]
fn fix_go_to_far() {
    assert_suggestion_result(
        "It's difficult to be sure when we go to far sometime when you don't exactly how the beast works in the background .",
        test_linter(),
        "It's difficult to be sure when we go too far sometime when you don't exactly how the beast works in the background .",
    );
}

// -goes to far-
#[test]
fn fix_goes_to_far() {
    assert_suggestion_result(
        "Memory consumption and cpu consumption goes to far like 900% and more than this",
        test_linter(),
        "Memory consumption and cpu consumption goes too far like 900% and more than this",
    );
}

// -going to far-
#[test]
fn fix_going_to_far() {
    assert_suggestion_result(
        "wsrun is going to far on this because debug 's devDependency shouldn't be considered in the cycle detection, should it?",
        test_linter(),
        "wsrun is going too far on this because debug 's devDependency shouldn't be considered in the cycle detection, should it?",
    );
}

// -gone to far-
#[test]
fn fix_gone_to_far() {
    assert_suggestion_result(
        "I might have gone to far with opening issues for small things.",
        test_linter(),
        "I might have gone too far with opening issues for small things.",
    );
}

// -went to far-
#[test]
fn fix_went_to_far() {
    assert_suggestion_result(
        "But I went to far compared to the initial request that seems talk about ...",
        test_linter(),
        "But I went too far compared to the initial request that seems talk about ...",
    );
}

// -life's too short-
#[test]
fn fix_life_s_too_short() {
    assert_suggestion_result(
        "Life's to short for messing around with git add , writing commit message.",
        test_linter(),
        "Life's too short for messing around with git add , writing commit message.",
    );
}

#[test]
fn fix_lifes_to_short() {
    assert_suggestion_result(
        "I wouldn't go back after the 3rd interview lifes to short.",
        test_linter(),
        "I wouldn't go back after the 3rd interview life's too short.",
    );
}

// -life is too short-
#[test]
fn fix_life_is_too_short() {
    assert_suggestion_result(
        "[Life is to short to use dated cli tools that suck]",
        test_linter(),
        "[Life is too short to use dated cli tools that suck]",
    );
}

// -put too fine a point-
#[test]
fn fix_put_too_fine_a_point() {
    assert_suggestion_result(
        "Not to put to fine a point on it... that's not the kind of team I think we want to be.",
        test_linter(),
        "Not to put too fine a point on it... that's not the kind of team I think we want to be.",
    );
}

// -speak too soon-
#[test]
fn fix_speak_too_soon() {
    assert_suggestion_result(
        "I don't want to speak to soon but I kept everything as I had before but included: http = httplib2.Http()",
        test_linter(),
        "I don't want to speak too soon but I kept everything as I had before but included: http = httplib2.Http()",
    );
}

// -speaking too soon-
#[test]
fn fix_speaking_too_soon() {
    assert_suggestion_result(
        "EDIT: Thats what I get for speaking to soon...",
        test_linter(),
        "EDIT: Thats what I get for speaking too soon...",
    );
}

// -spoke too soon-
#[test]
fn fix_spoke_too_soon() {
    assert_suggestion_result(
        "I spoke to soon. Ignore the previous post.",
        test_linter(),
        "I spoke too soon. Ignore the previous post.",
    );
}

// -spoken too soon-
#[test]
fn fix_spoken_too_soon() {
    assert_suggestion_result(
        "EDIT: I might have spoken to soon...",
        test_linter(),
        "EDIT: I might have spoken too soon...",
    );
}

// -think to much-
#[test]
fn fix_think_too_much() {
    assert_suggestion_result(
        "I don't think to much about it, but I don't think it's a big deal.",
        test_linter(),
        "I don't think too much about it, but I don't think it's a big deal.",
    );
}

// -too big for-
#[test]
fn fix_too_big_for() {
    assert_suggestion_result(
        "ng-relations form to big for small screens",
        test_linter(),
        "ng-relations form too big for small screens",
    );
}

// -too big to fail-
#[test]
fn fix_too_big_to_fail() {
    assert_suggestion_result(
        "The core alone has 50k LOC. Reminds me of \"to big to fail\".",
        test_linter(),
        "The core alone has 50k LOC. Reminds me of \"too big to fail\".",
    );
}

// -too good to be true-
#[test]
fn fix_too_good_to_be_true() {
    assert_suggestion_result(
        "This seemed to good to be true, but local to scene resources will not work when they are not contained in a node.",
        test_linter(),
        "This seemed too good to be true, but local to scene resources will not work when they are not contained in a node.",
    );
}

#[test]
fn fix_too_good_too_be_true() {
    assert_suggestion_result(
        "The normalization of rewards is making the plot in tensorboard look too good too be true, because they are not the actual reward ...",
        test_linter(),
        "The normalization of rewards is making the plot in tensorboard look too good to be true, because they are not the actual reward ...",
    );
}

// -too much information-
#[test]
fn fix_too_much_information() {
    assert_suggestion_result(
        "Live test are printing way to much information and is polluting our test output",
        test_linter(),
        "Live test are printing way too much information and is polluting our test output",
    );
}

// TooTo

// -too big too fail-
#[test]
fn fix_too_big_too_fail() {
    assert_suggestion_result(
        "In other words, pointer arithmetic is, at this point, too big too fail, regardless of the clever and sophisticated way C++ lawyercats worded it.",
        test_linter(),
        "In other words, pointer arithmetic is, at this point, too big to fail, regardless of the clever and sophisticated way C++ lawyercats worded it.",
    );
}

// WholeEntire

#[test]
fn detect_atomic_whole_entire() {
    assert_suggestion_result("whole entire", test_linter(), "whole");
}

#[test]
fn correct_real_world_whole_entire() {
    assert_suggestion_result(
        "[FR] support use system dns in whole entire app",
        test_linter(),
        "[FR] support use system dns in whole app",
    );
}

// -a whole entire-
#[test]
fn correct_atomic_a_whole_entire_to_a_whole() {
    assert_suggestion_result("a whole entire", test_linter(), "a whole");
}

#[test]
fn correct_atomic_a_whole_entire_to_an_entire() {
    assert_suggestion_result("a whole entire", test_linter(), "an entire");
}

#[test]
fn correct_real_world_a_whole_entire_to_a_whole() {
    assert_suggestion_result(
        "Start mapping a whole entire new planet using NASA’s MOLA.",
        test_linter(),
        "Start mapping a whole new planet using NASA’s MOLA.",
    );
}

#[test]
fn correct_real_world_a_whole_entire_to_an_entire() {
    assert_suggestion_result(
        "I am not sure I can pass in a whole entire query via the include.",
        test_linter(),
        "I am not sure I can pass in an entire query via the include.",
    );
}

// WorseOrWorst

// -a lot worst-
#[test]
fn detect_a_lot_worse_atomic() {
    assert_suggestion_result("a lot worst", test_linter(), "a lot worse");
}

#[test]
fn detect_a_lot_worse_real_world() {
    assert_suggestion_result(
        "On a debug build, it's even a lot worst.",
        test_linter(),
        "On a debug build, it's even a lot worse.",
    );
}

// -become worst-
#[test]
fn fix_became_worst() {
    assert_suggestion_result(
        "The problem became worst lately.",
        test_linter(),
        "The problem became worse lately.",
    );
}

#[test]
fn fix_become_worst() {
    assert_suggestion_result(
        "But results seems stay at one place or become worst.",
        test_linter(),
        "But results seems stay at one place or become worse.",
    );
}

#[test]
fn fix_becomes_worst() {
    assert_suggestion_result(
        "This becomes worst if you have an x64 dll and an x86 dll that you don't have thier source codes and want to use them in same project!",
        test_linter(),
        "This becomes worse if you have an x64 dll and an x86 dll that you don't have thier source codes and want to use them in same project!",
    );
}

#[test]
fn fix_becoming_worst() {
    assert_suggestion_result(
        "France is becoming worst than the Five Eyes",
        test_linter(),
        "France is becoming worse than the Five Eyes",
    );
}

// -far worse-
#[test]
fn detect_far_worse_atomic() {
    assert_suggestion_result("far worst", test_linter(), "far worse");
}

#[test]
fn detect_far_worse_real_world() {
    assert_suggestion_result(
        "I mainly use Firefox (personal preference) and have noticed it has far worst performance than Chrome",
        test_linter(),
        "I mainly use Firefox (personal preference) and have noticed it has far worse performance than Chrome",
    );
}

// -get worst-
#[test]
fn fix_get_worse() {
    assert_suggestion_result(
        "and the problem appears to get worst with 2025.5.1 and 2025.5.2.",
        test_linter(),
        "and the problem appears to get worse with 2025.5.1 and 2025.5.2.",
    );
}

#[test]
fn fix_gets_worse() {
    assert_suggestion_result(
        "It just starts after about 15 minutes of work and gradually gets worst.",
        test_linter(),
        "It just starts after about 15 minutes of work and gradually gets worse.",
    );
}

#[test]
#[ignore = "This kind of false positive is probably too subtle to detect"]
fn dont_flag_getting_worst() {
    // Here "getting" probably belongs to "I am getting" rather than "getting worst".
    // Which would not be an error but "I am getting the worst accuracy" would be better.
    // TODO: Maybe a noun following "getting" is enough context?
    assert_lint_count(
        "I am getting worst accuracy on the same dataste and 3 different models.",
        test_linter(),
        0,
    );
}

#[test]
fn fix_getting_worst() {
    assert_suggestion_result(
        "But, as I said, it is getting worst...",
        test_linter(),
        "But, as I said, it is getting worse...",
    );
}

#[test]
fn fix_got_worst() {
    assert_suggestion_result(
        "typescript support got worst.",
        test_linter(),
        "typescript support got worse.",
    );
}

#[test]
fn fix_gotten_worst() {
    assert_suggestion_result(
        "Has Claude gotten worst?",
        test_linter(),
        "Has Claude gotten worse?",
    );
}

// -much worse-
#[test]
fn detect_much_worse_atomic() {
    assert_suggestion_result("much worst", test_linter(), "much worse");
}

#[test]
fn detect_much_worse_real_world() {
    assert_suggestion_result(
        "the generated image quality is much worst (actually nearly broken)",
        test_linter(),
        "the generated image quality is much worse (actually nearly broken)",
    );
}

// -turn for the worse-
#[test]
fn detect_turn_for_the_worse_atomic() {
    assert_suggestion_result("turn for the worst", test_linter(), "turn for the worse");
}

#[test]
fn detect_turn_for_the_worse_real_world() {
    assert_suggestion_result(
        "Very surprised to see this repo take such a turn for the worst.",
        test_linter(),
        "Very surprised to see this repo take such a turn for the worse.",
    );
}

// -worse than-
#[test]
fn detect_worse_than_atomic() {
    assert_suggestion_result("worst than", test_linter(), "worse than");
}

#[test]
fn detect_worse_than_real_world() {
    assert_suggestion_result(
        "Project real image - inversion quality is worst than in StyleGAN2",
        test_linter(),
        "Project real image - inversion quality is worse than in StyleGAN2",
    );
}

// -worst ever-
#[test]
fn detect_worst_ever_atomic() {
    assert_suggestion_result("worse ever", test_linter(), "worst ever");
}

#[test]
fn detect_worst_ever_real_world() {
    assert_suggestion_result(
        "The Bcl package family is one of the worse ever published by Microsoft.",
        test_linter(),
        "The Bcl package family is one of the worst ever published by Microsoft.",
    );
}

// -worse and worse-
#[test]
fn detect_worst_and_worst_atomic() {
    assert_suggestion_result("worst and worst", test_linter(), "worse and worse");
}

#[test]
fn detect_worst_and_worst_real_world() {
    assert_suggestion_result(
        "This control-L trick does not work for me. The padding is getting worst and worst.",
        test_linter(),
        "This control-L trick does not work for me. The padding is getting worse and worse.",
    );
}

#[test]
fn detect_worse_and_worst_real_world() {
    assert_suggestion_result(
        "This progressively got worse and worst to the point that the machine (LEAD 1010) stopped moving alltogether.",
        test_linter(),
        "This progressively got worse and worse to the point that the machine (LEAD 1010) stopped moving alltogether.",
    );
}

// -at worst-
#[test]
fn detect_at_worst_atomic() {
    assert_suggestion_result(
        "Partial moving of core objects to interpreter state is incorrect at best, unsafe at worse.",
        test_linter(),
        "Partial moving of core objects to interpreter state is incorrect at best, unsafe at worst.",
    );
}

// -worst case scenario-
#[test]
fn correct_worse_case_space() {
    assert_suggestion_result(
        "In the worse case scenario, remote code execution could be achieved.",
        test_linter(),
        "In the worst-case scenario, remote code execution could be achieved.",
    );
}

#[test]
fn correct_worse_case_hyphen() {
    assert_suggestion_result(
        "Basically I want my pods to get the original client IP address... or at least have X-Forwarded-For header, in a worse-case scenario.",
        test_linter(),
        "Basically I want my pods to get the original client IP address... or at least have X-Forwarded-For header, in a worst-case scenario.",
    );
}

#[test]
fn correct_worse_case_two_hyphens() {
    assert_suggestion_result(
        "In a worse-case-scenario, the scenario class code and the results being analysed, become out of sync, and so the wrong labels are applied.",
        test_linter(),
        "In a worst-case scenario, the scenario class code and the results being analysed, become out of sync, and so the wrong labels are applied.",
    );
}

// -make it worst-
#[test]
fn detect_make_it_worst_atomic() {
    assert_suggestion_result(
        "And if you try to access before that, CloudFront will cache the error and it'll make it worst.",
        test_linter(),
        "And if you try to access before that, CloudFront will cache the error and it'll make it worse.",
    );
}

// -made it worst-
#[test]
fn detect_made_it_worst_atomic() {
    assert_suggestion_result(
        "However in couple of occasions the refresh made it worst and it showed commit differences that were already commited and pushed to origin.",
        test_linter(),
        "However in couple of occasions the refresh made it worse and it showed commit differences that were already commited and pushed to origin.",
    );
}

// -makes it worst-
#[test]
fn detect_makes_it_worst_atomic() {
    assert_suggestion_result(
        "What makes it worst, is if I use the returned SHA to try and update the newly created file I get the same error I show below.",
        test_linter(),
        "What makes it worse, is if I use the returned SHA to try and update the newly created file I get the same error I show below.",
    );
}

// -making it worst-
#[test]
fn detect_making_it_worst_atomic() {
    assert_suggestion_result(
        "PLease ai realled need help with this I think I'm making it worst.",
        test_linter(),
        "PLease ai realled need help with this I think I'm making it worse.",
    );
}

// -make them worst-
#[test]
fn detect_make_them_worst_atomic() {
    assert_suggestion_result(
        "Not sure if this makes things clearer or make them worst.",
        test_linter(),
        "Not sure if this makes things clearer or make them worse.",
    );
}

// -made them worst-
#[test]
fn detect_made_them_worst_atomic() {
    assert_suggestion_result(
        "if not outroght caused them / made them worst",
        test_linter(),
        "if not outroght caused them / made them worse",
    );
}

// -makes them worst-
#[test]
fn detect_makes_them_worst_atomic() {
    assert_suggestion_result(
        "(tried ~14 different hyperparameter and data format combos), however, always just makes them worst, they go from \"slightly\" wrong to \"complete nonsense\".",
        test_linter(),
        "(tried ~14 different hyperparameter and data format combos), however, always just makes them worse, they go from \"slightly\" wrong to \"complete nonsense\".",
    );
}

#[test]
#[ignore = "This false positive is not handled yet"]
fn dont_flag_makes_them_worst_case() {
    assert_lint_count(
        "Note 1: all hash tables has an Achilles heel that makes them worst case O(N)",
        test_linter(),
        0,
    );
}

// -making them worst-
#[test]
fn detect_making_them_worst_atomic() {
    assert_suggestion_result(
        "As for the last part about Apple deliberately making them worst in order for us to buy the 3s",
        test_linter(),
        "As for the last part about Apple deliberately making them worse in order for us to buy the 3s",
    );
}

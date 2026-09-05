use super::LintGroup;
use crate::linting::pooled_linter::for_tests::create_test_pool;
use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

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

// Ado

#[test]
fn corrects_further_ado() {
    assert_suggestion_result(
        "... but we finally hit a great spot, so without further adieu.",
        test_linter(),
        "... but we finally hit a great spot, so without further ado.",
    );
}

#[test]
fn corrects_much_ado() {
    assert_suggestion_result(
        "After much adieu this functionality is now available.",
        test_linter(),
        "After much ado this functionality is now available.",
    );
}

// ArgumentToBeMade

#[test]
fn corrects_theres_an_argument_to_be_said() {
    assert_suggestion_result(
        "I guess there s an argument to be said that if the TUCKR_HOME is already defined, it should use that instead.",
        test_linter(),
        "I guess there s an argument to be made that if the TUCKR_HOME is already defined, it should use that instead.",
    );
}

#[test]
fn corrects_there_is_an_argument_to_be_said() {
    assert_suggestion_result(
        "Same argument for smooth_image_crate, although there is an argument to be said this is more-generally useful than scale_image_crate",
        test_linter(),
        "Same argument for smooth_image_crate, although there is an argument to be made this is more-generally useful than scale_image_crate",
    );
}

#[test]
fn corrects_theres_theres_arguments_to_be_said() {
    assert_suggestion_result(
        "there's there's arguments to be said for all of it.",
        test_linter(),
        "there's there's arguments to be made for all of it.",
    );
}

// Bollocks

#[test]
fn fix_complete_bullocks() {
    assert_suggestion_result(
        "why you think some of them are complete bullocks or would be a bad idea",
        test_linter(),
        "why you think some of them are complete bollocks or would be a bad idea",
    );
}

#[test]
fn fix_dogs() {
    assert_suggestion_result(
        "The cat's ass, priceless! I have to steal that one. My go to phrase is “The dog's bullocks.",
        test_linter(),
        "The cat's ass, priceless! I have to steal that one. My go to phrase is “The dog's bollocks.",
    );
}

#[test]
fn fix_dogs_no_apostrophe_bullocks() {
    assert_suggestion_result(
        "some dumb rubbish that i do not give a dogs bullocks about",
        test_linter(),
        "some dumb rubbish that i do not give a dogs bollocks about",
    );
}

#[test]
fn fix_is_bullocks() {
    assert_suggestion_result(
        "for me this is bullocks, when the same user can sudo rm -rf",
        test_linter(),
        "for me this is bollocks, when the same user can sudo rm -rf",
    );
}

#[test]
fn fix_its_bullocks() {
    assert_suggestion_result(
        "I'm too lazy to explain why, but I think it's bullocks.",
        test_linter(),
        "I'm too lazy to explain why, but I think it's bollocks.",
    );
}

#[test]
fn fix_its_no_apostrophe_bullocks() {
    assert_suggestion_result(
        "but lance, dont claim to be clean, because we all know its bullocks",
        test_linter(),
        "but lance, dont claim to be clean, because we all know its bollocks",
    );
}

#[test]
fn fix_such_bullocks() {
    assert_suggestion_result(
        "This is why numerology is such bullocks.",
        test_linter(),
        "This is why numerology is such bollocks.",
    );
}

#[test]
fn fix_thats_bullocks() {
    assert_suggestion_result(
        "Respectfully, that's bullocks.",
        test_linter(),
        "Respectfully, that's bollocks.",
    );
}

#[test]
fn fix_thats_no_apostrophe_bullocks() {
    assert_suggestion_result(
        "In CSS thats bullocks as directives have priority in the order they are defined.",
        test_linter(),
        "In CSS thats bollocks as directives have priority in the order they are defined.",
    );
}

#[test]
fn fix_total_bullocks() {
    assert_suggestion_result(
        "Pointing out to the audience that their gravity explanation is total bullocks would seem an ethical must as well.",
        test_linter(),
        "Pointing out to the audience that their gravity explanation is total bollocks would seem an ethical must as well.",
    );
}

#[test]
fn fix_utter_bullocks() {
    assert_suggestion_result(
        "what utter bullocks a self employed person will get £94 under corona virus crisis",
        test_linter(),
        "what utter bollocks a self employed person will get £94 under corona virus crisis",
    );
}

#[test]
fn fix_was_bullocks() {
    assert_suggestion_result(
        "a few years ago I thought that was bullocks",
        test_linter(),
        "a few years ago I thought that was bollocks",
    );
}

#[test]
fn fix_bullocks_exclamation() {
    assert_suggestion_result(
        "throw(new Error('Bullocks!')));",
        test_linter(),
        "throw(new Error('Bollocks!')));",
    );
}

#[test]
fn dont_flag_herd_of_bullocks() {
    assert_no_lints(
        "driven back (literally) by a herd of bullocks across the path",
        test_linter(),
    );
}

// ChampAtTheBit
#[test]
fn correct_chomp_at_the_bit() {
    assert_suggestion_result(
        "so other than rolling back to older drivers i might have to chomp at the bit for a while longer yet",
        test_linter(),
        "so other than rolling back to older drivers i might have to champ at the bit for a while longer yet",
    );
}

#[test]
fn correct_chomped_at_the_bit() {
    assert_suggestion_result(
        "I chomped at the bit, frustrated by my urge to go faster, while my husband chafed at what I thought was a moderate pace.",
        test_linter(),
        "I champed at the bit, frustrated by my urge to go faster, while my husband chafed at what I thought was a moderate pace.",
    );
}

#[test]
fn correct_chomping_at_the_bit() {
    assert_suggestion_result(
        "Checking in to see when the Windows install will be ready. I am chomping at the bit!",
        test_linter(),
        "Checking in to see when the Windows install will be ready. I am champing at the bit!",
    );
}

#[test]
fn correct_chomps_at_the_bit() {
    assert_suggestion_result(
        "nobody chomps at the bit to make sure these are maintained, current, complete, and error free",
        test_linter(),
        "nobody champs at the bit to make sure these are maintained, current, complete, and error free",
    );
}

// ClientOrServerSide

// -client's side-
#[test]
fn correct_clients_side() {
    assert_suggestion_result(
        "I want to debug this server-side as I cannot find out why the connection is being refused from the client's side.",
        test_linter(),
        "I want to debug this server-side as I cannot find out why the connection is being refused from the client-side.",
    );
}

// -server's side-
#[test]
fn correct_servers_side() {
    assert_suggestion_result(
        "A client-server model where the client can execute commands in a terminal on the server's side",
        test_linter(),
        "A client-server model where the client can execute commands in a terminal on the server-side",
    );
}

// Combinate

#[test]
fn correct_combinate() {
    assert_suggestion_result(
        "I'm at chapter 11 and I can't craft and combinate abyss gear, what should I do to unlock both of them",
        test_linter(),
        "I'm at chapter 11 and I can't craft and combine abyss gear, what should I do to unlock both of them",
    );
}

#[test]
fn correct_combinated() {
    assert_suggestion_result(
        "NFS WORLD COMBINATED MAP (NEW!)",
        test_linter(),
        "NFS WORLD COMBINED MAP (NEW!)",
    );
}

#[test]
fn correct_combinates() {
    assert_suggestion_result(
        "is there a game that combinates ottd and rts?",
        test_linter(),
        "is there a game that combines ottd and rts?",
    );
}

#[test]
fn correct_combinating() {
    assert_suggestion_result(
        "This section discusses how the color combinating is accomplished",
        test_linter(),
        "This section discusses how the color combining is accomplished",
    );
}

// CompulseToCompel

#[test]
fn correct_compulse() {
    assert_suggestion_result(
        "Play Store will soon compulse to use SDK 30 on any app updates , and it's mandatory to have SDK 30 for new apps.",
        test_linter(),
        "Play Store will soon compel to use SDK 30 on any app updates , and it's mandatory to have SDK 30 for new apps.",
    );
}

#[test]
fn correct_compulsed() {
    assert_suggestion_result(
        "Just alpha, but now i am compulsed to work 10.6 into the github actions and insane docker environment :)",
        test_linter(),
        "Just alpha, but now i am compelled to work 10.6 into the github actions and insane docker environment :)",
    );
}

#[test]
fn correct_compulses() {
    assert_suggestion_result(
        "Occasionally, a film comes along that compulses me to make a fan poster.",
        test_linter(),
        "Occasionally, a film comes along that compels me to make a fan poster.",
    );
}

#[test]
fn correct_compulsing() {
    assert_suggestion_result(
        "We have an button enabled to prompt user to download the app whenever we find difference in version number in our servlet war file and apk verision compulsing user to update.",
        test_linter(),
        "We have an button enabled to prompt user to download the app whenever we find difference in version number in our servlet war file and apk verision compelling user to update.",
    );
}

// ConfirmThat

#[test]
fn correct_conform_that() {
    assert_suggestion_result(
        "the WCAG requires every view of the page to conform that we move this",
        test_linter(),
        "the WCAG requires every view of the page to confirm that we move this",
    );
}

#[test]
fn corrects_conformed_that() {
    assert_suggestion_result(
        "I have conformed that works now.",
        test_linter(),
        "I have confirmed that works now.",
    );
}

#[test]
fn corrects_conforms_that() {
    assert_suggestion_result(
        "I conformed that with the correct configuration, this is working correctly.",
        test_linter(),
        "I confirmed that with the correct configuration, this is working correctly.",
    );
}

#[test]
#[ignore = "False positive not yet handled."]
fn dont_flag_conforming_that() {
    assert_lint_count(
        "is there any example of a case that isn't fully conforming that is supported today?",
        test_linter(),
        0,
    );
}

#[test]
fn corrects_conforming_that() {
    assert_suggestion_result(
        "Thanks for conforming that this issue is fixed in the latest version.",
        test_linter(),
        "Thanks for confirming that this issue is fixed in the latest version.",
    );
}

// ConstituteAs

#[test]
fn corrects_constitute_as() {
    assert_suggestion_result(
        "This doesn't really constitute as an implicit cast in the eyes of the system.",
        test_linter(),
        "This doesn't really constitute an implicit cast in the eyes of the system.",
    );
}

#[test]
fn corrects_constituted_as() {
    assert_suggestion_result(
        "We do not recommend setting the number of threads to more than 20, as that can be constituted as a denial of service attack which we are not responsible for.",
        test_linter(),
        "We do not recommend setting the number of threads to more than 20, as that can be constituted a denial of service attack which we are not responsible for.",
    );
}

#[test]
fn corrects_constitutes_as() {
    assert_suggestion_result(
        "Hello! I was just wondering what constitutes as a prompt in GitHub CoPilot that consumes premium request tokens.",
        test_linter(),
        "Hello! I was just wondering what constitutes a prompt in GitHub CoPilot that consumes premium request tokens.",
    );
}

#[test]
fn corrects_constituting_as_example() {
    assert_suggestion_result(
        "This is the example constituting as hull-demo 's values.yaml",
        test_linter(),
        "This is the example constituting hull-demo 's values.yaml",
    );
}

#[test]
#[ignore = "Not sure if this would be a false positive or a true positive"]
fn ambiguous_constituting_as() {
    assert_suggestion_result(
        "Note that spinning up a Client is a non-trivial operation, constituting as much as a millisecond of overhead.",
        test_linter(),
        // Maybe this one was supposed to be "contributing"?
        "Note that spinning up a Client is a non-trivial operation, constituting as much as a millisecond of overhead.",
    );
}

// DefiniteArticle

#[test]
fn corrects_definite_article() {
    assert_suggestion_result(
        "As for format of outputs: the spec defines the field as using the singular definitive article \"the\"",
        test_linter(),
        "As for format of outputs: the spec defines the field as using the singular definite article \"the\"",
    );
}

#[test]
#[ignore = "Title case capitalization problem causes this one to fail too."]
fn corrects_definite_articles_title_case() {
    assert_suggestion_result(
        "01 Definitive Articles: De or Het. Before starting more complicated topics in Dutch grammar, you should be aware of the articles.",
        test_linter(),
        "01 Definite Articles: De or Het. Before starting more complicated topics in Dutch grammar, you should be aware of the articles.",
    );
}

#[test]
fn corrects_definite_articles_lowercase() {
    assert_suggestion_result(
        ".. definitive articles -та /-ta/ and -те /-te/ (postfixed in Bulgarian).",
        test_linter(),
        ".. definite articles -та /-ta/ and -те /-te/ (postfixed in Bulgarian).",
    );
}

// DigestiveTract

#[test]
fn dont_flag_digestive_track() {
    assert_suggestion_result(
        "In infants less than a year old, because their digestive track is not finished developing yet",
        test_linter(),
        "In infants less than a year old, because their digestive tract is not finished developing yet",
    );
}

#[test]
fn corrects_digestive_tracks() {
    assert_suggestion_result(
        "The digestive tracks of mammals are complex and diverse, with each species having its own unique digestive system.",
        test_linter(),
        "The digestive tracts of mammals are complex and diverse, with each species having its own unique digestive system.",
    );
}

// Discuss
// -none-

// DoesOrDose

// -does not-
#[test]
fn corrects_dose_not() {
    assert_suggestion_result(
        "It dose not run windows ?",
        test_linter(),
        "It does not run windows ?",
    );
}

// -dose it true positive-
#[test]
#[ignore = "due to false positives this can't be fixed yet"]
fn corrects_dose_it() {
    assert_suggestion_result(
        "dose it support zh_cn ？",
        test_linter(),
        "does it support zh_cn ？",
    );
}

// -dose it- noun false positives

// it should be noted that (in an excessive dose) (it might have an opposite effect)
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_excessive_dose_it_might() {
    assert_lint_count(
        "it should be noted that in an excessive dose it might have an opposite effect",
        test_linter(),
        0,
    );
}

// When the person receives (a prescribed second dose) (it is not counted ttwice)
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_second_dose_it_is_not() {
    assert_lint_count(
        "When the person receives a prescribed second dose it is not counted ttwice",
        test_linter(),
        0,
    );
}

// (At that small a dose) (it was pleasent).
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_a_dose_it_was() {
    assert_lint_count("At that small a dose it was pleasent.", test_linter(), 0);
}

// I do not know (what dose) (it takes) to trip out, but I don't think I could stay awake to find out.
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_what_dose_it_takes() {
    assert_lint_count(
        "I do not know what dose it takes to trip out, but I don't think I could stay awake to find out.",
        test_linter(),
        0,
    );
}

// -dose it- verb false positives

#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_to_dose_it() {
    assert_lint_count(
        "And then I have to re-add the salts back to it to dose it back up to drinkable.",
        test_linter(),
        0,
    );
}

#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_dont_dose_it_too_high() {
    assert_lint_count(
        "So my conclusion is: don't dose it too high or it actually is dangerous and not pleasant at all",
        test_linter(),
        0,
    );
}

#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_to_dose_it_off() {
    assert_lint_count(
        "the only solution the other hopefully-dominant-reasonable-adult-human mind can find, is to dose it off, hoping the drowsiness can keep the fear at bay",
        test_linter(),
        0,
    );
}

// -he/she/it does-
#[test]
fn corrects_he_does() {
    assert_suggestion_result(
        "This validate each and every field of your from with nice dotted red color warring for the user, incase he dose some mistakes.",
        test_linter(),
        "This validate each and every field of your from with nice dotted red color warring for the user, incase he does some mistakes.",
    );
}

#[test]
fn corrects_she_does() {
    assert_suggestion_result(
        "we wont agree on everything she dose thats what a real person would feel like",
        test_linter(),
        "we wont agree on everything she does thats what a real person would feel like",
    );
}

// -it does-
#[test]
fn corrects_it_dose() {
    assert_suggestion_result(
        "it dose work without WEBP enabled",
        test_linter(),
        "it does work without WEBP enabled",
    );
}

// -someone does-
#[test]
fn corrects_someone_dose() {
    assert_suggestion_result(
        "Hopefully someone dose, I'm not good at C programing....",
        test_linter(),
        "Hopefully someone does, I'm not good at C programing....",
    );
}

// -interrogatives-
#[test]
fn corrects_how_dose() {
    assert_suggestion_result(
        "How dose qsv-copy works?",
        test_linter(),
        "How does qsv-copy works?",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_how_dose_false_positive() {
    assert_lint_count(
        "Work in progress exploration of how dose modifications throughout a trial can also induce bias in the exposure-response relationships.",
        test_linter(),
        0,
    );
}

#[test]
fn corrects_what_dose() {
    assert_suggestion_result(
        "What dose this mean?",
        test_linter(),
        "What does this mean?",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_what_dose_false_positive() {
    assert_lint_count(
        "What dose of vitamin d should I take?",
        test_linter(),
        0,
    );
}


#[test]
fn corrects_when_dose() {
    assert_suggestion_result(
        "When dose reusebale variable sync between device? #2634",
        test_linter(),
        "When does reusebale variable sync between device? #2634",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_when_dose_false_positive() {
    assert_lint_count(
        "Should we remove the dose when dose has been applied",
        test_linter(),
        0,
    );
}

#[test]
fn corrects_where_dose() {
    assert_suggestion_result(
        "where dose the password store?",
        test_linter(),
        "where does the password store?",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_where_dose_false_positive() {
    assert_lint_count(
        "added some better error handling for the weird case where dose files have no dose...",
        test_linter(),
        0,
    );
}

#[test]
fn corrects_who_dose() {
    assert_suggestion_result(
        "Who dose knows the problem?",
        test_linter(),
        "Who does knows the problem?",
    );
}

#[test]
fn corrects_why_dose() {
    assert_suggestion_result(
        "why dose the path is random ?",
        test_linter(),
        "why does the path is random ?",
    );
}

// Note: no false positive detected for 'why does'. Only true positives.

// ExpandAlgorithm

#[test]
fn corrects_algo() {
    assert_suggestion_result(
        "Always glad when the algo feeds me a new dissident.",
        test_linter(),
        "Always glad when the algorithm feeds me a new dissident.",
    );
}

#[test]
fn corrects_algos() {
    assert_suggestion_result(
        "I moved algos development to a private repository.",
        test_linter(),
        "I moved algorithms development to a private repository.",
    );
}

// ExpandArgument

#[test]
fn corrects_arg() {
    assert_suggestion_result(
        "but I cannot figure out how to flag an arg as required",
        test_linter(),
        "but I cannot figure out how to flag an argument as required",
    );
}

#[test]
fn corrects_args() {
    assert_suggestion_result(
        "but every test I've done shows args as being about 65% faster",
        test_linter(),
        "but every test I've done shows arguments as being about 65% faster",
    );
}

// ExpandCoordinate

#[test]
fn corrects_coord() {
    assert_suggestion_result(
        "Prompted by #5684, we should probably emit more meaningful messages when position guides are specified in coord systems that do not support them",
        test_linter(),
        "Prompted by #5684, we should probably emit more meaningful messages when position guides are specified in coordinate systems that do not support them",
    );
}

#[test]
fn corrects_coords() {
    assert_suggestion_result(
        "Here is how you can extract the list of coords from any geometry:",
        test_linter(),
        "Here is how you can extract the list of coordinates from any geometry:",
    );
}

// ExpandDecl

#[test]
fn corrects_decl() {
    assert_suggestion_result(
        "Yeah, I agree a forward decl would be preferable in this case.",
        test_linter(),
        "Yeah, I agree a forward declaration would be preferable in this case.",
    );
}

#[test]
fn corrects_decls() {
    assert_suggestion_result(
        "Accessing type decls from pointer types",
        test_linter(),
        "Accessing type declarations from pointer types",
    );
}

// ExpandDependency
// -none-

// ExpandDereference

#[test]
fn expand_deref() {
    assert_suggestion_result(
        "Should raw pointer deref/projections have to be in-bounds?",
        test_linter(),
        "Should raw pointer dereference/projections have to be in-bounds?",
    );
}

#[test]
fn corrects_derefs() {
    assert_suggestion_result(
        "A contiguous-in-memory double-ended queue that derefs into a slice - gnzlbg/slice_deque.",
        test_linter(),
        "A contiguous-in-memory double-ended queue that dereferences into a slice - gnzlbg/slice_deque.",
    );
}

// ExpandDirectory

#[test]
fn expands_dir() {
    assert_suggestion_result(
        "Error: library dir does not exist: /Users/u/trr/node_modules/opencv",
        test_linter(),
        "Error: library directory does not exist: /Users/u/trr/node_modules/opencv",
    );
}

#[test]
fn expands_dirs() {
    assert_suggestion_result(
        "Dirs/files are missing when scanning on windows after 1.27.12",
        test_linter(),
        "Directories/files are missing when scanning on windows after 1.27.12",
    );
}

// ExpandNotification

#[test]
fn corrects_notif() {
    assert_suggestion_result(
        "Amazing to see the notif of this on my phone!",
        test_linter(),
        "Amazing to see the notification of this on my phone!",
    );
}

#[test]
fn corrects_notifs() {
    assert_suggestion_result(
        "I don't encourage you spending all your time on social media or keeping the notifs on if you're working on something serious.",
        test_linter(),
        "I don't encourage you spending all your time on social media or keeping the notifications on if you're working on something serious.",
    );
}

// ExpandParam

#[test]
fn corrects_param() {
    assert_suggestion_result(
        "If I use the following to set an endDate param with a default value",
        test_linter(),
        "If I use the following to set an endDate parameter with a default value",
    );
}

#[test]
fn corrects_params() {
    assert_suggestion_result(
        "the params are not loaded in the R environment when using the terminal",
        test_linter(),
        "the parameters are not loaded in the R environment when using the terminal",
    );
}

// ExpandPointer

fn correct_ptr() {
    assert_suggestion_result(
        "How else would you construct a slice from a ptr and a length?",
        test_linter(),
        "How else would you construct a slice from a pointer and a length?",
    );
}

fn correct_ptrs() {
    assert_suggestion_result(
        "FixedBufferAllocator.free not freeing ptrs",
        test_linter(),
        "FixedBufferAllocator.free not freeing pointers",
    );
}

// ExpandSpecification

// ExpandStandardInput
// -none-

// ExpandStandardOutput
// -none-

// ExpandVulnerability

#[test]
fn corrects_vuln() {
    assert_suggestion_result(
        "I did not understand this vuln in first place now I do not understand in 2nd place as well😢",
        test_linter(),
        "I did not understand this vulnerability in first place now I do not understand in 2nd place as well😢",
    );
}

#[test]
fn corrects_vulns() {
    // Fix just this lint
    assert_suggestion_result(
        "... when persisted, containing endpoints, vulns, WAF bypasses, sensitive params, and auth endpoints.",
        single_lint("ExpandVulnerability"),
        "... when persisted, containing endpoints, vulnerabilities, WAF bypasses, sensitive params, and auth endpoints.",
    );
    // Fix all lints in the `LintGroup`
    assert_suggestion_result(
        "... when persisted, containing endpoints, vulns, WAF bypasses, sensitive params, and auth endpoints.",
        test_linter(),
        "... when persisted, containing endpoints, vulnerabilities, WAF bypasses, sensitive parameters, and auth endpoints.",
    );
}

// ExplanationMark
#[test]
fn detect_explanation_mark_atomic() {
    assert_suggestion_result("explanation mark", test_linter(), "exclamation mark");
}

#[test]
fn detect_explanation_marks_atomic() {
    assert_suggestion_result("explanation marks", test_linter(), "exclamation marks");
}

#[test]
fn detect_explanation_mark_real_world() {
    assert_suggestion_result(
        "Note that circled explanation mark, question mark, plus and arrows may be significantly harder to distinguish than their uncircled variants.",
        test_linter(),
        "Note that circled exclamation mark, question mark, plus and arrows may be significantly harder to distinguish than their uncircled variants.",
    );
}

#[test]
fn detect_explanation_marks_real_world() {
    assert_suggestion_result(
        "this issue: html: properly handle explanation marks in comments",
        test_linter(),
        "this issue: html: properly handle exclamation marks in comments",
    );
}

#[test]
fn detect_explanation_point_atomic() {
    assert_suggestion_result("explanation point", test_linter(), "exclamation point");
}

#[test]
fn detect_explanation_point_real_world() {
    assert_suggestion_result(
        "js and makes an offhand mention that you can disable inbuilt plugin with an explanation point (e.g. !error ).",
        test_linter(),
        "js and makes an offhand mention that you can disable inbuilt plugin with an exclamation point (e.g. !error ).",
    );
}

// ExtendOrExtent

#[test]
fn correct_certain_extend() {
    assert_suggestion_result(
        "This is a PowerShell script to automate client pentests / checkups - at least to a certain extend.",
        test_linter(),
        "This is a PowerShell script to automate client pentests / checkups - at least to a certain extent.",
    );
}

#[test]
fn correct_to_the_extend() {
    assert_suggestion_result(
        "Our artifacts are carefully documented and well-structured to the extend that reuse is facilitated.",
        test_linter(),
        "Our artifacts are carefully documented and well-structured to the extent that reuse is facilitated.",
    );
}

#[test]
fn correct_to_some_extend() {
    assert_suggestion_result(
        "Hi, I'm new to Pydantic and to some extend python, and I have a question that I haven't been able to figure out from the Docs.",
        test_linter(),
        "Hi, I'm new to Pydantic and to some extent python, and I have a question that I haven't been able to figure out from the Docs.",
    );
}

#[test]
fn correct_to_an_extend() {
    assert_suggestion_result(
        "It mimics (to an extend) the way in which Chrome requests SSO cookies with the Windows 10 accounts extension.",
        test_linter(),
        "It mimics (to an extent) the way in which Chrome requests SSO cookies with the Windows 10 accounts extension.",
    );
}

// FlauntForFlout

#[test]
fn corrects_flaunt_the_rules() {
    assert_suggestion_result(
        "Some users flaunt the rules of punctuation.",
        test_linter(),
        "Some users flout the rules of punctuation.",
    );
}

#[test]
fn corrects_flaunted_the_law() {
    assert_suggestion_result(
        "He flaunted the law for personal gain.",
        test_linter(),
        "He flouted the law for personal gain.",
    );
}

#[test]
fn corrects_flaunting_authority() {
    assert_suggestion_result(
        "She was flaunting authority at every turn.",
        test_linter(),
        "She was flouting authority at every turn.",
    );
}

#[test]
fn allows_flaunt_wealth() {
    assert_no_lints("He likes to flaunt his wealth.", test_linter());
}

// FoamAtTheMouth

#[test]
fn correct_foam_out_the_mouth() {
    assert_suggestion_result(
        "and he gave him a drink that made him foam out the mouth and die",
        test_linter(),
        "and he gave him a drink that made him foam at the mouth and die",
    );
}

#[test]
fn correct_foamed_out_the_mouth() {
    assert_suggestion_result(
        "You can see in some shots they've foamed out the mouth, and it's apparent their poisoned.",
        test_linter(),
        "You can see in some shots they've foamed at the mouth, and it's apparent their poisoned.",
    );
}

#[test]
fn correct_foaming_out_the_mouth() {
    assert_suggestion_result(
        "choking or foaming out the mouth or something like that, leading up to death",
        test_linter(),
        "choking or foaming at the mouth or something like that, leading up to death",
    );
}

#[test]
fn correct_foams_out_the_mouth() {
    assert_suggestion_result(
        "Elaine can't swallow, foams out the mouth and Kramer says she has rabies just like his friend Bob Sacamano after she gets bit by the guy's dog",
        test_linter(),
        "Elaine can't swallow, foams at the mouth and Kramer says she has rabies just like his friend Bob Sacamano after she gets bit by the guy's dog",
    );
}

// FootTheBill

#[test]
fn correct_flip_the_bill() {
    assert_suggestion_result(
        "- SQL Compare (If the company will flip the bill)",
        test_linter(),
        "- SQL Compare (If the company will foot the bill)",
    );
}

#[test]
fn correct_flipped_the_bill() {
    assert_suggestion_result(
        "As a meetup we were extremely lucky that NOVI flipped the bill for our in-person events.",
        test_linter(),
        "As a meetup we were extremely lucky that NOVI footed the bill for our in-person events.",
    );
}

#[test]
fn correct_flipping_the_bill() {
    assert_suggestion_result(
        "for the simple reason that there were no multimillion dollar company flipping the bill",
        test_linter(),
        "for the simple reason that there were no multimillion dollar company footing the bill",
    );
}

#[test]
fn correct_flips_the_bill() {
    assert_suggestion_result(
        "There seems to be a perennial debate in Illinois between urbanites and rural folk about who really flips the bill.",
        test_linter(),
        "There seems to be a perennial debate in Illinois between urbanites and rural folk about who really foots the bill.",
    );
}

// GetUsedTo

//-get used of-
#[test]
fn corrects_get_used_of() {
    assert_suggestion_result(
        "I am following the examples in the documentation in order to get used of comets.",
        test_linter(),
        "I am following the examples in the documentation in order to get used to comets.",
    );
}

//-gets used of-
#[test]
fn corrects_gets_used_of() {
    assert_suggestion_result(
        "its like she gets used of her food and becomes spoiled",
        test_linter(),
        "its like she gets used to her food and becomes spoiled",
    );
}

//-getting used of-
#[test]
fn corrects_getting_used_of() {
    assert_suggestion_result(
        "Here you can find a guide to getting used of the most important methods of magum.",
        test_linter(),
        "Here you can find a guide to getting used to the most important methods of magum.",
    );
}

//-got used of-
#[test]
fn corrects_got_used_of() {
    assert_suggestion_result(
        "we users actually got used of such delays",
        test_linter(),
        "we users actually got used to such delays",
    );
}

//-gotten used of-
#[test]
fn corrects_gotten_used_of() {
    assert_suggestion_result(
        "The tutorial has indeed been of help, and I've gotten used of using Hull.",
        test_linter(),
        "The tutorial has indeed been of help, and I've gotten used to using Hull.",
    );
}

// GrindToAHalt

#[test]
fn corrects_grind_to_halt() {
    // Without this it will eventually grind to halt as it backs up upon itself
    assert_suggestion_result(
        "Without this it will eventually grind to halt as it backs up upon itself",
        test_linter(),
        "Without this it will eventually grind to a halt as it backs up upon itself",
    );
}

#[test]
#[ignore = "Fails due to how replace_with_matched_case works"]
fn corrects_grind_to_halt_title_case() {
    assert_suggestion_result(
        "Smart Search Tools Cause System to Grind to Halt",
        test_linter(),
        "Smart Search Tools Cause System to Grind to a Halt",
    );
}

#[test]
fn corrects_grinding_to_halt() {
    assert_suggestion_result(
        "app grinding to halt when loading many objects",
        test_linter(),
        "app grinding to a halt when loading many objects",
    );
}

#[test]
fn corrects_grinds_to_halt() {
    assert_suggestion_result(
        "If your machine grinds to halt due to memory oversubscription, you may want to try to set the MOLD_JOBS environment variable to 1",
        test_linter(),
        "If your machine grinds to a halt due to memory oversubscription, you may want to try to set the MOLD_JOBS environment variable to 1",
    );
}

#[test]
fn corrects_ground_to_halt() {
    assert_suggestion_result(
        "As you have probably guessed, my work on my fork has ground to halt.",
        test_linter(),
        "As you have probably guessed, my work on my fork has ground to a halt.",
    );
}

// HavePassed

#[test]
fn correct_has_past() {
    assert_suggestion_result(
        "Track the amount of time that has past since a point in time.",
        test_linter(),
        "Track the amount of time that has passed since a point in time.",
    );
}

#[test]
fn correct_have_past() {
    assert_suggestion_result(
        "Another 14+ days have past, any updates on this?",
        test_linter(),
        "Another 14+ days have passed, any updates on this?",
    );
}

#[test]
fn correct_had_past() {
    assert_suggestion_result(
        "Few days had past, so im starting to thinks there is a problem in my local version.",
        test_linter(),
        "Few days had passed, so im starting to thinks there is a problem in my local version.",
    );
}

#[test]
fn correct_having_past() {
    assert_suggestion_result(
        "Return to computer, with enough time having past for the computer to go to full sleep.",
        test_linter(),
        "Return to computer, with enough time having passed for the computer to go to full sleep.",
    );
}

// HitTheNailOnTheHead

#[test]
fn correct_hit_the_nail() {
    assert_suggestion_result(
        "Ahh, found it! You hit the nail in the head once again.",
        test_linter(),
        "Ahh, found it! You hit the nail on the head once again.",
    );
}

#[test]
fn correct_hits_the_nail() {
    assert_suggestion_result(
        "I'm not sure if this sentence hits the nail in the head",
        test_linter(),
        "I'm not sure if this sentence hits the nail on the head",
    );
}

#[test]
fn correct_hitting_the_nail() {
    assert_suggestion_result(
        "You are hitting the nail in the head of my issue with this game, too.",
        test_linter(),
        "You are hitting the nail on the head of my issue with this game, too.",
    );
}

#[test]
fn correct_hitted_the_nail() {
    assert_suggestion_result(
        "I mean, you just kinda hitted the nail in the head. You cannot do anything with this that you couldn't do in a Raspberry PI.",
        test_linter(),
        "I mean, you just kinda hitted the nail on the head. You cannot do anything with this that you couldn't do in a Raspberry PI.",
    );
}

// HomeInOn

#[test]
fn correct_hone_in_on() {
    assert_suggestion_result(
        "This way you can use an object detector algorithm to hone in on subjects and tell sam to only focus in certain areas when looking to extend ...",
        test_linter(),
        "This way you can use an object detector algorithm to home in on subjects and tell sam to only focus in certain areas when looking to extend ...",
    );
}

#[test]
fn correct_honing_in_on() {
    assert_suggestion_result(
        "I think I understand the syntax limitation you're honing in on.",
        test_linter(),
        "I think I understand the syntax limitation you're homing in on.",
    );
}

#[test]
fn correct_hones_in_on() {
    assert_suggestion_result(
        "[FEATURE] Add a magnet that hones in on mobs",
        test_linter(),
        "[FEATURE] Add a magnet that homes in on mobs",
    );
}

#[test]
fn correct_honed_in_on() {
    assert_suggestion_result(
        "But it took me quite a bit of faffing about checking things out before I honed in on the session as the problem and tried to dump out the ...",
        test_linter(),
        "But it took me quite a bit of faffing about checking things out before I homed in on the session as the problem and tried to dump out the ...",
    );
}

// InDetail

// -in details-
#[test]
fn in_detail_atomic() {
    assert_suggestion_result("in details", test_linter(), "in detail");
}

#[test]
fn in_detail_real_world() {
    assert_suggestion_result(
        "c++ - who can tell me \"*this pointer\" in details?",
        test_linter(),
        "c++ - who can tell me \"*this pointer\" in detail?",
    )
}

// -in more details-
#[test]
fn in_more_detail_atomic() {
    assert_suggestion_result("in more details", test_linter(), "in more detail");
}

#[test]
fn in_more_detail_real_world() {
    assert_suggestion_result(
        "Document the interface in more details · Issue #3 · owlbarn ...",
        test_linter(),
        "Document the interface in more detail · Issue #3 · owlbarn ...",
    );
}

// InThisThatRegard

#[test]
fn fix_in_this_regards() {
    assert_suggestion_result(
        "I am testing many apps for our custom TROMjaro Linux, so I can be helpful in this regards.",
        test_linter(),
        "I am testing many apps for our custom TROMjaro Linux, so I can be helpful in this regard.",
    );
}

#[test]
fn fix_in_that_regards() {
    assert_suggestion_result(
        "Looks like that are all settings I can make in the Buderus in that regards.",
        test_linter(),
        "Looks like that are all settings I can make in the Buderus in that regard.",
    );
}

// InflectionPoint

#[test]
fn corrects_infliction_point() {
    assert_suggestion_result(
        "You can also position the infliction point of the curve. By default it's exactly at the center in between the two connecting nodes.",
        test_linter(),
        "You can also position the inflection point of the curve. By default it's exactly at the center in between the two connecting nodes.",
    );
}

#[test]
fn corrects_infliction_points() {
    assert_suggestion_result(
        "... find where it touches the other side, and measure the distance. Potentially, I'd only have to do it for \"infliction points\".",
        test_linter(),
        "... find where it touches the other side, and measure the distance. Potentially, I'd only have to do it for \"inflection points\".",
    );
}

// InvestIn

#[test]
fn corrects_invest_into() {
    assert_suggestion_result(
        "which represents the amount of money they want to invest into a particular deal.",
        test_linter(),
        "which represents the amount of money they want to invest in a particular deal.",
    );
}

#[test]
fn corrects_investing_into() {
    assert_suggestion_result(
        "Taking dividends in cash (rather than automatically re-investing into the originating fund) can help alleviate the need for rebalancing.",
        test_linter(),
        "Taking dividends in cash (rather than automatically re-investing in the originating fund) can help alleviate the need for rebalancing.",
    );
}

#[test]
fn corrects_invested_into() {
    assert_suggestion_result(
        "it's all automatically invested into a collection of loans that match the criteria that ...",
        test_linter(),
        "it's all automatically invested in a collection of loans that match the criteria that ...",
    );
}

#[test]
fn corrects_invests_into() {
    assert_suggestion_result(
        "If a user invests into the protocol first using USDC but afterward changing to DAI, ...",
        test_linter(),
        "If a user invests in the protocol first using USDC but afterward changing to DAI, ...",
    );
}

#[test]
fn corrects_investment_into() {
    assert_suggestion_result(
        "A $10,000 investment into the fund made on February 28, 1997 would have grown to a value of $42,650 at the end of the 20-year period.",
        test_linter(),
        "A $10,000 investment in the fund made on February 28, 1997 would have grown to a value of $42,650 at the end of the 20-year period.",
    );
}

// LayoutVerb

#[test]
fn corrects_layouted() {
    assert_suggestion_result(
        "only the views that neeed it will be measured and layouted when the superview changes",
        test_linter(),
        "only the views that neeed it will be measured and laid out when the superview changes",
    );
}

#[test]
fn corrects_layouting() {
    assert_suggestion_result(
        "An R package for layouting tables, using the S4 method",
        test_linter(),
        "An R package for laying out tables, using the S4 method",
    );
}

// LitotesDirectPositive

#[test]
fn litotes_not_uncommon_atomic() {
    assert_suggestion_result("not uncommon", test_linter(), "common");
}

#[test]
fn litotes_not_uncommon_sentence() {
    assert_suggestion_result(
        "It is not uncommon to see outages during storms.",
        test_linter(),
        "It is common to see outages during storms.",
    );
}

#[test]
fn litotes_not_unlikely() {
    assert_suggestion_result(
        "This outcome is not unlikely given the data.",
        test_linter(),
        "This outcome is likely given the data.",
    );
}

#[test]
fn litotes_not_insignificant() {
    assert_suggestion_result(
        "That is not insignificant progress.",
        test_linter(),
        "That is significant progress.",
    );
}

#[test]
fn litotes_more_preferable() {
    assert_suggestion_result(
        "Is it more preferable to use process.env.variable or env.parsed.variable?",
        test_linter(),
        "Is it preferable to use process.env.variable or env.parsed.variable?",
    );
}

// LookForwardTo

#[test]
fn fix_look_forward_for() {
    assert_suggestion_result(
        "I will mark this issue as an enhancement and will look forward for enrolling it.",
        test_linter(),
        "I will mark this issue as an enhancement and will look forward to enrolling it.",
    );
}

#[test]
fn fix_looked_forward_for() {
    assert_suggestion_result(
        "Looked forward for standalone components so much, please fix this.",
        test_linter(),
        "Looked forward to standalone components so much, please fix this.",
    );
}

#[test]
fn fix_looking_forward_for() {
    assert_suggestion_result(
        "Looking forward for Typed version of this stack navigation",
        test_linter(),
        "Looking forward to Typed version of this stack navigation",
    );
}

#[test]
fn fix_looks_forward_for() {
    assert_suggestion_result(
        "Please take this words as from one of your fans who looks forward for a great and interesting project :)",
        test_linter(),
        "Please take this words as from one of your fans who looks forward to a great and interesting project :)",
    );
}

// MakeDoWith

#[test]
fn corrects_make_due_with() {
    assert_suggestion_result(
        "For now, I can make due with a bash script I have",
        test_linter(),
        "For now, I can make do with a bash script I have",
    );
}

#[test]
fn corrects_made_due_with() {
    assert_suggestion_result(
        "I made due with using actions.push for now but will try to do a codepen soon",
        test_linter(),
        "I made do with using actions.push for now but will try to do a codepen soon",
    );
}

#[test]
fn corrects_makes_due_with() {
    assert_suggestion_result(
        "but the code makes due with what is available",
        test_linter(),
        "but the code makes do with what is available",
    );
}

#[test]
fn corrects_making_due_with() {
    assert_suggestion_result(
        "I've been making due with the testMultiple script I wrote above.",
        test_linter(),
        "I've been making do with the testMultiple script I wrote above.",
    );
}

// MakeSense

#[test]
fn fix_make_senses() {
    assert_suggestion_result(
        "some symbols make senses only if you have a certain keyboard",
        test_linter(),
        "some symbols make sense only if you have a certain keyboard",
    );
}

#[test]
fn fix_made_senses() {
    assert_suggestion_result(
        "Usually on the examples of matlab central I have found all with positive magnitude and made senses to me.",
        test_linter(),
        "Usually on the examples of matlab central I have found all with positive magnitude and made sense to me.",
    );
}

#[test]
fn fix_makes_senses() {
    assert_suggestion_result(
        "If it makes senses I can open a PR.",
        test_linter(),
        "If it makes sense I can open a PR.",
    );
}

#[test]
fn fix_making_senses() {
    assert_suggestion_result(
        "I appreciate you mentioned the two use cases, which are making senses for both.",
        test_linter(),
        "I appreciate you mentioned the two use cases, which are making sense for both.",
    );
}

// MootPoint

// -point is mute-
#[test]
fn point_is_moot() {
    assert_suggestion_result("Your point is mute.", test_linter(), "Your point is moot.");
}

// OperatingSystem

#[test]
fn operative_system() {
    assert_suggestion_result(
        "COS is a operative system made with the COSMOS Kernel and written in C#, COS its literally the same than MS-DOS but written in C# and open-source.",
        test_linter(),
        "COS is a operating system made with the COSMOS Kernel and written in C#, COS its literally the same than MS-DOS but written in C# and open-source.",
    );
}

#[test]
fn operative_systems() {
    assert_suggestion_result(
        "My dotfiles for my operative systems and other configurations.",
        test_linter(),
        "My dotfiles for my operating systems and other configurations.",
    );
}

// PassersBy
#[test]
fn correct_passerbys() {
    assert_suggestion_result(
        "For any passerbys, you may replace visibility: hidden/collapsed with: opacity: 0; pointer-events: none;.",
        test_linter(),
        "For any passersby, you may replace visibility: hidden/collapsed with: opacity: 0; pointer-events: none;.",
    );
}

#[test]
fn correct_passer_bys_hyphen() {
    assert_suggestion_result(
        "Is there any way for random willing passer-bys to help with this effort?",
        test_linter(),
        "Is there any way for random willing passers-by to help with this effort?",
    );
}

// PeekBehindTheCurtain

#[test]
fn fix_peak() {
    assert_suggestion_result(
        "Offer a peak behind the curtain of what I look for when baselining a software installation.",
        test_linter(),
        "Offer a peek behind the curtain of what I look for when baselining a software installation.",
    );
}

#[test]
fn fix_peaked() {
    assert_suggestion_result(
        "I peaked behind the curtain of the new Autodraw tool and noticed some expected similarities to what I saw in Quickdraw.",
        test_linter(),
        "I peeked behind the curtain of the new Autodraw tool and noticed some expected similarities to what I saw in Quickdraw.",
    );
}

#[test]
fn fix_peaking() {
    assert_suggestion_result(
        "I can see how peaking behind the curtain got me to where I am today.",
        test_linter(),
        "I can see how peeking behind the curtain got me to where I am today.",
    );
}

#[test]
fn fix_peaks() {
    assert_suggestion_result(
        "The Daily Vlog Series that peaks behind the curtain of an Entrepreneur's day to day life in 2016 building a business.",
        test_linter(),
        "The Daily Vlog Series that peeks behind the curtain of an Entrepreneur's day to day life in 2016 building a business.",
    );
}

// Piggyback
// -none-

// Provocate

#[test]
fn fix_provocate() {
    assert_suggestion_result(
        "Hardcoded chainId can provocate a possible replay attacks between chains in the event of a future chain split.",
        test_linter(),
        "Hardcoded chainId can provoke a possible replay attacks between chains in the event of a future chain split.",
    );
}

#[test]
fn fix_provocated() {
    assert_suggestion_result(
        "tempering with inconsistent content lengths provocated the error which lead me to encoding?",
        test_linter(),
        "tempering with inconsistent content lengths provoked the error which lead me to encoding?",
    );
}

#[test]
fn fix_provocates() {
    assert_suggestion_result(
        "it wont mark the check on the solving square, it provocates a timeout and return 500 Server Error",
        test_linter(),
        "it wont mark the check on the solving square, it provokes a timeout and return 500 Server Error",
    );
}

#[test]
fn fix_provocating() {
    assert_suggestion_result(
        "could return incorrect balances provocating an incorrect calculation of rsETH price",
        test_linter(),
        "could return incorrect balances provoking an incorrect calculation of rsETH price",
    );
}

// RedundantSuperlatives

#[test]
fn redundant_more_optimal() {
    assert_suggestion_result("Is this more optimal?", test_linter(), "Is this optimal?");
}

#[test]
fn redundant_most_ideal() {
    assert_suggestion_result(
        "This is the most ideal scenario.",
        test_linter(),
        "This is the ideal scenario.",
    );
}

// ResponsibilityFor

#[test]
fn fix_take() {
    assert_suggestion_result(
        "Is anyone wanting to step up and take responsibility of this library, or should I put it in EOL and redirect to another tool? ",
        test_linter(),
        "Is anyone wanting to step up and take responsibility for this library, or should I put it in EOL and redirect to another tool? ",
    );
}

#[test]
fn fix_taken() {
    assert_suggestion_result(
        "if it had only taken responsibility of the manifest/info additions and extensionsID it would have made our life easier",
        test_linter(),
        "if it had only taken responsibility for the manifest/info additions and extensionsID it would have made our life easier",
    );
}

#[test]
fn fix_takes() {
    assert_suggestion_result(
        "If I have a message that i want to encode, who takes responsibility of pointers?",
        test_linter(),
        "If I have a message that i want to encode, who takes responsibility for pointers?",
    );
}

#[test]
fn fix_taking() {
    assert_suggestion_result(
        "This issue is about taking responsibility of the feature area auto indentation and start solving the bugs in the feature area.",
        test_linter(),
        "This issue is about taking responsibility for the feature area auto indentation and start solving the bugs in the feature area.",
    );
}

#[test]
fn fix_took() {
    assert_suggestion_result(
        "If the driver took responsibility of the locking, it could let these HTTP calls happen in parallel",
        test_linter(),
        "If the driver took responsibility for the locking, it could let these HTTP calls happen in parallel",
    );
}

#[test]
fn fix_assume() {
    assert_suggestion_result(
        "it's a relatively big chunk of behavior to assume responsibility of",
        test_linter(),
        "it's a relatively big chunk of behavior to assume responsibility for",
    );
}

#[test]
fn fix_assumed() {
    assert_suggestion_result(
        "and assumed responsibility of project managing the transition of Barclays",
        test_linter(),
        "and assumed responsibility for project managing the transition of Barclays",
    );
}

#[test]
fn fix_assumes() {
    assert_suggestion_result(
        "It means that the core development team assumes responsibility of the module",
        test_linter(),
        "It means that the core development team assumes responsibility for the module",
    );
}

#[test]
fn fix_assuming() {
    assert_suggestion_result(
        "The point of extract is essentially that you're assuming responsibility of maintenance for that version of the formula.",
        test_linter(),
        "The point of extract is essentially that you're assuming responsibility for maintenance for that version of the formula.",
    );
}

#[test]
fn fix_claim() {
    assert_suggestion_result(
        "so it doesn't need to claim responsibility of the reappearing containers lifecycle",
        test_linter(),
        "so it doesn't need to claim responsibility for the reappearing containers lifecycle",
    );
}

#[test]
fn fix_claimed() {
    assert_suggestion_result(
        "a group called The Impact Team had claimed responsibility of the data breach",
        test_linter(),
        "a group called The Impact Team had claimed responsibility for the data breach",
    );
}

#[test]
fn fix_claiming() {
    assert_suggestion_result(
        "I feel that there should be some other way of claiming responsibility of the promise's continuation.",
        test_linter(),
        "I feel that there should be some other way of claiming responsibility for the promise's continuation.",
    );
}

#[test]
fn fix_claims() {
    assert_suggestion_result(
        "yet the Lord claims responsibility of those boundaries",
        test_linter(),
        "yet the Lord claims responsibility for those boundaries",
    );
}

// ScapeGoat

#[test]
fn fix_an_escape_goat() {
    assert_suggestion_result(
        "I see too many times the cable and ps thingy being used as an escape goat.",
        test_linter(),
        "I see too many times the cable and ps thingy being used as a scapegoat.",
    );
}

#[test]
fn fix_escape_goat() {
    assert_suggestion_result(
        "It helps shift the reason for the failure on to what the manager did not do (making them the escape goat when it fails).",
        test_linter(),
        "It helps shift the reason for the failure on to what the manager did not do (making them the scapegoat when it fails).",
    );
}

#[test]
fn fix_escape_goats() {
    assert_suggestion_result(
        "People might be using Americans as escape goats for this, but these mishearings are becoming as common as a bowl in a china shop!",
        test_linter(),
        "People might be using Americans as scapegoats for this, but these mishearings are becoming as common as a bowl in a china shop!",
    );
}

// SeamToSeem

//-seam to be-
#[test]
fn fix_seam_to_be() {
    assert_suggestion_result(
        "amdvlk is deprecated but my system still uses it as default and I can't seam to be able to change it.",
        test_linter(),
        "amdvlk is deprecated but my system still uses it as default and I can't seem to be able to change it.",
    );
}

//-seams to be-
fn fix_seams_to_be() {
    assert_suggestion_result(
        "Problem: Docker image is seriously broken and everything seams to be related to trivial things like creating directory or dumping key",
        test_linter(),
        "Problem: Docker image is seriously broken and everything seems to be related to trivial things like creating directory or dumping key",
    );
}

//-I seam-
#[test]
fn fix_i_seam() {
    assert_suggestion_result(
        "so now whatever i seam to try it doesnt work",
        test_linter(),
        "so now whatever i seem to try it doesnt work",
    );
}

//-we seam-
#[test]
fn fix_we_seam() {
    assert_suggestion_result(
        "using a 4G network we seam to get ICE messages mixing Ipv6 and Ipv4",
        test_linter(),
        "using a 4G network we seem to get ICE messages mixing Ipv6 and Ipv4",
    );
}

//-we-all-seam-
#[test]
fn fix_we_all_seam() {
    assert_suggestion_result(
        "if it is your own nation then we all seam to get the update",
        test_linter(),
        "if it is your own nation then we all seem to get the update",
    );
}

//-we-both-seam-
#[test]
// because we both seam to have enough for frivolous things
fn fix_we_both_seam() {
    assert_suggestion_result(
        "because we both seam to have enough for frivolous things",
        test_linter(),
        "because we both seem to have enough for frivolous things",
    );
}

//-you seam-
#[test]
fn fix_you_seam() {
    assert_suggestion_result(
        "Assigning you, since you seam to have already made the fix.",
        test_linter(),
        "Assigning you, since you seem to have already made the fix.",
    );
}

//-you-all-seam
#[test]
fn fix_you_all_seam() {
    assert_suggestion_result(
        "That's a good advice which you all seam to agree upon.",
        test_linter(),
        "That's a good advice which you all seem to agree upon.",
    );
}

//-you-both-seam
#[test]
fn fix_you_both_seam() {
    assert_suggestion_result(
        "since you both seam to like the game",
        test_linter(),
        "since you both seem to like the game",
    );
}

//-he seams-
#[test]
fn fix_he_seams() {
    assert_suggestion_result(
        "tagging @PedroTroller as he seams to still be active on this project.",
        test_linter(),
        "tagging @PedroTroller as he seems to still be active on this project.",
    );
}

//-she seams-
#[test]
fn fix_she_seams() {
    assert_suggestion_result(
        "Here is the exact timestamp where she seams to talk about exactly this -> video.",
        test_linter(),
        "Here is the exact timestamp where she seems to talk about exactly this -> video.",
    );
}

//-it seams-
#[test]
fn fix_it_seams() {
    assert_suggestion_result(
        "It seams i cannot use $tries and $timeout properties on my queued listener class?",
        test_linter(),
        "It seems i cannot use $tries and $timeout properties on my queued listener class?",
    );
}

//-they seam-
#[test]
fn fix_they_seam() {
    assert_suggestion_result(
        "Lets start with the \"not\" and \"and\" gates because they seam the easiest.",
        test_linter(),
        "Lets start with the \"not\" and \"and\" gates because they seem the easiest.",
    );
}

//-they all seam-
#[test]
fn fix_they_all_seam() {
    assert_suggestion_result(
        "I have tried the sum, product, max and min functions and they all seam to work.",
        test_linter(),
        "I have tried the sum, product, max and min functions and they all seem to work.",
    );
}

//-they-both-seam-
#[test]
fn fix_they_both_seam() {
    assert_suggestion_result(
        "It's probably cause they both seam to combine martial arts with animal instincts",
        test_linter(),
        "It's probably cause they both seem to combine martial arts with animal instincts",
    );
}

//-everything seams-
#[test]
fn fix_everything_seams() {
    assert_suggestion_result(
        "Note that if you try to slider the slider first to the right and then to the left, everything seams alright.",
        test_linter(),
        "Note that if you try to slider the slider first to the right and then to the left, everything seems alright.",
    );
}

//-everybody seams-
#[test]
fn fix_everybody_seams() {
    assert_suggestion_result(
        "I'm currently a little disappointed because everybody seams to care only about the Rails framework",
        test_linter(),
        "I'm currently a little disappointed because everybody seems to care only about the Rails framework",
    );
}

//-everyone seams-
#[test]
fn fix_everyone_seams() {
    assert_suggestion_result(
        "everyone seams to use the editor now a days plus there is a tun of extensions available",
        single_lint("SeamToSeem"),
        "everyone seems to use the editor now a days plus there is a tun of extensions available",
    );
}

#[test]
fn fix_everyone_seams_combined_with_now_a_days() {
    assert_suggestion_result(
        "everyone seams to use the editor now a days plus there is a tun of extensions available",
        test_linter(),
        "everyone seems to use the editor nowadays plus there is a tun of extensions available",
    );
}

// SubjunctiveWasToWere

// -if only there was-
#[test]
fn if_only_there_was() {
    assert_suggestion_result(
        "if only there was an endpoint do to so",
        test_linter(),
        "if only there were an endpoint do to so",
    );
}

// -if only I-
#[test]
fn if_only_i_was() {
    assert_suggestion_result(
        "Oh If only I was that clever !!",
        test_linter(),
        "Oh If only I were that clever !!",
    );
}

// -if only he-
#[test]
fn if_only_he_was() {
    assert_suggestion_result(
        "If only he was kind enough to attempt to contact me in private first",
        test_linter(),
        "If only he were kind enough to attempt to contact me in private first",
    );
}

// -if only she-
#[test]
fn if_only_she_was() {
    assert_suggestion_result(
        "If only she was right.",
        test_linter(),
        "If only she were right.",
    );
}

// -it-
#[test]
fn if_only_it_was() {
    assert_suggestion_result(
        "if only it was accessible via USB connection - hint hint",
        test_linter(),
        "if only it were accessible via USB connection - hint hint",
    );
}

// -I wish there was-
#[test]
fn i_wish_there_was() {
    assert_suggestion_result(
        "I wish there was a keyboard shortcut or something that was \"bring back the suggestion you just made in the last 3 seconds\".",
        test_linter(),
        "I wish there were a keyboard shortcut or something that was \"bring back the suggestion you just made in the last 3 seconds\".",
    );
}

// -I wish I was-
#[test]
fn i_wish_i_was() {
    assert_suggestion_result(
        "I wish I was as smart as I think I am.",
        test_linter(),
        "I wish I were as smart as I think I am.",
    );
}

// -I wish he was-
#[test]
fn i_wish_he_was() {
    assert_suggestion_result(
        "However I wish he was that smart about ARM chips present in the current mobile devices.",
        test_linter(),
        "However I wish he were that smart about ARM chips present in the current mobile devices.",
    );
}

// -I wish she was-
#[test]
fn i_wish_she_was() {
    assert_suggestion_result(
        "I wish she was more accepting of her own interests.",
        test_linter(),
        "I wish she were more accepting of her own interests.",
    );
}

// -I wish it was-
#[test]
fn i_wish_it_was() {
    assert_suggestion_result(
        "but I wish it was more friendly to existing ecosystems",
        test_linter(),
        "but I wish it were more friendly to existing ecosystems",
    );
}

// TakeControlOf

#[test]
fn take() {
    assert_suggestion_result(
        "allowed .editorconfig to set editor ruler and take control over soft-wrap",
        test_linter(),
        "allowed .editorconfig to set editor ruler and take control of soft-wrap",
    );
}

#[test]
fn taken() {
    assert_suggestion_result(
        "I've taken control over the inputValue to be able to render the wanted menu items",
        test_linter(),
        "I've taken control of the inputValue to be able to render the wanted menu items",
    );
}

#[test]
fn takes() {
    assert_suggestion_result(
        "AI takes control over players hand",
        test_linter(),
        "AI takes control of players hand",
    );
}

#[test]
fn taking() {
    assert_suggestion_result(
        "this inconsistent behavior is very annoying for taking control over your dependency graph",
        test_linter(),
        "this inconsistent behavior is very annoying for taking control of your dependency graph",
    );
}

#[test]
fn took() {
    assert_suggestion_result(
        "Noted drone was NOT stoping and manually took control over it to stop it.",
        test_linter(),
        "Noted drone was NOT stoping and manually took control of it to stop it.",
    );
}

// UseToUsedTo

#[test]
fn corrects_getting_use_to() {
    assert_suggestion_result(
        "I'm getting use to it slowly.",
        test_linter(),
        "I'm getting used to it slowly.",
    );
}

#[test]
fn corrects_are_use_to() {
    assert_suggestion_result(
        "If you are use to Ubuntu, then the way sudo works should not be strange.",
        test_linter(),
        "If you are used to Ubuntu, then the way sudo works should not be strange.",
    );
}

#[test]
fn corrects_im_use_to() {
    assert_suggestion_result(
        "I'm use to doing a lot of work.",
        test_linter(),
        "I'm used to doing a lot of work.",
    );
}

#[test]
fn allows_use_to_as_verb() {
    assert_no_lints("This is the editor I use to write code.", test_linter());
}

#[test]
fn allows_used_to() {
    assert_no_lints("I used to develop with objects in JS.", test_linter());
}

// WreakHavoc

#[test]
fn fix_wreck_havoc() {
    assert_suggestion_result(
        "Tables with a \".\" in the name wreck havoc with the system",
        test_linter(),
        "Tables with a \".\" in the name wreak havoc with the system",
    );
}

#[test]
fn fix_wrecked_havoc() {
    assert_suggestion_result(
        "It would have been some weird local configuration of LO that wrecked havoc.",
        test_linter(),
        "It would have been some weird local configuration of LO that wreaked havoc.",
    );
}

#[test]
fn fix_wrecking_havoc() {
    assert_suggestion_result(
        "Multi-line edit is wrecking havoc with indention",
        test_linter(),
        "Multi-line edit is wreaking havoc with indention",
    );
}

#[test]
fn fix_wrecks_havoc() {
    assert_suggestion_result(
        "Small POC using rust with ptrace that wrecks havoc on msync",
        test_linter(),
        "Small POC using rust with ptrace that wreaks havoc on msync",
    );
}

// VerseAsVerb

#[test]
fn corrects_verse_against() {
    assert_suggestion_result(
        "A game of Morra, with 3 different AI you can verse against.",
        test_linter(),
        "A game of Morra, with 3 different AI you can play against.",
    );
}

#[test]
fn corrects_versing_against() {
    assert_suggestion_result(
        "This will help when you are versing against a particular boss.",
        test_linter(),
        "This will help when you are playing against a particular boss.",
    );
}

#[test]
fn corrects_verse_me() {
    assert_suggestion_result(
        "Come verse me in this game.",
        test_linter(),
        "Come play me in this game.",
    );
}

#[test]
fn allows_versus() {
    assert_no_lints("It was red versus blue in the finals.", test_linter());
}

// WroteToRote

#[test]
fn fix_by_wrote() {
    assert_suggestion_result(
        "Until one repeats and learns a fact by wrote it is the picture that sustains us.",
        test_linter(),
        "Until one repeats and learns a fact by rote it is the picture that sustains us.",
    );
}

#[test]
fn fix_by_wrote_hyphen() {
    assert_suggestion_result(
        "This specification may then be translated into a recursive-decent parser almost by-wrote.",
        test_linter(),
        "This specification may then be translated into a recursive-decent parser almost by-rote.",
    );
}

#[test]
fn fix_wrote_learning() {
    assert_suggestion_result(
        "I found that what turned me off math class was that teachers encouraged wrote learning instead of understanding.",
        test_linter(),
        "I found that what turned me off math class was that teachers encouraged rote learning instead of understanding.",
    );
}

#[test]
fn fix_wrote_memorisation() {
    assert_suggestion_result(
        "Not much of a wrote memorisation kind of guy, so I preferred to commit them to memory by framing them in the context of a paragraph.",
        test_linter(),
        "Not much of a rote memorisation kind of guy, so I preferred to commit them to memory by framing them in the context of a paragraph.",
    );
}

#[test]
fn fix_wrote_memorisation_hyphen() {
    assert_suggestion_result(
        "I find it helps me retain information much better and for longer compared to when I just blindly did wrote-memorisation.",
        test_linter(),
        "I find it helps me retain information much better and for longer compared to when I just blindly did rote-memorisation.",
    );
}

#[test]
fn fix_wrote_memorization() {
    assert_suggestion_result(
        "Outside websites are also no-go, exacerbating the need for wrote memorization.",
        test_linter(),
        "Outside websites are also no-go, exacerbating the need for rote memorization.",
    );
}

#[test]
fn fix_wrote_memorization_hyphen() {
    assert_suggestion_result(
        "The voicings was the biggest game-changer for me, coming from a wrote-memorization type classical piano background.",
        test_linter(),
        "The voicings was the biggest game-changer for me, coming from a rote-memorization type classical piano background.",
    );
}

#[test]
fn fix_wrote_memorizing() {
    assert_suggestion_result(
        "I have never been good at wrote memorizing abbreviations, initialisms, or acronyms.",
        test_linter(),
        "I have never been good at rote memorizing abbreviations, initialisms, or acronyms.",
    );
}

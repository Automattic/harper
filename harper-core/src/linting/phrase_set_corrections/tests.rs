use crate::linting::tests::{
    assert_good_and_bad_suggestions, assert_lint_count, assert_no_lints,
    assert_nth_suggestion_result, assert_suggestion_result, assert_top3_suggestion_result,
};

use super::lint_group;

// 1:1 tests

// Ado

#[test]
fn corrects_further_ado() {
    assert_suggestion_result(
        "... but we finally hit a great spot, so without further adieu.",
        lint_group(),
        "... but we finally hit a great spot, so without further ado.",
    );
}

#[test]
fn corrects_much_ado() {
    assert_suggestion_result(
        "After much adieu this functionality is now available.",
        lint_group(),
        "After much ado this functionality is now available.",
    );
}

// ChampAtTheBit
#[test]
fn correct_chomp_at_the_bit() {
    assert_suggestion_result(
        "so other than rolling back to older drivers i might have to chomp at the bit for a while longer yet",
        lint_group(),
        "so other than rolling back to older drivers i might have to champ at the bit for a while longer yet",
    );
}

#[test]
fn correct_chomped_at_the_bit() {
    assert_suggestion_result(
        "I chomped at the bit, frustrated by my urge to go faster, while my husband chafed at what I thought was a moderate pace.",
        lint_group(),
        "I champed at the bit, frustrated by my urge to go faster, while my husband chafed at what I thought was a moderate pace.",
    );
}

#[test]
fn correct_chomping_at_the_bit() {
    assert_suggestion_result(
        "Checking in to see when the Windows install will be ready. I am chomping at the bit!",
        lint_group(),
        "Checking in to see when the Windows install will be ready. I am champing at the bit!",
    );
}

#[test]
fn correct_chomps_at_the_bit() {
    assert_suggestion_result(
        "nobody chomps at the bit to make sure these are maintained, current, complete, and error free",
        lint_group(),
        "nobody champs at the bit to make sure these are maintained, current, complete, and error free",
    );
}

// ClientOrServerSide

// -client's side-
#[test]
fn correct_clients_side() {
    assert_suggestion_result(
        "I want to debug this server-side as I cannot find out why the connection is being refused from the client's side.",
        lint_group(),
        "I want to debug this server-side as I cannot find out why the connection is being refused from the client-side.",
    );
}

// -server's side-
#[test]
fn correct_servers_side() {
    assert_suggestion_result(
        "A client-server model where the client can execute commands in a terminal on the server's side",
        lint_group(),
        "A client-server model where the client can execute commands in a terminal on the server-side",
    );
}

// ConfirmThat

#[test]
fn correct_conform_that() {
    assert_suggestion_result(
        "the WCAG requires every view of the page to conform that we move this",
        lint_group(),
        "the WCAG requires every view of the page to confirm that we move this",
    );
}

#[test]
fn corrects_conformed_that() {
    assert_suggestion_result(
        "I have conformed that works now.",
        lint_group(),
        "I have confirmed that works now.",
    );
}

#[test]
fn corrects_conforms_that() {
    assert_suggestion_result(
        "I conformed that with the correct configuration, this is working correctly.",
        lint_group(),
        "I confirmed that with the correct configuration, this is working correctly.",
    );
}

#[test]
#[ignore = "False positive not yet handled."]
fn dont_flag_conforming_that() {
    assert_lint_count(
        "is there any example of a case that isn't fully conforming that is supported today?",
        lint_group(),
        0,
    );
}

#[test]
fn corrects_conforming_that() {
    assert_suggestion_result(
        "Thanks for conforming that this issue is fixed in the latest version.",
        lint_group(),
        "Thanks for confirming that this issue is fixed in the latest version.",
    );
}

// DefiniteArticle

#[test]
fn corrects_definite_article() {
    assert_suggestion_result(
        "As for format of outputs: the spec defines the field as using the singular definitive article \"the\"",
        lint_group(),
        "As for format of outputs: the spec defines the field as using the singular definite article \"the\"",
    );
}

#[test]
#[ignore = "Title case capitalization problem causes this one to fail too."]
fn corrects_definite_articles_title_case() {
    assert_suggestion_result(
        "01 Definitive Articles: De or Het. Before starting more complicated topics in Dutch grammar, you should be aware of the articles.",
        lint_group(),
        "01 Definite Articles: De or Het. Before starting more complicated topics in Dutch grammar, you should be aware of the articles.",
    );
}

#[test]
fn corrects_definite_articles_lowercase() {
    assert_suggestion_result(
        ".. definitive articles -та /-ta/ and -те /-te/ (postfixed in Bulgarian).",
        lint_group(),
        ".. definite articles -та /-ta/ and -те /-te/ (postfixed in Bulgarian).",
    );
}

// DigestiveTract

#[test]
fn dont_flag_digestive_track() {
    assert_suggestion_result(
        "In infants less than a year old, because their digestive track is not finished developing yet",
        lint_group(),
        "In infants less than a year old, because their digestive tract is not finished developing yet",
    );
}

#[test]
fn corrects_digestive_tracks() {
    assert_suggestion_result(
        "The digestive tracks of mammals are complex and diverse, with each species having its own unique digestive system.",
        lint_group(),
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
        lint_group(),
        "It does not run windows ?",
    );
}

// RedundantSuperlatives

#[test]
fn redundant_more_optimal() {
    assert_suggestion_result("Is this more optimal?", lint_group(), "Is this optimal?");
}

#[test]
fn redundant_most_ideal() {
    assert_suggestion_result(
        "This is the most ideal scenario.",
        lint_group(),
        "This is the ideal scenario.",
    );
}

// -dose it true positive-
#[test]
#[ignore = "due to false positives this can't be fixed yet"]
fn corrects_dose_it() {
    assert_suggestion_result(
        "dose it support zh_cn ？",
        lint_group(),
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
        lint_group(),
        0,
    );
}

// When the person receives (a prescribed second dose) (it is not counted ttwice)
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_second_dose_it_is_not() {
    assert_lint_count(
        "When the person receives a prescribed second dose it is not counted ttwice",
        lint_group(),
        0,
    );
}

// (At that small a dose) (it was pleasent).
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_a_dose_it_was() {
    assert_lint_count("At that small a dose it was pleasent.", lint_group(), 0);
}

// I do not know (what dose) (it takes) to trip out, but I don't think I could stay awake to find out.
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_what_dose_it_takes() {
    assert_lint_count(
        "I do not know what dose it takes to trip out, but I don't think I could stay awake to find out.",
        lint_group(),
        0,
    );
}

// -dose it- verb false positives

// And then I have to re-add the salts back to it to dose it back up to drinkable.
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_to_dose_it() {
    assert_lint_count(
        "And then I have to re-add the salts back to it to dose it back up to drinkable.",
        lint_group(),
        0,
    );
}

// So my conclusion is: don't dose it too high or it actually is dangerous and not pleasant at all
#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_dont_dose_it_too_high() {
    assert_lint_count(
        "So my conclusion is: don't dose it too high or it actually is dangerous and not pleasant at all",
        lint_group(),
        0,
    );
}

#[test]
#[ignore = "would be a false positive in a naive implementation"]
fn dont_flag_to_dose_it_off() {
    assert_lint_count(
        "the only solution the other hopefully-dominant-reasonable-adult-human mind can find, is to dose it off, hoping the drowsiness can keep the fear at bay",
        lint_group(),
        0,
    );
}

// -he/she/it does-
#[test]
fn corrects_he_does() {
    assert_suggestion_result(
        "This validate each and every field of your from with nice dotted red color warring for the user, incase he dose some mistakes.",
        lint_group(),
        "This validate each and every field of your from with nice dotted red color warring for the user, incase he does some mistakes.",
    );
}

#[test]
fn corrects_she_does() {
    assert_suggestion_result(
        "we wont agree on everything she dose thats what a real person would feel like",
        lint_group(),
        "we wont agree on everything she does thats what a real person would feel like",
    );
}

// -it does-
#[test]
fn corrects_it_dose() {
    assert_suggestion_result(
        "it dose work without WEBP enabled",
        lint_group(),
        "it does work without WEBP enabled",
    );
}

// -someone does-
#[test]
fn corrects_someone_dose() {
    assert_suggestion_result(
        "Hopefully someone dose, I'm not good at C programing....",
        lint_group(),
        "Hopefully someone does, I'm not good at C programing....",
    );
}

// -interrogatives-
#[test]
fn corrects_how_dose() {
    assert_suggestion_result(
        "How dose qsv-copy works?",
        lint_group(),
        "How does qsv-copy works?",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_how_dose_false_positive() {
    assert_lint_count(
        "Work in progress exploration of how dose modifications throughout a trial can also induce bias in the exposure-response relationships.",
        lint_group(),
        0,
    );
}

#[test]
fn corrects_when_dose() {
    assert_suggestion_result(
        "When dose reusebale variable sync between device? #2634",
        lint_group(),
        "When does reusebale variable sync between device? #2634",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_when_dose_false_positive() {
    assert_lint_count(
        "Should we remove the dose when dose has been applied",
        lint_group(),
        0,
    );
}

#[test]
fn corrects_where_dose() {
    assert_suggestion_result(
        "where dose the password store?",
        lint_group(),
        "where does the password store?",
    );
}

#[test]
#[ignore = "false positive not yet detected"]
fn dont_fix_where_dose_false_positive() {
    assert_lint_count(
        "added some better error handling for the weird case where dose files have no dose...",
        lint_group(),
        0,
    );
}

#[test]
fn corrects_who_dose() {
    assert_suggestion_result(
        "Who dose knows the problem?",
        lint_group(),
        "Who does knows the problem?",
    );
}

#[test]
fn corrects_why_dose() {
    assert_suggestion_result(
        "why dose the path is random ?",
        lint_group(),
        "why does the path is random ?",
    );
}

// Note: no false positive detected for 'why does'. Only true positives.

// ExpandArgument

#[test]
fn corrects_arg() {
    assert_suggestion_result(
        "but I cannot figure out how to flag an arg as required",
        lint_group(),
        "but I cannot figure out how to flag an argument as required",
    );
}

#[test]
fn corrects_args() {
    assert_suggestion_result(
        "but every test I've done shows args as being about 65% faster",
        lint_group(),
        "but every test I've done shows arguments as being about 65% faster",
    );
}

// ExpandDecl

#[test]
fn corrects_decl() {
    assert_suggestion_result(
        "Yeah, I agree a forward decl would be preferable in this case.",
        lint_group(),
        "Yeah, I agree a forward declaration would be preferable in this case.",
    );
}

#[test]
fn corrects_decls() {
    assert_suggestion_result(
        "Accessing type decls from pointer types",
        lint_group(),
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
        lint_group(),
        "Should raw pointer dereference/projections have to be in-bounds?",
    );
}

#[test]
fn corrects_derefs() {
    assert_suggestion_result(
        "A contiguous-in-memory double-ended queue that derefs into a slice - gnzlbg/slice_deque.",
        lint_group(),
        "A contiguous-in-memory double-ended queue that dereferences into a slice - gnzlbg/slice_deque.",
    );
}

// ExpandParam

#[test]
fn corrects_param() {
    assert_suggestion_result(
        "If I use the following to set an endDate param with a default value",
        lint_group(),
        "If I use the following to set an endDate parameter with a default value",
    );
}

#[test]
fn corrects_params() {
    assert_suggestion_result(
        "the params are not loaded in the R environment when using the terminal",
        lint_group(),
        "the parameters are not loaded in the R environment when using the terminal",
    );
}

// ExpandPointer

fn correct_ptr() {
    assert_suggestion_result(
        "How else would you construct a slice from a ptr and a length?",
        lint_group(),
        "How else would you construct a slice from a pointer and a length?",
    );
}

fn correct_ptrs() {
    assert_suggestion_result(
        "FixedBufferAllocator.free not freeing ptrs",
        lint_group(),
        "FixedBufferAllocator.free not freeing pointers",
    );
}

// ExpandSpecification

// ExpandStandardInput
// -none-

// ExpandStandardOutput
// -none-

// ExplanationMark
#[test]
fn detect_explanation_mark_atomic() {
    assert_suggestion_result("explanation mark", lint_group(), "exclamation mark");
}

#[test]
fn detect_explanation_marks_atomic() {
    assert_suggestion_result("explanation marks", lint_group(), "exclamation marks");
}

#[test]
fn detect_explanation_mark_real_world() {
    assert_suggestion_result(
        "Note that circled explanation mark, question mark, plus and arrows may be significantly harder to distinguish than their uncircled variants.",
        lint_group(),
        "Note that circled exclamation mark, question mark, plus and arrows may be significantly harder to distinguish than their uncircled variants.",
    );
}

#[test]
fn detect_explanation_marks_real_world() {
    assert_suggestion_result(
        "this issue: html: properly handle explanation marks in comments",
        lint_group(),
        "this issue: html: properly handle exclamation marks in comments",
    );
}

#[test]
fn detect_explanation_point_atomic() {
    assert_suggestion_result("explanation point", lint_group(), "exclamation point");
}

#[test]
fn detect_explanation_point_real_world() {
    assert_suggestion_result(
        "js and makes an offhand mention that you can disable inbuilt plugin with an explanation point (e.g. !error ).",
        lint_group(),
        "js and makes an offhand mention that you can disable inbuilt plugin with an exclamation point (e.g. !error ).",
    );
}

// ExtendOrExtent

#[test]
fn correct_certain_extend() {
    assert_suggestion_result(
        "This is a PowerShell script to automate client pentests / checkups - at least to a certain extend.",
        lint_group(),
        "This is a PowerShell script to automate client pentests / checkups - at least to a certain extent.",
    );
}

#[test]
fn correct_to_the_extend() {
    assert_suggestion_result(
        "Our artifacts are carefully documented and well-structured to the extend that reuse is facilitated.",
        lint_group(),
        "Our artifacts are carefully documented and well-structured to the extent that reuse is facilitated.",
    );
}

#[test]
fn correct_to_some_extend() {
    assert_suggestion_result(
        "Hi, I'm new to Pydantic and to some extend python, and I have a question that I haven't been able to figure out from the Docs.",
        lint_group(),
        "Hi, I'm new to Pydantic and to some extent python, and I have a question that I haven't been able to figure out from the Docs.",
    );
}

#[test]
fn correct_to_an_extend() {
    assert_suggestion_result(
        "It mimics (to an extend) the way in which Chrome requests SSO cookies with the Windows 10 accounts extension.",
        lint_group(),
        "It mimics (to an extent) the way in which Chrome requests SSO cookies with the Windows 10 accounts extension.",
    );
}

// FoamAtTheMouth

#[test]
fn correct_foam_out_the_mouth() {
    assert_suggestion_result(
        "and he gave him a drink that made him foam out the mouth and die",
        lint_group(),
        "and he gave him a drink that made him foam at the mouth and die",
    );
}

#[test]
fn correct_foamed_out_the_mouth() {
    assert_suggestion_result(
        "You can see in some shots they've foamed out the mouth, and it's apparent their poisoned.",
        lint_group(),
        "You can see in some shots they've foamed at the mouth, and it's apparent their poisoned.",
    );
}

#[test]
fn correct_foaming_out_the_mouth() {
    assert_suggestion_result(
        "choking or foaming out the mouth or something like that, leading up to death",
        lint_group(),
        "choking or foaming at the mouth or something like that, leading up to death",
    );
}

#[test]
fn correct_foams_out_the_mouth() {
    assert_suggestion_result(
        "Elaine can't swallow, foams out the mouth and Kramer says she has rabies just like his friend Bob Sacamano after she gets bit by the guy's dog",
        lint_group(),
        "Elaine can't swallow, foams at the mouth and Kramer says she has rabies just like his friend Bob Sacamano after she gets bit by the guy's dog",
    );
}

// FootTheBill

#[test]
fn correct_flip_the_bill() {
    assert_suggestion_result(
        "- SQL Compare (If the company will flip the bill)",
        lint_group(),
        "- SQL Compare (If the company will foot the bill)",
    );
}

#[test]
fn correct_flipped_the_bill() {
    assert_suggestion_result(
        "As a meetup we were extremely lucky that NOVI flipped the bill for our in-person events.",
        lint_group(),
        "As a meetup we were extremely lucky that NOVI footed the bill for our in-person events.",
    );
}

#[test]
fn correct_flipping_the_bill() {
    assert_suggestion_result(
        "for the simple reason that there were no multimillion dollar company flipping the bill",
        lint_group(),
        "for the simple reason that there were no multimillion dollar company footing the bill",
    );
}

#[test]
fn correct_flips_the_bill() {
    assert_suggestion_result(
        "There seems to be a perennial debate in Illinois between urbanites and rural folk about who really flips the bill.",
        lint_group(),
        "There seems to be a perennial debate in Illinois between urbanites and rural folk about who really foots the bill.",
    );
}

// HavePassed

#[test]
fn correct_has_past() {
    assert_suggestion_result(
        "Track the amount of time that has past since a point in time.",
        lint_group(),
        "Track the amount of time that has passed since a point in time.",
    );
}

#[test]
fn correct_have_past() {
    assert_suggestion_result(
        "Another 14+ days have past, any updates on this?",
        lint_group(),
        "Another 14+ days have passed, any updates on this?",
    );
}

#[test]
fn correct_had_past() {
    assert_suggestion_result(
        "Few days had past, so im starting to thinks there is a problem in my local version.",
        lint_group(),
        "Few days had passed, so im starting to thinks there is a problem in my local version.",
    );
}

#[test]
fn correct_having_past() {
    assert_suggestion_result(
        "Return to computer, with enough time having past for the computer to go to full sleep.",
        lint_group(),
        "Return to computer, with enough time having passed for the computer to go to full sleep.",
    );
}

// HomeInOn

#[test]
fn correct_hone_in_on() {
    assert_suggestion_result(
        "This way you can use an object detector algorithm to hone in on subjects and tell sam to only focus in certain areas when looking to extend ...",
        lint_group(),
        "This way you can use an object detector algorithm to home in on subjects and tell sam to only focus in certain areas when looking to extend ...",
    );
}

#[test]
fn correct_honing_in_on() {
    assert_suggestion_result(
        "I think I understand the syntax limitation you're honing in on.",
        lint_group(),
        "I think I understand the syntax limitation you're homing in on.",
    );
}

#[test]
fn correct_hones_in_on() {
    assert_suggestion_result(
        "[FEATURE] Add a magnet that hones in on mobs",
        lint_group(),
        "[FEATURE] Add a magnet that homes in on mobs",
    );
}

#[test]
fn correct_honed_in_on() {
    assert_suggestion_result(
        "But it took me quite a bit of faffing about checking things out before I honed in on the session as the problem and tried to dump out the ...",
        lint_group(),
        "But it took me quite a bit of faffing about checking things out before I homed in on the session as the problem and tried to dump out the ...",
    );
}

// InDetail

// -in details-
fn in_detail_atomic() {
    assert_suggestion_result("in details", lint_group(), "in detail");
}

#[test]
fn in_detail_real_world() {
    assert_suggestion_result(
        "c++ - who can tell me \"*this pointer\" in details?",
        lint_group(),
        "c++ - who can tell me \"*this pointer\" in detail?",
    )
}

// -in more details-
#[test]
fn in_more_detail_atomic() {
    assert_suggestion_result("in more details", lint_group(), "in more detail");
}

#[test]
fn in_more_detail_real_world() {
    assert_suggestion_result(
        "Document the interface in more details · Issue #3 · owlbarn ...",
        lint_group(),
        "Document the interface in more detail · Issue #3 · owlbarn ...",
    );
}

// InvestIn

#[test]
fn corrects_invest_into() {
    assert_suggestion_result(
        "which represents the amount of money they want to invest into a particular deal.",
        lint_group(),
        "which represents the amount of money they want to invest in a particular deal.",
    );
}

#[test]
fn corrects_investing_into() {
    assert_suggestion_result(
        "Taking dividends in cash (rather than automatically re-investing into the originating fund) can help alleviate the need for rebalancing.",
        lint_group(),
        "Taking dividends in cash (rather than automatically re-investing in the originating fund) can help alleviate the need for rebalancing.",
    );
}

#[test]
fn corrects_invested_into() {
    assert_suggestion_result(
        "it's all automatically invested into a collection of loans that match the criteria that ...",
        lint_group(),
        "it's all automatically invested in a collection of loans that match the criteria that ...",
    );
}

#[test]
fn corrects_invests_into() {
    assert_suggestion_result(
        "If a user invests into the protocol first using USDC but afterward changing to DAI, ...",
        lint_group(),
        "If a user invests in the protocol first using USDC but afterward changing to DAI, ...",
    );
}

#[test]
fn corrects_investment_into() {
    assert_suggestion_result(
        "A $10,000 investment into the fund made on February 28, 1997 would have grown to a value of $42,650 at the end of the 20-year period.",
        lint_group(),
        "A $10,000 investment in the fund made on February 28, 1997 would have grown to a value of $42,650 at the end of the 20-year period.",
    );
}

// LayoutVerb

#[test]
fn corrects_layouted() {
    assert_suggestion_result(
        "only the views that neeed it will be measured and layouted when the superview changes",
        lint_group(),
        "only the views that neeed it will be measured and laid out when the superview changes",
    );
}

#[test]
fn corrects_layouting() {
    assert_suggestion_result(
        "An R package for layouting tables, using the S4 method",
        lint_group(),
        "An R package for laying out tables, using the S4 method",
    );
}

// LitotesDirectPositive

#[test]
fn litotes_not_uncommon_atomic() {
    assert_suggestion_result("not uncommon", lint_group(), "common");
}

#[test]
fn litotes_not_uncommon_sentence() {
    assert_suggestion_result(
        "It is not uncommon to see outages during storms.",
        lint_group(),
        "It is common to see outages during storms.",
    );
}

#[test]
fn litotes_not_unlikely() {
    assert_suggestion_result(
        "This outcome is not unlikely given the data.",
        lint_group(),
        "This outcome is likely given the data.",
    );
}

#[test]
fn litotes_not_insignificant() {
    assert_suggestion_result(
        "That is not insignificant progress.",
        lint_group(),
        "That is significant progress.",
    );
}

#[test]
fn litotes_more_preferable() {
    assert_suggestion_result(
        "Is it more preferable to use process.env.variable or env.parsed.variable?",
        lint_group(),
        "Is it preferable to use process.env.variable or env.parsed.variable?",
    );
}

// MakeDoWith

#[test]
fn corrects_make_due_with() {
    assert_suggestion_result(
        "For now, I can make due with a bash script I have",
        lint_group(),
        "For now, I can make do with a bash script I have",
    );
}

#[test]
fn corrects_made_due_with() {
    assert_suggestion_result(
        "I made due with using actions.push for now but will try to do a codepen soon",
        lint_group(),
        "I made do with using actions.push for now but will try to do a codepen soon",
    );
}

#[test]
fn corrects_makes_due_with() {
    assert_suggestion_result(
        "but the code makes due with what is available",
        lint_group(),
        "but the code makes do with what is available",
    );
}

#[test]
fn corrects_making_due_with() {
    assert_suggestion_result(
        "I've been making due with the testMultiple script I wrote above.",
        lint_group(),
        "I've been making do with the testMultiple script I wrote above.",
    );
}

// MakeSense

#[test]
fn fix_make_senses() {
    assert_suggestion_result(
        "some symbols make senses only if you have a certain keyboard",
        lint_group(),
        "some symbols make sense only if you have a certain keyboard",
    );
}

#[test]
fn fix_made_senses() {
    assert_suggestion_result(
        "Usually on the examples of matlab central I have found all with positive magnitude and made senses to me.",
        lint_group(),
        "Usually on the examples of matlab central I have found all with positive magnitude and made sense to me.",
    );
}

#[test]
fn fix_makes_senses() {
    assert_suggestion_result(
        "If it makes senses I can open a PR.",
        lint_group(),
        "If it makes sense I can open a PR.",
    );
}

#[test]
fn fix_making_senses() {
    assert_suggestion_result(
        "I appreciate you mentioned the two use cases, which are making senses for both.",
        lint_group(),
        "I appreciate you mentioned the two use cases, which are making sense for both.",
    );
}

// MootPoint

// -point is mute-
#[test]
fn point_is_moot() {
    assert_suggestion_result("Your point is mute.", lint_group(), "Your point is moot.");
}

// OperatingSystem

#[test]
fn operative_system() {
    assert_suggestion_result(
        "COS is a operative system made with the COSMOS Kernel and written in C#, COS its literally the same than MS-DOS but written in C# and open-source.",
        lint_group(),
        "COS is a operating system made with the COSMOS Kernel and written in C#, COS its literally the same than MS-DOS but written in C# and open-source.",
    );
}

#[test]
fn operative_systems() {
    assert_suggestion_result(
        "My dotfiles for my operative systems and other configurations.",
        lint_group(),
        "My dotfiles for my operating systems and other configurations.",
    );
}

// PassersBy
#[test]
fn correct_passerbys() {
    assert_suggestion_result(
        "For any passerbys, you may replace visibility: hidden/collapsed with: opacity: 0; pointer-events: none;.",
        lint_group(),
        "For any passersby, you may replace visibility: hidden/collapsed with: opacity: 0; pointer-events: none;.",
    );
}

#[test]
fn correct_passer_bys_hyphen() {
    assert_suggestion_result(
        "Is there any way for random willing passer-bys to help with this effort?",
        lint_group(),
        "Is there any way for random willing passers-by to help with this effort?",
    );
}

// PeekBehindTheCurtain

#[test]
fn fix_peak() {
    assert_suggestion_result(
        "Offer a peak behind the curtain of what I look for when baselining a software installation.",
        lint_group(),
        "Offer a peek behind the curtain of what I look for when baselining a software installation.",
    );
}

#[test]
fn fix_peaked() {
    assert_suggestion_result(
        "I peaked behind the curtain of the new Autodraw tool and noticed some expected similarities to what I saw in Quickdraw.",
        lint_group(),
        "I peeked behind the curtain of the new Autodraw tool and noticed some expected similarities to what I saw in Quickdraw.",
    );
}

#[test]
fn fix_peaking() {
    assert_suggestion_result(
        "I can see how peaking behind the curtain got me to where I am today.",
        lint_group(),
        "I can see how peeking behind the curtain got me to where I am today.",
    );
}

#[test]
fn fix_peaks() {
    assert_suggestion_result(
        "The Daily Vlog Series that peaks behind the curtain of an Entrepreneur's day to day life in 2016 building a business.",
        lint_group(),
        "The Daily Vlog Series that peeks behind the curtain of an Entrepreneur's day to day life in 2016 building a business.",
    );
}

// Piggyback
// -none-

// RedundantSuperlatives
// -none-

// ScapeGoat

#[test]
fn fix_an_escape_goat() {
    assert_suggestion_result(
        "I see too many times the cable and ps thingy being used as an escape goat.",
        lint_group(),
        "I see too many times the cable and ps thingy being used as a scapegoat.",
    );
}

#[test]
fn fix_escape_goat() {
    assert_suggestion_result(
        "It helps shift the reason for the failure on to what the manager did not do (making them the escape goat when it fails).",
        lint_group(),
        "It helps shift the reason for the failure on to what the manager did not do (making them the scapegoat when it fails).",
    );
}

#[test]
fn fix_escape_goats() {
    assert_suggestion_result(
        "People might be using Americans as escape goats for this, but these mishearings are becoming as common as a bowl in a china shop!",
        lint_group(),
        "People might be using Americans as scapegoats for this, but these mishearings are becoming as common as a bowl in a china shop!",
    );
}

// WreakHavoc

#[test]
fn fix_wreck_havoc() {
    assert_suggestion_result(
        "Tables with a \".\" in the name wreck havoc with the system",
        lint_group(),
        "Tables with a \".\" in the name wreak havoc with the system",
    );
}

#[test]
fn fix_wrecked_havoc() {
    assert_suggestion_result(
        "It would have been some weird local configuration of LO that wrecked havoc.",
        lint_group(),
        "It would have been some weird local configuration of LO that wreaked havoc.",
    );
}

#[test]
fn fix_wrecking_havoc() {
    assert_suggestion_result(
        "Multi-line edit is wrecking havoc with indention",
        lint_group(),
        "Multi-line edit is wreaking havoc with indention",
    );
}

#[test]
fn fix_wrecks_havoc() {
    assert_suggestion_result(
        "Small POC using rust with ptrace that wrecks havoc on msync",
        lint_group(),
        "Small POC using rust with ptrace that wreaks havoc on msync",
    );
}

// WroteToRote

#[test]
fn fix_by_wrote() {
    assert_suggestion_result(
        "Until one repeats and learns a fact by wrote it is the picture that sustains us.",
        lint_group(),
        "Until one repeats and learns a fact by rote it is the picture that sustains us.",
    );
}

#[test]
fn fix_by_wrote_hyphen() {
    assert_suggestion_result(
        "This specification may then be translated into a recursive-decent parser almost by-wrote.",
        lint_group(),
        "This specification may then be translated into a recursive-decent parser almost by-rote.",
    );
}

#[test]
fn fix_wrote_learning() {
    assert_suggestion_result(
        "I found that what turned me off math class was that teachers encouraged wrote learning instead of understanding.",
        lint_group(),
        "I found that what turned me off math class was that teachers encouraged rote learning instead of understanding.",
    );
}

#[test]
fn fix_wrote_memorisation() {
    assert_suggestion_result(
        "Not much of a wrote memorisation kind of guy, so I preferred to commit them to memory by framing them in the context of a paragraph.",
        lint_group(),
        "Not much of a rote memorisation kind of guy, so I preferred to commit them to memory by framing them in the context of a paragraph.",
    );
}

#[test]
fn fix_wrote_memorisation_hyphen() {
    assert_suggestion_result(
        "I find it helps me retain information much better and for longer compared to when I just blindly did wrote-memorisation.",
        lint_group(),
        "I find it helps me retain information much better and for longer compared to when I just blindly did rote-memorisation.",
    );
}

#[test]
fn fix_wrote_memorization() {
    assert_suggestion_result(
        "Outside websites are also no-go, exacerbating the need for wrote memorization.",
        lint_group(),
        "Outside websites are also no-go, exacerbating the need for rote memorization.",
    );
}

#[test]
fn fix_wrote_memorization_hyphen() {
    assert_suggestion_result(
        "The voicings was the biggest game-changer for me, coming from a wrote-memorization type classical piano background.",
        lint_group(),
        "The voicings was the biggest game-changer for me, coming from a rote-memorization type classical piano background.",
    );
}

#[test]
fn fix_wrote_memorizing() {
    assert_suggestion_result(
        "I have never been good at wrote memorizing abbreviations, initialisms, or acronyms.",
        lint_group(),
        "I have never been good at rote memorizing abbreviations, initialisms, or acronyms.",
    );
}

// Many to many tests

// AwaitFor

#[test]
fn correct_awaits_for() {
    assert_good_and_bad_suggestions(
        "Headless mode awaits for requested user feedback without showing any text for what that feedback should be",
        lint_group(),
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
        lint_group(),
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
        lint_group(),
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
        lint_group(),
        &[
            "I have long awaited the rise of the Dagoat agenda, and it is glorious.",
            "I have long waited for the rise of the Dagoat agenda, and it is glorious.",
        ],
        &[],
    );
}

// Copyright

#[test]
fn copywritten() {
    assert_suggestion_result(
        "Including digital copies of copywritten artwork with the project isn't advised.",
        lint_group(),
        "Including digital copies of copyrighted artwork with the project isn't advised.",
    );
}

#[test]
fn copywrites() {
    assert_suggestion_result(
        "Code is 99% copy/pasted from OpenSSH with an attempt to retain all copywrites",
        lint_group(),
        "Code is 99% copy/pasted from OpenSSH with an attempt to retain all copyrights",
    );
}

#[test]
fn copywrited() {
    assert_suggestion_result(
        "Proprietary copywrited code",
        lint_group(),
        "Proprietary copyrighted code",
    );
}

#[test]
fn copywrited_all_caps() {
    assert_suggestion_result(
        "URLS MAY CONTAIN COPYWRITED MATERIAL",
        lint_group(),
        "URLS MAY CONTAIN COPYRIGHTED MATERIAL",
    );
}

#[test]
fn copywrote() {
    assert_suggestion_result(
        "How do you find out if someone copywrote a movie",
        lint_group(),
        "How do you find out if someone copyrighted a movie",
    );
}

// DoubleEdgedSword

#[test]
fn correct_double_edge_hyphen() {
    assert_suggestion_result(
        "I thought the global defaultTranslationValues was potentially a double-edge sword as it also obfuscates the full set of values",
        lint_group(),
        "I thought the global defaultTranslationValues was potentially a double-edged sword as it also obfuscates the full set of values",
    );
}

#[test]
fn correct_double_edge_space() {
    assert_suggestion_result(
        "It becomes a double edge sword when it should not be used in cases like this.",
        lint_group(),
        "It becomes a double-edged sword when it should not be used in cases like this.",
    );
}

#[test]
fn correct_double_edge_space_plural() {
    assert_suggestion_result(
        "Wake locks are really double edge swords.",
        lint_group(),
        "Wake locks are really double-edged swords.",
    );
}

#[test]
fn correct_double_edged_space() {
    assert_suggestion_result(
        "Use case. currently OPTIMIZE is a double edged sword and potentially a very dangerous tool to use.",
        lint_group(),
        "Use case. currently OPTIMIZE is a double-edged sword and potentially a very dangerous tool to use.",
    );
}

#[test]
fn correct_double_edged_space_plural() {
    assert_suggestion_result(
        "Change: Ambushers and Crusaders now protect their targets too, making them double edged swords",
        lint_group(),
        "Change: Ambushers and Crusaders now protect their targets too, making them double-edged swords",
    );
}

// ExpandAlloc

#[test]
fn corrects_allocs() {
    assert_top3_suggestion_result(
        "cmd/compile: avoid allocs by better tracking of literals for interface conversions and make",
        lint_group(),
        "cmd/compile: avoid allocations by better tracking of literals for interface conversions and make",
    );
}

#[test]
fn expand_alloc() {
    assert_top3_suggestion_result(
        "Used to find system libraries that alloc RWX regions on load.",
        lint_group(),
        "Used to find system libraries that allocate RWX regions on load.",
    );
}

// Expat

#[test]
fn correct_ex_pat_hyphen() {
    assert_suggestion_result(
        "It seems ex-pat means the person will be in a foreign country temporarily",
        lint_group(),
        "It seems expat means the person will be in a foreign country temporarily",
    );
}

#[test]
fn correct_ex_pats_hyphen() {
    assert_suggestion_result(
        "So, it might be correct to call most Brits ex-pats.",
        lint_group(),
        "So, it might be correct to call most Brits expats.",
    );
}

#[test]
fn correct_ex_pat_space() {
    assert_suggestion_result(
        "For me, the term ex pat embodies the exquisite hypocrisy of certain people feeling entitled",
        lint_group(),
        "For me, the term expat embodies the exquisite hypocrisy of certain people feeling entitled",
    );
}

#[test]
#[ignore = "replace_with_match_case results in ExPats"]
fn correct_ex_pats_space() {
    assert_suggestion_result(
        "Why are Brits who emigrate \"Ex Pats\" but people who come here \"immigrants\"?",
        lint_group(),
        "Why are Brits who emigrate \"Expats\" but people who come here \"immigrants\"?",
    );
}

// Expatriate

#[test]
fn correct_expatriot() {
    assert_suggestion_result(
        "Another expatriot of the era, James Joyce, also followed Papa's writing and drinking schedule.",
        lint_group(),
        "Another expatriate of the era, James Joyce, also followed Papa's writing and drinking schedule.",
    );
}

#[test]
fn correct_expatriots() {
    assert_suggestion_result(
        "Expatriots, upon discovering the delightful nuances of Dutch pronunciation, often find themselves in stitches.",
        lint_group(),
        "Expatriates, upon discovering the delightful nuances of Dutch pronunciation, often find themselves in stitches.",
    );
}

#[test]
fn correct_ex_patriot_hyphen() {
    assert_suggestion_result(
        "Then I added we should all be using the word 移民 immigrant, not ex-patriot, not 外国人 gaikokujin, and definitely not 外人 gaijin",
        lint_group(),
        "Then I added we should all be using the word 移民 immigrant, not expatriate, not 外国人 gaikokujin, and definitely not 外人 gaijin",
    );
}

#[test]
fn correct_ex_patriots_hyphen() {
    assert_suggestion_result(
        "Ex-patriots who move to Hong Kong to seek greener pastures and to experience a new culture seem to bring their own cultural baggage with them.",
        lint_group(),
        "Expatriates who move to Hong Kong to seek greener pastures and to experience a new culture seem to bring their own cultural baggage with them.",
    );
}

// GetRidOf

#[test]
fn get_rid_off() {
    assert_suggestion_result(
        "Please bump axios version to get rid off npm warning #624",
        lint_group(),
        "Please bump axios version to get rid of npm warning #624",
    );
}

#[test]
fn gets_rid_off() {
    assert_suggestion_result(
        "Adding at as a runtime dependency gets rid off that error",
        lint_group(),
        "Adding at as a runtime dependency gets rid of that error",
    );
}

#[test]
fn getting_rid_off() {
    assert_suggestion_result(
        "getting rid off of all the complexity of the different accesses method of API service providers",
        lint_group(),
        "getting rid of of all the complexity of the different accesses method of API service providers",
    );
}

#[test]
fn got_rid_off() {
    assert_suggestion_result(
        "For now we got rid off circular deps in model tree structure and it's API.",
        lint_group(),
        "For now we got rid of circular dependencies in model tree structure and it's API.",
    );
}

#[test]
fn gotten_rid_off() {
    assert_suggestion_result(
        "The baX variable thingy I have gotten rid off, that was due to a bad character in the encryption key.",
        lint_group(),
        "The baX variable thingy I have gotten rid of, that was due to a bad character in the encryption key.",
    );
}

#[test]
fn get_ride_of() {
    assert_suggestion_result(
        "Get ride of \"WARNING Deprecated: markdown_github. Use gfm\"",
        lint_group(),
        "Get rid of \"WARNING Deprecated: markdown_github. Use gfm\"",
    );
}

#[test]
fn get_ride_off() {
    assert_suggestion_result(
        "This exact hack was what I trying to get ride off. ",
        lint_group(),
        "This exact hack was what I trying to get rid of. ",
    );
}

#[test]
fn getting_ride_of() {
    assert_suggestion_result(
        "If you have any idea how to fix this without getting ride of bootstrap I would be thankfull.",
        lint_group(),
        "If you have any idea how to fix this without getting rid of bootstrap I would be thankfull.",
    );
}

#[test]
fn gets_ride_of() {
    assert_suggestion_result(
        ".. gets ride of a central back-end/server and eliminates all the risks associated to it.",
        lint_group(),
        ".. gets rid of a central back-end/server and eliminates all the risks associated to it.",
    );
}

#[test]
fn gotten_ride_of() {
    assert_suggestion_result(
        "I have gotten ride of the react-table and everything works just fine.",
        lint_group(),
        "I have gotten rid of the react-table and everything works just fine.",
    );
}

#[test]
fn got_ride_of() {
    assert_suggestion_result(
        "I had to adjust the labels on the free version because you guys got ride of ...",
        lint_group(),
        "I had to adjust the labels on the free version because you guys got rid of ...",
    );
}

// HolyWar

#[test]
#[ignore = "Known failure due to replace_with_match_case working by character index"]
fn correct_holy_war() {
    assert_suggestion_result(
        "I know it is Holly War about idempotent in HTTP and DELETE",
        lint_group(),
        "I know it is Holy War about idempotent in HTTP and DELETE",
    );
}

#[test]
fn correct_holly_wars() {
    assert_suggestion_result(
        "Anyway I'm not starting some holly wars about this point.",
        lint_group(),
        "Anyway I'm not starting some holy wars about this point.",
    );
}

// HowItLooksLike

#[test]
fn correct_how_it_looks_like_1() {
    assert_suggestion_result(
        "And here is how it looks like: As you can see, there is no real difference in the diagram itself.",
        lint_group(),
        "And here is how it looks: As you can see, there is no real difference in the diagram itself.",
    );
}

#[test]
fn correct_how_it_looks_like_2() {
    assert_nth_suggestion_result(
        "This is how it looks like when run from Windows PowerShell or Cmd: image.",
        lint_group(),
        "This is what it looks like when run from Windows PowerShell or Cmd: image.",
        1,
    );
}

#[test]
fn correct_how_they_look_like_1() {
    assert_suggestion_result(
        "This is a sample project illustrating a demo of how to use the new Material 3 components and how they look like.",
        lint_group(),
        "This is a sample project illustrating a demo of how to use the new Material 3 components and how they look.",
    );
}

#[test]
fn correct_how_they_look_like_2() {
    assert_nth_suggestion_result(
        "So for now I'll just leave this issue here of how they look like in the XLSX",
        lint_group(),
        "So for now I'll just leave this issue here of what they look like in the XLSX",
        1,
    );
}

#[test]
fn correct_how_they_looks_like_1() {
    assert_suggestion_result(
        "Here I demonstrate how disney works and how they looks like Don't miss to give me a star.",
        lint_group(),
        "Here I demonstrate how disney works and how they look Don't miss to give me a star.",
    );
}

#[test]
fn correct_how_they_looks_like_2() {
    assert_nth_suggestion_result(
        "You can check how they looks like on Android app by this command:",
        lint_group(),
        "You can check what they look like on Android app by this command:",
        1,
    );
}

#[test]
fn correct_how_she_looks_like_1() {
    assert_suggestion_result(
        "You all know how she looks like.",
        lint_group(),
        "You all know how she looks.",
    );
}

#[test]
fn correct_how_he_looks_like_2() {
    assert_nth_suggestion_result(
        "Here's how he looks like, when he's supposed to just look like his old fatui design.",
        lint_group(),
        "Here's what he looks like, when he's supposed to just look like his old fatui design.",
        1,
    );
}

#[test]
fn correct_how_it_look_like_1() {
    assert_suggestion_result(
        "And I don't mind how it look like, language code subpath or the last subpath as below.",
        lint_group(),
        "And I don't mind how it looks, language code subpath or the last subpath as below.",
    );
}

#[test]
fn correct_how_it_look_like_2() {
    assert_nth_suggestion_result(
        "Here is how it look like in your browser:",
        lint_group(),
        "Here is what it looks like in your browser:",
        1,
    );
}

#[test]
fn correct_how_it_looks_like_with_apostrophe() {
    assert_suggestion_result(
        "In the picture we can see how It look's like on worker desktop.",
        lint_group(),
        "In the picture we can see how It looks on worker desktop.",
    );
}

// MakeItSeem

#[test]
fn corrects_make_it_seems() {
    assert_suggestion_result(
        "but put it into unlisted list may make it seems like listed for GitHub",
        lint_group(),
        "but put it into unlisted list may make it seem like listed for GitHub",
    );
}

#[test]
fn corrects_made_it_seems() {
    assert_suggestion_result(
        "previous explanations made it seems like it would be n",
        lint_group(),
        "previous explanations made it seem like it would be n",
    );
}

#[test]
fn corrects_makes_it_seems() {
    assert_suggestion_result(
        "bundle gives an error that makes it seems like esbuild is trying to use lib/index.js from main",
        lint_group(),
        "bundle gives an error that makes it seem like esbuild is trying to use lib/index.js from main",
    );
}

#[test]
fn corrects_making_it_seems() {
    assert_suggestion_result(
        "Is it possible to teach the concept of assignment/reassignment at the very beginner stage instead of making it seems like constants?",
        lint_group(),
        "Is it possible to teach the concept of assignment/reassignment at the very beginner stage instead of making it seem like constants?",
    );
}

#[test]
fn corrects_made_it_seemed() {
    assert_suggestion_result(
        "The path made it seemed a bit \"internal\".",
        lint_group(),
        "The path made it seem a bit \"internal\".",
    );
}

// NervousWreck

#[test]
#[ignore = "Harper matches case by letter index as 'How Not to Be a Complete NervoUs wreck in an Interview'"]
fn correct_nerve_wreck_space_title_case() {
    assert_suggestion_result(
        "How Not to Be a Complete Nerve Wreck in an Interview",
        lint_group(),
        "How Not to Be a Complete Nervous Wreck in an Interview",
    );
}

#[test]
fn correct_nerve_wreck_space() {
    assert_suggestion_result(
        "The nerve wreck you are makes you seem anxious and agitated so your employer will believe the complaints.",
        lint_group(),
        "The nervous wreck you are makes you seem anxious and agitated so your employer will believe the complaints.",
    );
}

#[test]
fn correct_nerve_wreck_hyphen() {
    assert_suggestion_result(
        "the child receives little education and grows up to be a nerve-wreck",
        lint_group(),
        "the child receives little education and grows up to be a nervous wreck",
    );
}

#[test]
fn correct_nerve_wreck_hyphen_plural() {
    assert_suggestion_result(
        "This helps us not to become nerve wrecks while looking at the side mirrors",
        lint_group(),
        "This helps us not to become nervous wrecks while looking at the side mirrors",
    );
}

#[test]
#[ignore = "We can't detect when the altered form is used for an event rather than a person."]
fn dont_correct_it_was_a_nerve_wreck() {
    assert_no_lints(
        "It was a nerve-wreck, but I was also excited to see what would happen next.",
        lint_group(),
    );
}

#[test]
#[ignore = "We can't detect when the altered form is used for an event rather than a person."]
fn dont_correct_so_much_nerve_wreck() {
    assert_no_lints(
        "So much nerve wreck for such a simple game ...",
        lint_group(),
    );
}

// NotOnly

// -not only are-
#[test]
fn fix_no_only_are() {
    assert_suggestion_result(
        "No only are tests run on my pipeline but once successful, my app is deployed differently",
        lint_group(),
        "Not only are tests run on my pipeline but once successful, my app is deployed differently",
    );
}

// -not only is-
#[test]
fn fix_no_only_is() {
    assert_suggestion_result(
        "No only is it simple, it's efficient!",
        lint_group(),
        "Not only is it simple, it's efficient!",
    );
}

// -not only was-
#[test]
fn fix_no_only_was() {
    assert_suggestion_result(
        "No only was he happily creating shapes, but he was actively using distances and angles to do so.",
        lint_group(),
        "Not only was he happily creating shapes, but he was actively using distances and angles to do so.",
    );
}

// -not only were-
#[test]
fn fix_no_only_were() {
    assert_suggestion_result(
        "No only were there UI inconsistencies, but Safari lags behind chrome with things like the Popover API",
        lint_group(),
        "Not only were there UI inconsistencies, but Safari lags behind chrome with things like the Popover API",
    );
}

// RaiseTheQuestion

// -raise the question-
#[test]
fn detect_rise_the_question() {
    assert_suggestion_result(
        "That would rise the question how to deal with syntax errors etc.",
        lint_group(),
        "That would raise the question how to deal with syntax errors etc.",
    );
}

#[test]
fn detect_arise_the_question() {
    assert_suggestion_result(
        "As e.g. UTC+1, might arise the question whether it includes summer and winter time",
        lint_group(),
        "As e.g. UTC+1, might raise the question whether it includes summer and winter time",
    );
}

// -raises the question-
#[test]
fn detect_rises_the_question() {
    assert_suggestion_result(
        "However, this rises the question as to whether this test is conceptually sound.",
        lint_group(),
        "However, this raises the question as to whether this test is conceptually sound.",
    );
}

#[test]
fn detect_arises_the_question() {
    assert_suggestion_result(
        "And it arises the question, why?",
        lint_group(),
        "And it raises the question, why?",
    );
}

// -raising the question-
#[test]
fn detect_rising_the_question() {
    assert_suggestion_result(
        "as soon as a infoHash query is performed, a Torrent file is retried, rising the question of:",
        lint_group(),
        "as soon as a infoHash query is performed, a Torrent file is retried, raising the question of:",
    );
}

#[test]
fn detect_arising_the_question() {
    assert_suggestion_result(
        "arising the question whether the requirement of wgpu::Features::DEPTH24PLUS_STENCIL8 is precise",
        lint_group(),
        "raising the question whether the requirement of wgpu::Features::DEPTH24PLUS_STENCIL8 is precise",
    );
}

// -raised the question-
#[test]
fn detect_rose_the_question() {
    assert_suggestion_result(
        "Here is an example that rose the question at first: What works.",
        lint_group(),
        "Here is an example that raised the question at first: What works.",
    );
}

#[test]
fn detect_risen_the_question() {
    assert_suggestion_result(
        "That has risen the question in my mind if it is still possible to embed your own Flash player on Facebook today?",
        lint_group(),
        "That has raised the question in my mind if it is still possible to embed your own Flash player on Facebook today?",
    );
}

#[test]
fn detect_rised_the_question() {
    assert_suggestion_result(
        "I rised the question to Emax Support and they just came back to me inmediately with the below response.",
        lint_group(),
        "I raised the question to Emax Support and they just came back to me inmediately with the below response.",
    );
}

#[test]
#[ignore = "Not actually an error after when it's 'there arose'"]
fn dont_fag_there_arose_the_question() {
    assert_suggestion_result(
        "Hello, while I have been using modals manager there arose the question related to customizing of modal header.",
        lint_group(),
        "Hello, while I have been using modals manager there arose the question related to customizing of modal header.",
    );
}

#[test]
fn detect_arised_the_question() {
    assert_suggestion_result(
        "and that fact arised the question in my mind, what does exactly is happening",
        lint_group(),
        "and that fact raised the question in my mind, what does exactly is happening",
    );
}

#[test]
fn detect_arose_the_question() {
    assert_suggestion_result(
        "This arose the question, could I store 32 digits on the stack?",
        lint_group(),
        "This raised the question, could I store 32 digits on the stack?",
    );
}

#[test]
fn detect_arisen_the_question() {
    assert_suggestion_result(
        "Some have arisen the question like how to use this wireless HD mini camera",
        lint_group(),
        "Some have raised the question like how to use this wireless HD mini camera",
    );
}

// ToToo

// -a bridge too far-
#[test]
fn fix_a_bridge_too_far() {
    assert_suggestion_result(
        "If Winforms can ever be conquered by the Mono developers may be a bridge to far.",
        lint_group(),
        "If Winforms can ever be conquered by the Mono developers may be a bridge too far.",
    );
}

// -cake and eat it too-
#[test]
fn fix_cake_and_eat_it_too() {
    assert_suggestion_result(
        "The solution: wouldn't it be great if I could have my cake and eat it to?",
        lint_group(),
        "The solution: wouldn't it be great if I could have my cake and eat it too?",
    );
}

// -go to far-
#[test]
fn fix_go_to_far() {
    assert_suggestion_result(
        "It's difficult to be sure when we go to far sometime when you don't exactly how the beast works in the background .",
        lint_group(),
        "It's difficult to be sure when we go too far sometime when you don't exactly how the beast works in the background .",
    );
}

// -goes to far-
#[test]
fn fix_goes_to_far() {
    assert_suggestion_result(
        "Memory consumption and cpu consumption goes to far like 900% and more than this",
        lint_group(),
        "Memory consumption and cpu consumption goes too far like 900% and more than this",
    );
}

// -going to far-
#[test]
fn fix_going_to_far() {
    assert_suggestion_result(
        "wsrun is going to far on this because debug 's devDependency shouldn't be considered in the cycle detection, should it?",
        lint_group(),
        "wsrun is going too far on this because debug 's devDependency shouldn't be considered in the cycle detection, should it?",
    );
}

// -gone to far-
#[test]
fn fix_gone_to_far() {
    assert_suggestion_result(
        "I might have gone to far with opening issues for small things.",
        lint_group(),
        "I might have gone too far with opening issues for small things.",
    );
}

// -went to far-
#[test]
fn fix_went_to_far() {
    assert_suggestion_result(
        "But I went to far compared to the initial request that seems talk about ...",
        lint_group(),
        "But I went too far compared to the initial request that seems talk about ...",
    );
}

// -life's too short-
#[test]
fn fix_life_s_too_short() {
    assert_suggestion_result(
        "Life's to short for messing around with git add , writing commit message.",
        lint_group(),
        "Life's too short for messing around with git add , writing commit message.",
    );
}

#[test]
fn fix_lifes_to_short() {
    assert_suggestion_result(
        "I wouldn't go back after the 3rd interview lifes to short.",
        lint_group(),
        "I wouldn't go back after the 3rd interview life's too short.",
    );
}

// -life is too short-
#[test]
fn fix_life_is_too_short() {
    assert_suggestion_result(
        "[Life is to short to use dated cli tools that suck]",
        lint_group(),
        "[Life is too short to use dated cli tools that suck]",
    );
}

// -put too fine a point-
#[test]
fn fix_put_too_fine_a_point() {
    assert_suggestion_result(
        "Not to put to fine a point on it... that's not the kind of team I think we want to be.",
        lint_group(),
        "Not to put too fine a point on it... that's not the kind of team I think we want to be.",
    );
}

// -speak too soon-
#[test]
fn fix_speak_too_soon() {
    assert_suggestion_result(
        "I don't want to speak to soon but I kept everything as I had before but included: http = httplib2.Http()",
        lint_group(),
        "I don't want to speak too soon but I kept everything as I had before but included: http = httplib2.Http()",
    );
}

// -speaking too soon-
#[test]
fn fix_speaking_too_soon() {
    assert_suggestion_result(
        "EDIT: Thats what I get for speaking to soon...",
        lint_group(),
        "EDIT: Thats what I get for speaking too soon...",
    );
}

// -spoke too soon-
#[test]
fn fix_spoke_too_soon() {
    assert_suggestion_result(
        "I spoke to soon. Ignore the previous post.",
        lint_group(),
        "I spoke too soon. Ignore the previous post.",
    );
}

// -spoken too soon-
#[test]
fn fix_spoken_too_soon() {
    assert_suggestion_result(
        "EDIT: I might have spoken to soon...",
        lint_group(),
        "EDIT: I might have spoken too soon...",
    );
}

// -think to much-
#[test]
fn fix_think_too_much() {
    assert_suggestion_result(
        "I don't think to much about it, but I don't think it's a big deal.",
        lint_group(),
        "I don't think too much about it, but I don't think it's a big deal.",
    );
}

// -too big for-
#[test]
fn fix_too_big_for() {
    assert_suggestion_result(
        "ng-relations form to big for small screens",
        lint_group(),
        "ng-relations form too big for small screens",
    );
}

// -too big to fail-
#[test]
fn fix_too_big_to_fail() {
    assert_suggestion_result(
        "The core alone has 50k LOC. Reminds me of \"to big to fail\".",
        lint_group(),
        "The core alone has 50k LOC. Reminds me of \"too big to fail\".",
    );
}

// -too good to be true-
#[test]
fn fix_too_good_to_be_true() {
    assert_suggestion_result(
        "This seemed to good to be true, but local to scene resources will not work when they are not contained in a node.",
        lint_group(),
        "This seemed too good to be true, but local to scene resources will not work when they are not contained in a node.",
    );
}

#[test]
fn fix_too_good_too_be_true() {
    assert_suggestion_result(
        "The normalization of rewards is making the plot in tensorboard look too good too be true, because they are not the actual reward ...",
        lint_group(),
        "The normalization of rewards is making the plot in tensorboard look too good to be true, because they are not the actual reward ...",
    );
}

// -too much information-
#[test]
fn fix_too_much_information() {
    assert_suggestion_result(
        "Live test are printing way to much information and is polluting our test output",
        lint_group(),
        "Live test are printing way too much information and is polluting our test output",
    );
}

// TooTo

// -too big too fail-
#[test]
fn fix_too_big_too_fail() {
    assert_suggestion_result(
        "In other words, pointer arithmetic is, at this point, too big too fail, regardless of the clever and sophisticated way C++ lawyercats worded it.",
        lint_group(),
        "In other words, pointer arithmetic is, at this point, too big to fail, regardless of the clever and sophisticated way C++ lawyercats worded it.",
    );
}

// WholeEntire

#[test]
fn detect_atomic_whole_entire() {
    assert_suggestion_result("whole entire", lint_group(), "whole");
}

#[test]
fn correct_real_world_whole_entire() {
    assert_suggestion_result(
        "[FR] support use system dns in whole entire app",
        lint_group(),
        "[FR] support use system dns in whole app",
    );
}

// -a whole entire-
#[test]
fn correct_atomic_a_whole_entire_to_a_whole() {
    assert_suggestion_result("a whole entire", lint_group(), "a whole");
}

#[test]
fn correct_atomic_a_whole_entire_to_an_entire() {
    assert_nth_suggestion_result("a whole entire", lint_group(), "an entire", 1);
}

#[test]
fn correct_real_world_a_whole_entire_to_a_whole() {
    assert_suggestion_result(
        "Start mapping a whole entire new planet using NASA’s MOLA.",
        lint_group(),
        "Start mapping a whole new planet using NASA’s MOLA.",
    );
}

#[test]
fn correct_real_world_a_whole_entire_to_an_entire() {
    assert_nth_suggestion_result(
        "I am not sure I can pass in a whole entire query via the include.",
        lint_group(),
        "I am not sure I can pass in an entire query via the include.",
        1,
    );
}

// WorseOrWorst

// -a lot worst-
#[test]
fn detect_a_lot_worse_atomic() {
    assert_suggestion_result("a lot worst", lint_group(), "a lot worse");
}

#[test]
fn detect_a_lot_worse_real_world() {
    assert_suggestion_result(
        "On a debug build, it's even a lot worst.",
        lint_group(),
        "On a debug build, it's even a lot worse.",
    );
}

// -become worst-
#[test]
fn fix_became_worst() {
    assert_suggestion_result(
        "The problem became worst lately.",
        lint_group(),
        "The problem became worse lately.",
    );
}

#[test]
fn fix_become_worst() {
    assert_suggestion_result(
        "But results seems stay at one place or become worst.",
        lint_group(),
        "But results seems stay at one place or become worse.",
    );
}

#[test]
fn fix_becomes_worst() {
    assert_suggestion_result(
        "This becomes worst if you have an x64 dll and an x86 dll that you don't have thier source codes and want to use them in same project!",
        lint_group(),
        "This becomes worse if you have an x64 dll and an x86 dll that you don't have thier source codes and want to use them in same project!",
    );
}

#[test]
fn fix_becoming_worst() {
    assert_suggestion_result(
        "France is becoming worst than the Five Eyes",
        lint_group(),
        "France is becoming worse than the Five Eyes",
    );
}

// -far worse-
#[test]
fn detect_far_worse_atomic() {
    assert_suggestion_result("far worst", lint_group(), "far worse");
}

#[test]
fn detect_far_worse_real_world() {
    assert_suggestion_result(
        "I mainly use Firefox (personal preference) and have noticed it has far worst performance than Chrome",
        lint_group(),
        "I mainly use Firefox (personal preference) and have noticed it has far worse performance than Chrome",
    );
}

// -get worst-
#[test]
fn fix_get_worse() {
    assert_suggestion_result(
        "and the problem appears to get worst with 2025.5.1 and 2025.5.2.",
        lint_group(),
        "and the problem appears to get worse with 2025.5.1 and 2025.5.2.",
    );
}

#[test]
fn fix_gets_worse() {
    assert_suggestion_result(
        "It just starts after about 15 minutes of work and gradually gets worst.",
        lint_group(),
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
        lint_group(),
        0,
    );
}

#[test]
fn fix_getting_worst() {
    assert_suggestion_result(
        "But, as I said, it is getting worst...",
        lint_group(),
        "But, as I said, it is getting worse...",
    );
}

#[test]
fn fix_got_worst() {
    assert_suggestion_result(
        "typescript support got worst.",
        lint_group(),
        "typescript support got worse.",
    );
}

#[test]
fn fix_gotten_worst() {
    assert_suggestion_result(
        "Has Claude gotten worst?",
        lint_group(),
        "Has Claude gotten worse?",
    );
}

// -much worse-
#[test]
fn detect_much_worse_atomic() {
    assert_suggestion_result("much worst", lint_group(), "much worse");
}

#[test]
fn detect_much_worse_real_world() {
    assert_suggestion_result(
        "the generated image quality is much worst (actually nearly broken)",
        lint_group(),
        "the generated image quality is much worse (actually nearly broken)",
    );
}

// -turn for the worse-
#[test]
fn detect_turn_for_the_worse_atomic() {
    assert_suggestion_result("turn for the worst", lint_group(), "turn for the worse");
}

#[test]
fn detect_turn_for_the_worse_real_world() {
    assert_suggestion_result(
        "Very surprised to see this repo take such a turn for the worst.",
        lint_group(),
        "Very surprised to see this repo take such a turn for the worse.",
    );
}

// -worse than-
#[test]
fn detect_worse_than_atomic() {
    assert_suggestion_result("worst than", lint_group(), "worse than");
}

#[test]
fn detect_worse_than_real_world() {
    assert_suggestion_result(
        "Project real image - inversion quality is worst than in StyleGAN2",
        lint_group(),
        "Project real image - inversion quality is worse than in StyleGAN2",
    );
}

// -worst ever-
#[test]
fn detect_worst_ever_atomic() {
    assert_suggestion_result("worse ever", lint_group(), "worst ever");
}

#[test]
fn detect_worst_ever_real_world() {
    assert_suggestion_result(
        "The Bcl package family is one of the worse ever published by Microsoft.",
        lint_group(),
        "The Bcl package family is one of the worst ever published by Microsoft.",
    );
}

// -worse and worse-
#[test]
fn detect_worst_and_worst_atomic() {
    assert_suggestion_result("worst and worst", lint_group(), "worse and worse");
}

#[test]
fn detect_worst_and_worst_real_world() {
    assert_suggestion_result(
        "This control-L trick does not work for me. The padding is getting worst and worst.",
        lint_group(),
        "This control-L trick does not work for me. The padding is getting worse and worse.",
    );
}

#[test]
fn detect_worse_and_worst_real_world() {
    assert_suggestion_result(
        "This progressively got worse and worst to the point that the machine (LEAD 1010) stopped moving alltogether.",
        lint_group(),
        "This progressively got worse and worse to the point that the machine (LEAD 1010) stopped moving alltogether.",
    );
}

// -at worst-
#[test]
fn detect_at_worst_atomic() {
    assert_suggestion_result(
        "Partial moving of core objects to interpreter state is incorrect at best, unsafe at worse.",
        lint_group(),
        "Partial moving of core objects to interpreter state is incorrect at best, unsafe at worst.",
    );
}

// -worst case scenario-
#[test]
fn correct_worse_case_space() {
    assert_suggestion_result(
        "In the worse case scenario, remote code execution could be achieved.",
        lint_group(),
        "In the worst-case scenario, remote code execution could be achieved.",
    );
}

#[test]
fn correct_worse_case_hyphen() {
    assert_suggestion_result(
        "Basically I want my pods to get the original client IP address... or at least have X-Forwarded-For header, in a worse-case scenario.",
        lint_group(),
        "Basically I want my pods to get the original client IP address... or at least have X-Forwarded-For header, in a worst-case scenario.",
    );
}

#[test]
fn correct_worse_case_two_hyphens() {
    assert_suggestion_result(
        "In a worse-case-scenario, the scenario class code and the results being analysed, become out of sync, and so the wrong labels are applied.",
        lint_group(),
        "In a worst-case scenario, the scenario class code and the results being analysed, become out of sync, and so the wrong labels are applied.",
    );
}

// -make it worst-
#[test]
fn detect_make_it_worst_atomic() {
    assert_suggestion_result(
        "And if you try to access before that, CloudFront will cache the error and it'll make it worst.",
        lint_group(),
        "And if you try to access before that, CloudFront will cache the error and it'll make it worse.",
    );
}

// -made it worst-
#[test]
fn detect_made_it_worst_atomic() {
    assert_suggestion_result(
        "However in couple of occasions the refresh made it worst and it showed commit differences that were already commited and pushed to origin.",
        lint_group(),
        "However in couple of occasions the refresh made it worse and it showed commit differences that were already commited and pushed to origin.",
    );
}

// -makes it worst-
#[test]
fn detect_makes_it_worst_atomic() {
    assert_suggestion_result(
        "What makes it worst, is if I use the returned SHA to try and update the newly created file I get the same error I show below.",
        lint_group(),
        "What makes it worse, is if I use the returned SHA to try and update the newly created file I get the same error I show below.",
    );
}

// -making it worst-
#[test]
fn detect_making_it_worst_atomic() {
    assert_suggestion_result(
        "PLease ai realled need help with this I think I'm making it worst.",
        lint_group(),
        "PLease ai realled need help with this I think I'm making it worse.",
    );
}

// -make them worst-
#[test]
fn detect_make_them_worst_atomic() {
    assert_suggestion_result(
        "Not sure if this makes things clearer or make them worst.",
        lint_group(),
        "Not sure if this makes things clearer or make them worse.",
    );
}

// -made them worst-
#[test]
fn detect_made_them_worst_atomic() {
    assert_suggestion_result(
        "if not outroght caused them / made them worst",
        lint_group(),
        "if not outroght caused them / made them worse",
    );
}

// -makes them worst-
#[test]
fn detect_makes_them_worst_atomic() {
    assert_suggestion_result(
        "(tried ~14 different hyperparameter and data format combos), however, always just makes them worst, they go from \"slightly\" wrong to \"complete nonsense\".",
        lint_group(),
        "(tried ~14 different hyperparameter and data format combos), however, always just makes them worse, they go from \"slightly\" wrong to \"complete nonsense\".",
    );
}

#[test]
#[ignore = "This false positive is not handled yet"]
fn dont_flag_makes_them_worst_case() {
    assert_lint_count(
        "Note 1: all hash tables has an Achilles heel that makes them worst case O(N)",
        lint_group(),
        0,
    );
}

// -making them worst-
#[test]
fn detect_making_them_worst_atomic() {
    assert_suggestion_result(
        "As for the last part about Apple deliberately making them worst in order for us to buy the 3s",
        lint_group(),
        "As for the last part about Apple deliberately making them worse in order for us to buy the 3s",
    );
}

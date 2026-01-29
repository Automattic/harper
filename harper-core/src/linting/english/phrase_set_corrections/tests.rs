use crate::linting::tests::{
    assert_lint_count, assert_no_lints, assert_nth_suggestion_result, assert_suggestion_result,
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
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_much_ado() {
    assert_suggestion_result(
        "After much adieu this functionality is now available.",
        lint_group(),
        "After much ado this functionality is now available.",
        crate::languages::LanguageFamily::English,
    );
}

// ChampAtTheBit
#[test]
fn correct_chomp_at_the_bit() {
    assert_suggestion_result(
        "so other than rolling back to older drivers i might have to chomp at the bit for a while longer yet",
        lint_group(),
        "so other than rolling back to older drivers i might have to champ at the bit for a while longer yet",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_chomped_at_the_bit() {
    assert_suggestion_result(
        "I chomped at the bit, frustrated by my urge to go faster, while my husband chafed at what I thought was a moderate pace.",
        lint_group(),
        "I champed at the bit, frustrated by my urge to go faster, while my husband chafed at what I thought was a moderate pace.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_chomping_at_the_bit() {
    assert_suggestion_result(
        "Checking in to see when the Windows install will be ready. I am chomping at the bit!",
        lint_group(),
        "Checking in to see when the Windows install will be ready. I am champing at the bit!",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_chomps_at_the_bit() {
    assert_suggestion_result(
        "nobody chomps at the bit to make sure these are maintained, current, complete, and error free",
        lint_group(),
        "nobody champs at the bit to make sure these are maintained, current, complete, and error free",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

// -server's side-
#[test]
fn correct_servers_side() {
    assert_suggestion_result(
        "A client-server model where the client can execute commands in a terminal on the server's side",
        lint_group(),
        "A client-server model where the client can execute commands in a terminal on the server-side",
        crate::languages::LanguageFamily::English,
    );
}

// ConfirmThat

#[test]
fn correct_conform_that() {
    assert_suggestion_result(
        "the WCAG requires every view of the page to conform that we move this",
        lint_group(),
        "the WCAG requires every view of the page to confirm that we move this",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_conformed_that() {
    assert_suggestion_result(
        "I have conformed that works now.",
        lint_group(),
        "I have confirmed that works now.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_conforms_that() {
    assert_suggestion_result(
        "I conformed that with the correct configuration, this is working correctly.",
        lint_group(),
        "I confirmed that with the correct configuration, this is working correctly.",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

// DefiniteArticle

#[test]
fn corrects_definite_article() {
    assert_suggestion_result(
        "As for format of outputs: the spec defines the field as using the singular definitive article \"the\"",
        lint_group(),
        "As for format of outputs: the spec defines the field as using the singular definite article \"the\"",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
#[ignore = "Title case capitalization problem causes this one to fail too."]
fn corrects_definite_articles_title_case() {
    assert_suggestion_result(
        "01 Definitive Articles: De or Het. Before starting more complicated topics in Dutch grammar, you should be aware of the articles.",
        lint_group(),
        "01 Definite Articles: De or Het. Before starting more complicated topics in Dutch grammar, you should be aware of the articles.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_definite_articles_lowercase() {
    assert_suggestion_result(
        ".. definitive articles -та /-ta/ and -те /-te/ (postfixed in Bulgarian).",
        lint_group(),
        ".. definite articles -та /-ta/ and -те /-te/ (postfixed in Bulgarian).",
        crate::languages::LanguageFamily::English,
    );
}

// DigestiveTract

#[test]
fn dont_flag_digestive_track() {
    assert_suggestion_result(
        "In infants less than a year old, because their digestive track is not finished developing yet",
        lint_group(),
        "In infants less than a year old, because their digestive tract is not finished developing yet",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_digestive_tracks() {
    assert_suggestion_result(
        "The digestive tracks of mammals are complex and diverse, with each species having its own unique digestive system.",
        lint_group(),
        "The digestive tracts of mammals are complex and diverse, with each species having its own unique digestive system.",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

// LitotesDirectPositive

#[test]
fn litotes_not_uncommon_atomic() {
    assert_suggestion_result(
        "not uncommon",
        lint_group(),
        "common",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn litotes_not_uncommon_sentence() {
    assert_suggestion_result(
        "It is not uncommon to see outages during storms.",
        lint_group(),
        "It is common to see outages during storms.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn litotes_not_unlikely() {
    assert_suggestion_result(
        "This outcome is not unlikely given the data.",
        lint_group(),
        "This outcome is likely given the data.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn litotes_not_insignificant() {
    assert_suggestion_result(
        "That is not insignificant progress.",
        lint_group(),
        "That is significant progress.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn litotes_more_preferable() {
    assert_suggestion_result(
        "Is it more preferable to use process.env.variable or env.parsed.variable?",
        lint_group(),
        "Is it preferable to use process.env.variable or env.parsed.variable?",
        crate::languages::LanguageFamily::English,
    );
}

// RedundantSuperlatives

#[test]
fn redundant_more_optimal() {
    assert_suggestion_result(
        "Is this more optimal?",
        lint_group(),
        "Is this optimal?",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn redundant_most_ideal() {
    assert_suggestion_result(
        "This is the most ideal scenario.",
        lint_group(),
        "This is the ideal scenario.",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_she_does() {
    assert_suggestion_result(
        "we wont agree on everything she dose thats what a real person would feel like",
        lint_group(),
        "we wont agree on everything she does thats what a real person would feel like",
        crate::languages::LanguageFamily::English,
    );
}

// -it does-
#[test]
fn corrects_it_dose() {
    assert_suggestion_result(
        "it dose work without WEBP enabled",
        lint_group(),
        "it does work without WEBP enabled",
        crate::languages::LanguageFamily::English,
    );
}

// -someone does-
#[test]
fn corrects_someone_dose() {
    assert_suggestion_result(
        "Hopefully someone dose, I'm not good at C programing....",
        lint_group(),
        "Hopefully someone does, I'm not good at C programing....",
        crate::languages::LanguageFamily::English,
    );
}

// -interrogatives-
#[test]
fn corrects_how_dose() {
    assert_suggestion_result(
        "How dose qsv-copy works?",
        lint_group(),
        "How does qsv-copy works?",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_why_dose() {
    assert_suggestion_result(
        "why dose the path is random ?",
        lint_group(),
        "why does the path is random ?",
        crate::languages::LanguageFamily::English,
    );
}

// Note: no false positive detected for 'why does'. Only true positives.

// ExpandDependency
// -none-

// ExpandStandardInput
// -none-

// ExpandStandardOutput
// -none-

// ExplanationMark
#[test]
fn detect_explanation_mark_atomic() {
    assert_suggestion_result(
        "explanation mark",
        lint_group(),
        "exclamation mark",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_explanation_marks_atomic() {
    assert_suggestion_result(
        "explanation marks",
        lint_group(),
        "exclamation marks",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_explanation_mark_real_world() {
    assert_suggestion_result(
        "Note that circled explanation mark, question mark, plus and arrows may be significantly harder to distinguish than their uncircled variants.",
        lint_group(),
        "Note that circled exclamation mark, question mark, plus and arrows may be significantly harder to distinguish than their uncircled variants.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_explanation_marks_real_world() {
    assert_suggestion_result(
        "this issue: html: properly handle explanation marks in comments",
        lint_group(),
        "this issue: html: properly handle exclamation marks in comments",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_explanation_point_atomic() {
    assert_suggestion_result(
        "explanation point",
        lint_group(),
        "exclamation point",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_explanation_point_real_world() {
    assert_suggestion_result(
        "js and makes an offhand mention that you can disable inbuilt plugin with an explanation point (e.g. !error ).",
        lint_group(),
        "js and makes an offhand mention that you can disable inbuilt plugin with an exclamation point (e.g. !error ).",
        crate::languages::LanguageFamily::English,
    );
}

// ExtendOrExtent

#[test]
fn correct_certain_extend() {
    assert_suggestion_result(
        "This is a PowerShell script to automate client pentests / checkups - at least to a certain extend.",
        lint_group(),
        "This is a PowerShell script to automate client pentests / checkups - at least to a certain extent.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_to_the_extend() {
    assert_suggestion_result(
        "Our artifacts are carefully documented and well-structured to the extend that reuse is facilitated.",
        lint_group(),
        "Our artifacts are carefully documented and well-structured to the extent that reuse is facilitated.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_to_some_extend() {
    assert_suggestion_result(
        "Hi, I'm new to Pydantic and to some extend python, and I have a question that I haven't been able to figure out from the Docs.",
        lint_group(),
        "Hi, I'm new to Pydantic and to some extent python, and I have a question that I haven't been able to figure out from the Docs.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_to_an_extend() {
    assert_suggestion_result(
        "It mimics (to an extend) the way in which Chrome requests SSO cookies with the Windows 10 accounts extension.",
        lint_group(),
        "It mimics (to an extent) the way in which Chrome requests SSO cookies with the Windows 10 accounts extension.",
        crate::languages::LanguageFamily::English,
    );
}

// FootTheBill

#[test]
fn correct_flip_the_bill() {
    assert_suggestion_result(
        "- SQL Compare (If the company will flip the bill)",
        lint_group(),
        "- SQL Compare (If the company will foot the bill)",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_flipped_the_bill() {
    assert_suggestion_result(
        "As a meetup we were extremely lucky that NOVI flipped the bill for our in-person events.",
        lint_group(),
        "As a meetup we were extremely lucky that NOVI footed the bill for our in-person events.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_flipping_the_bill() {
    assert_suggestion_result(
        "for the simple reason that there were no multimillion dollar company flipping the bill",
        lint_group(),
        "for the simple reason that there were no multimillion dollar company footing the bill",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_flips_the_bill() {
    assert_suggestion_result(
        "There seems to be a perennial debate in Illinois between urbanites and rural folk about who really flips the bill.",
        lint_group(),
        "There seems to be a perennial debate in Illinois between urbanites and rural folk about who really foots the bill.",
        crate::languages::LanguageFamily::English,
    );
}

// HavePassed

#[test]
fn correct_has_past() {
    assert_suggestion_result(
        "Track the amount of time that has past since a point in time.",
        lint_group(),
        "Track the amount of time that has passed since a point in time.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_have_past() {
    assert_suggestion_result(
        "Another 14+ days have past, any updates on this?",
        lint_group(),
        "Another 14+ days have passed, any updates on this?",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_had_past() {
    assert_suggestion_result(
        "Few days had past, so im starting to thinks there is a problem in my local version.",
        lint_group(),
        "Few days had passed, so im starting to thinks there is a problem in my local version.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_having_past() {
    assert_suggestion_result(
        "Return to computer, with enough time having past for the computer to go to full sleep.",
        lint_group(),
        "Return to computer, with enough time having passed for the computer to go to full sleep.",
        crate::languages::LanguageFamily::English,
    );
}

// HomeInOn

#[test]
fn correct_hone_in_on() {
    assert_suggestion_result(
        "This way you can use an object detector algorithm to hone in on subjects and tell sam to only focus in certain areas when looking to extend ...",
        lint_group(),
        "This way you can use an object detector algorithm to home in on subjects and tell sam to only focus in certain areas when looking to extend ...",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_honing_in_on() {
    assert_suggestion_result(
        "I think I understand the syntax limitation you're honing in on.",
        lint_group(),
        "I think I understand the syntax limitation you're homing in on.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_hones_in_on() {
    assert_suggestion_result(
        "[FEATURE] Add a magnet that hones in on mobs",
        lint_group(),
        "[FEATURE] Add a magnet that homes in on mobs",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_honed_in_on() {
    assert_suggestion_result(
        "But it took me quite a bit of faffing about checking things out before I honed in on the session as the problem and tried to dump out the ...",
        lint_group(),
        "But it took me quite a bit of faffing about checking things out before I homed in on the session as the problem and tried to dump out the ...",
        crate::languages::LanguageFamily::English,
    );
}

// InDetail

// -in details-
fn in_detail_atomic() {
    assert_suggestion_result(
        "in details",
        lint_group(),
        "in detail",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn in_detail_real_world() {
    assert_suggestion_result(
        "c++ - who can tell me \"*this pointer\" in details?",
        lint_group(),
        "c++ - who can tell me \"*this pointer\" in detail?",
        crate::languages::LanguageFamily::English,
    )
}

// -in more details-
#[test]
fn in_more_detail_atomic() {
    assert_suggestion_result(
        "in more details",
        lint_group(),
        "in more detail",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn in_more_detail_real_world() {
    assert_suggestion_result(
        "Document the interface in more details · Issue #3 · owlbarn ...",
        lint_group(),
        "Document the interface in more detail · Issue #3 · owlbarn ...",
        crate::languages::LanguageFamily::English,
    );
}

// InvestIn

#[test]
fn corrects_invest_into() {
    assert_suggestion_result(
        "which represents the amount of money they want to invest into a particular deal.",
        lint_group(),
        "which represents the amount of money they want to invest in a particular deal.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_investing_into() {
    assert_suggestion_result(
        "Taking dividends in cash (rather than automatically re-investing into the originating fund) can help alleviate the need for rebalancing.",
        lint_group(),
        "Taking dividends in cash (rather than automatically re-investing in the originating fund) can help alleviate the need for rebalancing.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_invested_into() {
    assert_suggestion_result(
        "it's all automatically invested into a collection of loans that match the criteria that ...",
        lint_group(),
        "it's all automatically invested in a collection of loans that match the criteria that ...",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_invests_into() {
    assert_suggestion_result(
        "If a user invests into the protocol first using USDC but afterward changing to DAI, ...",
        lint_group(),
        "If a user invests in the protocol first using USDC but afterward changing to DAI, ...",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_investment_into() {
    assert_suggestion_result(
        "A $10,000 investment into the fund made on February 28, 1997 would have grown to a value of $42,650 at the end of the 20-year period.",
        lint_group(),
        "A $10,000 investment in the fund made on February 28, 1997 would have grown to a value of $42,650 at the end of the 20-year period.",
        crate::languages::LanguageFamily::English,
    );
}

// MakeDoWith

#[test]
fn corrects_make_due_with() {
    assert_suggestion_result(
        "For now, I can make due with a bash script I have",
        lint_group(),
        "For now, I can make do with a bash script I have",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_made_due_with() {
    assert_suggestion_result(
        "I made due with using actions.push for now but will try to do a codepen soon",
        lint_group(),
        "I made do with using actions.push for now but will try to do a codepen soon",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_makes_due_with() {
    assert_suggestion_result(
        "but the code makes due with what is available",
        lint_group(),
        "but the code makes do with what is available",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_making_due_with() {
    assert_suggestion_result(
        "I've been making due with the testMultiple script I wrote above.",
        lint_group(),
        "I've been making do with the testMultiple script I wrote above.",
        crate::languages::LanguageFamily::English,
    );
}

// MootPoint

// -point is mute-
#[test]
fn point_is_moot() {
    assert_suggestion_result(
        "Your point is mute.",
        lint_group(),
        "Your point is moot.",
        crate::languages::LanguageFamily::English,
    );
}

// OperatingSystem

#[test]
fn operative_system() {
    assert_suggestion_result(
        "COS is a operative system made with the COSMOS Kernel and written in C#, COS its literally the same than MS-DOS but written in C# and open-source.",
        lint_group(),
        "COS is a operating system made with the COSMOS Kernel and written in C#, COS its literally the same than MS-DOS but written in C# and open-source.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn operative_systems() {
    assert_suggestion_result(
        "My dotfiles for my operative systems and other configurations.",
        lint_group(),
        "My dotfiles for my operating systems and other configurations.",
        crate::languages::LanguageFamily::English,
    );
}

// PassersBy
#[test]
fn correct_passerbys() {
    assert_suggestion_result(
        "For any passerbys, you may replace visibility: hidden/collapsed with: opacity: 0; pointer-events: none;.",
        lint_group(),
        "For any passersby, you may replace visibility: hidden/collapsed with: opacity: 0; pointer-events: none;.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_passer_bys_hyphen() {
    assert_suggestion_result(
        "Is there any way for random willing passer-bys to help with this effort?",
        lint_group(),
        "Is there any way for random willing passers-by to help with this effort?",
        crate::languages::LanguageFamily::English,
    );
}

// Piggyback
// -none-

// Many to many tests

// GetRidOf

#[test]
fn get_rid_off() {
    assert_suggestion_result(
        "Please bump axios version to get rid off npm warning #624",
        lint_group(),
        "Please bump axios version to get rid of npm warning #624",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn gets_rid_off() {
    assert_suggestion_result(
        "Adding at as a runtime dependency gets rid off that error",
        lint_group(),
        "Adding at as a runtime dependency gets rid of that error",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn getting_rid_off() {
    assert_suggestion_result(
        "getting rid off of all the complexity of the different accesses method of API service providers",
        lint_group(),
        "getting rid of of all the complexity of the different accesses method of API service providers",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn got_rid_off() {
    assert_suggestion_result(
        "For now we got rid off circular deps in model tree structure and it's API.",
        lint_group(),
        "For now we got rid of circular dependencies in model tree structure and it's API.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn gotten_rid_off() {
    assert_suggestion_result(
        "The baX variable thingy I have gotten rid off, that was due to a bad character in the encryption key.",
        lint_group(),
        "The baX variable thingy I have gotten rid of, that was due to a bad character in the encryption key.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn get_ride_of() {
    assert_suggestion_result(
        "Get ride of \"WARNING Deprecated: markdown_github. Use gfm\"",
        lint_group(),
        "Get rid of \"WARNING Deprecated: markdown_github. Use gfm\"",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn get_ride_off() {
    assert_suggestion_result(
        "This exact hack was what I trying to get ride off. ",
        lint_group(),
        "This exact hack was what I trying to get rid of. ",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn getting_ride_of() {
    assert_suggestion_result(
        "If you have any idea how to fix this without getting ride of bootstrap I would be thankfull.",
        lint_group(),
        "If you have any idea how to fix this without getting rid of bootstrap I would be thankfull.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn gets_ride_of() {
    assert_suggestion_result(
        ".. gets ride of a central back-end/server and eliminates all the risks associated to it.",
        lint_group(),
        ".. gets rid of a central back-end/server and eliminates all the risks associated to it.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn gotten_ride_of() {
    assert_suggestion_result(
        "I have gotten ride of the react-table and everything works just fine.",
        lint_group(),
        "I have gotten rid of the react-table and everything works just fine.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn got_ride_of() {
    assert_suggestion_result(
        "I had to adjust the labels on the free version because you guys got ride of ...",
        lint_group(),
        "I had to adjust the labels on the free version because you guys got rid of ...",
        crate::languages::LanguageFamily::English,
    );
}

// HowItLooksLike

#[test]
fn correct_how_it_looks_like_1() {
    assert_suggestion_result(
        "And here is how it looks like: As you can see, there is no real difference in the diagram itself.",
        lint_group(),
        "And here is how it looks: As you can see, there is no real difference in the diagram itself.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_how_it_looks_like_2() {
    assert_nth_suggestion_result(
        "This is how it looks like when run from Windows PowerShell or Cmd: image.",
        lint_group(),
        "This is what it looks like when run from Windows PowerShell or Cmd: image.",
        crate::languages::LanguageFamily::English,
        1,
    );
}

#[test]
fn correct_how_they_look_like_1() {
    assert_suggestion_result(
        "This is a sample project illustrating a demo of how to use the new Material 3 components and how they look like.",
        lint_group(),
        "This is a sample project illustrating a demo of how to use the new Material 3 components and how they look.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_how_they_look_like_2() {
    assert_nth_suggestion_result(
        "So for now I'll just leave this issue here of how they look like in the XLSX",
        lint_group(),
        "So for now I'll just leave this issue here of what they look like in the XLSX",
        crate::languages::LanguageFamily::English,
        1,
    );
}

#[test]
fn correct_how_they_looks_like_1() {
    assert_suggestion_result(
        "Here I demonstrate how disney works and how they looks like Don't miss to give me a star.",
        lint_group(),
        "Here I demonstrate how disney works and how they look Don't miss to give me a star.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_how_they_looks_like_2() {
    assert_nth_suggestion_result(
        "You can check how they looks like on Android app by this command:",
        lint_group(),
        "You can check what they look like on Android app by this command:",
        crate::languages::LanguageFamily::English,
        1,
    );
}

#[test]
fn correct_how_she_looks_like_1() {
    assert_suggestion_result(
        "You all know how she looks like.",
        lint_group(),
        "You all know how she looks.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_how_he_looks_like_2() {
    assert_nth_suggestion_result(
        "Here's how he looks like, when he's supposed to just look like his old fatui design.",
        lint_group(),
        "Here's what he looks like, when he's supposed to just look like his old fatui design.",
        crate::languages::LanguageFamily::English,
        1,
    );
}

#[test]
fn correct_how_it_look_like_1() {
    assert_suggestion_result(
        "And I don't mind how it look like, language code subpath or the last subpath as below.",
        lint_group(),
        "And I don't mind how it looks, language code subpath or the last subpath as below.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_how_it_look_like_2() {
    assert_nth_suggestion_result(
        "Here is how it look like in your browser:",
        lint_group(),
        "Here is what it looks like in your browser:",
        crate::languages::LanguageFamily::English,
        1,
    );
}

#[test]
fn correct_how_it_looks_like_with_apostrophe() {
    assert_suggestion_result(
        "In the picture we can see how It look's like on worker desktop.",
        lint_group(),
        "In the picture we can see how It looks on worker desktop.",
        crate::languages::LanguageFamily::English,
    );
}

// MakeItSeem

#[test]
fn corrects_make_it_seems() {
    assert_suggestion_result(
        "but put it into unlisted list may make it seems like listed for GitHub",
        lint_group(),
        "but put it into unlisted list may make it seem like listed for GitHub",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_made_it_seems() {
    assert_suggestion_result(
        "previous explanations made it seems like it would be n",
        lint_group(),
        "previous explanations made it seem like it would be n",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_makes_it_seems() {
    assert_suggestion_result(
        "bundle gives an error that makes it seems like esbuild is trying to use lib/index.js from main",
        lint_group(),
        "bundle gives an error that makes it seem like esbuild is trying to use lib/index.js from main",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_making_it_seems() {
    assert_suggestion_result(
        "Is it possible to teach the concept of assignment/reassignment at the very beginner stage instead of making it seems like constants?",
        lint_group(),
        "Is it possible to teach the concept of assignment/reassignment at the very beginner stage instead of making it seem like constants?",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn corrects_made_it_seemed() {
    assert_suggestion_result(
        "The path made it seemed a bit \"internal\".",
        lint_group(),
        "The path made it seem a bit \"internal\".",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_nerve_wreck_space() {
    assert_suggestion_result(
        "The nerve wreck you are makes you seem anxious and agitated so your employer will believe the complaints.",
        lint_group(),
        "The nervous wreck you are makes you seem anxious and agitated so your employer will believe the complaints.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_nerve_wreck_hyphen() {
    assert_suggestion_result(
        "the child receives little education and grows up to be a nerve-wreck",
        lint_group(),
        "the child receives little education and grows up to be a nervous wreck",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_nerve_wreck_hyphen_plural() {
    assert_suggestion_result(
        "This helps us not to become nerve wrecks while looking at the side mirrors",
        lint_group(),
        "This helps us not to become nervous wrecks while looking at the side mirrors",
        crate::languages::LanguageFamily::English,
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

// RaiseTheQuestion

// -raise the question-
#[test]
fn detect_raise_the_question() {
    assert_suggestion_result(
        "That would rise the question how to deal with syntax errors etc.",
        lint_group(),
        "That would raise the question how to deal with syntax errors etc.",
        crate::languages::LanguageFamily::English,
    );
}

// -raises the question-
#[test]
fn detect_raises_the_question() {
    assert_suggestion_result(
        "However, this rises the question as to whether this test is conceptually sound.",
        lint_group(),
        "However, this raises the question as to whether this test is conceptually sound.",
        crate::languages::LanguageFamily::English,
    );
}

// -raising the question-
#[test]
fn detect_raising_the_question() {
    assert_suggestion_result(
        "as soon as a infoHash query is performed, a Torrent file is retried, rising the question of:",
        lint_group(),
        "as soon as a infoHash query is performed, a Torrent file is retried, raising the question of:",
        crate::languages::LanguageFamily::English,
    );
}

// -rose the question-
#[test]
fn detect_rose_the_question() {
    assert_suggestion_result(
        "Here is an example that rose the question at first: What works.",
        lint_group(),
        "Here is an example that raised the question at first: What works.",
        crate::languages::LanguageFamily::English,
    );
}

// -risen the question-
#[test]
fn detect_risen_the_question() {
    assert_suggestion_result(
        "That has risen the question in my mind if it is still possible to embed your own Flash player on Facebook today?",
        lint_group(),
        "That has raised the question in my mind if it is still possible to embed your own Flash player on Facebook today?",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

// -cake and eat it too-
#[test]
fn fix_cake_and_eat_it_too() {
    assert_suggestion_result(
        "The solution: wouldn't it be great if I could have my cake and eat it to?",
        lint_group(),
        "The solution: wouldn't it be great if I could have my cake and eat it too?",
        crate::languages::LanguageFamily::English,
    );
}

// -go to far-
#[test]
fn fix_go_to_far() {
    assert_suggestion_result(
        "It's difficult to be sure when we go to far sometime when you don't exactly how the beast works in the background .",
        lint_group(),
        "It's difficult to be sure when we go too far sometime when you don't exactly how the beast works in the background .",
        crate::languages::LanguageFamily::English,
    );
}

// -goes to far-
#[test]
fn fix_goes_to_far() {
    assert_suggestion_result(
        "Memory consumption and cpu consumption goes to far like 900% and more than this",
        lint_group(),
        "Memory consumption and cpu consumption goes too far like 900% and more than this",
        crate::languages::LanguageFamily::English,
    );
}

// -going to far-
#[test]
fn fix_going_to_far() {
    assert_suggestion_result(
        "wsrun is going to far on this because debug 's devDependency shouldn't be considered in the cycle detection, should it?",
        lint_group(),
        "wsrun is going too far on this because debug 's devDependency shouldn't be considered in the cycle detection, should it?",
        crate::languages::LanguageFamily::English,
    );
}

// -gone to far-
#[test]
fn fix_gone_to_far() {
    assert_suggestion_result(
        "I might have gone to far with opening issues for small things.",
        lint_group(),
        "I might have gone too far with opening issues for small things.",
        crate::languages::LanguageFamily::English,
    );
}

// -went to far-
#[test]
fn fix_went_to_far() {
    assert_suggestion_result(
        "But I went to far compared to the initial request that seems talk about ...",
        lint_group(),
        "But I went too far compared to the initial request that seems talk about ...",
        crate::languages::LanguageFamily::English,
    );
}

// -life's too short-
#[test]
fn fix_life_s_too_short() {
    assert_suggestion_result(
        "Life's to short for messing around with git add , writing commit message.",
        lint_group(),
        "Life's too short for messing around with git add , writing commit message.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_lifes_to_short() {
    assert_suggestion_result(
        "I wouldn't go back after the 3rd interview lifes to short.",
        lint_group(),
        "I wouldn't go back after the 3rd interview life's too short.",
        crate::languages::LanguageFamily::English,
    );
}

// -life is too short-
#[test]
fn fix_life_is_too_short() {
    assert_suggestion_result(
        "[Life is to short to use dated cli tools that suck]",
        lint_group(),
        "[Life is too short to use dated cli tools that suck]",
        crate::languages::LanguageFamily::English,
    );
}

// -put too fine a point-
#[test]
fn fix_put_too_fine_a_point() {
    assert_suggestion_result(
        "Not to put to fine a point on it... that's not the kind of team I think we want to be.",
        lint_group(),
        "Not to put too fine a point on it... that's not the kind of team I think we want to be.",
        crate::languages::LanguageFamily::English,
    );
}

// -speak too soon-
#[test]
fn fix_speak_too_soon() {
    assert_suggestion_result(
        "I don't want to speak to soon but I kept everything as I had before but included: http = httplib2.Http()",
        lint_group(),
        "I don't want to speak too soon but I kept everything as I had before but included: http = httplib2.Http()",
        crate::languages::LanguageFamily::English,
    );
}

// -speaking too soon-
#[test]
fn fix_speaking_too_soon() {
    assert_suggestion_result(
        "EDIT: Thats what I get for speaking to soon...",
        lint_group(),
        "EDIT: Thats what I get for speaking too soon...",
        crate::languages::LanguageFamily::English,
    );
}

// -spoke too soon-
#[test]
fn fix_spoke_too_soon() {
    assert_suggestion_result(
        "I spoke to soon. Ignore the previous post.",
        lint_group(),
        "I spoke too soon. Ignore the previous post.",
        crate::languages::LanguageFamily::English,
    );
}

// -spoken too soon-
#[test]
fn fix_spoken_too_soon() {
    assert_suggestion_result(
        "EDIT: I might have spoken to soon...",
        lint_group(),
        "EDIT: I might have spoken too soon...",
        crate::languages::LanguageFamily::English,
    );
}

// -think to much-
#[test]
fn fix_think_too_much() {
    assert_suggestion_result(
        "I don't think to much about it, but I don't think it's a big deal.",
        lint_group(),
        "I don't think too much about it, but I don't think it's a big deal.",
        crate::languages::LanguageFamily::English,
    );
}

// -too big for-
#[test]
fn fix_too_big_for() {
    assert_suggestion_result(
        "ng-relations form to big for small screens",
        lint_group(),
        "ng-relations form too big for small screens",
        crate::languages::LanguageFamily::English,
    );
}

// -too big to fail-
#[test]
fn fix_too_big_to_fail() {
    assert_suggestion_result(
        "The core alone has 50k LOC. Reminds me of \"to big to fail\".",
        lint_group(),
        "The core alone has 50k LOC. Reminds me of \"too big to fail\".",
        crate::languages::LanguageFamily::English,
    );
}

// -too good to be true-
#[test]
fn fix_too_good_to_be_true() {
    assert_suggestion_result(
        "This seemed to good to be true, but local to scene resources will not work when they are not contained in a node.",
        lint_group(),
        "This seemed too good to be true, but local to scene resources will not work when they are not contained in a node.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_too_good_too_be_true() {
    assert_suggestion_result(
        "The normalization of rewards is making the plot in tensorboard look too good too be true, because they are not the actual reward ...",
        lint_group(),
        "The normalization of rewards is making the plot in tensorboard look too good to be true, because they are not the actual reward ...",
        crate::languages::LanguageFamily::English,
    );
}

// -too much information-
#[test]
fn fix_too_much_information() {
    assert_suggestion_result(
        "Live test are printing way to much information and is polluting our test output",
        lint_group(),
        "Live test are printing way too much information and is polluting our test output",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

// WholeEntire

#[test]
fn detect_atomic_whole_entire() {
    assert_suggestion_result(
        "whole entire",
        lint_group(),
        "whole",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_real_world_whole_entire() {
    assert_suggestion_result(
        "[FR] support use system dns in whole entire app",
        lint_group(),
        "[FR] support use system dns in whole app",
        crate::languages::LanguageFamily::English,
    );
}

// -a whole entire-
#[test]
fn correct_atomic_a_whole_entire_to_a_whole() {
    assert_suggestion_result(
        "a whole entire",
        lint_group(),
        "a whole",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_atomic_a_whole_entire_to_an_entire() {
    assert_nth_suggestion_result(
        "a whole entire",
        lint_group(),
        "an entire",
        crate::languages::LanguageFamily::English,
        1,
    );
}

#[test]
fn correct_real_world_a_whole_entire_to_a_whole() {
    assert_suggestion_result(
        "Start mapping a whole entire new planet using NASA’s MOLA.",
        lint_group(),
        "Start mapping a whole new planet using NASA’s MOLA.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_real_world_a_whole_entire_to_an_entire() {
    assert_nth_suggestion_result(
        "I am not sure I can pass in a whole entire query via the include.",
        lint_group(),
        "I am not sure I can pass in an entire query via the include.",
        crate::languages::LanguageFamily::English,
        1,
    );
}

// WorseOrWorst

// -a lot worst-
#[test]
fn detect_a_lot_worse_atomic() {
    assert_suggestion_result(
        "a lot worst",
        lint_group(),
        "a lot worse",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_a_lot_worse_real_world() {
    assert_suggestion_result(
        "On a debug build, it's even a lot worst.",
        lint_group(),
        "On a debug build, it's even a lot worse.",
        crate::languages::LanguageFamily::English,
    );
}

// -become worst-
#[test]
fn fix_became_worst() {
    assert_suggestion_result(
        "The problem became worst lately.",
        lint_group(),
        "The problem became worse lately.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_become_worst() {
    assert_suggestion_result(
        "But results seems stay at one place or become worst.",
        lint_group(),
        "But results seems stay at one place or become worse.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_becomes_worst() {
    assert_suggestion_result(
        "This becomes worst if you have an x64 dll and an x86 dll that you don't have thier source codes and want to use them in same project!",
        lint_group(),
        "This becomes worse if you have an x64 dll and an x86 dll that you don't have thier source codes and want to use them in same project!",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_becoming_worst() {
    assert_suggestion_result(
        "France is becoming worst than the Five Eyes",
        lint_group(),
        "France is becoming worse than the Five Eyes",
        crate::languages::LanguageFamily::English,
    );
}

// -far worse-
#[test]
fn detect_far_worse_atomic() {
    assert_suggestion_result(
        "far worst",
        lint_group(),
        "far worse",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_far_worse_real_world() {
    assert_suggestion_result(
        "I mainly use Firefox (personal preference) and have noticed it has far worst performance than Chrome",
        lint_group(),
        "I mainly use Firefox (personal preference) and have noticed it has far worse performance than Chrome",
        crate::languages::LanguageFamily::English,
    );
}

// -get worst-
#[test]
fn fix_get_worse() {
    assert_suggestion_result(
        "and the problem appears to get worst with 2025.5.1 and 2025.5.2.",
        lint_group(),
        "and the problem appears to get worse with 2025.5.1 and 2025.5.2.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_gets_worse() {
    assert_suggestion_result(
        "It just starts after about 15 minutes of work and gradually gets worst.",
        lint_group(),
        "It just starts after about 15 minutes of work and gradually gets worse.",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_got_worst() {
    assert_suggestion_result(
        "typescript support got worst.",
        lint_group(),
        "typescript support got worse.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn fix_gotten_worst() {
    assert_suggestion_result(
        "Has Claude gotten worst?",
        lint_group(),
        "Has Claude gotten worse?",
        crate::languages::LanguageFamily::English,
    );
}

// -much worse-
#[test]
fn detect_much_worse_atomic() {
    assert_suggestion_result(
        "much worst",
        lint_group(),
        "much worse",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_much_worse_real_world() {
    assert_suggestion_result(
        "the generated image quality is much worst (actually nearly broken)",
        lint_group(),
        "the generated image quality is much worse (actually nearly broken)",
        crate::languages::LanguageFamily::English,
    );
}

// -turn for the worse-
#[test]
fn detect_turn_for_the_worse_atomic() {
    assert_suggestion_result(
        "turn for the worst",
        lint_group(),
        "turn for the worse",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_turn_for_the_worse_real_world() {
    assert_suggestion_result(
        "Very surprised to see this repo take such a turn for the worst.",
        lint_group(),
        "Very surprised to see this repo take such a turn for the worse.",
        crate::languages::LanguageFamily::English,
    );
}

// -worse than-
#[test]
fn detect_worse_than_atomic() {
    assert_suggestion_result(
        "worst than",
        lint_group(),
        "worse than",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_worse_than_real_world() {
    assert_suggestion_result(
        "Project real image - inversion quality is worst than in StyleGAN2",
        lint_group(),
        "Project real image - inversion quality is worse than in StyleGAN2",
        crate::languages::LanguageFamily::English,
    );
}

// -worst ever-
#[test]
fn detect_worst_ever_atomic() {
    assert_suggestion_result(
        "worse ever",
        lint_group(),
        "worst ever",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_worst_ever_real_world() {
    assert_suggestion_result(
        "The Bcl package family is one of the worse ever published by Microsoft.",
        lint_group(),
        "The Bcl package family is one of the worst ever published by Microsoft.",
        crate::languages::LanguageFamily::English,
    );
}

// -worse and worse-
#[test]
fn detect_worst_and_worst_atomic() {
    assert_suggestion_result(
        "worst and worst",
        lint_group(),
        "worse and worse",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_worst_and_worst_real_world() {
    assert_suggestion_result(
        "This control-L trick does not work for me. The padding is getting worst and worst.",
        lint_group(),
        "This control-L trick does not work for me. The padding is getting worse and worse.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn detect_worse_and_worst_real_world() {
    assert_suggestion_result(
        "This progressively got worse and worst to the point that the machine (LEAD 1010) stopped moving alltogether.",
        lint_group(),
        "This progressively got worse and worse to the point that the machine (LEAD 1010) stopped moving alltogether.",
        crate::languages::LanguageFamily::English,
    );
}

// -at worst-
#[test]
fn detect_at_worst_atomic() {
    assert_suggestion_result(
        "Partial moving of core objects to interpreter state is incorrect at best, unsafe at worse.",
        lint_group(),
        "Partial moving of core objects to interpreter state is incorrect at best, unsafe at worst.",
        crate::languages::LanguageFamily::English,
    );
}

// -worst case scenario-
#[test]
fn correct_worse_case_space() {
    assert_suggestion_result(
        "In the worse case scenario, remote code execution could be achieved.",
        lint_group(),
        "In the worst-case scenario, remote code execution could be achieved.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_worse_case_hyphen() {
    assert_suggestion_result(
        "Basically I want my pods to get the original client IP address... or at least have X-Forwarded-For header, in a worse-case scenario.",
        lint_group(),
        "Basically I want my pods to get the original client IP address... or at least have X-Forwarded-For header, in a worst-case scenario.",
        crate::languages::LanguageFamily::English,
    );
}

#[test]
fn correct_worse_case_two_hyphens() {
    assert_suggestion_result(
        "In a worse-case-scenario, the scenario class code and the results being analysed, become out of sync, and so the wrong labels are applied.",
        lint_group(),
        "In a worst-case scenario, the scenario class code and the results being analysed, become out of sync, and so the wrong labels are applied.",
        crate::languages::LanguageFamily::English,
    );
}

// -make it worst-
#[test]
fn detect_make_it_worst_atomic() {
    assert_suggestion_result(
        "And if you try to access before that, CloudFront will cache the error and it'll make it worst.",
        lint_group(),
        "And if you try to access before that, CloudFront will cache the error and it'll make it worse.",
        crate::languages::LanguageFamily::English,
    );
}

// -made it worst-
#[test]
fn detect_made_it_worst_atomic() {
    assert_suggestion_result(
        "However in couple of occasions the refresh made it worst and it showed commit differences that were already commited and pushed to origin.",
        lint_group(),
        "However in couple of occasions the refresh made it worse and it showed commit differences that were already commited and pushed to origin.",
        crate::languages::LanguageFamily::English,
    );
}

// -makes it worst-
#[test]
fn detect_makes_it_worst_atomic() {
    assert_suggestion_result(
        "What makes it worst, is if I use the returned SHA to try and update the newly created file I get the same error I show below.",
        lint_group(),
        "What makes it worse, is if I use the returned SHA to try and update the newly created file I get the same error I show below.",
        crate::languages::LanguageFamily::English,
    );
}

// -making it worst-
#[test]
fn detect_making_it_worst_atomic() {
    assert_suggestion_result(
        "PLease ai realled need help with this I think I'm making it worst.",
        lint_group(),
        "PLease ai realled need help with this I think I'm making it worse.",
        crate::languages::LanguageFamily::English,
    );
}

// -make them worst-
#[test]
fn detect_make_them_worst_atomic() {
    assert_suggestion_result(
        "Not sure if this makes things clearer or make them worst.",
        lint_group(),
        "Not sure if this makes things clearer or make them worse.",
        crate::languages::LanguageFamily::English,
    );
}

// -made them worst-
#[test]
fn detect_made_them_worst_atomic() {
    assert_suggestion_result(
        "if not outroght caused them / made them worst",
        lint_group(),
        "if not outroght caused them / made them worse",
        crate::languages::LanguageFamily::English,
    );
}

// -makes them worst-
#[test]
fn detect_makes_them_worst_atomic() {
    assert_suggestion_result(
        "(tried ~14 different hyperparameter and data format combos), however, always just makes them worst, they go from \"slightly\" wrong to \"complete nonsense\".",
        lint_group(),
        "(tried ~14 different hyperparameter and data format combos), however, always just makes them worse, they go from \"slightly\" wrong to \"complete nonsense\".",
        crate::languages::LanguageFamily::English,
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
        crate::languages::LanguageFamily::English,
    );
}

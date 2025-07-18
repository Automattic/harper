use crate::linting::tests::{
    assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result,
    assert_top3_suggestion_result,
};

use super::lint_group;

// ACoupleMore
#[test]
fn corrects_a_couple_of_more() {
    assert_suggestion_result(
        "There are a couple of more rules that could be added, how can I contribute?",
        lint_group(),
        "There are a couple more rules that could be added, how can I contribute?",
    )
}

// AfterAWhile
#[test]
fn correct_after_while() {
    assert_suggestion_result(
        "bromite Crashes on all sites after while.",
        lint_group(),
        "bromite Crashes on all sites after a while.",
    );
}

// AllOfASudden
#[test]
fn corrects_all_of_a_sudden() {
    assert_suggestion_result(
        "On an app that has been released since December, all of the sudden around February 5th ANRs started going up.",
        lint_group(),
        "On an app that has been released since December, all of a sudden around February 5th ANRs started going up.",
    )
}

// ALongTime
#[test]
fn detect_a_long_time() {
    assert_suggestion_result("along time", lint_group(), "a long time");
}

#[test]
fn detect_a_long_time_real_world() {
    assert_suggestion_result(
        "Fast refreshing is very slow had to wait along time for it to update.",
        lint_group(),
        "Fast refreshing is very slow had to wait a long time for it to update.",
    );
}

// AlzheimersDisease
// -none-

// AnAnother
#[test]
fn correct_an_another() {
    assert_suggestion_result(
        "Render shader to use it as texture in an another shader.",
        lint_group(),
        "Render shader to use it as texture in another shader.",
    );
}

#[test]
fn correct_a_another() {
    assert_suggestion_result(
        "Audit login is a another package for laravel framework.",
        lint_group(),
        "Audit login is another package for laravel framework.",
    );
}

// AndIn
// -none-

// AndTheLike
// -none-

// AnotherAn
#[test]
fn correct_another_an() {
    assert_suggestion_result(
        "Yet another an atomic deployment tool.",
        lint_group(),
        "Yet another atomic deployment tool.",
    );
}

// AnotherOnes
#[test]
fn correct_another_ones() {
    assert_nth_suggestion_result(
        "Change list params of a resource, another ones change too",
        lint_group(),
        "Change list params of a resource, other ones change too",
        2,
    );
}

// AnotherThings
#[test]
fn correct_another_things() {
    assert_nth_suggestion_result(
        "Another things to fix in the Mask editor",
        lint_group(),
        "Other things to fix in the Mask editor",
        1,
    );
}

// AsFarBackAs
#[test]
fn detect_as_early_back_as() {
    assert_suggestion_result("as early back as", lint_group(), "as far back as");
}

#[test]
fn detect_as_early_back_as_real_world() {
    assert_suggestion_result(
        "skin overrides also supports a wide variety of minecraft versions - as early back as 1.14.4.",
        lint_group(),
        "skin overrides also supports a wide variety of minecraft versions - as far back as 1.14.4.",
    );
}

// AsItHappens
#[test]
fn correct_as_it_happens() {
    assert_suggestion_result(
        "As it so happens, we have language currently in review that basically states that a major version break means backwards incompatibility ...",
        lint_group(),
        "As it happens, we have language currently in review that basically states that a major version break means backwards incompatibility ...",
    );
}

// AsOfLate
#[test]
fn corrects_as_of_lately() {
    assert_suggestion_result(
        "I haven't noticed any crashing with AMDGPU as of lately, so this looks to not be an issue anymore.",
        lint_group(),
        "I haven't noticed any crashing with AMDGPU as of late, so this looks to not be an issue anymore.",
    )
}

// AsWell
// -none-

// AtFaceValue
#[test]
fn correct_on_face_value() {
    assert_suggestion_result(
        "Obviously what you want is possible and on face value it's a trivial change on our end.",
        lint_group(),
        "Obviously what you want is possible and at face value it's a trivial change on our end.",
    );
}

// AtTheEndOfTheDay
#[test]
fn correct_at_the_end_of_the_day() {
    assert_suggestion_result(
        "In the end of the day, it's not a big deal.",
        lint_group(),
        "At the end of the day, it's not a big deal.",
    );
}

// AvoidAndAlso
// -none-

// BadRap
#[test]
fn bad_rep() {
    assert_suggestion_result("bad rep", lint_group(), "bad rap");
}

// BaitedBreath
#[test]
fn baited_breath() {
    assert_suggestion_result("baited breath", lint_group(), "bated breath");
}

// BandTogether
// -none-

// BareInMind
// -none-

// BeckAndCall
// -none-

// BeenThere
// -none-

// BeforeHand
#[test]
fn corrects_before_hand() {
    assert_suggestion_result(
        "Let me know before hand if you will attend.",
        lint_group(),
        "Let me know beforehand if you will attend.",
    );
}

#[test]
fn corrects_before_hand_hyphen() {
    assert_suggestion_result(
        "I prepared the documents before-hand.",
        lint_group(),
        "I prepared the documents beforehand.",
    );
}

#[test]
fn allows_beforehand() {
    assert_lint_count("We finished the preparations beforehand.", lint_group(), 0);
}

// BestRegards
// -none-

// BlanketStatement
#[test]
fn issue_790() {
    assert_suggestion_result(
        "This seems like a blanketed statement and I have not found any info to back up whether PyJWT is affected.",
        lint_group(),
        "This seems like a blanket statement and I have not found any info to back up whether PyJWT is affected.",
    );
}

// Brutality
#[test]
fn corrects_brutalness() {
    assert_suggestion_result(
        "the mildness and brutalness of the story rises.",
        lint_group(),
        "the mildness and brutality of the story rises.",
    )
}

// ByAccident
// -none-

// CanBeSeen
// -none-

// CaseInPoint
#[test]
fn correct_case_and_point_spaced() {
    assert_suggestion_result(
        "They are just not as high of a priority as other tasks that user's are requesting for, a case and point is I have never ran into this issue.",
        lint_group(),
        "They are just not as high of a priority as other tasks that user's are requesting for, a case in point is I have never ran into this issue.",
    );
}

// CaseSensitive
// -none-

// CondenseAllThe
// -none-

// CoursingThroughVeins
#[test]
fn detect_cursing_through_veins_atomic() {
    assert_suggestion_result(
        "cursing through veins",
        lint_group(),
        "coursing through veins",
    );
}

#[test]
fn detect_cursing_through_veins_real_world() {
    assert_suggestion_result(
        "It felt like the drugs were cursing through veins.",
        lint_group(),
        "It felt like the drugs were coursing through veins.",
    );
}

#[test]
fn does_not_flag_other_contexts() {
    assert_lint_count(
        "He was cursing through the entire meeting.",
        lint_group(),
        0,
    );
}

// DampSquib
// -none-

// DayAndAge
// -none

// DoNotWant
#[test]
fn corrects_dont_wan() {
    assert_suggestion_result(
        "I don't wan to pay for this.",
        lint_group(),
        "I don't want to pay for this.",
    );
}

#[test]
fn corrects_mixed_case() {
    assert_suggestion_result(
        "Don't Wan that option.",
        lint_group(),
        "Don't Want that option.",
    );
}

#[test]
fn does_not_flag_already_correct() {
    assert_lint_count("I don't want to leave.", lint_group(), 0);
}

// EachAndEveryOne
#[test]
fn detect_each_and_everyone() {
    assert_suggestion_result("each and everyone", lint_group(), "each and every one");
}

#[test]
fn detect_each_and_everyone_real_world() {
    assert_suggestion_result(
        "I have modified each and everyone of them to keep only the best of the best!",
        lint_group(),
        "I have modified each and every one of them to keep only the best of the best!",
    );
}

// EludedTo
// -none-

// EnMasse
#[test]
fn in_mass() {
    assert_suggestion_result("in mass", lint_group(), "en masse");
}

// EnRoute
#[test]
fn on_route() {
    assert_suggestion_result("on route to", lint_group(), "en route to");
}

#[test]
fn in_route() {
    assert_suggestion_result("in route to", lint_group(), "en route to");
}

#[test]
fn on_route_real_world() {
    assert_suggestion_result(
        "vehicles may already be on route to one end of a Shipment",
        lint_group(),
        "vehicles may already be en route to one end of a Shipment",
    );
}

#[test]
fn on_hyphen_route_real_world() {
    assert_top3_suggestion_result(
        "I ultimately just want a slight preference for matches that are on-route to correct cases like the above.",
        lint_group(),
        "I ultimately just want a slight preference for matches that are en-route to correct cases like the above.",
    );
}

#[test]
fn in_route_real_world() {
    assert_suggestion_result(
        "TF-South is in route to conduct SSE on the strike.",
        lint_group(),
        "TF-South is en route to conduct SSE on the strike.",
    );
}

// EverPresent
#[test]
fn detect_ever_present_atomic() {
    assert_suggestion_result("ever present", lint_group(), "ever-present");
}

#[test]
fn detect_ever_present_real_world() {
    assert_suggestion_result(
        "Distrust was an ever present tension in the negotiations.",
        lint_group(),
        "Distrust was an ever-present tension in the negotiations.",
    );
}

// Excellent
#[test]
fn excellent_clean() {
    assert_lint_count(
        "The performance was excellent, drawing praise from all critics.",
        lint_group(),
        0,
    );
}

#[test]
fn excellent_incorrect() {
    assert_suggestion_result(
        "Her results were very good this semester.",
        lint_group(),
        "Her results were excellent this semester.",
    );
}

#[test]
fn excellent_no_false_positive() {
    assert_lint_count(
        "He radiated a sense of very goodness in his charitable acts.",
        lint_group(),
        0,
    );
}

// ExpandBecause
#[test]
fn expand_cuz() {
    assert_suggestion_result(
        "Stick around cuz I got a surprise for you at the end.",
        lint_group(),
        "Stick around because I got a surprise for you at the end.",
    );
}

// ExpandMinimum
// -none

// ExpandWith
// -none-

// ExpandWithout
// -none-

// Expatriate
// -none-

// FaceFirst
// -none-

// FairBit
#[test]
fn corrects_fair_bit() {
    assert_suggestion_result(
        "I've read through a fare bit of the ecosystem framework, but I am not clear on what is modified...",
        lint_group(),
        "I've read through a fair bit of the ecosystem framework, but I am not clear on what is modified...",
    );
}

// FastPaste
// -none-

// FatalOutcome
// -none-

// FetalPosition
// -none-

// ForAllIntentsAndPurposes
// -none-

// ForALongTime
#[test]
fn test_for_a_long_time() {
    assert_suggestion_result(
        "I was stuck there for along time.",
        lint_group(),
        "I was stuck there for a long time.",
    );
}

// ForAWhile
#[test]
fn correct_for_while() {
    assert_suggestion_result(
        "Build flutter releases in github actions for production only android for while.",
        lint_group(),
        "Build flutter releases in github actions for production only android for a while.",
    );
}

// FreeRein
// -none-

// Freezing
// -none-

// GildedAge
#[test]
#[ignore = "There's a bug when changing the length of title case phrases.\nI believe there's a fix coming in a PR. Uncomment when fixed."]
fn corrects_gilded_age_capitalized() {
    assert_suggestion_result(
        "It is especially a reflection of the socio-economic patterns in the Guilded Age.",
        lint_group(),
        "It is especially a reflection of the socio-economic patterns in the Gilded Age.",
    );
}

#[test]
#[ignore = "Currently the correct spelling is suggested but the case is not changed.\nThis may also be fixed in the coming PR mentioned above."]
fn corrects_gilded_age_lowercase() {
    assert_suggestion_result(
        "It is especially a reflection of the socio-economic patterns in the guilded age.",
        lint_group(),
        "It is especially a reflection of the socio-economic patterns in the Gilded Age.",
    );
}

// GoingTo
// -none-

// GuineaBissau
#[test]
fn guinea_bissau_missing_hyphen_only() {
    assert_suggestion_result("Guinea Bissau", lint_group(), "Guinea-Bissau");
}

// HadOf
// -none-

// HalfAnHour
#[test]
fn test_half_an_hour() {
    assert_suggestion_result(
        "It took half an our to get there.",
        lint_group(),
        "It took half an hour to get there.",
    );
}

// Haphazard
// -none-

// HumanBeings
#[test]
fn test_human_beings() {
    assert_suggestion_result(
        "All humans beings deserve empathy.",
        lint_group(),
        "All human beings deserve empathy.",
    );
    assert_suggestion_result(
        "We should respect a human's beings fundamental rights.",
        lint_group(),
        "We should respect a human beings fundamental rights.",
    );
}

// HumanLife
// -none-

// HungerPang
#[test]
fn hunger_pain() {
    assert_suggestion_result("hunger pain", lint_group(), "hunger pang");
}

// IAm
// -none-

// IAmAgreement
#[test]
fn corrects_i_are() {
    assert_suggestion_result(
        "I are really happy about this release.",
        lint_group(),
        "I am really happy about this release.",
    );
}

// IDo
#[test]
fn corrects_i_does() {
    assert_suggestion_result(
        "I does enjoy writing Rust.",
        lint_group(),
        "I do enjoy writing Rust.",
    );
}

// InAndOfItself
#[test]
fn detect_atomic_in_of_itself() {
    assert_top3_suggestion_result("in of itself", lint_group(), "in and of itself");
}

#[test]
fn correct_real_world_in_of_itself() {
    assert_top3_suggestion_result(
        "This is not entirely unexpected in of itself, as Git and GitHub Desktop both generally prove fairly bad at delineating context intelligently...",
        lint_group(),
        "This is not entirely unexpected in and of itself, as Git and GitHub Desktop both generally prove fairly bad at delineating context intelligently...",
    )
}

// InAnyWay
#[test]
fn detect_in_anyway_atomic() {
    assert_suggestion_result("in anyway", lint_group(), "in any way");
}

#[test]
fn detect_in_anyway_real_world() {
    assert_suggestion_result(
        "The names should not affect your application in anyway and you can override extension names.",
        lint_group(),
        "The names should not affect your application in any way and you can override extension names.",
    );
}

// InAWhile
#[test]
fn test_in_a_while() {
    assert_suggestion_result(
        "We’ll talk again in while.",
        lint_group(),
        "We’ll talk again in a while.",
    );
}

// InCase
// -none-

// InNeedOf
#[test]
fn corrects_in_need_of() {
    assert_suggestion_result(
        "In need for a native control for map symbols (map legend) #5203.",
        lint_group(),
        "In need of a native control for map symbols (map legend) #5203.",
    );
}

// InOneFellSwoop
// -none-

// Insensitive
#[test]
fn test_insensitive() {
    assert_suggestion_result(
        "We want to potentially make an unsensitive header",
        lint_group(),
        "We want to potentially make an insensitive header",
    );
}

// InsteadOf
#[test]
fn test_instead_of() {
    assert_suggestion_result(
        "He used water in stead of soda.",
        lint_group(),
        "He used water instead of soda.",
    );
}

#[test]
fn test_instead_of_clean() {
    // Ensure no lint is triggered when it's already correct
    assert_lint_count("He used water instead of soda.", lint_group(), 0);
}

// Insurmountable
#[test]
fn corrects_unsurmountable() {
    assert_suggestion_result(
        "That being said, if you find upgrading to newer versions to be unsurmountable, please open an issue.",
        lint_group(),
        "That being said, if you find upgrading to newer versions to be insurmountable, please open an issue.",
    )
}

// Intact
#[test]
fn test_intact() {
    assert_suggestion_result(
        "The code remains in tact after the merge.",
        lint_group(),
        "The code remains intact after the merge.",
    );
}

#[test]
fn test_intact_clean() {
    assert_lint_count("The data set remains intact.", lint_group(), 0);
}

// InThe
// -none-

// IsKnownFor
// -none-

// ItCan
// -none-

// IveGotTo
#[test]
fn test_ive_got_to() {
    assert_suggestion_result(
        "I've go to finish this before Monday.",
        lint_group(),
        "I've got to finish this before Monday.",
    );
}

// JawDropping
// -none-

// JustDeserts
// -none-

// KindOf
#[test]
fn corrects_kinda_of() {
    assert_suggestion_result(
        "Some kinda of Sync issue only with 0.79.1",
        lint_group(),
        "Some kind of Sync issue only with 0.79.1",
    );
}

// KindRegards
// -none-

// LastButNotLeast
#[test]
fn issue_777() {
    assert_suggestion_result(
        "Last but not the least, with VS2013 you can use Web Essentials 2013",
        lint_group(),
        "Last but not least, with VS2013 you can use Web Essentials 2013",
    );
}

#[test]
fn test_last_but_not_least() {
    assert_suggestion_result(
        "Last but not last, I'd like to thank my parents.",
        lint_group(),
        "Last but not least, I'd like to thank my parents.",
    );
}

// LastDitch
#[test]
fn correct_last_ditched() {
    assert_suggestion_result(
        "I was actually just trying that as a last ditched attempt to get it working, previously those ...",
        lint_group(),
        "I was actually just trying that as a last-ditch attempt to get it working, previously those ...",
    );
}

#[test]
fn correct_last_ditch_space() {
    assert_suggestion_result(
        "There are unique use cases and is meant to be a last ditch option.",
        lint_group(),
        "There are unique use cases and is meant to be a last-ditch option.",
    );
}

// LetAlone
#[test]
fn let_along() {
    assert_suggestion_result("let along", lint_group(), "let alone");
}

// LikeThePlague
#[test]
fn correct_like_a_plague() {
    assert_suggestion_result(
        "Below is the worst example of them all (avoid such coding like a plague):",
        lint_group(),
        "Below is the worst example of them all (avoid such coding like the plague):",
    );
}

// LowHangingFruit
#[test]
fn corrects_low_hanging_fruit() {
    assert_suggestion_result(
        "If you add me as a collaborator i can start merging some of the low hanging fruit.",
        lint_group(),
        "If you add me as a collaborator i can start merging some of the low-hanging fruit.",
    )
}

#[test]
fn corrects_low_hanging_fruits_hyphen() {
    assert_suggestion_result(
        "Field guide to gather low-hanging fruits.",
        lint_group(),
        "Field guide to gather low-hanging fruit.",
    )
}

#[test]
fn corrects_low_hanging_fruits_space() {
    assert_suggestion_result(
        "Will search for low hanging fruits and useful information for escalation on a compromised workstation.",
        lint_group(),
        "Will search for low-hanging fruit and useful information for escalation on a compromised workstation.",
    )
}

// Monumentous
#[test]
fn detect_monumentous_atomic() {
    assert_suggestion_result("monumentous", lint_group(), "momentous");
}

#[test]
fn detect_monumentous_real_world() {
    assert_suggestion_result(
        "I think that would be a monumentous step in the right direction, and would DEFINATLY turn heads in not just the music industry, but every ...",
        lint_group(),
        "I think that would be a momentous step in the right direction, and would DEFINATLY turn heads in not just the music industry, but every ...",
    );
}

// MyHouse
// -none-

// NeedHelp
// -none-

// NerveRacking
#[test]
fn detect_nerve_wracking_hyphen() {
    assert_suggestion_result(
        "We've gone through several major changes / upgrades to atlantis, and it's always a little bit nerve-wracking because if we mess something up we ...",
        lint_group(),
        "We've gone through several major changes / upgrades to atlantis, and it's always a little bit nerve-racking because if we mess something up we ...",
    );
}

#[test]
fn detect_nerve_wrecking_hyphen() {
    assert_suggestion_result(
        "The issue happens to me on a daily basis, and it is nerve-wrecking because I become unsure if I have actually saved the diagram, but every time ...",
        lint_group(),
        "The issue happens to me on a daily basis, and it is nerve-racking because I become unsure if I have actually saved the diagram, but every time ...",
    );
}

#[test]
fn detect_nerve_wracking_no_hyphen() {
    assert_suggestion_result(
        "Very nerve wracking landing in an unfamiliar mountainous airport in dead of night with no radar to show surrounding terrain.",
        lint_group(),
        "Very nerve-racking landing in an unfamiliar mountainous airport in dead of night with no radar to show surrounding terrain.",
    );
}

#[test]
fn detect_nerve_wrecking_no_hyphen() {
    assert_suggestion_result(
        "I appreciate any kind of help since this is kind of nerve wrecking.",
        lint_group(),
        "I appreciate any kind of help since this is kind of nerve-racking.",
    );
}

#[test]
fn detect_nerve_racking_no_hyphen() {
    assert_suggestion_result(
        "It's nerve racking to think about it because I have code inside the callback that resolves the member and somehow I feel like it's so ..",
        lint_group(),
        "It's nerve-racking to think about it because I have code inside the callback that resolves the member and somehow I feel like it's so ..",
    );
}

// NotIn
// -none-

// NotTo
// -none-

// OfCourse
// See also: tests in `of_course.rs` for "of curse/corse" → "of course" corrections
#[test]
fn off_course() {
    assert_suggestion_result(
        "Yes, off course we should do that.",
        lint_group(),
        "Yes, of course we should do that.",
    );
}

#[test]
fn o_course() {
    assert_suggestion_result(
        "Yes, o course we should do that.",
        lint_group(),
        "Yes, of course we should do that.",
    );
}

#[test]
fn ofcourse() {
    assert_suggestion_result(
        "Ofcourse, I like other languages.. uzulla has 183 repositories available.",
        lint_group(),
        "Of course, I like other languages.. uzulla has 183 repositories available.",
    );
}

// OffTheCuff
// -none-

// OldWivesTale
// -none-

// OnceInAWhile
#[test]
fn corrects_once_a_while() {
    assert_suggestion_result(
        "For me it is a SMB mount I have on the client device that I sync only once a while for a backup into the cloud.",
        lint_group(),
        "For me it is a SMB mount I have on the client device that I sync only once in a while for a backup into the cloud.",
    );
}

#[test]
fn corrects_once_and_a_while() {
    assert_suggestion_result(
        "Every once and a while all the links on my page seem to stop working.",
        lint_group(),
        "Every once in a while all the links on my page seem to stop working.",
    );
}

// OnSecondThought
#[test]
fn on_second_thought_clean() {
    assert_lint_count(
        "She considered driving home, but on second thought, she decided to walk.",
        lint_group(),
        0,
    );
}

#[test]
fn on_second_thought_incorrect() {
    assert_suggestion_result(
        "I was going to buy it, but on second though, maybe I'll wait.",
        lint_group(),
        "I was going to buy it, but on second thought, maybe I'll wait.",
    );
}

#[test]
fn on_second_thought_no_false_positive() {
    assert_lint_count(
        "My second though is that I'd prefer something else entirely.",
        lint_group(),
        0,
    );
}

// OnTheSpurOfTheMoment
fn fix_on_the_spurt_of_the_moment() {
    assert_suggestion_result(
        "Quite often in the spurt of the moment, someone will say something which they think is witty.",
        lint_group(),
        "Quite often on the spur of the moment, someone will say something which they think is witty.",
    );
}

fn fix_at_the_spur_of_the_moment() {
    assert_suggestion_result(
        "but at the spur of the moment, I'd say that ansible-lint should work exactly like ansible",
        lint_group(),
        "but on the spur of the moment, I'd say that ansible-lint should work exactly like ansible",
    );
}

fn fix_in_the_spur_of_the_moment() {
    assert_suggestion_result(
        "an assortment of things I started yesterday in the spur of the moment",
        lint_group(),
        "an assortment of things I started yesterday on the spur of the moment",
    );
}

// OnTopOf
#[test]
fn correct_on_top_of() {
    assert_suggestion_result(
        "Initcpio hooks for overlayfs ontop of root.",
        lint_group(),
        "Initcpio hooks for overlayfs on top of root.",
    );
}

// PeaceOfMind
#[test]
fn corrects_piece_of_mind() {
    assert_suggestion_result(
        "A Discord bot that gives you piece of mind knowing you are free from obnoxious intrusions in a Discord Voice Channel",
        lint_group(),
        "A Discord bot that gives you peace of mind knowing you are free from obnoxious intrusions in a Discord Voice Channel",
    )
}

// PointsOfView
#[test]
fn corrects_points_of_view() {
    assert_suggestion_result(
        "This will produce a huge amount of raw data, representing the region in multiple point of views.",
        lint_group(),
        "This will produce a huge amount of raw data, representing the region in multiple points of view.",
    )
}

// PrayingMantis
// -none-

// RapidFire
// -none-

// RealTrouper
// -none-

// RedundantIIRC
#[test]
#[ignore = "The bug in replace_with_match_case erroneously generates `iiRC`."]
fn correct_if_iirc_caps() {
    assert_suggestion_result(
        "This is due to the fact that if IIRC up to 2 processes mpirun will bind to core and then it will be socket.",
        lint_group(),
        "This is due to the fact that IIRC up to 2 processes mpirun will bind to core and then it will be socket.",
    );
}

#[test]
fn correct_if_iirc() {
    assert_suggestion_result(
        "if iirc getting it to work with the SQLite storage engine was turning into a whole project and we decided to punt it",
        lint_group(),
        "iirc getting it to work with the SQLite storage engine was turning into a whole project and we decided to punt it",
    );
}

#[test]
fn correct_iirc_correctly() {
    assert_suggestion_result(
        "IIRC correctly, someone on the Home Assistant forums went as far as discovering that RS-485 was being used.",
        lint_group(),
        "IIRC, someone on the Home Assistant forums went as far as discovering that RS-485 was being used.",
    );
}

// RifeWith
// -none-

// RoadMap
// -none-

// SameAs
// -none-

// ScantilyClad
// -none-

// SimpleGrammatical
// -none-

// SneakingSuspicion
#[test]
fn sneaky_suspicion() {
    assert_suggestion_result("sneaky suspicion", lint_group(), "sneaking suspicion");
}

// SomebodyElses
#[test]
fn correct_somebodys_else() {
    assert_suggestion_result(
        "I really like your component and change to somebody's else would be really bad for now.",
        lint_group(),
        "I really like your component and change to somebody else's would be really bad for now.",
    );
}

#[test]
fn correct_somebodys_elses() {
    assert_suggestion_result(
        "Nice to know it's somebody's else's problem for a change.",
        lint_group(),
        "Nice to know it's somebody else's problem for a change.",
    );
}

// SomeOfThe
#[test]
fn corrects_some_the_beginning() {
    assert_suggestion_result(
        "Some the trees are too thick to climb.",
        lint_group(),
        "Some of the trees are too thick to climb.",
    );
}

#[test]
fn corrects_some_the() {
    assert_suggestion_result(
        "You have misplaced some the config files.",
        lint_group(),
        "You have misplaced some of the config files.",
    );
}

// SoonerOrLater
// -none-

// SpecialAttention
#[test]
fn spacial_attention() {
    assert_suggestion_result("spacial attention", lint_group(), "special attention");
}

// SpokeTooSoon
// -none-

// Starving
// -none-

// StateOfTheArt
// -none-

// StatuteOfLimitations
#[test]
fn statute_of_limitations() {
    assert_suggestion_result(
        "Shouldn't there be a grandfathered-in or statue of limitations for posts before the original punishment?",
        lint_group(),
        "Shouldn't there be a grandfathered-in or statute of limitations for posts before the original punishment?",
    );
}

// SufficeItToSay
#[test]
fn suffice_it_to_say() {
    assert_suggestion_result(
        "I don't fully grok the bug, but suffice to say it is not an RCD issue.",
        lint_group(),
        "I don't fully grok the bug, but suffice it to say it is not an RCD issue.",
    );
}

// SupposedTo
#[test]
fn supposed_to() {
    assert_suggestion_result("suppose to", lint_group(), "supposed to");
}

// TakeItPersonally
#[test]
fn corrects_take_it_personal() {
    assert_suggestion_result(
        "This is not personal, do not take it personal, we also think Thingsboard is a extraordinary tool (we are using in several scenarios in fact)",
        lint_group(),
        "This is not personal, do not take it personally, we also think Thingsboard is a extraordinary tool (we are using in several scenarios in fact)",
    );
}

// TakeItSeriously
// -none-

// ThatChallenged
// -none-

// ThatThis
// -none-

// TheAnother
#[test]
fn correct_the_another() {
    assert_suggestion_result(
        "Another possible cause is simply that the application does not have file creation permissions on the another machine.",
        lint_group(),
        "Another possible cause is simply that the application does not have file creation permissions on the other machine.",
    );
}

// ThereIsAny
// -none-

// ThoughtProcess
// -none-

// TickingTimeClock
#[test]
fn suggests_ticking_time_bomb() {
    assert_suggestion_result(
        "One element that can help up the stakes (and tension!) is a “ticking time clock.”",
        lint_group(),
        "One element that can help up the stakes (and tension!) is a “ticking time bomb.”",
    );
}

#[test]
fn suggests_ticking_clock() {
    assert_nth_suggestion_result(
        "The opportunity itself has a ticking time clock as all great opportunities do.",
        lint_group(),
        "The opportunity itself has a ticking clock as all great opportunities do.",
        1,
    );
}

// ToDoHyphen
// -none-

// ToTheMannerBorn
// -none-

// Towards
// -none-

// TrialAndError
#[test]
fn correct_trail_and_error() {
    assert_suggestion_result(
        "It was produced through trail and error.",
        lint_group(),
        "It was produced through trial and error.",
    );
}

// TurnItOff
#[test]
fn turn_it_off_clean_lower() {
    assert_lint_count("turn it off", lint_group(), 0);
}

#[test]
fn turn_it_off_clean_upper() {
    assert_lint_count("Turn it off", lint_group(), 0);
}

#[test]
fn of_confusion() {
    assert_suggestion_result("Turn it of", lint_group(), "Turn it off");
}

#[test]
fn i_and_of_confusion() {
    assert_suggestion_result("Turn i of", lint_group(), "Turn it off");
}

#[test]
fn issue_574() {
    assert_lint_count("run by one", lint_group(), 0);
}

// Unless
#[test]
fn correct_unless_if() {
    assert_suggestion_result(
        "Simplex does not interpret the following invite link as an invite link unless if it has https:// in front of it.",
        lint_group(),
        "Simplex does not interpret the following invite link as an invite link unless it has https:// in front of it.",
    );
}

// WantBe
// -none-

// WaveFunction
// -none-

// WellBeing
// -noun-

// WellKept
#[test]
fn correct_highly_kept_space() {
    assert_suggestion_result(
        "I assure you that frequency/angle dependence is a highly kept secret.",
        lint_group(),
        "I assure you that frequency/angle dependence is a well-kept secret.",
    );
}

#[test]
fn correct_highly_kept_no_hyphen() {
    assert_suggestion_result(
        "Well, Kushina's giving birth was already a highly-kept secret so it makes sense to operate with only the completely necessary personnel.",
        lint_group(),
        "Well, Kushina's giving birth was already a well-kept secret so it makes sense to operate with only the completely necessary personnel.",
    );
}

// WhetYourAppetite
// -none-

// WillContain
// -none-

// WorldWarII
// -none-

// WorstCaseScenario
#[test]
fn correct_worst_case_space() {
    assert_suggestion_result(
        "The worst case scenario can be calculated without looking at streams of data.",
        lint_group(),
        "The worst-case scenario can be calculated without looking at streams of data.",
    );
}

#[test]
fn correct_worst_case_two_hyphens() {
    assert_suggestion_result(
        "CAPD worst-case-scenario cloud simulator for naughty clouds.",
        lint_group(),
        "CAPD worst-case scenario cloud simulator for naughty clouds.",
    );
}

// Used to belong to LoAndBehold which disappeared in #efb3b82
#[test]
fn now_on_hold() {
    assert_lint_count("Those are now on hold for month.", lint_group(), 0);
}

// ThanksALot
#[test]
fn thanks_lot() {
    assert_suggestion_result("thanks lot", lint_group(), "thanks a lot");
}

#[test]
fn thanks_alot() {
    assert_suggestion_result("thanks alot", lint_group(), "thanks a lot");
}

#[test]
fn thanks_a_lot_clean() {
    assert_lint_count("thanks a lot", lint_group(), 0);
}

// WroughtIron
#[test]
fn corrects_rod_iron() {
    assert_suggestion_result(
        "The gate was crafted from rod iron.",
        lint_group(),
        "The gate was crafted from wrought iron.",
    );
}

#[test]
fn corrects_rot_iron() {
    assert_suggestion_result(
        "The artisan works in rot iron.",
        lint_group(),
        "The artisan works in wrought iron.",
    );
}

#[test]
fn allows_wrought_iron() {
    assert_lint_count("She specialized in wrought iron artwork.", lint_group(), 0);
}

#[test]
fn brain_storm() {
    assert_suggestion_result(
        "let's brain storm some ideas",
        lint_group(),
        "let's brainstorm some ideas",
    );
}

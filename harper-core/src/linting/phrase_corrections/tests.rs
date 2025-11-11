use crate::linting::tests::{
    assert_lint_count, assert_no_lints, assert_nth_suggestion_result, assert_suggestion_result,
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

// AheadAnd
#[test]
fn correct_ahead_and() {
    assert_suggestion_result(
        "If it's important, go ahead an open an issue.",
        lint_group(),
        "If it's important, go ahead and open an issue.",
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

#[test]
fn corrects_all_the_sudden_basic() {
    assert_suggestion_result(
        "It happened all the sudden when the lights went out.",
        lint_group(),
        "It happened all of a sudden when the lights went out.",
    );
}

#[test]
fn corrects_all_the_sudden_sentence_start() {
    assert_suggestion_result(
        "All the sudden the room fell quiet.",
        lint_group(),
        "All of a sudden the room fell quiet.",
    );
}

#[test]
fn corrects_all_the_sudden_with_comma() {
    assert_suggestion_result(
        "The music stopped, all the sudden, during the chorus.",
        lint_group(),
        "The music stopped, all of a sudden, during the chorus.",
    );
}

#[test]
fn corrects_all_the_sudden_question() {
    assert_suggestion_result(
        "Did the power cut all the sudden?",
        lint_group(),
        "Did the power cut all of a sudden?",
    );
}

#[test]
fn corrects_all_the_sudden_in_quotes() {
    assert_suggestion_result(
        "He whispered, \"all the sudden we were alone.\"",
        lint_group(),
        "He whispered, \"all of a sudden we were alone.\"",
    );
}

#[test]
fn corrects_all_the_sudden_all_caps() {
    assert_suggestion_result(
        "ALL THE SUDDEN THE ROOM WENT DARK.",
        lint_group(),
        "ALL OF A SUDDEN THE ROOM WENT DARK.",
    );
}

#[test]
fn corrects_all_the_sudden_end_period() {
    assert_suggestion_result(
        "They were laughing all the sudden.",
        lint_group(),
        "They were laughing all of a sudden.",
    );
}

#[test]
fn counts_all_the_sudden_once() {
    assert_lint_count(
        "This all the sudden change surprised everyone.",
        lint_group(),
        1,
    );
}

#[test]
fn corrects_all_of_sudden_variant() {
    assert_suggestion_result(
        "It stormed all of sudden after a warm morning.",
        lint_group(),
        "It stormed all of a sudden after a warm morning.",
    );
}

#[test]
fn ignores_all_the_suddenness() {
    assert_no_lints(
        "Their excitement and suddenness were all the suddenness she remembered.",
        lint_group(),
    );
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

// Alongside
#[test]
fn corrects_along_side_basic() {
    assert_suggestion_result(
        "They walked along side the river.",
        lint_group(),
        "They walked alongside the river.",
    );
}

#[test]
fn corrects_along_side_sentence_start() {
    assert_suggestion_result(
        "Along side the road, we saw a parade.",
        lint_group(),
        "Alongside the road, we saw a parade.",
    );
}

#[test]
fn corrects_along_side_all_caps() {
    assert_suggestion_result(
        "The banner read ALONG SIDE THE TEAM!",
        lint_group(),
        "The banner read ALONGSIDE THE TEAM!",
    );
}

#[test]
fn corrects_along_side_with_period() {
    assert_suggestion_result(
        "The skiff pulled along side.",
        lint_group(),
        "The skiff pulled alongside.",
    );
}

#[test]
fn corrects_along_side_in_quotes() {
    assert_suggestion_result(
        "\"We drifted along side,\" she said.",
        lint_group(),
        "\"We drifted alongside,\" she said.",
    );
}

#[test]
fn corrects_along_side_before_comma() {
    assert_suggestion_result(
        "They stood along side, waiting patiently.",
        lint_group(),
        "They stood alongside, waiting patiently.",
    );
}

#[test]
fn corrects_along_side_plural_subject() {
    assert_suggestion_result(
        "Cars lined up along side the curb.",
        lint_group(),
        "Cars lined up alongside the curb.",
    );
}

#[test]
fn allows_correct_alongside() {
    assert_lint_count("They walked alongside the river.", lint_group(), 0);
}

#[test]
fn allows_along_the_side_phrase() {
    assert_lint_count("They walked along the side of the river.", lint_group(), 0);
}

#[test]
fn allows_lakeside_usage() {
    assert_lint_count("We camped along the lakeside all weekend.", lint_group(), 0);
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

// AsIfThough
#[test]
fn correct_as_if_though_usagi_1() {
    assert_top3_suggestion_result(
        "It's coming back to you. and looking as if though it's very bright red.",
        lint_group(),
        "It's coming back to you. and looking as if it's very bright red.",
    );
}

#[test]
fn correct_as_if_though_usagi_2() {
    assert_top3_suggestion_result(
        "it passes right on by it as if though nothing happened.",
        lint_group(),
        "it passes right on by it as though nothing happened.",
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

// AsLongAs
#[test]
fn correct_as_long_as() {
    assert_suggestion_result(
        "server loads up fine but cant log on client side aslong as the plugin is installed",
        lint_group(),
        "server loads up fine but cant log on client side as long as the plugin is installed",
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

// BesideThePoint
#[test]
fn beside_the_point() {
    assert_suggestion_result(
        "we kind of focus on GPUs a lot but uh that's besides the point so uh sometime ago",
        lint_group(),
        "we kind of focus on GPUs a lot but uh that's beside the point so uh sometime ago",
    );
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

// DegreesKelvin
#[test]
fn corrects_degrees_kelvin() {
    assert_suggestion_result("degrees kelvin", lint_group(), "kelvins");
    assert_suggestion_result("°K", lint_group(), "K");
}

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

// EggYolk
#[test]
fn corrects_simple_egg_yoke() {
    assert_suggestion_result(
        "She whisked the egg yoke briskly.",
        lint_group(),
        "She whisked the egg yolk briskly.",
    );
}

#[test]
fn corrects_sentence_start_egg_yoke() {
    assert_suggestion_result(
        "Egg yoke is rich in nutrients.",
        lint_group(),
        "Egg yolk is rich in nutrients.",
    );
}

#[test]
fn corrects_all_caps_egg_yoke() {
    assert_suggestion_result(
        "Add the EGG YOKE to the batter.",
        lint_group(),
        "Add the EGG YOLK to the batter.",
    );
}

#[test]
fn corrects_punctuated_egg_yoke() {
    assert_suggestion_result(
        "Separate the egg yoke, then fold it in.",
        lint_group(),
        "Separate the egg yolk, then fold it in.",
    );
}

#[test]
fn corrects_adjective_egg_yoke() {
    assert_suggestion_result(
        "The runny egg yoke spilled over the toast.",
        lint_group(),
        "The runny egg yolk spilled over the toast.",
    );
}

#[test]
fn corrects_plural_context_egg_yoke() {
    assert_suggestion_result(
        "Blend the cream with each egg yoke before baking.",
        lint_group(),
        "Blend the cream with each egg yolk before baking.",
    );
}

#[test]
fn allows_correct_egg_yolk() {
    assert_lint_count("The custard calls for one egg yolk.", lint_group(), 0);
}

#[test]
fn allows_plural_egg_yolks() {
    assert_lint_count("Reserve the egg yolks for later.", lint_group(), 0);
}

#[test]
fn allows_yoke_without_egg() {
    assert_lint_count(
        "The artisan carved a wooden yoke for the oxen.",
        lint_group(),
        0,
    );
}

#[test]
fn does_not_flag_partial_phrase() {
    assert_lint_count("Crack the eggs so no yoke spills.", lint_group(), 0);
}

// DontCan
#[test]
fn corrects_dont_can() {
    assert_suggestion_result(
        "And currently uh I'm looking at it when I don't can see it like you know where it is, right?",
        lint_group(),
        "And currently uh I'm looking at it when I can't see it like you know where it is, right?",
    );
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

// EverSince
#[test]
fn detect_ever_since() {
    assert_suggestion_result(
        "einstein been real quiet every since this dropped",
        lint_group(),
        "einstein been real quiet ever since this dropped",
    );
}

// EveryTime
#[test]
fn fix_everytime() {
    assert_suggestion_result(
        "Init tool everytime a file in a directory is modified",
        lint_group(),
        "Init tool every time a file in a directory is modified",
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

// FarAndFewBetween
#[test]
fn corrects_far_and_few_between() {
    assert_suggestion_result(
        "Their neighbors were far and few between, which only made it even more unlikely that surveillance footage recovered from their properties could help.",
        lint_group(),
        "Their neighbors were few and far between, which only made it even more unlikely that surveillance footage recovered from their properties could help.",
    );
}

// FastPaste
// -none-

// FatalOutcome
// -none-

// FetalPosition
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

// LikeAsIf
#[test]
fn correct_like_as_if() {
    assert_top3_suggestion_result(
        "And looks like as if linux-personality hasn't got any changes for 8 years.",
        lint_group(),
        "And looks as if linux-personality hasn't got any changes for 8 years.",
    );
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

// ManagerialReins
#[test]
fn fixes_managerial_reigns_basic() {
    assert_suggestion_result(
        "She grabbed the managerial reigns during the crisis.",
        lint_group(),
        "She grabbed the managerial reins during the crisis.",
    );
}

#[test]
fn fixes_managerial_reigns_sentence_start() {
    assert_suggestion_result(
        "Managerial reigns are never easy to hand over.",
        lint_group(),
        "Managerial reins are never easy to hand over.",
    );
}

#[test]
fn fixes_managerial_reigns_with_article() {
    assert_suggestion_result(
        "The managerial reigns belong to Carla now.",
        lint_group(),
        "The managerial reins belong to Carla now.",
    );
}

#[test]
fn fixes_managerial_reigns_with_comma() {
    assert_suggestion_result(
        "By winter, he held the managerial reigns, and morale improved.",
        lint_group(),
        "By winter, he held the managerial reins, and morale improved.",
    );
}

#[test]
fn fixes_managerial_reigns_question() {
    assert_suggestion_result(
        "Who will hold the managerial reigns after April?",
        lint_group(),
        "Who will hold the managerial reins after April?",
    );
}

#[test]
fn fixes_managerial_reigns_in_quotes() {
    assert_suggestion_result(
        "\"managerial reigns\" showed up in the draft notes.",
        lint_group(),
        "\"managerial reins\" showed up in the draft notes.",
    );
}

#[test]
fn counts_managerial_reigns_error() {
    assert_lint_count(
        "They debated who should manage the managerial reigns for the quarter.",
        lint_group(),
        1,
    );
}

#[test]
fn counts_managerial_reigns_caps() {
    assert_lint_count("Their memo shouted MANAGERIAL REIGNS.", lint_group(), 1);
}

#[test]
fn allows_managerial_reins_correct() {
    assert_no_lints(
        "He kept the managerial reins despite the reshuffle.",
        lint_group(),
    );
}

#[test]
fn allows_reigns_without_managerial() {
    assert_no_lints("Legends of ancient reigns filled the museum.", lint_group());
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

// NotUncommon moved to phrase_set_corrections as part of the
// generalized double negative mapping.

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

// QuiteMany
#[test]
fn corrects_quite_many() {
    assert_suggestion_result(
        "To me it seems it might be caused by a2aaa55 which contains quite many build-related changes.",
        lint_group(),
        "To me it seems it might be caused by a2aaa55 which contains quite a few build-related changes.",
    );
}

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

// RulesOfThumb

#[test]
fn correct_rules_of_thumbs() {
    assert_suggestion_result(
        "Thanks. 0.2 is just from my rule of thumbs.",
        lint_group(),
        "Thanks. 0.2 is just from my rules of thumb.",
    );
}

#[test]
fn correct_rules_of_thumbs_hyphenated() {
    assert_suggestion_result(
        "Add rule-of-thumbs for basic metrics, like \"Spill more than 1GB is a red flag\".",
        lint_group(),
        "Add rules of thumb for basic metrics, like \"Spill more than 1GB is a red flag\".",
    );
}

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

// ToGreatLengths
#[test]
fn correct_through_great_lengths() {
    assert_suggestion_result(
        "Bloomberg's sponsored paid for content goes through great lengths to market Nvidia's products and in particular its AI products that we've frequently criticized.",
        lint_group(),
        "Bloomberg's sponsored paid for content goes to great lengths to market Nvidia's products and in particular its AI products that we've frequently criticized.",
    );
}

#[test]
fn correct_to_a_great_length() {
    assert_suggestion_result(
        "While ratatui-image goes to a great length to detect a rendered image's pixel size in terms of \"character cells that will be covered\", via font pixel size detection, ultimately it's up to the terminal emulator to decide what exactly a pixel is.",
        lint_group(),
        "While ratatui-image goes to great lengths to detect a rendered image's pixel size in terms of \"character cells that will be covered\", via font pixel size detection, ultimately it's up to the terminal emulator to decide what exactly a pixel is.",
    );
}

// ToTheMannerBorn
// -none-

// TongueInCheek
#[test]
fn tongue_and_cheek_plain() {
    assert_suggestion_result(
        "The remark was entirely tongue and cheek.",
        lint_group(),
        "The remark was entirely tongue in cheek.",
    );
}

#[test]
fn tongue_and_cheek_with_article() {
    assert_suggestion_result(
        "It was a tongue and cheek response.",
        lint_group(),
        "It was a tongue in cheek response.",
    );
}

#[test]
fn tongue_and_cheek_with_comma() {
    assert_suggestion_result(
        "He delivered it tongue and cheek, expecting a laugh.",
        lint_group(),
        "He delivered it tongue in cheek, expecting a laugh.",
    );
}

#[test]
fn tongue_and_cheek_in_quotes() {
    assert_suggestion_result(
        "\"tongue and cheek\" jokes are tough to read.",
        lint_group(),
        "\"tongue in cheek\" jokes are tough to read.",
    );
}

#[test]
fn tongue_and_cheek_all_caps() {
    assert_suggestion_result(
        "Their tone was TONGUE AND CHEEK all night.",
        lint_group(),
        "Their tone was TONGUE IN CHEEK all night.",
    );
}

#[test]
fn tongue_and_cheek_capitalized() {
    assert_suggestion_result(
        "Tongue and cheek banter kept the meeting light.",
        lint_group(),
        "Tongue in cheek banter kept the meeting light.",
    );
}

#[test]
fn tongue_and_cheek_in_parentheses() {
    assert_suggestion_result(
        "Her note (totally tongue and cheek) made us smile.",
        lint_group(),
        "Her note (totally tongue in cheek) made us smile.",
    );
}

#[test]
fn tongue_and_cheek_question() {
    assert_suggestion_result(
        "Was that tongue and cheek or sincere?",
        lint_group(),
        "Was that tongue in cheek or sincere?",
    );
}

#[test]
fn tongue_in_cheek_is_allowed() {
    assert_lint_count(
        "Their comments were deliberately tongue in cheek.",
        lint_group(),
        0,
    );
}

#[test]
fn tongue_in_cheek_hyphenated_is_allowed() {
    assert_lint_count("That was a tongue-in-cheek reply.", lint_group(), 0);
}

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
fn thanks_a_lot_clean() {
    assert_lint_count("thanks a lot", lint_group(), 0);
}

#[test]
fn fixes_to_worry_about() {
    assert_top3_suggestion_result(
        "I don't want you to worried about it.",
        lint_group(),
        "I don't want you to worry about it.",
    );
    assert_top3_suggestion_result(
        "I don't want you to worried about it.",
        lint_group(),
        "I don't want you too worried about it.",
    );
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
fn fixes_teh() {
    assert_suggestion_result(
        "I adore teh light of the moon.",
        lint_group(),
        "I adore the light of the moon.",
    );
}

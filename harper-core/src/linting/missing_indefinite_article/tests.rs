use super::*;
use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

mod coverage_gaps {
    use super::*;
    use crate::linting::tests::assert_first_suggestion_result;

    macro_rules! corrections {
        ($($name:ident: $input:literal => $expected:literal;)*) => {
            $(
                #[test]
                fn $name() {
                    assert_no_lints(
                        $expected,
                        MissingIndefiniteArticle::new(Dialect::American),
                    );
                    assert_first_suggestion_result(
                        $input,
                        MissingIndefiniteArticle::new(Dialect::American),
                        $expected,
                    );
                }
            )*
        };
    }

    corrections! {
        after_make: "The intern made urgent request." => "The intern made an urgent request.";
        after_take: "The pilot took unexpected detour." => "The pilot took an unexpected detour.";
        after_carry: "The gardener carried heavy bucket." => "The gardener carried a heavy bucket.";
        after_use: "The carpenter used electric drill." => "The carpenter used an electric drill.";
        after_for: "Mira waited for available appointment." => "Mira waited for an available appointment.";
        ask_for: "The visitor asked for printed map." => "The visitor asked for a printed map.";
        adverb_before_adjective: "They worked in unusually quiet room." => "They worked in an unusually quiet room.";
        participial_modifier: "A fox hid behind fallen tree." => "A fox hid behind a fallen tree.";
        sentence_initial: "Careful editor can spot this error." => "A careful editor can spot this error.";
        initial_prepositional_phrase: "During storm, the beacon remained visible." => "During a storm, the beacon remained visible.";
        after_such: "The rescue required such unusual vehicle." => "The rescue required such an unusual vehicle.";
        modified_object: "The gallery displayed unusually vivid portrait." => "The gallery displayed an unusually vivid portrait.";
        multiple_adjectives: "The cyclist has bright yellow bicycle." => "The cyclist has a bright yellow bicycle.";
        compound_head: "The hostel needs emergency exit sign." => "The hostel needs an emergency exit sign.";
        capitalized_modifier: "The clerk bought European travel guide." => "The clerk bought a European travel guide.";
        silent_h_compound: "The designer has hour-long meeting tomorrow." => "The designer has an hour-long meeting tomorrow.";
        existential: "There was slight delay at customs." => "There was a slight delay at customs.";
        other_object: "The artist painted vivid portrait." => "The artist painted a vivid portrait.";
        other_preposition: "They waited beside old tower." => "They waited beside an old tower.";
        other_subject: "Young musician played the piano." => "A young musician played the piano.";
        other_approximation: "Almost million people attended." => "Almost a million people attended.";
        other_quote: "She has \"old bicycle\" in the shed." => "She has an \"old bicycle\" in the shed.";
        quoted_noun_phrase: "The cabin has \"spare key\" in the drawer." => "The cabin has a \"spare key\" in the drawer.";
        approximate_number: "Nearly thousand visitors attended the fair." => "Nearly a thousand visitors attended the fair.";
        gerund_modifier: "The puppy found hiding place beneath the stairs." => "The puppy found a hiding place beneath the stairs.";
    }

    #[test]
    fn allows_valid_near_matches() {
        for input in [
            "The carpenter used electric drills.",
            "The visitor asked for directions.",
            "The gardener carried water.",
            "The carpenter used her electric drill.",
            "Careful editors can spot this error.",
            "They worked in quiet rooms.",
            "The cyclist has two yellow bicycles.",
            "The gallery displayed unusually vivid portraits.",
            "They worked in unusually cold water.",
            "They worked in a quiet room.",
            "They worked in unusually quiet rooms.",
            "She has a hiding place.",
            "She found hiding places.",
            "He has finished the book.",
            "She is carrying water.",
            "They walked side by side.",
            "Almost two million people attended.",
            "Nearly a thousand visitors arrived.",
            "The artist painted her vivid portrait.",
            "The artist painted vivid portraits.",
            "Young musicians played the piano.",
            "They waited beside the old tower.",
            "She has \"old bicycles\" in the shed.",
            "Rune is a TUI editor.",
            "Successful completion carrying an optional payload.",
            "Voluntary cessation initiated by the caller.",
            "But there is less consensus on this.",
            "They did it for good measure.",
            "She has painted doors.",
            "They are carrying buckets.",
        ] {
            assert_no_lints(input, MissingIndefiniteArticle::new(Dialect::American));
        }
    }
}

mod corpus_regressions {
    use super::*;
    use crate::linting::tests::assert_first_suggestion_result;

    macro_rules! allows {
        ($($name:ident: $text:literal;)*) => {$(
            #[test]
            fn $name() {
                assert_no_lints($text, MissingIndefiniteArticle::new(Dialect::American));
            }
        )*};
    }
    allows! {
        content_creation: "They offer fields for content creation.";
        community_engagement: "Community engagement is central to this project.";
        determined_participle: "A meticulously maintained card catalog system.";
        determined_adverb: "She has a comfortably furnished therapy office.";
        possessive_participle: "She has her carefully restored vintage bicycle.";
        possessive_noun: "They used John's carefully restored vintage bicycle.";
        gerund_complement: "They focus on building rapport.";
        gerund_object: "They considered providing food for the visitors.";
        gerund_preposition: "They are responsible for providing food and water.";
        coordinated_predicate: "These bowls are oven, freezer, and microwave safe.";
        two_coordinated_modifiers: "The container is oven and microwave safe.";
        reduced_passive: "Local pickup preferred.";
        reduced_passive_variant: "Prompt payment required.";
        abstract_definition: "They use this for batch task definition.";
        mass_attention: "Pay close attention to the opening solo.";
        finite_clause: "A careful editor reviewed the manuscript.";
        determined_quote: "She has a \"carefully restored bicycle\" in the shed.";
    }

    macro_rules! corrects {
        ($($name:ident: $input:literal => $expected:literal;)*) => {$(
            #[test]
            fn $name() {
                assert_no_lints($expected, MissingIndefiniteArticle::new(Dialect::American));
                assert_first_suggestion_result($input, MissingIndefiniteArticle::new(Dialect::American), $expected);
            }
        )*};
    }
    corrects! {
        modified_system: "She has unusual card catalog system." => "She has an unusual card catalog system.";
        later_phrase: "She has a carefully restored bicycle and needs new helmet." => "She has a carefully restored bicycle and needs a new helmet.";
        gerund_modifier: "They bought sewing machine." => "They bought a sewing machine.";
        nominal_building: "They worked in building basement." => "They worked in a building basement.";
        participial_modifier: "They waited beside fallen tree." => "They waited beside a fallen tree.";
        finite_subject: "Local editor reviewed the manuscript." => "A local editor reviewed the manuscript.";
        finite_past_subject: "Young musician played the piano." => "A young musician played the piano.";
        count_creation: "She bought unusual creation." => "She bought an unusual creation.";
        count_engagement: "She has prior engagement." => "She has a prior engagement.";
        count_definition: "We need precise definition." => "We need a precise definition.";
        ordinary_predicate: "This is useful tool and it is safe." => "This is a useful tool and it is safe.";
        working_computer: "They sat beside working computer." => "They sat beside a working computer.";
        swimming_pool: "They waited beside swimming pool." => "They waited beside a swimming pool.";
        intransitive_predicate: "Experienced teacher arrived." => "An experienced teacher arrived.";
        predicate_creation: "This is original creation." => "This is an original creation.";
        attended_engagement: "We attended formal engagement." => "We attended a formal engagement.";
        predicate_definition: "This is precise definition." => "This is a precise definition.";
    }
}

mod further_corpus_regressions {
    use super::*;
    use crate::linting::tests::assert_first_suggestion_result;

    macro_rules! allows {
        ($($name:ident: $text:literal;)*) => {$(
            #[test]
            fn $name() {
                assert_no_lints($text, MissingIndefiniteArticle::new(Dialect::American));
            }
        )*};
    }
    allows! {
        modal_contact: "They should contact customer support.";
        modal_track: "We must track project progress.";
        modal_coordinate: "The group will coordinate volunteer planting days.";
        modal_switch: "They can quickly switch between projects.";
        modal_document: "They will carefully document each incident.";
        help_infinitive: "Help spread compost and prepare the soil for planting.";
        helped_infinitive: "They helped spread fresh compost.";
        to_switch: "The plugin allows users to quickly switch between projects.";
        to_switch_after_code: "Simply put, `telescope-project` allows you to quickly switch between projects based on the root directory containing a recognized project marker file.";
        to_review: "They want to carefully review the report.";
        stewardship_subject: "Environmental stewardship is important.";
        stewardship_object: "They focus on responsible stewardship.";
        reclamation_subject: "Water reclamation is operating normally.";
        reclamation_object: "They improved water reclamation.";
        attendance_subject: "Regular attendance is crucial.";
        attendance_object: "They expect regular attendance.";
    }

    macro_rules! corrects {
        ($($name:ident: $input:literal => $expected:literal;)*) => {$(
            #[test]
            fn $name() {
                assert_no_lints($expected, MissingIndefiniteArticle::new(Dialect::American));
                assert_first_suggestion_result($input, MissingIndefiniteArticle::new(Dialect::American), $expected);
            }
        )*};
    }
    corrects! {
        object_after_modal: "They can buy electric drill." => "They can buy an electric drill.";
        help_nominal_object: "They helped injured child." => "They helped an injured child.";
        help_later_object: "Help buy electric drill." => "Help buy an electric drill.";
        nominal_to: "They went to old tower." => "They went to an old tower.";
        nominal_switch: "The plugin has useful switch." => "The plugin has a useful switch.";
        latency_modifier: "They need latency monitor." => "They need a latency monitor.";
        compost_modifier: "They need compost container." => "They need a compost container.";
        adverbial_nominal: "They waited beside unusually tall tower." => "They waited beside an unusually tall tower.";
        attendance_modifier: "They need regular attendance record." => "They need a regular attendance record.";
        reclamation_modifier: "They bought reclamation machine." => "They bought a reclamation machine.";
        stewardship_modifier: "They need stewardship manual." => "They need a stewardship manual.";
    }
}

mod mass_senses {
    use super::*;
    use crate::linting::tests::assert_first_suggestion_result;

    #[test]
    fn allows_materials_aggregate_activity_and_public_services() {
        for text in [
            "They monitored economic activity.",
            "Economic activity is increasing.",
            "They measured regional economic activity.",
            "The report describes changes in economic activity.",
            "They monitored unusual economic activity.",
            "They recycled agricultural waste.",
            "Agricultural waste is collected daily.",
            "They worked with agricultural waste.",
            "They reduced industrial waste.",
            "We need less agricultural waste.",
            "She devoted her life to public service.",
            "Public service is important.",
            "They provide reliable public service.",
            "She is known for excellent customer service.",
            "They restored regular bus service.",
            "They planned activities for the children.",
            "They attended religious services.",
            "It is a waste of time.",
        ] {
            assert_no_lints(text, MissingIndefiniteArticle::new(Dialect::American));
        }
    }

    #[test]
    fn preserves_individual_events_and_waste_of_construction() {
        for (input, expected) in [
            (
                "They planned entertaining activity.",
                "They planned an entertaining activity.",
            ),
            (
                "They scheduled outdoor activity.",
                "They scheduled an outdoor activity.",
            ),
            (
                "We attended memorial service.",
                "We attended a memorial service.",
            ),
            (
                "They scheduled religious service.",
                "They scheduled a religious service.",
            ),
            ("It is waste of time.", "It is a waste of time."),
            ("That was waste of money.", "That was a waste of money."),
        ] {
            assert_no_lints(expected, MissingIndefiniteArticle::new(Dialect::American));
            assert_first_suggestion_result(
                input,
                MissingIndefiniteArticle::new(Dialect::American),
                expected,
            );
        }
    }
}

#[test]
fn private_children_are_enabled_and_handle_distinct_contexts() {
    let document = Document::new_markdown_default_curated(
        "She needs umbrella. They waited beside old tower. Careful editor can spot this error. They need such unusual vehicle. Nearly thousand visitors attended.",
    );
    let mut rule = MissingIndefiniteArticle::new(Dialect::American);
    let results = rule.group.organized_lints(&document);
    assert_eq!(results.len(), 5);
    for (name, lints) in results {
        assert_eq!(lints.len(), 1, "{name}");
    }
    rule.group.set_all_rules_to(Some(false));
    assert!(rule.lint(&document).is_empty());
}

#[test]
fn dialect_reaches_each_article_inserting_child() {
    use crate::linting::tests::assert_first_suggestion_result;
    for (input, american, british) in [
        (
            "She needs herb garden.",
            "She needs an herb garden.",
            "She needs a herb garden.",
        ),
        (
            "They waited beside herb garden.",
            "They waited beside an herb garden.",
            "They waited beside a herb garden.",
        ),
        (
            "Herb garden can grow.",
            "An herb garden can grow.",
            "A herb garden can grow.",
        ),
        (
            "They need such herb garden.",
            "They need such an herb garden.",
            "They need such a herb garden.",
        ),
        (
            "Nearly hundred visitors attended.",
            "Nearly a hundred visitors attended.",
            "Nearly a hundred visitors attended.",
        ),
    ] {
        for (dialect, expected) in [(Dialect::American, american), (Dialect::British, british)] {
            assert_first_suggestion_result(input, MissingIndefiniteArticle::new(dialect), expected);
        }
    }
}

#[test]
fn overlapping_subject_and_verb_candidates_insert_only_once() {
    use crate::linting::tests::assert_first_suggestion_result;
    let input = "Wanted poster can identify the suspect.";
    let mut rule = MissingIndefiniteArticle::new(Dialect::American);
    let document = Document::new_markdown_default_curated(input);
    assert_eq!(rule.lint(&document).len(), 1);
    assert_first_suggestion_result(input, rule, "A wanted poster can identify the suspect.");
}

#[test]
fn public_setting_controls_all_private_children() {
    use crate::spell::FstDictionary;
    let document = Document::new_markdown_default_curated("She needs umbrella.");
    let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    group.set_all_rules_to(Some(false));
    assert!(group.lint(&document).is_empty());
    group
        .config
        .set_rule_enabled("MissingIndefiniteArticle", true);
    let results = group.organized_lints(&document);
    assert_eq!(results.len(), 1);
    assert_eq!(results["MissingIndefiniteArticle"].len(), 1);
}

#[test]
fn merged_group_preserves_source_order_without_duplicate_insertions() {
    let mut rule = MissingIndefiniteArticle::new(Dialect::American);
    let document = Document::new_markdown_default_curated(
        "They waited beside fallen tree. She needs umbrella. Nearly thousand visitors attended.",
    );
    let lints = rule.lint(&document);
    assert_eq!(lints.len(), 3);
    assert!(
        lints
            .windows(2)
            .all(|pair| pair[0].span.start < pair[1].span.start)
    );
}

#[test]
fn teacher() {
    assert_suggestion_result(
        "She is teacher.",
        MissingIndefiniteArticle::new(Dialect::American),
        "She is a teacher.",
    );
}

#[test]
fn engineer() {
    assert_suggestion_result(
        "He is engineer.",
        MissingIndefiniteArticle::new(Dialect::American),
        "He is an engineer.",
    );
}

#[test]
fn idea() {
    assert_suggestion_result(
        "This is good idea.",
        MissingIndefiniteArticle::new(Dialect::American),
        "This is a good idea.",
    );
}

#[test]
fn book() {
    assert_suggestion_result(
        "I bought book.",
        MissingIndefiniteArticle::new(Dialect::American),
        "I bought a book.",
    );
}

#[test]
fn umbrella() {
    assert_suggestion_result(
        "We need umbrella.",
        MissingIndefiniteArticle::new(Dialect::American),
        "We need an umbrella.",
    );
}

#[test]
fn doctor() {
    assert_suggestion_result(
        "She became doctor.",
        MissingIndefiniteArticle::new(Dialect::American),
        "She became a doctor.",
    );
}

#[test]
fn compound() {
    assert_suggestion_result(
        "He is university student.",
        MissingIndefiniteArticle::new(Dialect::American),
        "He is a university student.",
    );
}

#[test]
fn silent_h() {
    assert_suggestion_result(
        "It is honest mistake.",
        MissingIndefiniteArticle::new(Dialect::American),
        "It is an honest mistake.",
    );
}

#[test]
fn car() {
    assert_suggestion_result(
        "I have car.",
        MissingIndefiniteArticle::new(Dialect::American),
        "I have a car.",
    );
}

#[test]
fn accident() {
    assert_suggestion_result(
        "It was accident.",
        MissingIndefiniteArticle::new(Dialect::American),
        "It was an accident.",
    );
}

#[test]
fn hour() {
    assert_suggestion_result(
        "We need hour.",
        MissingIndefiniteArticle::new(Dialect::American),
        "We need an hour.",
    );
}

#[test]
fn adjectives() {
    assert_suggestion_result(
        "This is interesting new idea.",
        MissingIndefiniteArticle::new(Dialect::American),
        "This is an interesting new idea.",
    );
}

#[test]
fn whitespace() {
    assert_suggestion_result(
        "I need\tcar.",
        MissingIndefiniteArticle::new(Dialect::American),
        "I need\ta car.",
    );
}

#[test]
fn capitalization() {
    assert_suggestion_result(
        "She IS teacher.",
        MissingIndefiniteArticle::new(Dialect::American),
        "She IS a teacher.",
    );
}

#[test]
fn unicode_prefix() {
    assert_suggestion_result(
        "Élodie has car.",
        MissingIndefiniteArticle::new(Dialect::American),
        "Élodie has a car.",
    );
}

#[test]
fn supports_consonant_pronunciation() {
    assert_suggestion_result(
        "This is ubiquitous problem.",
        MissingIndefiniteArticle::new(Dialect::American),
        "This is a ubiquitous problem.",
    );
}

#[test]
fn supports_vowel_pronunciation() {
    assert_suggestion_result(
        "This is ubiquitous problem.",
        MissingIndefiniteArticle::new(Dialect::American),
        "This is an ubiquitous problem.",
    );
}

#[test]
fn curated_rule_inserts_once() {
    use crate::{linting::LintGroup, spell::FstDictionary};
    assert_suggestion_result(
        "He is engineer.",
        LintGroup::new_curated(FstDictionary::curated(), Dialect::American),
        "He is an engineer.",
    );
}

#[test]
fn allows_indefinite_article_a() {
    assert_no_lints(
        "She is a teacher.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_indefinite_article_an() {
    assert_no_lints(
        "He is an engineer.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_definite_article() {
    assert_no_lints(
        "This is the idea.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_possessive_determiner() {
    assert_no_lints(
        "I have my car.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_quantified_plural() {
    assert_no_lints(
        "We need some books.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_plural_predicate() {
    assert_no_lints(
        "They are teachers.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_plural_compound() {
    assert_no_lints(
        "I bought book reviews.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_happy() {
    assert_no_lints(
        "He is happy.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_advice() {
    assert_no_lints(
        "They need advice.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_water() {
    assert_no_lints(
        "This is water.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_work() {
    assert_no_lints(
        "We have work.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_president() {
    assert_no_lints(
        "She became president.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_home() {
    assert_no_lints(
        "He is home.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_lunch() {
    assert_no_lints(
        "We had lunch.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_proper_noun() {
    assert_no_lints(
        "This is London.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_possessive_noun() {
    assert_no_lints(
        "I have John's book.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_perfect_progressive() {
    assert_no_lints(
        "She has been working.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_infinitive() {
    assert_no_lints(
        "I need to go.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_likely_intended() {
    assert_no_lints(
        "It is likely intended.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_finished() {
    assert_no_lints(
        "She has finished.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_running() {
    assert_no_lints(
        "He is running.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_part_of() {
    assert_no_lints(
        "It is part of a compound.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_mayor() {
    assert_no_lints(
        "She became mayor.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_plenty_of() {
    assert_no_lints(
        "She had plenty of time.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_good_practice() {
    assert_no_lints(
        "It was good practice to say it.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_judge_and_jury() {
    assert_no_lints(
        "I'll be judge, I'll be jury.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_silence() {
    assert_no_lints(
        "There was silence.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_postpositive_enough() {
    assert_no_lints(
        "There was mouth enough for it to speak.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_title_in_question() {
    assert_no_lints(
        "Is Mr. Smith in?",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_alert() {
    assert_no_lints(
        "He were alert for something.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_operative() {
    assert_no_lints(
        "It becomes operative from today.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_party() {
    assert_no_lints(
        "They shall be Party.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_subject_for() {
    assert_no_lints(
        "No person shall be subject for the offense.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_comparative() {
    assert_no_lints(
        "Laughter is easier minute by minute.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_nephew() {
    assert_no_lints(
        "He was nephew to the king.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_right_next_door() {
    assert_no_lints(
        "The house is right next door.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_common_property() {
    assert_no_lints(
        "They were common property.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_luncheon() {
    assert_no_lints(
        "We had luncheon.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_such_majority() {
    assert_no_lints(
        "They have such majority.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_unchecked_as_phrase() {
    assert_no_lints(
        "She works as nurse.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_institutional_school() {
    assert_no_lints(
        "Go to school.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_empty_input() {
    assert_no_lints("", MissingIndefiniteArticle::new(Dialect::American));
}

#[test]
fn allows_standalone_noun() {
    assert_no_lints("teacher", MissingIndefiniteArticle::new(Dialect::American));
}

#[test]
fn allows_good() {
    assert_no_lints(
        "This is good.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_software() {
    assert_no_lints(
        "We need software.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_trouble() {
    assert_no_lints(
        "I have trouble.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_intervening_punctuation() {
    assert_no_lints(
        "She is, teacher.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_inline_code() {
    assert_no_lints(
        "I need `car`.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_fine() {
    assert_no_lints(
        "I am fine.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_well() {
    assert_no_lints(
        "He is well.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_kind() {
    assert_no_lints(
        "She is kind.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_great() {
    assert_no_lints(
        "This is great.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_nice() {
    assert_no_lints(
        "It is nice.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_tall() {
    assert_no_lints(
        "He is tall.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_short() {
    assert_no_lints(
        "She is short.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_ready() {
    assert_no_lints(
        "It is ready.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_smart() {
    assert_no_lints(
        "She is smart.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_safe() {
    assert_no_lints(
        "It is safe.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_calm() {
    assert_no_lints(
        "He is calm.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_quiet() {
    assert_no_lints(
        "She is quiet.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_right() {
    assert_no_lints(
        "He is right.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_wrong() {
    assert_no_lints(
        "She is wrong.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_mine() {
    assert_no_lints(
        "It is mine.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_yours() {
    assert_no_lints(
        "This is yours.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_permission() {
    assert_no_lints(
        "We have permission.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_access() {
    assert_no_lints(
        "They have access.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_cancer() {
    assert_no_lints(
        "He has cancer.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_flu() {
    assert_no_lints(
        "She has flu.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_hope() {
    assert_no_lints(
        "I have hope.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_sleep() {
    assert_no_lints(
        "We need sleep.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_help() {
    assert_no_lints(
        "We need help.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_peace() {
    assert_no_lints(
        "I want peace.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_rest() {
    assert_no_lints(
        "They need rest.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_in_bed() {
    assert_no_lints(
        "He is in bed.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_at_school() {
    assert_no_lints(
        "He is at school.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_milk() {
    assert_no_lints(
        "This is milk.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_gold() {
    assert_no_lints(
        "This is gold.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_wood() {
    assert_no_lints(
        "It is wood.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_courage() {
    assert_no_lints(
        "She has courage.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_famous() {
    assert_no_lints(
        "He became famous.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_breakfast() {
    assert_no_lints(
        "They have breakfast.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_faith() {
    assert_no_lints(
        "I have faith.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_father() {
    assert_no_lints(
        "He is father to three children.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_mother() {
    assert_no_lints(
        "She is mother of twins.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_proof() {
    assert_no_lints(
        "This is proof of success.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_winter() {
    assert_no_lints(
        "It is winter.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_spring() {
    assert_no_lints(
        "It was spring.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_night() {
    assert_no_lints(
        "It is night.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_nonsense() {
    assert_no_lints(
        "This is nonsense.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_business() {
    assert_no_lints(
        "This is business.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_news() {
    assert_no_lints(
        "It is good news.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_company() {
    assert_no_lints(
        "We have company.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_life() {
    assert_no_lints(
        "This is real life.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_in_love() {
    assert_no_lints(
        "They are in love.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_laughter() {
    assert_no_lints(
        "There was laughter.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_joy() {
    assert_no_lints(
        "There was joy.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_experience() {
    assert_no_lints(
        "He has experience.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_second_in_command() {
    assert_no_lints(
        "He is second in command.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_professor() {
    assert_no_lints(
        "She is professor of physics.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_impossible() {
    assert_no_lints(
        "It is impossible.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_available() {
    assert_no_lints(
        "He is available.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_gone() {
    assert_no_lints(
        "She has gone.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_happened() {
    assert_no_lints(
        "It has happened.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_left() {
    assert_no_lints(
        "They have left.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_arrived() {
    assert_no_lints(
        "We have arrived.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_asleep() {
    assert_no_lints(
        "He is asleep.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_awake() {
    assert_no_lints(
        "She is awake.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_alive() {
    assert_no_lints(
        "We are alive.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_alone() {
    assert_no_lints(
        "He is alone.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_worth() {
    assert_no_lints(
        "It is worth the effort.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_done() {
    assert_no_lints(
        "This is done.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

#[test]
fn allows_exchange() {
    assert_no_lints(
        "There has been exchange of ideas.",
        MissingIndefiniteArticle::new(Dialect::American),
    );
}

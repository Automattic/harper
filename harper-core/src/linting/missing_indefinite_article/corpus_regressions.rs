use super::MissingIndefiniteArticle;
use crate::{
    Dialect,
    linting::tests::{assert_first_suggestion_result, assert_no_lints},
};

macro_rules! allows {
    ($($name:ident: $text:literal;)*) => {$(
        #[test]
        fn $name() { assert_no_lints($text, MissingIndefiniteArticle::new(Dialect::American)); }
    )*};
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

allows! {
    food: "They serve delicious food.";
    low_latency: "They require low latency.";
    passive_sown: "The seeds were being sown.";
    passive_temporal_suffix: "The applications will be evaluated post-launch.";
    gerund_tying: "They learn advanced knot-tying.";
    prefixed_infinitive: "They need time to re-emerge.";
    prefixed_roll: "They chose to re-roll the dice.";
    compound_infinitive: "They need to fine-tune the instrument.";
    attack: "They protect the material from microbial attack.";
    burden: "The process reduces landfill burden.";
    survey: "They measured satisfaction via in-app survey.";
    wheat: "They grow blight-resistant wheat.";
    robotics: "They study bio-inspired robotics.";
    nomenclature: "They are familiar with botanical nomenclature.";
    pneumonia: "He has community-acquired pneumonia.";
    material_substrate: "They use sterilized substrate.";
    adverbial_site: "Parking is available on-site.";
    adverbial_post: "They collect user feedback post-launch.";
    progressive_preposition: "They restored service following rollback.";
    finite_after_gerund: "One state of being gives way to another.";
    finite_after_mass: "Handling sensitive information requires strict adherence.";
    meat_shoulder: "They serve slow-roasted pork shoulder.";
    meat_breast: "They serve pan-seared duck breast.";
    repeated_side: "They worked side-by-side.";
    repeated_eye: "They are not seeing eye-to-eye.";
    processing_latency: "They measured order processing latency.";
    adjective_teacher: "She is a teacher.";
    nominal_after_plural: "They reported significant concerns regarding toxicity.";
    nominal_before_auxiliary: "They work within libraries that are open.";
    adverbial_hyphen: "The trail is passable year-round.";
    relative_clause: "They empower families facing adversity.";
    assessment: "The process requires careful assessment.";
    completion: "Upon successful completion, students receive a certificate.";
    delivery: "They offer free delivery.";
    admission: "Daily admission is free.";
    composition: "Atmospheric composition remains stable.";
    utilization: "They observed high CPU utilization.";
    revenue: "They reported increased tax revenue.";
    fatigue: "She reported mild fatigue.";
    oversight: "The project requires careful oversight.";
    review: "The proposal requires manual review.";
    intervention: "The system requires manual intervention.";
    identity: "They protect personal identity.";
    error: "The process is prone to human error.";
    application: "The course emphasizes practical application.";
    scholarship: "The library supports quiet scholarship.";
    purchase: "Inspect each item before purchase.";
    reference: "Keep the manual for future reference.";
    supply: "The tokens are in short supply.";
    correspondence: "They found extensive personal correspondence.";
    note: "Of particular note is the final chapter.";
    plurals: "Preliminary findings indicate several problems.";
    populations: "They studied vulnerable populations.";
    phenomena: "They studied unusual phenomena.";
    definitions: "The book contains clear definitions.";
    currents: "They observed strong currents.";
    station_name: "They waited at Central Station.";
    product_name: "They are configuring Telescope to find files.";
    relative_time: "They met last week.";
    future_time: "They are back next week.";
    peak_season: "They visited during peak season.";
    placeholder: "Please visit [insert website address here].";
    gif_placeholder: "[insert victory-dance GIF]";
    determined_comma: "They found a small, locked wooden box.";
    determined_untagged: "She wore a faded denim jacket.";
    determined_hyphen: "It has a built-in accountability system.";
    gerund_head: "They need efficient file handling.";
    plural_compound: "They reviewed recurring client needs.";
    adjective_predicate: "Please be patient with the children.";
    firm_predicate: "The shrimp should be firm but tender.";
    file_browsing: "They use traditional file browsing tools.";
    acronym_modifier: "They need a USB cable.";
    finite_monitors: "The editor monitors changes.";
}

corrects! {
    count_review: "She wrote helpful review." => "She wrote a helpful review.";
    count_survey: "They conducted detailed survey." => "They conducted a detailed survey.";
    count_attack: "They launched surprise attack." => "They launched a surprise attack.";
    count_burden: "They carried heavy burden." => "They carried a heavy burden.";
    ordinary_shoulder: "He has sore shoulder." => "He has a sore shoulder.";
    hyphen_modifier: "They need built-in amplifier." => "They need a built-in amplifier.";
    count_assessment: "She wrote detailed assessment." => "She wrote a detailed assessment.";
    count_response: "She sent detailed response." => "She sent a detailed response.";
    count_composition: "He wrote musical composition." => "He wrote a musical composition.";
    count_entry: "She wrote diary entry." => "She wrote a diary entry.";
    count_note: "She left short note." => "She left a short note.";
    count_error: "They found syntax error." => "They found a syntax error.";
    count_application: "She submitted detailed application." => "She submitted a detailed application.";
    count_assignment: "She completed difficult assignment." => "She completed a difficult assignment.";
    count_reference: "He added bibliographic reference." => "He added a bibliographic reference.";
    count_pickup: "He bought new pickup." => "He bought a new pickup.";
    count_scope: "She bought expensive scope." => "She bought an expensive scope.";
    count_study: "She published detailed study." => "She published a detailed study.";
    mass_modifier: "They need food container." => "They need a food container.";
    plural_modifier: "They need findings summary." => "They need a findings summary.";
    latency_monitor: "They need latency monitor." => "They need a latency monitor.";
    ordinary_station: "They waited at old station." => "They waited at an old station.";
    nominal_hyphen_after_to: "They went to red-brick tower." => "They went to a red-brick tower.";
    capital_adjective: "She is French teacher." => "She is a French teacher.";
    initial_subject: "Careful editor finds mistakes." => "A careful editor finds mistakes.";
    finite_needs: "Careful editor needs help." => "A careful editor needs help.";
    ordinary_time: "They waited for entire week." => "They waited for an entire week.";
    bracketed_prose: "[She needs useful tool.]" => "[She needs a useful tool.]";
    insert_imperative: "Insert valid address here." => "Insert a valid address here.";
    modified_patient: "They helped injured patient." => "They helped an injured patient.";
}

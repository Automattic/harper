//! Synthetic training data that closes the model's out-of-distribution gaps:
//! the error constructions the linters check, which never appear in the
//! grammatical UD treebanks, so the model mistags them out-of-distribution —
//! almost always defaulting an ambiguous content word to NOUN. We generate many
//! *natural, good-length* sentences from varied vocabulary pools so the model
//! learns the PATTERN (and generalizes), with the lint-critical words mixed in
//! as instances. Each injected sentence is the literal ERROR construction but
//! tagged with the word's CORRECT underlying POS.
//!
//! IMPORTANT (round 7): the corrective templates must contain the LITERAL error
//! trigger words the rule scans for ("all ready", "each others", "there",
//! "rainbow colored", "tomorrows", "make up") with the neighbouring POS slot
//! tagged correctly. Round 6 taught the grammatical/corrected form ("already",
//! "their") and so never moved those weir rules. Round 7 also drops the
//! "Its"+ADJECTIVE family (it destabilized its_possessive without fixing its
//! target) and keeps protective examples for the must-not-flag cases. Families:
//!   1. noun-noun possessive head ("the"/"a" determiner)   possessive_noun
//!   2. -s verb after a noun subject                        noun_verb / weir::wildcard
//!   3. control-verb + bare verb                            missing_to
//!   4. "way to" + adjective                                way_too_adjective
//!   5. "Its" + past/participle verb                        its_contraction
//!   6. affect/effect: attributive NOUN+NOUN & protective   noun_verb_confusion
//!   7. weir error-form slots (literal trigger + POS slot)  weir_rules
//!   8. imperative sentence-initial VERB                    were_where

use crate::UPOS;

/// A template cell: either a fixed function word or a slot drawn from a pool.
enum Cell {
    Fixed(&'static str),
    Pool(&'static [&'static str]),
}
use Cell::{Fixed, Pool};

// ---- Vocabulary pools (lint-critical words mixed with varied vocab) ---------

const PLURAL_NOUNS: &[&str] = &[
    "birds", "farmers", "neighbors", "flowers", "students", "teachers", "writers",
    "singers", "villagers", "painters", "dogs", "workers", "tourists", "theses", "managers",
];
const HEAD_NOUNS: &[&str] = &[
    "song", "garden", "scent", "crops", "apples", "code", "report", "notes",
    "fields", "stories", "music", "budget", "schedule", "designs", "houses",
];
const ADJS: &[&str] = &[
    "elusive", "stiff", "quiet", "ancient", "clever", "fragile", "bright",
    "complicated", "subtle", "restless",
];
const SUBJECTS: &[&str] = &[
    "offer", "rumor", "greenhouse", "proposal", "manager", "economy", "storm",
    "silence", "lecture", "weather",
];
const VERBS_S: &[&str] = &[
    "tempts", "warms", "affects", "shapes", "guides", "alters", "drives",
    "comforts", "frightens", "puzzles",
];
const OBJECTS: &[&str] = &[
    "buyers", "planet", "readers", "students", "market", "crowd", "audience",
    "investors", "public", "travelers",
];
/// Control verbs that take "to + VERB"; in the error form "to" is dropped.
const CONTROL_VERBS: &[&str] = &[
    "need", "forgot", "want", "tried", "agreed", "refused", "hoped", "decided",
    "planned", "managed", "promised", "offered", "started", "learned",
];
/// Bare (base-form) verbs that follow a control verb.
const BARE_VERBS: &[&str] = &[
    "talk", "send", "meet", "help", "write", "call", "leave", "finish", "answer",
    "get", "read", "build", "fix", "speak", "plan", "review",
];
/// Adjectives in the "way to[o] <adj>" error.
const WAY_ADJS: &[&str] = &[
    "fast", "complicated", "slow", "expensive", "late", "loud", "bright",
    "risky", "hard", "big", "heavy", "cold", "simple", "tight",
];
/// Past/participle verbs after the contraction error "Its named ...".
const ITS_VERBS: &[&str] = &[
    "named", "located", "designed", "written", "built", "owned", "painted",
    "founded", "ranked", "considered", "shaped",
];

// affect/effect cluster — attributive NOUN modifiers the tagger wrongly calls
// VERB. HEAD pool includes affect/affects/effect/effects so the noun role is
// taught directly ("sound affects" / "sound affect").
const EA_ATTR_NOUNS: &[&str] = &[
    "sound", "employee", "customer", "policy", "software", "student",
    "house", "market", "user", "team", "network", "product",
];
const EA_HEAD_NOUNS: &[&str] = &[
    "morale", "effects", "affects", "effect", "affect", "engagement", "price",
    "quality", "design", "behavior", "feedback", "focus", "performance", "results",
];
const EA_SUBJECTS: &[&str] = &[
    "policy", "outage", "decision", "rollout", "change", "update",
    "merger", "delay", "shortage", "redesign", "ruling", "tariff",
];
const EA_VERBS_S: &[&str] = &[
    "effects", "affects", "shapes", "alters", "drives", "shifts",
    "guides", "boosts", "harms", "limits", "reshapes", "disrupts",
];
const EA_PRED_ADJS: &[&str] = &[
    "immediate", "obvious", "dramatic", "subtle", "lasting", "minimal",
    "significant", "noticeable", "temporary", "profound",
];

// weir cluster slots.
const WR_PRED_ADJS: &[&str] = &[
    "available", "clear", "stable", "aware", "excited", "possible", "complete",
    "relevant", "public", "consistent", "accurate", "confident", "certain", "visible",
];
const WR_ATTR_ADJS: &[&str] = &[
    "reproducible", "minimal", "detailed", "quick", "clear", "brief", "simple",
    "dramatic", "bridal", "everyday", "elusive", "notorious", "skilled", "thorough",
];
const WR_HEAD_NOUNS: &[&str] = &[
    "code", "notes", "schedules", "drafts", "voices", "grammar", "reports",
    "backpacks", "maps", "evidence", "patience", "ladder", "meeting", "agenda",
    "roadmap", "leaves", "banners", "posters", "napkins", "answer",
];
const WR_BARE_NOUNS: &[&str] = &[
    "homework", "teachers", "students", "buyers", "readers", "workers",
    "managers", "writers", "tourists", "neighbors", "investors", "travelers",
];
const WR_VERBS_S: &[&str] = &[
    "tempts", "rewards", "distracts", "challenges", "motivates", "frustrates",
    "inspires", "confuses", "exhausts", "delights",
];
/// Color words in the "rainbow/cream colored <noun>" hyphenation error.
const WR_COLORS: &[&str] = &["rainbow", "cream"];

// possessive "a"-determiner cluster ("novel" added for writers_novel).
const MP_PLURAL_A: &[&str] = &[
    "teachers", "writers", "farmers", "workers", "neighbors", "doctors",
    "builders", "singers", "painters", "drivers", "managers", "readers",
];
const MP_HEAD_A: &[&str] = &[
    "lounge", "meeting", "report", "schedule", "garden", "office", "novel",
    "budget", "handbook", "review", "notebook", "contract", "survey",
];

/// Each template is a list of (cell, upos, np-membership). Templates are natural
/// English of ~11-14 tokens. Attributive adjectives/determiners/nouns/pronouns
/// inside a noun phrase = true; predicative adjectives, verbs, adverbs,
/// adpositions, aux, conjunctions, particles, punctuation = false.
fn templates() -> Vec<Vec<(Cell, UPOS, bool)>> {
    use UPOS::*;
    vec![
        // ===== Family 1: noun-noun head =====================================
        vec![
            (Fixed("We"), PRON, true), (Fixed("heard"), VERB, false),
            (Fixed("the"), DET, true), (Pool(PLURAL_NOUNS), NOUN, true), (Pool(HEAD_NOUNS), NOUN, true),
            (Fixed("drifting"), VERB, false), (Fixed("across"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("quiet"), ADJ, true), (Fixed("valley"), NOUN, true),
            (Fixed("after"), ADP, false), (Fixed("dawn"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        vec![
            (Fixed("The"), DET, true), (Pool(PLURAL_NOUNS), NOUN, true), (Pool(HEAD_NOUNS), NOUN, true),
            (Fixed("near"), ADP, false), (Fixed("the"), DET, true), (Fixed("river"), NOUN, true),
            (Fixed("looked"), VERB, false), (Fixed("unusually"), ADV, false), (Fixed("calm"), ADJ, false),
            (Fixed("this"), DET, true), (Fixed("year"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // "a" determiner (fixes possessive_noun "a teachers lounge").
        vec![
            (Fixed("Every"), DET, true), (Fixed("office"), NOUN, true), (Fixed("needs"), VERB, false),
            (Fixed("a"), DET, true), (Pool(MP_PLURAL_A), NOUN, true), (Pool(MP_HEAD_A), NOUN, true),
            (Fixed("for"), ADP, false), (Fixed("the"), DET, true), (Fixed("whole"), ADJ, true),
            (Fixed("team"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // "the" determiner, plural-noun + head (protects writers_novel, neighbors_garden).
        vec![
            (Fixed("The"), DET, true), (Pool(MP_PLURAL_A), NOUN, true), (Pool(MP_HEAD_A), NOUN, true),
            (Fixed("impressed"), VERB, false), (Fixed("every"), DET, true), (Fixed("visitor"), NOUN, true),
            (Fixed("at"), ADP, false), (Fixed("the"), DET, true), (Fixed("county"), NOUN, true),
            (Fixed("fair"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // ===== Family 2: -s verb after a noun subject =======================
        vec![
            (Fixed("The"), DET, true), (Pool(ADJS), ADJ, true), (Pool(SUBJECTS), NOUN, true),
            (Pool(VERBS_S), VERB, false), (Fixed("the"), DET, true), (Pool(OBJECTS), NOUN, true),
            (Fixed("more"), ADV, false), (Fixed("than"), SCONJ, false), (Fixed("anyone"), PRON, false),
            (Fixed("had"), AUX, false), (Fixed("expected"), VERB, false), (Fixed("."), PUNCT, false),
        ],
        vec![
            (Fixed("Every"), DET, true), (Fixed("season"), NOUN, true), (Fixed("the"), DET, true),
            (Pool(SUBJECTS), NOUN, true), (Fixed("quietly"), ADV, false), (Pool(VERBS_S), VERB, false),
            (Fixed("the"), DET, true), (Pool(OBJECTS), NOUN, true), (Fixed("across"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("entire"), ADJ, true), (Fixed("region"), NOUN, true),
            (Fixed("."), PUNCT, false),
        ],
        // tight bare NOUN + -s VERB + bare NOUN (fixes weir::wildcard "homework tempts teachers").
        vec![
            (Pool(WR_BARE_NOUNS), NOUN, true), (Pool(WR_VERBS_S), VERB, false), (Pool(WR_BARE_NOUNS), NOUN, true),
            (Fixed("far"), ADV, false), (Fixed("more"), ADV, false), (Fixed("than"), SCONJ, false),
            (Fixed("most"), ADJ, true), (Fixed("parents"), NOUN, true), (Fixed("would"), AUX, false),
            (Fixed("ever"), ADV, false), (Fixed("admit"), VERB, false), (Fixed("."), PUNCT, false),
        ],
        // ===== Family 3: control verb + bare verb (missing "to") ============
        vec![
            (Fixed("She"), PRON, true), (Pool(CONTROL_VERBS), VERB, false), (Pool(BARE_VERBS), VERB, false),
            (Fixed("the"), DET, true), (Fixed("report"), NOUN, true), (Fixed("before"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("upcoming"), ADJ, true), (Fixed("deadline"), NOUN, true),
            (Fixed("."), PUNCT, false),
        ],
        vec![
            (Fixed("We"), PRON, true), (Pool(CONTROL_VERBS), VERB, false), (Pool(BARE_VERBS), VERB, false),
            (Fixed("about"), ADP, false), (Fixed("the"), DET, true), (Fixed("issue"), NOUN, true),
            (Fixed("at"), ADP, false), (Fixed("the"), DET, true), (Fixed("next"), ADJ, true),
            (Fixed("meeting"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        vec![
            (Fixed("He"), PRON, true), (Pool(CONTROL_VERBS), VERB, false), (Pool(BARE_VERBS), VERB, false),
            (Fixed("the"), DET, true), (Fixed("file"), NOUN, true), (Fixed("to"), ADP, false),
            (Fixed("his"), DET, true), (Fixed("manager"), NOUN, true), (Fixed("early"), ADV, false),
            (Fixed("that"), DET, true), (Fixed("morning"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // ===== Family 4: "way to[o] <adj>" ==================================
        vec![
            (Fixed("That"), DET, true), (Fixed("plan"), NOUN, true), (Fixed("is"), AUX, false),
            (Fixed("way"), ADV, false), (Fixed("to"), PART, false), (Pool(WAY_ADJS), ADJ, false),
            (Fixed("for"), ADP, false), (Fixed("our"), DET, true), (Fixed("current"), ADJ, true),
            (Fixed("budget"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        vec![
            (Fixed("You"), PRON, true), (Fixed("always"), ADV, false), (Fixed("drive"), VERB, false),
            (Fixed("way"), ADV, false), (Fixed("to"), PART, false), (Pool(WAY_ADJS), ADJ, false),
            (Fixed("on"), ADP, false), (Fixed("the"), DET, true), (Fixed("narrow"), ADJ, true),
            (Fixed("mountain"), NOUN, true), (Fixed("road"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // ===== Family 5: "Its" + past/participle VERB =======================
        vec![
            (Fixed("Its"), PRON, true), (Pool(ITS_VERBS), VERB, false), (Fixed("after"), ADP, false),
            (Fixed("a"), DET, true), (Fixed("famous"), ADJ, true), (Fixed("explorer"), NOUN, true),
            (Fixed("from"), ADP, false), (Fixed("the"), DET, true), (Fixed("far"), ADJ, true),
            (Fixed("north"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // ===== Family 6: affect/effect ======================================
        // CORRECTIVE: attributive NOUN + head NOUN object after an -s verb.
        vec![
            (Fixed("The"), DET, true), (Pool(EA_SUBJECTS), NOUN, true), (Pool(EA_VERBS_S), VERB, false),
            (Fixed("our"), DET, true), (Pool(EA_ATTR_NOUNS), NOUN, true), (Pool(EA_HEAD_NOUNS), NOUN, true),
            (Fixed("more"), ADV, false), (Fixed("than"), SCONJ, false), (Fixed("anyone"), PRON, false),
            (Fixed("had"), AUX, false), (Fixed("expected"), VERB, false), (Fixed("."), PUNCT, false),
        ],
        // CORRECTIVE: attributive NOUN + head NOUN subject + passive AUX
        // ("Sound affects were added ...") — affect/affects are in EA_HEAD_NOUNS.
        vec![
            (Pool(EA_ATTR_NOUNS), NOUN, true), (Pool(EA_HEAD_NOUNS), NOUN, true), (Fixed("were"), AUX, false),
            (Fixed("added"), VERB, false), (Fixed("during"), ADP, false), (Fixed("the"), DET, true),
            (Fixed("final"), ADJ, true), (Fixed("editing"), NOUN, true), (Fixed("pass"), NOUN, true),
            (Fixed("last"), ADJ, true), (Fixed("week"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // CORRECTIVE: possessive + attributive NOUN + head NOUN + verb
        // ("its sound affect remains ...").
        vec![
            (Fixed("Its"), PRON, true), (Pool(EA_ATTR_NOUNS), NOUN, true), (Pool(EA_HEAD_NOUNS), NOUN, true),
            (Fixed("remains"), VERB, false), (Fixed("stuck"), ADJ, false), (Fixed("in"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("same"), ADJ, true), (Fixed("broken"), ADJ, true),
            (Fixed("state"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // PROTECTIVE: result-noun + predicative ADJ.
        vec![
            (Fixed("The"), DET, true), (Fixed("overall"), ADJ, true), (Fixed("outcome"), NOUN, true),
            (Fixed("was"), AUX, false), (Pool(EA_PRED_ADJS), ADJ, false), (Fixed("and"), CCONJ, false),
            (Fixed("clearly"), ADV, false), (Fixed("visible"), ADJ, false), (Fixed("to"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("whole"), ADJ, true), (Fixed("team"), NOUN, true),
            (Fixed("."), PUNCT, false),
        ],
        // PROTECTIVE: AUX + VERB "influence".
        vec![
            (Fixed("The"), DET, true), (Pool(EA_SUBJECTS), NOUN, true), (Fixed("will"), AUX, false),
            (Fixed("influence"), VERB, false), (Fixed("our"), DET, true), (Fixed("quarterly"), ADJ, true),
            (Fixed("revenue"), NOUN, true), (Fixed("quite"), ADV, false), (Fixed("significantly"), ADV, false),
            (Fixed("this"), DET, true), (Fixed("year"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // ===== Family 7: weir error-form (literal trigger + POS slot) =======
        // all_ready: literal "all ready" + predicative ADJ.
        vec![
            (Fixed("The"), DET, true), (Fixed("firmware"), NOUN, true), (Fixed("is"), AUX, false),
            (Fixed("all"), ADV, false), (Fixed("ready"), ADJ, false), (Pool(WR_PRED_ADJS), ADJ, false),
            (Fixed("on"), ADP, false), (Fixed("every"), DET, true), (Fixed("supported"), ADJ, true),
            (Fixed("device"), NOUN, true), (Fixed("today"), NOUN, false), (Fixed("."), PUNCT, false),
        ],
        // each_others: literal "each others" + NOUN head.
        vec![
            (Fixed("They"), PRON, true), (Fixed("reviewed"), VERB, false), (Fixed("each"), DET, true),
            (Fixed("others"), PRON, true), (Pool(WR_HEAD_NOUNS), NOUN, true), (Fixed("before"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("final"), ADJ, true), (Fixed("release"), NOUN, true),
            (Fixed("went"), VERB, false), (Fixed("live"), ADV, false), (Fixed("."), PUNCT, false),
        ],
        // there_to_their: literal "there" + NOUN head (possessive misuse).
        vec![
            (Fixed("They"), PRON, true), (Fixed("left"), VERB, false), (Fixed("there"), ADV, false),
            (Pool(WR_HEAD_NOUNS), NOUN, true), (Fixed("near"), ADP, false), (Fixed("the"), DET, true),
            (Fixed("front"), ADJ, true), (Fixed("entrance"), NOUN, true), (Fixed("this"), DET, true),
            (Fixed("morning"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // rainbow_colored: literal "[rainbow/cream] colored" + NOUN head.
        vec![
            (Fixed("We"), PRON, true), (Fixed("bought"), VERB, false), (Pool(WR_COLORS), NOUN, true),
            (Fixed("colored"), ADJ, true), (Pool(WR_HEAD_NOUNS), NOUN, true), (Fixed("for"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("spring"), NOUN, true), (Fixed("festival"), NOUN, true),
            (Fixed("parade"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // makeup: literal "make up" preceded by determiner + attributive ADJ.
        vec![
            (Fixed("Her"), DET, true), (Pool(WR_ATTR_ADJS), ADJ, true), (Fixed("make"), VERB, false),
            (Fixed("up"), ADP, false), (Fixed("looked"), VERB, false), (Fixed("perfect"), ADJ, false),
            (Fixed("for"), ADP, false), (Fixed("the"), DET, true), (Fixed("evening"), NOUN, true),
            (Fixed("gala"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // tomorrow_possessive: literal "Tomorrows" + NOUN head (protect).
        vec![
            (Fixed("Tomorrows"), NOUN, true), (Pool(WR_HEAD_NOUNS), NOUN, true), (Fixed("starts"), VERB, false),
            (Fixed("promptly"), ADV, false), (Fixed("at"), ADP, false), (Fixed("nine"), NUM, true),
            (Fixed("in"), ADP, false), (Fixed("the"), DET, true), (Fixed("main"), ADJ, true),
            (Fixed("hall"), NOUN, true), (Fixed("."), PUNCT, false),
        ],
        // attributive ADJ inside an NP (protects missing_determiner, thieve_noun).
        vec![
            (Fixed("Please"), INTJ, false), (Fixed("send"), VERB, false), (Fixed("the"), DET, true),
            (Pool(WR_ATTR_ADJS), ADJ, true), (Fixed("report"), NOUN, true), (Fixed("to"), ADP, false),
            (Fixed("the"), DET, true), (Fixed("whole"), ADJ, true), (Fixed("team"), NOUN, true),
            (Fixed("before"), ADP, false), (Fixed("noon"), NOUN, true), (Fixed("today"), NOUN, false),
            (Fixed("."), PUNCT, false),
        ],
        // ===== Family 8: imperative sentence-initial VERB (were_where) ======
        // "Check the logs ..." teaches sentence-initial "Check" as VERB so the
        // verb_were_clause rule ("Check were the ...") can fire.
        vec![
            (Fixed("Check"), VERB, false), (Fixed("the"), DET, true), (Pool(HEAD_NOUNS), NOUN, true),
            (Fixed("before"), ADP, false), (Fixed("the"), DET, true), (Fixed("next"), ADJ, true),
            (Fixed("release"), NOUN, true), (Fixed("goes"), VERB, false), (Fixed("out"), ADP, false),
            (Fixed("tonight"), NOUN, false), (Fixed("."), PUNCT, false),
        ],
    ]
}

/// Sentences generated per template. With ~26 templates this yields ~1040
/// injected sentences — comprehensive coverage at modest volume.
const PER_TEMPLATE: usize = 40;

/// Build the synthetic injection dataset: `(sentences, tags, np-membership)`,
/// shaped to append directly to `extract_records_from_files` output.
pub fn build_injection() -> (Vec<Vec<String>>, Vec<Vec<Option<UPOS>>>, Vec<Vec<bool>>) {
    let (mut sents, mut tags, mut nps) = (Vec::new(), Vec::new(), Vec::new());
    for tmpl in templates() {
        for j in 0..PER_TEMPLATE {
            let mut s = Vec::with_capacity(tmpl.len());
            let mut t = Vec::with_capacity(tmpl.len());
            let mut n = Vec::with_capacity(tmpl.len());
            for (c, (cell, upos, np)) in tmpl.iter().enumerate() {
                let word = match cell {
                    Fixed(w) => (*w).to_string(),
                    // `(j + c*7) % len`: consecutive j sweep every pool index, so
                    // EVERY pool word (incl. each lint-critical one) is injected;
                    // the per-cell `c*7` offset decorrelates slots within a sentence.
                    Pool(pool) => pool[(j + c * 7) % pool.len()].to_string(),
                };
                s.push(word);
                t.push(Some(*upos));
                n.push(*np);
            }
            sents.push(s);
            tags.push(t);
            nps.push(n);
        }
    }
    (sents, tags, nps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_varied_well_formed_examples() {
        let (sents, tags, nps) = build_injection();
        assert_eq!(sents.len(), templates().len() * PER_TEMPLATE);
        assert_eq!(sents.len(), tags.len());
        assert_eq!(sents.len(), nps.len());
        // every sentence is well-formed (aligned lengths) and a good length.
        for (s, t) in sents.iter().zip(tags.iter()) {
            assert_eq!(s.len(), t.len());
            assert!(s.len() >= 10, "sentences should be a realistic length");
        }
        // the noun-noun frame teaches a head noun after a plural-noun modifier.
        let song = sents.iter().zip(tags.iter()).find(|(s, _)| {
            s.windows(2).any(|w| w[0] == "birds" && w[1] == "song")
        });
        if let Some((s, t)) = song {
            let i = s.iter().position(|w| w == "song").unwrap();
            assert_eq!(t[i], Some(UPOS::NOUN));
            assert_eq!(t[i - 1], Some(UPOS::NOUN), "the plural modifier is a NOUN too");
        }
        // a control-verb template teaches a bare VERB after the control verb.
        let ctrl = sents.iter().zip(tags.iter()).find(|(s, _)| {
            s.first().map(|w| w == "She").unwrap_or(false)
        });
        if let Some((_s, t)) = ctrl {
            assert_eq!(t[1], Some(UPOS::VERB), "control verb is a VERB");
            assert_eq!(t[2], Some(UPOS::VERB), "the bare verb after it is a VERB");
        }
        // weir error-form: literal "all ready" is followed by a predicative ADJ.
        let allready = sents.iter().zip(tags.iter()).find(|(s, _)| {
            s.windows(2).any(|w| w[0] == "all" && w[1] == "ready")
        });
        if let Some((s, t)) = allready {
            let i = s.iter().position(|w| w == "ready").unwrap();
            assert_eq!(t[i + 1], Some(UPOS::ADJ), "the word after 'all ready' is an ADJ");
        }
    }
}

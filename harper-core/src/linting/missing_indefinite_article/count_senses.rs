use crate::CharStringExt;

/// Supplies missing mass readings for dictionary nouns with default countability.
/// This is a conservative fallback, not a ban on articles: `selected` and the
/// contextual count selectors in `phrase` take precedence. Modifying a mass noun
/// does not establish a count sense ("tiny plankton", "absolute certainty").
pub(super) fn allows_mass_reading(head: &[char]) -> bool {
    head.eq_any_ignore_ascii_case_str(&[
        // Materials, substances, and aggregate organisms.
        "plankton",
        "ozone",
        "flax",
        "feedstock",
        "upholstery",
        "litter",
        "regalia",
        "peroxide",
        "adobe",
        "yarn",
        "sustenance",
        "currency",
        "pavement",
        "purée",
        // Qualities and states, rather than discrete objects or events.
        "amusement",
        "familiarity",
        "willpower",
        "certainty",
        "rigor",
        "rigour",
        "conductivity",
        "porosity",
        "homogeneity",
        "volatility",
        "irritability",
        "gingivitis",
        "hypertension",
        "solitude",
        "unease",
        "despair",
        "camaraderie",
        "dexterity",
        "virtuosity",
        "practicality",
        "inertia",
        "innocence",
        "ignorance",
        "authenticity",
        "impulsiveness",
        "longevity",
        "sharpness",
        "reactivity",
        "spontaneity",
        "conformity",
        "regularity",
        "stickiness",
        "efficacy",
        "silliness",
        "stubbornness",
        "heartiness",
        "exasperation",
        "forgetfulness",
        // Processes and disciplines that commonly occur without an article.
        "colonization",
        "sterilization",
        "pasteurization",
        "retrieval",
        "recall",
        "duplication",
        "prioritization",
        "reconciliation",
        "harmonization",
        "homogenization",
        "revitalization",
        "standardization",
        "replenishment",
        "comprehension",
        "physiotherapy",
        "epidemiology",
        "botany",
        "physiology",
        "pathology",
        "anatomy",
        "demolition",
        "renewal",
        "deprivation",
        "scrutiny",
        "wear",
        "runoff",
        "advocacy",
        "suspense",
        "methodology",
        "coordination",
        "meditation",
        "mentorship",
        "encouragement",
        "restraint",
        "dismay",
        "chatter",
        "downtime",
        "proficiency",
        "legislation",
        "rationalism",
        "interdependence",
        "overhead",
        "sanitizer",
        "encrustation",
        "timbre",
        "filigree",
        "conductance",
        "coercion",
        "coexistence",
        "expediency",
        "toxicity",
        "severity",
        "praise",
        "uncertainty",
        "conservation",
        "depletion",
        "imagination",
        "reputation",
        "accommodation",
        "teamwork",
        "help",
        "locomotion",
        "appeal",
        "excitement",
        "curation",
        "snowfall",
        "spaciousness",
        "connectivity",
        "retention",
        "sonar",
        "fertilizer",
        "fertiliser",
        "jazz",
        "calculus",
        "comparison",
        "quantification",
        "disappearance",
        "lateness",
        "preference",
        "irritation",
        "sanitiser",
        "self-efficacy",
        "volunteerism",
        "bluntness",
        "excellence",
        "merit",
        "inundation",
        "adversity",
        "sophistication",
        "certification",
        "entanglement",
        "impregnation",
        "dissemination",
        "hardness",
        "thickness",
        "democracy",
        "palmetto",
        "ramen",
        "marble",
        "ribbon",
        "hide",
        "burial",
        "specialization",
        "self-sacrifice",
        "self-comparison",
        "self-proclamation",
        "similarity",
        "disappointment",
        "misfortune",
        "turnover",
        "declination",
        "ruin",
        "constancy",
        "enjoyment",
        "survival",
        "humanity",
        "passage",
        "necessity",
        "determination",
        "impairment",
        "reimbursement",
        "incubation",
        "bias",
        "indentation",
        "overflow",
        "airspace",
        "burnout",
        "coloration",
        "ambiguity",
        "luck",
        "traversal",
        "disposal",
        "allusion",
        "immersion",
        "speculation",
        "inoculation",
        "deceleration",
        "abrasion",
        "emphasis",
        "discourse",
        "intellect",
        "dialect",
        "exhaustion",
        "turmoil",
        "resignation",
        "promise",
        "workload",
        "diagnosis",
        "cleanup",
        "clearance",
        "submission",
        "conversion",
        "adjustment",
        "transition",
        "glaze",
    ])
}

/// Selects concrete senses of nouns that also name activities or materials.
/// Writing a review produces a countable text; requiring review names a process.
/// Keep selectors local to each sense instead of treating every modified mass
/// noun, or every object of a transitive verb, as countable.
pub(super) fn selected(head: &[char], governor: &[char], modifier: &[char]) -> bool {
    let text = head.eq_any_ignore_ascii_case_str(&[
        "assessment",
        "review",
        "composition",
        "study",
        "entry",
        "application",
    ]) && governor.eq_any_ignore_ascii_case_str(&[
        "write",
        "writes",
        "wrote",
        "written",
        "draft",
        "drafts",
        "drafted",
        "publish",
        "publishes",
        "published",
        "submit",
        "submits",
        "submitted",
    ]);
    let instance = (head.eq_any_ignore_ascii_case_str(&["promise", "diagnosis", "adjustment"])
        && governor.eq_any_ignore_ascii_case_str(&["make", "makes", "made"]))
        || (head.eq_any_ignore_ascii_case_str(&["diagnosis"])
            && governor.eq_any_ignore_ascii_case_str(&[
                "receive", "receives", "received", "get", "gets", "got",
            ]));
    let comparison = head.eq_any_ignore_ascii_case_str(&["comparison"])
        && governor
            .eq_any_ignore_ascii_case_str(&["make", "makes", "made", "draw", "draws", "drew"]);
    let message = head.eq_any_ignore_ascii_case_str(&["note", "response"])
        && governor.eq_any_ignore_ascii_case_str(&[
            "write", "writes", "wrote", "leave", "leaves", "left", "send", "sends", "sent",
        ]);
    let reference = head.eq_any_ignore_ascii_case_str(&["reference"])
        && governor
            .eq_any_ignore_ascii_case_str(&["add", "adds", "added", "cite", "cites", "cited"]);
    let assignment = head.eq_any_ignore_ascii_case_str(&["assignment"])
        && governor.eq_any_ignore_ascii_case_str(&[
            "complete",
            "completes",
            "completed",
            "finish",
            "finishes",
            "finished",
            "submit",
            "submits",
            "submitted",
        ])
        && !modifier.eq_any_ignore_ascii_case_str(&["manual", "automatic", "automated"]);
    let object = head.eq_any_ignore_ascii_case_str(&["pickup", "scope", "curiosity", "football"])
        && governor.eq_any_ignore_ascii_case_str(&[
            "buy",
            "buys",
            "bought",
            "purchase",
            "purchases",
            "purchased",
        ]);
    let error = head.eq_any_ignore_ascii_case_str(&["error"])
        && modifier.eq_any_ignore_ascii_case_str(&[
            "syntax",
            "spelling",
            "grammatical",
            "typographical",
        ])
        && governor.eq_any_ignore_ascii_case_str(&[
            "find",
            "finds",
            "found",
            "fix",
            "fixes",
            "fixed",
            "correct",
            "corrects",
            "corrected",
        ]);
    let survey = head.eq_any_ignore_ascii_case_str(&["survey"])
        && governor.eq_any_ignore_ascii_case_str(&[
            "conduct",
            "conducts",
            "conducted",
            "commission",
            "commissions",
            "commissioned",
        ]);
    let attack = head.eq_any_ignore_ascii_case_str(&["attack"])
        && governor.eq_any_ignore_ascii_case_str(&["launch", "launches", "launched"]);
    let burden = head.eq_any_ignore_ascii_case_str(&["burden"])
        && governor.eq_any_ignore_ascii_case_str(&[
            "carry",
            "carries",
            "carried",
            "bear",
            "bears",
            "bore",
            "shoulder",
            "shoulders",
            "shouldered",
        ]);
    text || instance
        || comparison
        || message
        || reference
        || assignment
        || object
        || error
        || survey
        || attack
        || burden
}

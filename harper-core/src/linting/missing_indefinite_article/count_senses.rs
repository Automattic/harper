use crate::CharStringExt;

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
    text || message || reference || assignment || object || error || survey || attack || burden
}

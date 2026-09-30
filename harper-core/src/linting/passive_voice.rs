// SPDX-License-Identifier: Apache-2.0
// Original project: JaredTweed/PassiveVoiceDetector.

use harper_brill::UPOS;

use super::{Lint, LintKind, Linter};
use crate::{Document, Span, Token, TokenStringExt};

/// Detects likely passive-voice constructions.
///
/// The rule deliberately favors precision over recall. Passive voice is a style
/// choice rather than a grammatical error, so a false positive is more annoying
/// than an occasional missed, genuinely ambiguous construction.
#[derive(Debug, Clone, Copy, Default)]
pub struct PassiveVoice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PassiveSource {
    Be,
    Get,
    Become,
    Reduced,
}

impl Linter for PassiveVoice {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let source = document.get_source();
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let word_indices: Vec<usize> = sentence.iter_word_indices().collect();
            let is_question = sentence_is_question(sentence, source);
            let mut covered_through = None;

            for (word_pos, &token_idx) in word_indices.iter().enumerate() {
                if covered_through.is_some_and(|end| token_idx <= end) {
                    continue;
                }

                let candidate = &sentence[token_idx];
                // Most words cannot be participles. Avoid scanning their context.
                if is_descriptive_compound(sentence, token_idx, source)
                    || !is_participle_candidate(candidate, source)
                {
                    continue;
                }
                let by_agent = has_agentive_by(sentence, &word_indices, word_pos, source);

                let Some((passive_source, start_idx)) = classify_passive(
                    sentence,
                    &word_indices,
                    word_pos,
                    source,
                    by_agent,
                    is_question,
                ) else {
                    continue;
                };

                if has_state_complement(sentence, &word_indices, word_pos, source, by_agent)
                    || is_attributive_after_get(
                        sentence,
                        &word_indices,
                        word_pos,
                        source,
                        passive_source,
                    )
                    || (should_suppress_adjectival(
                        candidate,
                        source,
                        by_agent,
                        passive_source,
                        has_event_evidence(sentence, &word_indices, word_pos, source),
                    ) && !has_clear_done_passive(sentence, &word_indices, word_pos, source))
                {
                    continue;
                }

                let end_idx =
                    coordinated_tail(sentence, &word_indices, word_pos, source, token_idx);
                covered_through = Some(end_idx);

                let start_idx = auxiliary_chain_start(sentence, &word_indices, start_idx, source);
                let start_idx = if is_question {
                    inverted_question_chain_start(sentence, &word_indices, start_idx, source)
                        .unwrap_or(start_idx)
                } else {
                    start_idx
                };
                lints.push(Lint {
                    span: Span::new(sentence[start_idx].span.start, sentence[end_idx].span.end),
                    lint_kind: LintKind::Style,
                    suggestions: vec![],
                    message: if by_agent {
                        "Possible passive voice. The actor is named; consider active voice if it would improve the emphasis or clarity."
                    } else {
                        "Possible passive voice. Is the actor relevant but unclear? If so, name the actor or use active voice; otherwise this wording may be appropriate."
                    }
                    .to_owned(),
                    priority: 180,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects likely passive-voice constructions while avoiding common participial-adjective false positives."
    }
}

/// Lexicalized compounds describe attributes, even when a component is a verb
/// ("left-handed"). Do not turn either half into an independent passive.
fn is_descriptive_compound(sentence: &[Token], idx: usize, source: &[char]) -> bool {
    let descriptive_suffix = |token: &Token| {
        matches!(
            normalized_word(token, source).as_str(),
            "handed"
                | "haired"
                | "hearted"
                | "minded"
                | "legged"
                | "eyed"
                | "skinned"
                | "faced"
                | "headed"
                | "tempered"
        )
    };
    (idx > 0 && sentence[idx - 1].kind.is_hyphen() && descriptive_suffix(&sentence[idx]))
        || (sentence.get(idx + 1).is_some_and(|t| t.kind.is_hyphen())
            && sentence.get(idx + 2).is_some_and(descriptive_suffix))
}

fn normalized_word(token: &Token, source: &[char]) -> String {
    token
        .get_ch(source)
        .iter()
        .map(|c| match c {
            '’' => '\'',
            _ => c.to_ascii_lowercase(),
        })
        .collect()
}

/// A by-phrase alone does not make a finite active verb passive.
fn is_contextual_noun(token: &Token) -> bool {
    token.kind.is_upos(UPOS::NOUN)
        || token.kind.is_upos(UPOS::PROPN)
        || token
            .kind
            .as_word()
            .and_then(|m| m.as_ref())
            .is_some_and(|m| m.pos_tag.is_none() && (m.is_noun() || m.is_proper_noun()))
}

fn can_be_reduced_passive(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
) -> bool {
    let candidate_idx = word_indices[word_pos];
    // "got" ordinarily heads the get-passive rather than serving as its
    // lexical participle ("the employees got fired by the manager").
    if matches!(
        normalized_word(&sentence[candidate_idx], source).as_str(),
        "got" | "come" | "become"
    ) {
        return false;
    }
    // Reject active perfects before considering reduced clauses.
    for &idx in word_indices[..word_pos].iter().rev().take(6) {
        if has_hard_boundary(&sentence[idx + 1..candidate_idx]) {
            break;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        if matches!(
            lower.as_str(),
            "have" | "has" | "had" | "having" | "i've" | "you've" | "we've" | "they've"
        ) || lower.ends_with("'d")
        {
            return false;
        }
        if !is_gap_modifier(token, &lower) {
            break;
        }
    }
    // A second participle can share the first passive's subject even when
    // each verb has its own agent: "was signed by Alice and filed by Bob".
    if word_pos >= 3
        && matches!(
            normalized_word(&sentence[word_indices[word_pos - 1]], source).as_str(),
            "and" | "or" | "but" | "yet"
        )
    {
        for by_pos in (1..word_pos - 1).rev().take(6) {
            let by_idx = word_indices[by_pos];
            if sentence[by_idx + 1..candidate_idx].iter().any(|token| {
                (token.kind.is_chunk_terminator() && !token.kind.is_comma())
                    || token.kind.is_unlintable()
                    || (token.kind.is_punctuation()
                        && !token.kind.is_hyphen()
                        && !token.kind.is_comma())
            }) {
                break;
            }
            if normalized_word(&sentence[by_idx], source) == "by"
                && is_participle_candidate(&sentence[word_indices[by_pos - 1]], source)
            {
                return true;
            }
        }
    }
    let Some(mut previous_pos) = word_pos.checked_sub(1) else {
        return true;
    };
    for _ in 0..5 {
        let idx = word_indices[previous_pos];
        if has_hard_boundary(&sentence[idx + 1..candidate_idx]) {
            return true;
        }
        if !is_gap_modifier(&sentence[idx], &normalized_word(&sentence[idx], source)) {
            break;
        }
        let Some(pos) = previous_pos.checked_sub(1) else {
            return true;
        };
        previous_pos = pos;
    }
    let previous_idx = word_indices[previous_pos];
    if has_hard_boundary(&sentence[previous_idx + 1..candidate_idx]) {
        return true;
    }
    let previous = &sentence[previous_idx];
    let lower = normalized_word(previous, source);
    // 's can mean either is or has; only recover it with a local explicit agent.
    if lower.ends_with("'s") {
        return true;
    }
    if has_measure_by_after(sentence, candidate_idx, source) {
        return false;
    }
    is_contextual_noun(previous)
}

fn has_measure_by_after(sentence: &[Token], candidate_idx: usize, source: &[char]) -> bool {
    let tail = &sentence[candidate_idx + 1..];
    let Some(by_pos) = tail
        .iter()
        .take(32)
        .take_while(|t| !t.kind.is_chunk_terminator())
        .position(|t| normalized_word(t, source) == "by")
    else {
        return false;
    };
    tail[by_pos + 1..]
        .iter()
        .filter(|t| !t.kind.is_whitespace())
        .take(5)
        .take_while(|t| !t.kind.is_chunk_terminator())
        .any(|t| {
            matches!(
                normalized_word(t, source).as_str(),
                "%" | "percent"
                    | "percentage"
                    | "points"
                    | "margin"
                    | "vote"
                    | "votes"
                    | "degrees"
                    | "inches"
                    | "miles"
                    | "centimeters"
                    | "cm"
                    | "mm"
                    | "m"
                    | "ft"
                    | "in"
            )
        })
}

fn classify_passive(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
    by_agent: bool,
    is_question: bool,
) -> Option<(PassiveSource, usize)> {
    if let Some((kind, aux_idx)) = preceding_passive_aux(sentence, word_indices, word_pos, source) {
        return Some((kind, aux_idx));
    }

    // In questions the auxiliary can precede the subject: "Was the report written?"
    // The ordinary backwards scan intentionally stops at nominals, so handle this
    // inversion separately and only when the sentence is actually interrogative.
    if let Some((kind, aux_idx)) =
        preceding_inverted_passive_aux(sentence, word_indices, word_pos, source, is_question)
    {
        return Some((kind, aux_idx));
    }

    // Perfect auxiliaries are not lexical reduced participles, even when the
    // dictionary also marks their surface form as a participle ("had").
    if sentence[word_indices[word_pos]].kind.is_upos(UPOS::AUX) {
        return None;
    }

    if by_agent && can_be_reduced_passive(sentence, word_indices, word_pos, source) {
        return Some((PassiveSource::Reduced, word_indices[word_pos]));
    }

    if is_existential_reduced_passive(sentence, word_indices, word_pos, source)
        || is_temporal_reduced_passive(sentence, word_indices, word_pos, source)
        || is_strong_irregular_reduced_passive(
            sentence,
            word_indices,
            word_pos,
            source,
            is_question,
        )
    {
        return Some((PassiveSource::Reduced, word_indices[word_pos]));
    }

    None
}

fn preceding_passive_aux(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
) -> Option<(PassiveSource, usize)> {
    let candidate_idx = *word_indices.get(word_pos)?;
    let mut pos = word_pos;
    let mut skipped = 0usize;

    while pos > 0 && skipped <= 5 {
        pos -= 1;
        let idx = word_indices[pos];

        if has_hard_boundary(&sentence[idx + 1..candidate_idx]) {
            return None;
        }

        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        if (is_contextual_noun(token)
            || (idx > 0 && sentence[idx - 1].kind.is_hyphen())
            || sentence.get(idx + 1).is_some_and(|t| t.kind.is_hyphen()))
            && (is_be_form(&lower) || is_get_form(&lower) || is_become_form(&lower))
        {
            break;
        }

        if is_be_form(&lower) || is_unambiguous_be_contraction(&lower) {
            return Some((PassiveSource::Be, idx));
        }

        if is_get_form(&lower) {
            return Some((PassiveSource::Get, idx));
        }

        if is_become_form(&lower) {
            return Some((PassiveSource::Become, idx));
        }

        if is_gap_modifier(token, &lower)
            || (idx + 2 == candidate_idx
                && sentence[idx + 1].kind.is_hyphen()
                && !token.kind.is_upos(UPOS::ADJ))
        {
            skipped += 1;
            continue;
        }

        // Coordinate adverbial modifiers without mistaking coordinated verbs for
        // an auxiliary chain: "was recently and deliberately removed".
        if skipped > 0 && matches!(lower.as_str(), "and" | "or") && pos > 0 {
            let before = &sentence[word_indices[pos - 1]];
            let before_lower = normalized_word(before, source);
            if is_gap_modifier(before, &before_lower) {
                skipped += 1;
                continue;
            }
        }

        break;
    }

    None
}

fn preceding_inverted_passive_aux(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
    is_question: bool,
) -> Option<(PassiveSource, usize)> {
    if !is_question || word_pos == 0 {
        return None;
    }

    let candidate_idx = word_indices[word_pos];
    let lower_bound = word_pos.saturating_sub(16);
    let mut saw_subject = false;
    let mut in_relative = false;

    for pos in (lower_bound..word_pos).rev() {
        let idx = word_indices[pos];
        if has_hard_boundary(&sentence[idx + 1..candidate_idx]) {
            return None;
        }

        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        let auxiliary_allowed =
            !is_contextual_noun(token) && !(idx > 0 && sentence[idx - 1].kind.is_hyphen());

        if matches!(
            lower.as_str(),
            "when"
                | "while"
                | "because"
                | "although"
                | "unless"
                | "since"
                | "as"
                | "where"
                | "after"
                | "before"
                | "if"
                | "whether"
                | "to"
        ) || (token.kind.is_upos(UPOS::DET) && !saw_subject)
        {
            return None;
        }
        if auxiliary_allowed && (is_be_form(&lower) || is_unambiguous_be_contraction(&lower)) {
            return (saw_subject
                && !in_relative
                && is_question_aux_position(sentence, word_indices, pos, source))
            .then_some((PassiveSource::Be, idx));
        }
        if auxiliary_allowed && is_get_form(&lower) {
            return (saw_subject
                && !in_relative
                && is_question_aux_position(sentence, word_indices, pos, source))
            .then_some((PassiveSource::Get, idx));
        }
        if auxiliary_allowed && is_become_form(&lower) {
            return (saw_subject
                && !in_relative
                && is_question_aux_position(sentence, word_indices, pos, source))
            .then_some((PassiveSource::Become, idx));
        }

        // Subject material (determiners, adjectives, nouns, pronouns, numerals,
        // possessives) is expected between an inverted auxiliary and participle.
        // A different predicate must belong to a relative clause within the
        // subject before we can accept the earlier inverted auxiliary.
        if matches!(lower.as_str(), "that" | "which" | "who" | "whom") {
            in_relative = false;
        } else if token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX) {
            in_relative = true;
        }
        saw_subject |= token.kind.is_upos(UPOS::NOUN)
            || token.kind.is_upos(UPOS::PROPN)
            || token.kind.is_upos(UPOS::PRON)
            || token
                .kind
                .as_word()
                .and_then(|m| m.as_ref())
                .is_some_and(|m| m.pos_tag.is_none() && m.is_nominal());
    }

    None
}

fn is_question_aux_position(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
) -> bool {
    if pos == 0 || has_hard_boundary(&sentence[indices[pos - 1] + 1..indices[pos]]) {
        return true;
    }
    indices[..pos].iter().take(6).any(|&idx| {
        matches!(
            normalized_word(&sentence[idx], source).as_str(),
            "why" | "when" | "where" | "how" | "what" | "which" | "who" | "whom" | "whose"
        )
    })
}

fn sentence_is_question(sentence: &[Token], source: &[char]) -> bool {
    for token in sentence.iter().rev() {
        if token.kind.is_whitespace() {
            continue;
        }

        let content = token.get_str(source);
        let text = content.trim();
        if matches!(text, "\"" | "'" | "”" | "’") {
            continue;
        }

        return text == "?";
    }

    false
}

/// Once a passive has been established, include the fronted auxiliary in an
/// inverted question. The subject intervenes, so the ordinary chain scan must
/// stop before it. A separate finite predicate or clause boundary blocks the
/// extension.
fn inverted_question_chain_start(
    sentence: &[Token],
    indices: &[usize],
    start_idx: usize,
    source: &[char],
) -> Option<usize> {
    // Finite auxiliaries already start their own clause. Extending one to an
    // earlier auxiliary can accidentally swallow a containing question, as in
    // "Is that the reason the cups are put away?".
    let start_word = normalized_word(&sentence[start_idx], source);
    if !matches!(
        start_word.as_str(),
        "to" | "be"
            | "been"
            | "being"
            | "have"
            | "get"
            | "got"
            | "gotten"
            | "getting"
            | "become"
            | "becoming"
    ) {
        return None;
    }
    let start_pos = indices.partition_point(|&idx| idx < start_idx);
    let mut saw_subject = false;
    let mut in_relative = false;

    for pos in (0..start_pos).rev().take(16) {
        let idx = indices[pos];
        if has_hard_boundary(&sentence[idx + 1..start_idx]) {
            return None;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        if matches!(
            lower.as_str(),
            "when"
                | "while"
                | "because"
                | "although"
                | "unless"
                | "since"
                | "after"
                | "before"
                | "if"
                | "whether"
        ) {
            return None;
        }
        let auxiliary = is_be_form(&lower)
            || is_get_form(&lower)
            || is_become_form(&lower)
            || matches!(
                lower.as_str(),
                "have"
                    | "has"
                    | "had"
                    | "haven't"
                    | "hasn't"
                    | "hadn't"
                    | "can"
                    | "could"
                    | "may"
                    | "might"
                    | "must"
                    | "shall"
                    | "should"
                    | "will"
                    | "would"
                    | "can't"
                    | "cannot"
                    | "couldn't"
                    | "shouldn't"
                    | "won't"
                    | "wouldn't"
                    | "mustn't"
                    | "do"
                    | "does"
                    | "did"
            );
        let modal_or_do = matches!(
            lower.as_str(),
            "can"
                | "could"
                | "may"
                | "might"
                | "must"
                | "shall"
                | "should"
                | "will"
                | "would"
                | "can't"
                | "cannot"
                | "couldn't"
                | "shouldn't"
                | "won't"
                | "wouldn't"
                | "mustn't"
                | "do"
                | "does"
                | "did"
        );
        if auxiliary
            && (!matches!(start_word.as_str(), "have" | "get") || modal_or_do)
            && saw_subject
            && !in_relative
            && is_question_aux_position(sentence, indices, pos, source)
        {
            return Some(idx);
        }
        if matches!(lower.as_str(), "that" | "which" | "who" | "whom") {
            in_relative = false;
        } else if (token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX)) && !auxiliary {
            in_relative = true;
        }
        saw_subject |= token.kind.is_upos(UPOS::NOUN)
            || token.kind.is_upos(UPOS::PROPN)
            || token.kind.is_upos(UPOS::PRON)
            || token
                .kind
                .as_word()
                .and_then(|m| m.as_ref())
                .is_some_and(|m| m.pos_tag.is_none() && m.is_nominal());
    }
    None
}

/// Include modal/perfect/progressive auxiliaries, but never absorb a subject.
fn auxiliary_chain_start(
    sentence: &[Token],
    word_indices: &[usize],
    start_idx: usize,
    source: &[char],
) -> usize {
    let start_pos = word_indices.partition_point(|&idx| idx < start_idx);
    let mut start = start_idx;
    for &idx in word_indices[..start_pos].iter().rev().take(8) {
        if has_hard_boundary(&sentence[idx + 1..start_idx]) {
            break;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        let next_word_pos = word_indices.partition_point(|&word_idx| word_idx <= idx);
        let going_to = lower == "going"
            && word_indices
                .get(next_word_pos)
                .is_some_and(|&next_idx| normalized_word(&sentence[next_idx], source) == "to");
        if is_contextual_noun(token) || (idx > 0 && sentence[idx - 1].kind.is_hyphen()) {
            break;
        }
        if is_be_form(&lower)
            || going_to
            || is_unambiguous_be_contraction(&lower)
            || lower.ends_with("'s")
            || lower.ends_with("'d")
            || matches!(
                lower.as_str(),
                "have"
                    | "has"
                    | "had"
                    | "having"
                    | "can"
                    | "could"
                    | "may"
                    | "might"
                    | "must"
                    | "shall"
                    | "should"
                    | "will"
                    | "would"
                    | "to"
                    | "can't"
                    | "cannot"
                    | "couldn't"
                    | "shouldn't"
                    | "won't"
                    | "wouldn't"
                    | "mustn't"
                    | "haven't"
                    | "hasn't"
                    | "hadn't"
                    | "i've"
                    | "you've"
                    | "we've"
                    | "they've"
            )
        {
            start = idx;
        } else if !is_gap_modifier(token, &lower) {
            break;
        }
    }
    let pos = word_indices.partition_point(|&idx| idx < start);
    if pos >= 2
        && normalized_word(&sentence[word_indices[pos - 1]], source) == "or"
        && normalized_word(&sentence[word_indices[pos - 2]], source)
            == normalized_word(&sentence[start], source)
        && !has_hard_boundary(&sentence[word_indices[pos - 2] + 1..start])
    {
        start = word_indices[pos - 2];
    }
    start
}

fn coordinated_tail(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
    fallback_end: usize,
) -> usize {
    let mut pos = word_pos + 1;
    let mut end_idx = fallback_end;

    while let Some(&next_idx) = word_indices.get(pos) {
        let separator = &sentence[end_idx + 1..next_idx];
        // Commas are allowed only within a participle list; other clause
        // boundaries and code tokens always stop the match.
        if separator
            .iter()
            .any(|t| !t.kind.is_whitespace() && !t.kind.is_comma())
        {
            break;
        }
        let lower = normalized_word(&sentence[next_idx], source);
        let conjunction = matches!(lower.as_str(), "and" | "or" | "but" | "yet");
        if conjunction {
            pos += 1;
        } else if !separator.iter().any(|t| t.kind.is_comma()) {
            break;
        }
        while let Some(&idx) = word_indices.get(pos) {
            let token = &sentence[idx];
            if is_gap_modifier(token, &normalized_word(token, source)) {
                pos += 1;
            } else {
                break;
            }
        }
        let Some(&idx) = word_indices.get(pos) else {
            break;
        };
        let token = &sentence[idx];
        // A contrast can introduce a new active predicate sharing the same
        // subject: "The candidate was interviewed but rejected the offer."
        // A following object is a useful signal that the auxiliary does not
        // carry over to this verb.
        let contrast_with_object = matches!(lower.as_str(), "but" | "yet")
            && !allows_retained_object(&normalized_word(token, source))
            && word_indices.get(pos + 1).is_some_and(|&following_idx| {
                !has_hard_boundary(&sentence[idx + 1..following_idx])
                    && (sentence[following_idx].kind.is_determiner()
                        || sentence[following_idx].kind.is_pronoun()
                        || is_contextual_noun(&sentence[following_idx]))
            });
        if contrast_with_object
            || sentence[next_idx..idx]
                .iter()
                .any(|t| !t.kind.is_word() && !t.kind.is_whitespace())
            || is_descriptive_compound(sentence, idx, source)
            || !is_participle_candidate(token, source)
            || should_suppress_adjectival(
                token,
                source,
                has_agentive_by(sentence, word_indices, pos, source),
                PassiveSource::Be,
                has_event_evidence(sentence, word_indices, pos, source),
            )
        {
            break;
        }
        end_idx = idx;
        pos += 1;
    }
    end_idx
}

fn is_existential_reduced_passive(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
) -> bool {
    if word_pos < 3 {
        return false;
    }

    let candidate_idx = word_indices[word_pos];
    if let Some(&next_idx) = word_indices.get(word_pos + 1)
        && !has_hard_boundary(&sentence[candidate_idx + 1..next_idx])
        && (is_contextual_noun(&sentence[next_idx])
            || (normalized_word(&sentence[next_idx], source).ends_with("ing")
                && !sentence[candidate_idx].kind.is_verb_past_participle_form()))
    {
        // "There was a light dignified knocking" contains an attributive
        // adjective; the nominal before it is not its passive subject.
        return false;
    }
    let mut nominal_pos = word_pos - 1;
    while nominal_pos > 0
        && is_gap_modifier(
            &sentence[word_indices[nominal_pos]],
            &normalized_word(&sentence[word_indices[nominal_pos]], source),
        )
    {
        nominal_pos -= 1;
    }
    if !is_contextual_noun(&sentence[word_indices[nominal_pos]]) {
        return false;
    }
    let mut saw_nominal = false;

    for pos in (0..word_pos).rev().take(7) {
        let idx = word_indices[pos];
        if has_hard_boundary(&sentence[idx + 1..candidate_idx]) {
            break;
        }

        let token = &sentence[idx];
        if token.kind.is_upos(UPOS::SCONJ)
            || matches!(
                normalized_word(token, source).as_str(),
                "who" | "whom" | "whose" | "which" | "that" | "where" | "when" | "as"
            )
        {
            return false;
        }
        if is_contextual_noun(token) {
            saw_nominal = true;
        }
        if (token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX))
            && !is_be_form(&normalized_word(token, source))
        {
            return false;
        }

        let text = token.get_str(source);
        if is_be_form(&text.to_ascii_lowercase()) && pos > 0 && saw_nominal {
            let previous = sentence[word_indices[pos - 1]].get_str(source);
            return previous.eq_ignore_ascii_case("there");
        }
    }

    false
}

/// A noun followed by a participle, a time modifier, and a separate finite
/// predicate has the shape of a reduced relative, even for forms shared with
/// the simple past: "the package sent yesterday arrived".
fn is_temporal_reduced_passive(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
) -> bool {
    let Some(previous_pos) = word_pos.checked_sub(1) else {
        return false;
    };
    let Some(&time_idx) = word_indices.get(word_pos + 1) else {
        return false;
    };
    let Some(&predicate_idx) = word_indices.get(word_pos + 2) else {
        return false;
    };
    let previous_idx = word_indices[previous_pos];
    let candidate_idx = word_indices[word_pos];
    // Many verbs are also intransitive ("the door closed yesterday remains
    // shut"). Only recover common transitive forms in this ambiguous shape.
    let lower = normalized_word(&sentence[candidate_idx], source);
    matches!(
        lower.as_str(),
        "sent"
            | "read"
            | "written"
            | "seen"
            | "found"
            | "given"
            | "taken"
            | "built"
            | "chosen"
            | "reviewed"
            | "approved"
            | "published"
            | "reported"
            | "filed"
            | "signed"
    ) && is_contextual_noun(&sentence[previous_idx])
        && sentence[candidate_idx].kind.is_upos(UPOS::VERB)
        && matches!(
            normalized_word(&sentence[time_idx], source).as_str(),
            "yesterday" | "today" | "recently"
        )
        && (sentence[predicate_idx].kind.is_upos(UPOS::VERB)
            || sentence[predicate_idx].kind.is_upos(UPOS::AUX))
        && !has_hard_boundary(&sentence[previous_idx + 1..predicate_idx])
}

fn is_strong_irregular_reduced_passive(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
    is_question: bool,
) -> bool {
    let candidate_idx = word_indices[word_pos];
    let candidate = &sentence[candidate_idx];

    // In a question, the finite perfect auxiliary can precede the subject:
    // "Has Alice written the report?" is active, not a reduced relative.
    if is_question
        && word_indices[..word_pos].iter().take(3).any(|&idx| {
            matches!(
                normalized_word(&sentence[idx], source).as_str(),
                "have" | "has" | "had" | "haven't" | "hasn't" | "hadn't"
            )
        })
        && !word_indices[..word_pos]
            .iter()
            .any(|&idx| is_be_form(&normalized_word(&sentence[idx], source)))
    {
        return false;
    }

    // For regular verbs the preterite and past participle are identical, so without
    // a dependency parser "the door closed" cannot safely be distinguished from
    // "the data collected". Restrict agentless reduced passives to participle-only
    // forms such as "the report written yesterday".
    // Lemma/participle homographs are not high-confidence reduced passives;
    // dictionary merges can retain the participle flag without the lemma flag.
    if matches!(
        normalized_word(candidate, source).as_str(),
        "come"
            | "become"
            | "run"
            | "read"
            | "cut"
            | "hit"
            | "set"
            | "put"
            | "shut"
            | "split"
            | "cost"
            | "hurt"
            | "let"
            | "spread"
            | "burst"
            | "cast"
            | "quit"
    ) {
        return false;
    }
    if !candidate.kind.is_verb_past_participle_only()
        || candidate.kind.is_upos(UPOS::ADJ)
        || candidate.kind.is_verb_lemma()
        || normalized_word(candidate, source).ends_with("ed")
    {
        return false;
    }

    if word_pos == 0 {
        return false;
    }
    let previous_idx = word_indices[word_pos - 1];

    if has_hard_boundary(&sentence[previous_idx + 1..candidate_idx]) {
        return false;
    }

    let previous = &sentence[previous_idx];
    let previous_lower = previous.get_str(source).to_ascii_lowercase();

    // A reduced relative normally modifies a noun phrase, not a personal pronoun.
    // This also prevents contractions/possessives such as `it's broken` or
    // `John's written report` from being mistaken for agentless reduced passives.
    if previous_lower.ends_with("'s") || previous_lower.ends_with("’s") {
        return false;
    }

    is_contextual_noun(previous)
}

fn is_participle_candidate(token: &Token, source: &[char]) -> bool {
    let lower = normalized_word(token, source);
    if token
        .kind
        .as_word()
        .and_then(|m| m.as_ref())
        .is_some_and(|m| {
            m.pos_tag
                .is_some_and(|tag| !matches!(tag, UPOS::VERB | UPOS::AUX | UPOS::ADJ))
        })
    {
        return false;
    }
    // "been" participates in an auxiliary chain; it is not itself the passive
    // lexical verb. Treating it as one creates duplicate lints.
    if is_be_form(&lower) {
        return false;
    }
    if token.kind.is_verb_past_participle_form() {
        return true;
    }
    // Many dictionary verbs have no tense metadata (e.g. deleted, reviewed,
    // delivered). Recover regular forms using lexical verb evidence, rather
    // than treating every adjective ending in -ed or -en as a participle.
    // These lemma spellings end in -ed without being inflected participles.
    if matches!(
        lower.as_str(),
        "need"
            | "feed"
            | "bleed"
            | "breed"
            | "speed"
            | "heed"
            | "weed"
            | "seed"
            | "proceed"
            | "exceed"
            | "succeed"
    ) {
        return false;
    }
    (token.kind.is_verb() || token.kind.is_upos(UPOS::VERB))
        && looks_like_participle_surface(&lower)
}

fn has_personal_subject(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
) -> bool {
    for &idx in indices[..pos].iter().rev().take(8) {
        if has_hard_boundary(&sentence[idx + 1..indices[pos]]) {
            break;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        let root = lower.split('\'').next().unwrap_or(&lower);
        if matches!(root, "i" | "you" | "he" | "she" | "we" | "they") {
            return true;
        }
        if !is_be_form(&lower)
            && !is_get_form(&lower)
            && !is_gap_modifier(token, &lower)
            && !matches!(
                lower.as_str(),
                "have"
                    | "has"
                    | "had"
                    | "will"
                    | "would"
                    | "can"
                    | "cannot"
                    | "can't"
                    | "could"
                    | "couldn't"
                    | "may"
                    | "might"
                    | "must"
                    | "should"
            )
        {
            break;
        }
    }
    false
}

fn has_state_complement(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
    by_agent: bool,
) -> bool {
    if by_agent {
        return false;
    }
    let lower = normalized_word(&sentence[indices[pos]], source);
    let next = indices
        .get(pos + 1)
        .map(|&idx| normalized_word(&sentence[idx], source));
    match (lower.as_str(), next.as_deref()) {
        ("grown" | "fed", Some("up")) => true,
        ("worn", Some("out")) => true,
        ("relieved", next) => next != Some("of"),
        ("obliged", next) => next != Some("to"),
        ("bound", Some("to")) => indices
            .get(pos + 2)
            .is_some_and(|&idx| sentence[idx].kind.is_upos(UPOS::VERB)),
        ("determined", Some("to" | "not")) => has_personal_subject(sentence, indices, pos, source),
        ("mixed", Some("up")) => {
            indices
                .get(pos + 2)
                .is_some_and(|&idx| normalized_word(&sentence[idx], source) == "in")
                && has_personal_subject(sentence, indices, pos, source)
        }
        ("tied", Some("up")) => indices.iter().skip(pos + 2).take(6).any(|&idx| {
            matches!(
                normalized_word(&sentence[idx], source).as_str(),
                "business" | "work" | "meeting" | "meetings"
            )
        }),
        ("headed", Some("for" | "toward" | "towards" | "to")) => true,
        ("mistaken", next) => next != Some("for"),
        ("used", Some("to")) => indices.get(pos + 2).is_some_and(|&idx| {
            let token = &sentence[idx];
            !token.kind.is_upos(UPOS::VERB)
                && !token.kind.is_upos(UPOS::AUX)
                && !is_be_form(&normalized_word(token, source))
        }),
        _ => false,
    }
}

fn allows_retained_object(lower: &str) -> bool {
    matches!(
        lower,
        "given"
            | "told"
            | "paid"
            | "offered"
            | "awarded"
            | "taught"
            | "denied"
            | "shown"
            | "asked"
            | "sent"
            | "promised"
    )
}

fn is_attributive_after_get(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
    kind: PassiveSource,
) -> bool {
    if kind != PassiveSource::Get {
        return false;
    }
    let Some(&next) = indices.get(pos + 1) else {
        return false;
    };
    if has_hard_boundary(&sentence[indices[pos] + 1..next]) {
        return false;
    }
    let lower = normalized_word(&sentence[indices[pos]], source);
    // Ditransitive passives can legitimately retain a nominal object.
    if allows_retained_object(&lower) {
        return false;
    }
    let next_lower = normalized_word(&sentence[next], source);
    is_contextual_noun(&sentence[next])
        && !is_time_word(&next_lower)
        && !matches!(next_lower.as_str(), "yesterday" | "today" | "tomorrow")
}

/// Event cues disambiguate common result states such as "are married" from
/// constructions such as "were married in June" or "is being prepared".
fn has_event_evidence(sentence: &[Token], indices: &[usize], pos: usize, source: &[char]) -> bool {
    let candidate = indices[pos];
    for &idx in indices[..pos].iter().rev().take(6) {
        if has_hard_boundary(&sentence[idx + 1..candidate]) {
            break;
        }
        let lower = normalized_word(&sentence[idx], source);
        if matches!(
            lower.as_str(),
            "being"
                | "be"
                | "getting"
                | "recently"
                | "deliberately"
                | "intentionally"
                | "newly"
                | "just"
        ) {
            return true;
        }
        if !is_gap_modifier(&sentence[idx], &lower) && !is_be_form(&lower) {
            break;
        }
    }
    let mut previous = String::new();
    for &idx in indices.iter().skip(pos + 1).take(4) {
        if has_hard_boundary(&sentence[candidate + 1..idx]) {
            break;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        if matches!(
            lower.as_str(),
            "yesterday" | "recently" | "earlier" | "every"
        ) || (matches!(previous.as_str(), "in" | "on" | "at" | "last") && is_time_word(&lower))
        {
            return true;
        }
        if token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX) {
            break;
        }
        previous = lower;
    }
    false
}

fn has_clear_done_passive(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
) -> bool {
    if normalized_word(&sentence[indices[pos]], source) != "done" {
        return false;
    }
    let subject = indices[..pos]
        .iter()
        .rev()
        .take(8)
        .find(|&&idx| is_contextual_noun(&sentence[idx]) || sentence[idx].kind.is_pronoun());
    if subject.is_some_and(|&idx| {
        matches!(
            normalized_word(&sentence[idx], source).as_str(),
            "i" | "you" | "he" | "she" | "we" | "they"
        )
    }) {
        return false;
    }
    let mut previous_pos = pos;
    while let Some(idx) = previous_pos.checked_sub(1) {
        let lower = normalized_word(&sentence[indices[idx]], source);
        if is_gap_modifier(&sentence[indices[idx]], &lower) {
            previous_pos = idx;
            continue;
        }
        let before = indices[..idx].iter().rev().take(4).find_map(|&prev_idx| {
            let word = normalized_word(&sentence[prev_idx], source);
            (!is_gap_modifier(&sentence[prev_idx], &word)).then_some(word)
        });
        return match lower.as_str() {
            "being" => true,
            "been" => before.is_some_and(|word| matches!(word.as_str(), "have" | "has" | "had")),
            "be" => before.is_some_and(|word| {
                matches!(
                    word.as_str(),
                    "to" | "can"
                        | "could"
                        | "may"
                        | "might"
                        | "must"
                        | "shall"
                        | "should"
                        | "will"
                        | "would"
                        | "can't"
                        | "cannot"
                        | "couldn't"
                        | "shouldn't"
                        | "won't"
                        | "wouldn't"
                        | "mustn't"
                )
            }),
            _ => false,
        };
    }
    false
}

fn should_suppress_adjectival(
    token: &Token,
    source: &[char],
    by_agent: bool,
    passive_source: PassiveSource,
    event_evidence: bool,
) -> bool {
    if by_agent {
        return false;
    }

    let lower = normalized_word(token, source);
    // "Get started" commonly means "begin", including tutorial headings.
    if lower == "started" && passive_source == PassiveSource::Get {
        return true;
    }

    // Perfect aspect and degree modifiers do not make an emotional state
    // eventive: "I've been tired" and "I'm just worried" remain copular.
    if matches!(
        lower.as_str(),
        "tired"
            | "bored"
            | "interested"
            | "excited"
            | "worried"
            | "concerned"
            | "satisfied"
            | "disappointed"
            | "confused"
            | "annoyed"
            | "frightened"
            | "scared"
            | "terrified"
            | "accustomed"
            | "related"
            | "surprised"
            | "puzzled"
            | "offended"
            | "embarrassed"
            | "flattered"
            | "engrossed"
            | "paralyzed"
            | "delighted"
            | "astounded"
    ) {
        return true;
    }

    if token.kind.is_upos(UPOS::ADJ) && !event_evidence {
        return true;
    }

    // Keep PassivePy's deliberately conservative ambiguity set even when a
    // contextual tagger happens to call the token a verb. These lemmas are a
    // major source of false positives in simple "be + participle" detectors.
    if passivepy_ambiguous_participle(&lower) {
        return true;
    }

    // These are strongly lexicalized result/state readings in ordinary copular
    // use. Even if the tagger calls them verbs, flagging `he is gone`, `I am
    // done`, or `he is drunk` as passive is more harmful than the small recall
    // gain. An explicit by-agent still overrides this suppression above.
    if lexicalized_nonpassive_state(&lower) {
        return true;
    }

    // A VERB tag alone cannot distinguish an event from a result state. Require
    // additional event evidence for the broader ambiguity list.
    if likely_participial_adjective(&lower) && !event_evidence {
        return true;
    }

    // Get/become are especially productive with predicative adjectives ("got
    // tired", "became interested"), so require the contextual tagger to regard
    // the complement as a verb.
    if matches!(passive_source, PassiveSource::Get | PassiveSource::Become)
        && !token.kind.is_upos(UPOS::VERB)
    {
        return true;
    }

    false
}

fn is_gap_modifier(token: &Token, lower: &str) -> bool {
    token.kind.is_upos(UPOS::ADV)
        || token
            .kind
            .as_word()
            .and_then(|m| m.as_ref())
            .is_some_and(|m| m.pos_tag.is_none() && m.is_adverb())
        || matches!(
            lower,
            "not"
                | "never"
                | "just"
                | "only"
                | "already"
                | "still"
                | "almost"
                | "also"
                | "then"
                | "now"
                | "recently"
                | "previously"
                | "currently"
                | "carefully"
                | "deliberately"
                | "intentionally"
                | "automatically"
                | "manually"
                | "completely"
                | "fully"
                | "partially"
                | "partly"
                | "successfully"
                | "originally"
                | "newly"
                | "widely"
                | "commonly"
                | "generally"
                | "directly"
                | "immediately"
                | "finally"
                | "eventually"
                | "reportedly"
                | "allegedly"
                | "subsequently"
                | "slightly"
                | "highly"
                | "well"
        )
}

fn is_be_form(lower: &str) -> bool {
    matches!(
        lower,
        "am" | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "being"
            | "isn't"
            | "aren't"
            | "wasn't"
            | "weren't"
    )
}

fn is_unambiguous_be_contraction(lower: &str) -> bool {
    matches!(lower, "i'm" | "you're" | "we're" | "they're")
}

fn is_get_form(lower: &str) -> bool {
    matches!(lower, "get" | "gets" | "got" | "gotten" | "getting")
}

fn is_become_form(lower: &str) -> bool {
    matches!(lower, "become" | "becomes" | "became" | "becoming")
}

fn has_agentive_by(
    sentence: &[Token],
    word_indices: &[usize],
    word_pos: usize,
    source: &[char],
) -> bool {
    let candidate_idx = word_indices[word_pos];
    let mut in_prepositional_phrase = false;

    for (offset, &idx) in word_indices.iter().skip(word_pos + 1).take(9).enumerate() {
        if has_hard_boundary(&sentence[candidate_idx + 1..idx]) {
            return false;
        }

        let token = &sentence[idx];
        let text = normalized_word(token, source);
        if text == "by" {
            return !by_phrase_is_nonagentive(sentence, idx, source);
        }
        // An adjunct may intervene between the participle and its agent:
        // "guaranteed on any input by a finite state-space". Do not let the
        // search cross a new finite clause. A coordinated participle can
        // share the same agent: "was stunned and exhausted by the speech".
        if matches!(text.as_str(), "and" | "or" | "but" | "yet") {
            let mut next_pos = word_pos + offset + 2;
            while let Some(&next_idx) = word_indices.get(next_pos) {
                let next = &sentence[next_idx];
                if is_gap_modifier(next, &normalized_word(next, source)) {
                    next_pos += 1;
                } else {
                    break;
                }
            }
            if !word_indices.get(next_pos).is_some_and(|&next_idx| {
                is_participle_candidate(&sentence[next_idx], source)
                    && !has_hard_boundary(&sentence[idx + 1..next_idx])
            }) {
                return false;
            }
            continue;
        }
        if token.kind.is_upos(UPOS::SCONJ) {
            return false;
        }
        if token.kind.is_upos(UPOS::ADP)
            || matches!(
                text.as_str(),
                "on" | "in" | "at" | "for" | "with" | "to" | "from"
            )
        {
            in_prepositional_phrase = true;
            continue;
        }
        if (token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX))
            && !is_participle_candidate(token, source)
        {
            return false;
        }
        if !in_prepositional_phrase
            && (token.kind.is_determiner()
                || token.kind.is_pronoun()
                || token.kind.is_upos(UPOS::NOUN)
                || token.kind.is_upos(UPOS::PROPN))
        {
            return false;
        }
    }

    false
}

fn by_phrase_is_nonagentive(sentence: &[Token], by_idx: usize, source: &[char]) -> bool {
    let mut iter = sentence[by_idx + 1..]
        .iter()
        .filter(|tok| !tok.kind.is_whitespace());

    let Some(mut next) = iter.next() else {
        return true;
    };

    if next.kind.is_punctuation() || next.kind.is_unlintable() || next.kind.is_upos(UPOS::SCONJ) {
        return true;
    }
    if next.kind.is_number() {
        // Bare numeric deadlines ("by 5", "by 2027") are usually temporal,
        // but a following non-time word can turn the phrase into an agent or
        // measure ("by 3 judges", "by 10 percent").
        return iter
            .take_while(|tok| !tok.kind.is_chunk_terminator())
            .find(|tok| tok.kind.is_word_like())
            .map(|tok| {
                let lower = normalized_word(tok, source);
                is_time_word(&lower) || lower == "by"
            })
            .unwrap_or(true);
    }

    let mut lower = next.get_str(source).to_ascii_lowercase();
    // Bare communication/transport phrases describe means, not actors. Keep
    // determiner-led phrases available as agents (e.g. "hit by a train").
    if matches!(
        lower.as_str(),
        "phone"
            | "email"
            | "fax"
            | "train"
            | "bus"
            | "plane"
            | "air"
            | "sea"
            | "candlelight"
            | "cesarean"
            | "caesarean"
    ) {
        return true;
    }
    if matches!(lower.as_str(), "the" | "a" | "an") {
        let Some(after_determiner) = iter.find(|tok| tok.kind.is_word_like()) else {
            return false;
        };
        next = after_determiner;
        if next.kind.is_number() {
            return iter
                .take_while(|tok| !tok.kind.is_chunk_terminator())
                .find(|tok| tok.kind.is_word_like())
                .map(|tok| {
                    let lower = normalized_word(tok, source);
                    is_time_word(&lower) || lower == "by"
                })
                .unwrap_or(true);
        }
        lower = next.get_str(source).to_ascii_lowercase();
    }

    if matches!(
        lower.as_str(),
        "next" | "last" | "following" | "previous" | "same" | "early" | "late"
    ) {
        return iter
            .take_while(|tok| !tok.kind.is_chunk_terminator())
            .find(|tok| tok.kind.is_word_like())
            .is_some_and(|tok| is_time_word(&normalized_word(tok, source)));
    }
    is_time_word(&lower)
        || matches!(
            lower.as_str(),
            "hand"
                | "design"
                | "default"
                | "chance"
                | "accident"
                | "mistake"
                | "definition"
                | "window"
                | "door"
                | "river"
                | "lake"
                | "island"
                | "coast"
                | "shore"
                | "beach"
                | "harbor"
                | "ocean"
                | "road"
                | "stairs"
        )
}

fn is_time_word(lower: &str) -> bool {
    matches!(
        lower,
        "am" | "pm"
            | "a.m."
            | "p.m."
            | "o'clock"
            | "noon"
            | "midnight"
            | "morning"
            | "afternoon"
            | "evening"
            | "night"
            | "dawn"
            | "dusk"
            | "today"
            | "tomorrow"
            | "yesterday"
            | "then"
            | "now"
            | "monday"
            | "tuesday"
            | "wednesday"
            | "thursday"
            | "friday"
            | "saturday"
            | "sunday"
            | "january"
            | "february"
            | "march"
            | "april"
            | "may"
            | "june"
            | "july"
            | "august"
            | "september"
            | "october"
            | "november"
            | "december"
            | "deadline"
            | "time"
            | "end"
            | "start"
            | "hour"
            | "hours"
            | "minute"
            | "minutes"
            | "day"
            | "days"
            | "week"
            | "weeks"
            | "month"
            | "months"
            | "year"
            | "years"
    )
}

fn passivepy_ambiguous_participle(lower: &str) -> bool {
    // Surface forms corresponding to PassivePy's ambiguity lemmas. PassivePy
    // generally requires explicit "by" evidence for these to avoid adjective
    // readings such as "I was exhausted" and "I am involved".
    matches!(
        lower,
        "associated"
            | "involved"
            | "exhausted"
            | "based"
            | "led"
            | "stunned"
            | "overrated"
            | "filled"
            | "born"
            | "borne"
            | "complicated"
            | "reserved"
            | "heated"
            | "screwed"
    )
}

fn lexicalized_nonpassive_state(lower: &str) -> bool {
    matches!(lower, "gone" | "done" | "drunk" | "fainted")
}

fn likely_participial_adjective(lower: &str) -> bool {
    // Additional high-frequency result/state participles. Unlike the PassivePy
    // ambiguity set above, these are only suppressors when Harper does not give
    // the token a contextual VERB tag.
    matches!(
        lower,
        "tired"
            | "bored"
            | "interested"
            | "excited"
            | "worried"
            | "concerned"
            | "satisfied"
            | "disappointed"
            | "confused"
            | "annoyed"
            | "frightened"
            | "scared"
            | "terrified"
            | "married"
            | "divorced"
            | "engaged"
            | "retired"
            | "accustomed"
            | "related"
            | "located"
            | "situated"
            | "finished"
            | "prepared"
            | "closed"
            | "broken"
            | "dressed"
            | "seated"
            | "settled"
            | "descended"
            | "fit"
            | "ready"
    )
}

fn looks_like_participle_surface(lower: &str) -> bool {
    lower.ends_with("ed")
        || matches!(
            lower,
            "born"
                | "beat"
                | "beaten"
                | "bitten"
                | "broken"
                | "chosen"
                | "fallen"
                | "forgotten"
                | "frozen"
                | "hidden"
                | "ridden"
                | "risen"
                | "shaken"
                | "spoken"
                | "stolen"
                | "woken"
                | "woven"
                | "forgiven"
                | "forbidden"
                | "built"
                | "bought"
                | "brought"
                | "caught"
                | "dealt"
                | "done"
                | "drawn"
                | "drunk"
                | "driven"
                | "eaten"
                | "felt"
                | "found"
                | "given"
                | "gone"
                | "grown"
                | "held"
                | "kept"
                | "known"
                | "led"
                | "left"
                | "lost"
                | "made"
                | "meant"
                | "paid"
                | "read"
                | "run"
                | "said"
                | "seen"
                | "sent"
                | "shown"
                | "sold"
                | "spent"
                | "stuck"
                | "taught"
                | "taken"
                | "told"
                | "thought"
                | "thrown"
                | "understood"
                | "won"
                | "worn"
                | "written"
        )
}

fn has_hard_boundary(tokens: &[Token]) -> bool {
    tokens.iter().any(|tok| {
        tok.kind.is_chunk_terminator()
            || tok.kind.is_unlintable()
            || (tok.kind.is_punctuation() && !tok.kind.is_hyphen())
    })
}

#[cfg(test)]
mod tests {
    use super::PassiveVoice;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    fn passive(text: &str) {
        assert_lint_count(text, PassiveVoice, 1);
    }

    fn active(text: &str) {
        assert_no_lints(text, PassiveVoice);
    }

    #[test]
    fn detects_basic_be_passives() {
        passive("The ball was dropped.");
        passive("The president was impeached before his second term.");
        passive("The ballots were clearly marked on Election Day.");
        passive("An error was made in the tabulation of votes.");
    }

    #[test]
    fn detects_agentless_passives() {
        passive("The issue will be resolved.");
        passive("The file can be opened.");
        passive("Politics would have to be considered.");
        passive("Some politicians are not to be trusted.");
        passive("The work must not be done.");
        passive("The task has not been done.");
        passive("The work can't be done.");
    }

    #[test]
    fn detects_complex_auxiliary_chains() {
        passive("The package has been successfully delivered.");
        passive("The proposal is being carefully reviewed.");
        passive("The result should not have been reported.");
        passive("The files may already have been deleted.");
    }

    #[test]
    fn detects_get_passives() {
        passive("He got fired yesterday.");
        passive("Donald Trump got beat by Joe Biden in the election.");
        passive("The files are getting deleted automatically.");
    }

    #[test]
    fn detects_become_passive_patterns() {
        passive("The candidate became surrounded by reporters.");
        passive("The method became widely accepted.");
    }

    #[test]
    fn detects_intervening_modifiers() {
        passive("The result was not fully explained.");
        passive("The package was recently and deliberately removed.");
        passive("The gift was carefully hand-wrapped.");
    }

    #[test]
    fn detects_coordinated_passives_as_one_lint() {
        passive("The rule was determined and implemented.");
        passive("The file was reviewed and carefully approved.");
    }

    #[test]
    fn detects_contrasting_coordinated_passives() {
        for text in [
            "The report was reviewed but later rejected.",
            "The plan was proposed but not adopted.",
            "The contract was signed, but not delivered.",
            "The company was launched yet quickly closed.",
        ] {
            passive(text);
        }
        passive("The candidate was interviewed but rejected the offer.");
        passive("The candidate was interviewed yet declined the job.");
        passive("The candidate was interviewed but rejected it.");
        passive("The candidate was interviewed but offered a job.");
    }

    #[test]
    fn detects_reduced_passives_with_agents() {
        passive("Arrested by the police, he never thought this would be his end.");
        passive("The report written by Alice was useful.");
        passive("Resources exhausted by humans can recover slowly.");
    }

    #[test]
    fn detects_high_confidence_agentless_reduced_passives() {
        passive("The report written yesterday contains several errors.");
        passive("The man seen near the station called police.");
        passive("There was no change detected in her behavior.");
    }

    #[test]
    fn avoids_common_participial_adjectives() {
        active("I am stunned at the impact politics is having on our country.");
        active("The mayor was satisfied with the voters' opinion.");
        active("The debate tonight was heated.");
        active("Politics are very complicated.");
        active("I am tired of politics.");
        active("She was interested in the result.");
        active("They are married.");
        active("The office is located downtown.");
    }

    #[test]
    fn explicit_agent_overrides_adjective_suppression() {
        passive("Natural resources were exhausted by humans.");
        passive("The audience was stunned by the announcement.");
        passive("The project was complicated by new regulations.");
    }

    #[test]
    fn temporal_by_does_not_force_adjective_into_passive() {
        active("He was tired by noon.");
        active("She was exhausted by the end of the day.");
    }

    #[test]
    fn does_not_confuse_active_perfect_with_passive() {
        active("She has written the letter.");
        active("They have ruined the country.");
        active("He had finished the report.");
        active("The senator had won the race.");
        active("He's written three books.");
    }

    #[test]
    fn does_not_confuse_simple_past_with_reduced_passive() {
        active("The door closed at noon.");
        active("The candidate lost the election.");
        active("The committee approved the proposal.");
    }

    #[test]
    fn does_not_flag_ordinary_copular_adjectives() {
        active("The world of politics is quite fascinating.");
        active("I wish the government was better.");
        active("Politics was the focus of her television viewing.");
        active("Politics are very controversial.");
    }

    #[test]
    fn detects_passive_inside_longer_sentences() {
        passive("I have seen that your paper has been accepted by JAIR.");
        passive("The initial intent was that the configuration could be easily replaced.");
    }

    #[test]
    fn detects_inverted_question_passives() {
        passive("Was the report written?");
        passive("Why was the report carefully written?");
        passive("Were the ballots counted?");
        passive("Did he get fired?");
        passive("Has the report been written?");
    }

    #[test]
    fn avoids_active_or_adjectival_questions() {
        active("Was John tired?");
        active("Has she written the letter?");
        active("Hasn't Alice written the report?");
        active("Hasn’t Alice written the report?");
        active("Did the committee approve the report?");
    }

    #[test]
    fn by_number_can_be_agent_or_measure() {
        passive("The proposal was reviewed by 3 judges.");
        passive("Reduced by 10 percent, the rate became manageable.");
        active("He was tired by 5 pm.");
    }

    #[test]
    fn suppresses_lexicalized_result_states() {
        active("He is gone.");
        active("I am done.");
        active("He was born in Canada.");
        active("He is drunk.");
        active("I can't be done with this yet.");
    }

    #[test]
    fn avoids_ambiguous_s_contraction_without_strong_evidence() {
        active("It's broken.");
        active("He's finished.");
        passive("It's been broken by vandals.");
    }
    #[test]
    fn regression_corpus() {
        let cases = [
            ("Getting started with Harper.", 0),
            ("getting-started", 0),
            ("The engine got started by the mechanic.", 1),
            ("The engine was started by the mechanic.", 1),
            ("His count of enchanted objects had diminished by one.", 0),
            (
                "She turned her head as there was a light dignified knocking at the front door.",
                0,
            ),
            ("There was a beautifully painted door.", 0),
            ("There were people caught stealing.", 1),
            ("She has written the report by hand.", 0),
            ("They had completed the task by noon.", 0),
            ("He walked by the river.", 0),
            ("The vessel sailed by the coast.", 0),
            ("The hikers walked by the beach.", 0),
            ("The committee approved the proposal by a wide margin.", 0),
            ("She read by the window.", 0),
            ("He is tired and walks by the river.", 0),
            ("The exhausted hikers walked by the lake.", 0),
            ("The report written by Alice was published by Bob.", 2),
            ("He got written permission from his manager.", 0),
            ("Was the written report useful?", 0),
            ("Was the report that Alice wrote published?", 1),
            ("The file was deleted and the report was printed.", 2),
            ("The file was reviewed, approved, and published.", 1),
            ("The file was reviewed but later rejected.", 1),
            ("The candidate was interviewed but rejected the offer.", 1),
            ("The file was reviewed and Alice approved it.", 1),
            ("The report was reviewed and approved by Alice.", 1),
            ("The file WAS DELETED.", 1),
            ("The report wasn’t reviewed.", 1),
            ("They’re being watched.", 1),
            ("You're invited to the meeting.", 1),
            ("The report is well-written.", 1),
            ("She was red-haired.", 0),
            ("The file was deleted; the folder was renamed.", 2),
            ("The file was deleted. The folder was renamed.", 2),
            ("The file was\ncarefully deleted.", 1),
            ("The file was\n\ndeleted.", 0),
            ("She is exhausted by now.", 0),
            ("She was exhausted by Friday morning.", 0),
            ("She was exhausted by the next day.", 0),
            ("She was exhausted by running.", 1),
            ("The room was filled by the time we arrived.", 0),
            ("The proposal was approved by three judges.", 1),
            ("He was bored by 5:30.", 0),
            ("By Alice, the report was written.", 1),
            ("Wasn’t the report written?", 1),
            ("The plan must be carried out.", 1),
            ("The report needs to be rewritten.", 1),
            ("She was running by the lake.", 0),
            ("The retired teacher arrived.", 0),
            ("There was a tired person by the door.", 0),
            ("The window was broken by vandals.", 1),
            ("She had a broken window by the stairs.", 0),
            ("The door is closed.", 0),
            ("They were married in June.", 1),
            ("I am well prepared.", 0),
            ("The meeting is scheduled for tomorrow.", 1),
            ("The price dropped by 10 percent.", 0),
            ("Prices increased by a wide margin.", 0),
            ("The committee approved by a narrow margin.", 0),
            ("The committee walked by the river.", 0),
            ("The floor is wooden by design.", 0),
            ("The floor is open by noon.", 0),
            ("She was left-handed.", 0),
            ("He got signed copies of the book.", 0),
            ("The issues discussed by the team are complex.", 1),
            ("The report carefully reviewed by Alice was published.", 2),
            ("He got paid overtime.", 1),
            ("She got tired.", 0),
            ("He became interested.", 0),
            ("The report is being written by Alice.", 1),
            ("It has been written by Alice.", 1),
            ("She has not yet written the report by hand.", 0),
            ("I have seen that the report was written by Alice.", 1),
            ("They’re watched by cameras.", 1),
            ("The report isn’t reviewed by Alice.", 1),
            ("I've been tired all week.", 0),
            ("I am just tired.", 0),
            ("She has been worried recently.", 0),
            ("They have been married for twenty years.", 0),
            ("The report is being prepared.", 1),
            ("The audience was stunned and exhausted by the speech.", 1),
            ("The committee voted by phone.", 0),
            ("The senator voted by email.", 0),
            ("The committee travelled by train.", 0),
            ("The report was written by hand.", 1),
            ("The employees got fired by the manager.", 1),
            ("The files got deleted by Alice.", 1),
            ("The company is in profit.", 0),
            ("There are many well-known plays by William Shakespeare.", 0),
            ("The wall measured 10 by 20 by 30 cm.", 0),
            ("Come by before you leave.", 0),
            ("Come by our house tomorrow.", 0),
            ("He was headed for the door.", 0),
            ("There was so much to read.", 0),
            ("There were men who had hated his guts.", 0),
            ("There was a story that he'd agreed to pay.", 0),
            ("There was a boom as John shut the windows.", 0),
            ("There was a line where my ragged lawn ended.", 0),
            ("Have you got everything you need?", 0),
            ("I am quite surprised.", 0),
            ("I'm grown up now.", 0),
            ("I'm used to it.", 0),
            ("I'm used to cold weather.", 0),
            ("The tool is used to cut paper.", 1),
            ("The things get used up.", 1),
            ("She got used to the noise.", 0),
            ("She was seated on the throne.", 0),
            ("We're descended from that family.", 0),
            ("It was when I asked you?", 0),
            ("Was I the same when I got up?", 0),
            ("Was I ready to read?", 0),
            ("Is this a written report?", 0),
            ("Some words have got altered.", 1),
            ("The company was headed by Alice.", 1),
            ("I was surprised by the announcement.", 1),
            ("There were files detected on the disk.", 1),
            ("The runners run fast.", 0),
            ("Several cars come quickly.", 0),
            ("Reports read well.", 0),
            ("The company run by Alice was successful.", 1),
            ("A feeling of well-being radiated from him.", 0),
            ("He was determined to win.", 0),
            ("The method was determined to be safe.", 1),
            ("I was relieved.", 0),
            ("She was relieved of duty.", 1),
            ("I am much obliged.", 0),
            ("He was obliged to answer.", 1),
            ("I am tied up in important business.", 0),
            ("He was tied up by Alice.", 1),
            ("I can't get mixed up in this.", 0),
            ("The ingredients were mixed by the chef.", 1),
            ("He was bound to get ahead.", 0),
            ("The bundles were bound by the workers.", 1),
            ("Who was it fainted?", 0),
            ("She got dressed before lunch.", 0),
            ("The child was dressed by her mother.", 1),
            ("She was delighted.", 0),
            ("He was astounded.", 0),
        ];
        let failures: Vec<_> = cases
            .into_iter()
            .filter_map(|(text, expected)| {
                let doc = crate::Document::new_plain_english_curated(text);
                let actual = super::Linter::lint(&mut PassiveVoice, &doc).len();
                (actual != expected).then(|| format!("{text:?}: expected {expected}, got {actual}"))
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
    #[test]
    fn highlights_complete_auxiliary_chains() {
        use crate::{Document, linting::Linter};
        for (text, expected) in [
            (
                "The package has been successfully delivered.",
                "has been successfully delivered",
            ),
            (
                "The proposal is being carefully reviewed.",
                "is being carefully reviewed",
            ),
            (
                "The files may already have been deleted.",
                "may already have been deleted",
            ),
            ("The files are getting deleted.", "are getting deleted"),
            (
                "The report was reviewed, approved, and published.",
                "was reviewed, approved, and published",
            ),
            ("Was the report written?", "Was the report written"),
            ("The employee was promoted and left-handed.", "was promoted"),
            (
                "The report was reviewed but later rejected.",
                "was reviewed but later rejected",
            ),
            (
                "The plan was proposed but not adopted.",
                "was proposed but not adopted",
            ),
            (
                "The candidate was interviewed but rejected the offer.",
                "was interviewed",
            ),
            (
                "The candidate was interviewed but rejected it.",
                "was interviewed",
            ),
        ] {
            let doc = Document::new_plain_english_curated(text);
            let lints = PassiveVoice.lint(&doc);
            assert_eq!(lints.len(), 1, "{text}");
            assert_eq!(
                lints[0].span.get_content_string(doc.get_source()),
                expected,
                "{text}"
            );
        }
    }

    #[test]
    fn respects_clause_boundaries() {
        active("She was happy; he written nonsense.");
        active("She was happy: he written nonsense.");
        active("She was happy, he written nonsense.");
        active("She was happy — he written nonsense.");
        passive("The file was deleted, but the folder survived.");
    }
}

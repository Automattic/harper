// SPDX-License-Identifier: Apache-2.0
// Original project: JaredTweed/PassiveVoiceDetector.

use std::borrow::Cow;

use harper_brill::UPOS;

use super::{Lint, LintKind, Linter};
use crate::char_string::CharStringExt;
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
    Coordinated,
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

                if is_license_notice(sentence, &word_indices, word_pos, source)
                    || has_state_complement(sentence, &word_indices, word_pos, source, by_agent)
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
                        has_technical_subject(sentence, &word_indices, word_pos, source),
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
        "Detects likely passive predicates, excluding reduced relatives, licensing declarations, and common state or intention phrases."
    }
}

/// Lexicalized compounds describe attributes, even when a component is a verb
/// ("left-handed"). Do not turn either half into an independent passive.
fn is_descriptive_compound(sentence: &[Token], idx: usize, source: &[char]) -> bool {
    let descriptive_suffix = |token: &Token| {
        normalized_word(token, source).eq_any_ignore_ascii_case_str(&[
            "eyed", "faced", "haired", "handed", "headed", "hearted", "legged", "minded",
            "skinned", "tempered",
        ])
    };
    (idx > 0 && sentence[idx - 1].kind.is_hyphen() && descriptive_suffix(&sentence[idx]))
        || (sentence.get(idx + 1).is_some_and(|t| t.kind.is_hyphen())
            && sentence.get(idx + 2).is_some_and(descriptive_suffix))
}

/// Normalize a token's characters (curly quotes and dashes to ASCII). Callers
/// compare the result through [`CharStringExt`], which is case-insensitive and
/// avoids allocating a lowercased copy of every scanned word.
// Borrow ordinary words; allocate only when Unicode punctuation needs normalization.
// Comparisons below handle case without allocating lowercase strings.
fn normalized_word<'a>(token: &Token, source: &'a [char]) -> Cow<'a, [char]> {
    token.get_ch(source).normalized()
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

/// Recover a shared auxiliary across a preceding agent phrase, but never
/// classify an isolated noun modifier ("software written by Alice") as a lint.
fn preceding_coordinated_passive(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
) -> bool {
    let mut current_pos = pos;
    // Each step crosses just one agent phrase. Bound the number of shared
    // predicates, and require an explicit auxiliary at the head of the chain.
    for _ in 0..4 {
        let candidate = indices[current_pos];
        let Some(earlier) = (0..current_pos)
            .rev()
            .take(12)
            .find(|&earlier| is_participle_candidate(&sentence[indices[earlier]], source))
        else {
            return false;
        };
        let idx = indices[earlier];
        if sentence[idx + 1..candidate].iter().any(|token| {
            (token.kind.is_chunk_terminator() && !token.kind.is_comma())
                || token.kind.is_unlintable()
                || (token.kind.is_punctuation() && !token.kind.is_comma())
        }) {
            return false;
        }
        let tail = &indices[earlier + 1..current_pos];
        let separator = sentence[..candidate]
            .iter()
            .rev()
            .find(|t| !t.kind.is_whitespace());
        if !tail
            .first()
            .is_some_and(|&i| normalized_word(&sentence[i], source).eq_str("by"))
            || !(tail.last().is_some_and(|&i| {
                normalized_word(&sentence[i], source)
                    .eq_any_ignore_ascii_case_str(&["and", "but", "or", "yet"])
            }) || separator.is_some_and(|t| t.kind.is_comma()))
            || tail.iter().any(|&i| {
                let token = &sentence[i];
                token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX)
            })
        {
            return false;
        }
        if preceding_passive_aux(sentence, indices, earlier, source).is_some() {
            return true;
        }
        current_pos = earlier;
    }
    false
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

    // Reduced relatives and standalone participial labels are deliberately
    // outside this style rule's scope: they seldom benefit from active rewrites.
    if by_agent && preceding_coordinated_passive(sentence, word_indices, word_pos, source) {
        return Some((PassiveSource::Coordinated, word_indices[word_pos]));
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
            && !lower.ends_with_ignore_ascii_case_str("n't")
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

        if lower.eq_str("deal")
            && pos >= 2
            && normalized_word(&sentence[word_indices[pos - 2]], source).eq_str("a")
            && normalized_word(&sentence[word_indices[pos - 1]], source)
                .eq_any_ignore_ascii_case_str(&["good", "great"])
            && !has_hard_boundary(&sentence[word_indices[pos - 2] + 1..idx])
        {
            pos -= 2;
            skipped += 3;
            continue;
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
        if skipped > 0 && lower.eq_any_ignore_ascii_case_str(&["and", "or"]) && pos > 0 {
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

        if lower.eq_any_ignore_ascii_case_str(&[
            "after", "although", "as", "because", "before", "if", "since", "to", "unless", "when",
            "where", "whether", "while",
        ]) || (token.kind.is_upos(UPOS::DET) && !saw_subject)
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
        if lower.eq_any_ignore_ascii_case_str(&["that", "which", "who", "whom"]) {
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
        normalized_word(&sentence[idx], source).eq_any_ignore_ascii_case_str(&[
            "how", "what", "when", "where", "which", "who", "whom", "whose", "why",
        ])
    })
}

fn sentence_is_question(sentence: &[Token], source: &[char]) -> bool {
    for token in sentence.iter().rev() {
        if token.kind.is_whitespace() {
            continue;
        }

        let chars = token.get_ch(source);
        let start = chars
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or(chars.len());
        let end = chars
            .iter()
            .rposition(|c| !c.is_whitespace())
            .map_or(start, |end| end + 1);
        let text = &chars[start..end];
        if text.eq_ch(&['"']) || text.eq_ch(&['\'']) || text.eq_ch(&['”']) || text.eq_ch(&['’'])
        {
            continue;
        }

        return text.eq_ch(&['?']);
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
    if !start_word.eq_any_ignore_ascii_case_str(&[
        "be", "become", "becoming", "been", "being", "get", "getting", "got", "gotten", "have",
        "to",
    ]) {
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
        if lower.eq_any_ignore_ascii_case_str(&[
            "after", "although", "because", "before", "if", "since", "unless", "when", "whether",
            "while",
        ]) {
            return None;
        }
        let auxiliary = is_be_form(&lower)
            || is_get_form(&lower)
            || is_become_form(&lower)
            || lower.eq_any_ignore_ascii_case_str(&[
                "can",
                "can't",
                "cannot",
                "could",
                "couldn't",
                "did",
                "do",
                "does",
                "had",
                "hadn't",
                "has",
                "hasn't",
                "have",
                "haven't",
                "may",
                "might",
                "must",
                "mustn't",
                "shall",
                "should",
                "shouldn't",
                "will",
                "won't",
                "would",
                "wouldn't",
            ]);
        let modal_or_do = lower.eq_any_ignore_ascii_case_str(&[
            "can",
            "can't",
            "cannot",
            "could",
            "couldn't",
            "did",
            "do",
            "does",
            "may",
            "might",
            "must",
            "mustn't",
            "shall",
            "should",
            "shouldn't",
            "will",
            "won't",
            "would",
            "wouldn't",
        ]);
        if auxiliary
            && (!start_word.eq_any_ignore_ascii_case_str(&["get", "have"]) || modal_or_do)
            && saw_subject
            && !in_relative
            && is_question_aux_position(sentence, indices, pos, source)
        {
            return Some(idx);
        }
        if lower.eq_any_ignore_ascii_case_str(&["that", "which", "who", "whom"]) {
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
        let going_to = lower.eq_str("going")
            && word_indices
                .get(next_word_pos)
                .is_some_and(|&next_idx| normalized_word(&sentence[next_idx], source).eq_str("to"));
        // Ambiguous words such as "can" and "will" may be tagged as verbs
        // even after an article. Keep that nominal subject out of the span.
        let follows_article = next_word_pos.checked_sub(2).is_some_and(|pos| {
            let article = word_indices[pos];
            !has_hard_boundary(&sentence[article + 1..idx])
                && normalized_word(&sentence[article], source)
                    .eq_any_ignore_ascii_case_str(&["a", "an", "the"])
        });
        if is_contextual_noun(token)
            || follows_article
            || (idx > 0 && sentence[idx - 1].kind.is_hyphen())
        {
            break;
        }
        if is_be_form(&lower)
            || going_to
            || is_unambiguous_be_contraction(&lower)
            || lower.ends_with_ignore_ascii_case_str("'s")
            || lower.ends_with_ignore_ascii_case_str("'d")
            || lower.eq_any_ignore_ascii_case_str(&[
                "can",
                "can't",
                "cannot",
                "could",
                "couldn't",
                "had",
                "hadn't",
                "has",
                "hasn't",
                "have",
                "haven't",
                "having",
                "i've",
                "may",
                "might",
                "must",
                "mustn't",
                "shall",
                "should",
                "shouldn't",
                "they've",
                "to",
                "we've",
                "will",
                "won't",
                "would",
                "wouldn't",
                "you've",
            ])
        {
            start = idx;
        } else if !is_gap_modifier(token, &lower) {
            break;
        }
    }
    let pos = word_indices.partition_point(|&idx| idx < start);
    if pos >= 2
        && normalized_word(&sentence[word_indices[pos - 1]], source).eq_str("or")
        && normalized_word(&sentence[word_indices[pos - 2]], source)
            .iter()
            .map(char::to_ascii_lowercase)
            .eq(normalized_word(&sentence[start], source)
                .iter()
                .map(char::to_ascii_lowercase))
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
        let conjunction = lower.eq_any_ignore_ascii_case_str(&["and", "but", "or", "yet"]);
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
        let contrast_with_object = lower.eq_any_ignore_ascii_case_str(&["but", "yet"])
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
            || is_license_notice(sentence, word_indices, pos, source)
            || has_state_complement(
                sentence,
                word_indices,
                pos,
                source,
                has_agentive_by(sentence, word_indices, pos, source),
            )
            || should_suppress_adjectival(
                token,
                source,
                has_agentive_by(sentence, word_indices, pos, source),
                PassiveSource::Be,
                has_event_evidence(sentence, word_indices, pos, source),
                has_technical_subject(sentence, word_indices, pos, source),
            )
        {
            break;
        }
        end_idx = idx;
        pos += 1;
    }
    end_idx
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
    if lower.eq_any_ignore_ascii_case_str(&[
        "bleed", "breed", "exceed", "feed", "heed", "need", "proceed", "seed", "speed", "succeed",
        "weed",
    ]) {
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
        let root_len = lower.iter().position(|&c| c == '\'').unwrap_or(lower.len());
        let root = &lower[..root_len];
        if root.eq_any_ignore_ascii_case_str(&["he", "i", "she", "they", "we", "you"]) {
            return true;
        }
        if !is_be_form(&lower)
            && !is_get_form(&lower)
            && !is_gap_modifier(token, &lower)
            && !lower.eq_any_ignore_ascii_case_str(&[
                "can", "can't", "cannot", "could", "couldn't", "had", "has", "have", "may",
                "might", "must", "should", "will", "would",
            ])
        {
            break;
        }
    }
    false
}

/// Conventional license declarations are not useful targets for style advice.
/// Keep this local to the declaration, rather than skipping whole documents.
fn is_license_notice(sentence: &[Token], indices: &[usize], pos: usize, source: &[char]) -> bool {
    let word = |offset| {
        indices.get(pos + offset).and_then(|&idx| {
            (!has_hard_boundary(&sentence[indices[pos] + 1..idx]))
                .then(|| normalized_word(&sentence[idx], source))
        })
    };
    let lower = normalized_word(&sentence[indices[pos]], source);
    if lower.eq_str("licensed") && word(1).is_some_and(|w| w.eq_str("under")) {
        return true;
    }
    let under = if lower.eq_any_ignore_ascii_case_str(&["distributed", "released"]) {
        word(1).is_some_and(|w| w.eq_str("under"))
    } else if lower.eq_str("made") {
        word(1).is_some_and(|w| w.eq_str("available")) && word(2).is_some_and(|w| w.eq_str("under"))
    } else if lower.eq_str("governed") {
        word(1).is_some_and(|w| w.eq_str("by"))
    } else {
        false
    };
    under
        && (1..=8).filter_map(word).any(|word| {
            word.eq_any_ignore_ascii_case_str(&[
                "apache", "bsd", "gpl", "licence", "licences", "license", "licenses", "mit",
                "terms",
            ])
        })
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
    let next_is = |w: &str| next.as_ref().is_some_and(|n| n.eq_str(w));
    let next_any = |ws: &[&str]| {
        next.as_ref()
            .is_some_and(|n| n.eq_any_ignore_ascii_case_str(ws))
    };

    if lower.eq_str("bound") && next_is("to") {
        return indices
            .get(pos + 2)
            .is_some_and(|&idx| sentence[idx].kind.is_upos(UPOS::VERB));
    }
    if lower.eq_str("built") && next_is("into") {
        return true;
    }
    if lower.eq_str("composed") && next_is("of") {
        return true;
    }
    if lower.eq_str("damned") && next_is("if") {
        return true;
    }
    if lower.eq_any_ignore_ascii_case_str(&["designed", "expected", "meant", "supposed"])
        && next_is("to")
    {
        return true;
    }
    if lower.eq_str("determined") && next_any(&["not", "to"]) {
        return has_personal_subject(sentence, indices, pos, source);
    }
    if lower.eq_any_ignore_ascii_case_str(&["fed", "grown"]) && next_is("up") {
        return true;
    }
    // Postnominal availability idioms: "no time to be lost" and
    // "not a single pro to be found". Keep ordinary passive infinitives.
    if lower.eq_any_ignore_ascii_case_str(&["found", "lost"]) && pos >= 3 {
        let be = indices[pos - 1];
        let to = indices[pos - 2];
        let noun = indices[pos - 3];
        return normalized_word(&sentence[be], source).eq_str("be")
            && normalized_word(&sentence[to], source).eq_str("to")
            && is_contextual_noun(&sentence[noun])
            && !has_hard_boundary(&sentence[noun + 1..indices[pos]]);
    }
    if lower.eq_str("headed") && next_any(&["for", "to", "toward", "towards"]) {
        return true;
    }
    if lower.eq_str("intended") && next_any(&["for", "to"]) {
        return true;
    }
    if lower.eq_str("left") && next_any(&["alone", "to"]) {
        return true;
    }
    if lower.eq_any_ignore_ascii_case_str(&["located", "situated"]) {
        return !indices[..pos]
            .iter()
            .rev()
            .take(3)
            .any(|&idx| normalized_word(&sentence[idx], source).eq_str("being"));
    }
    if lower.eq_str("made") && next_is("possible") {
        return true;
    }
    if lower.eq_str("mistaken") {
        return !next_is("for");
    }
    if lower.eq_str("mixed") && next_is("up") {
        return indices
            .get(pos + 2)
            .is_some_and(|&idx| normalized_word(&sentence[idx], source).eq_str("in"))
            && has_personal_subject(sentence, indices, pos, source);
    }
    if lower.eq_str("obliged") {
        return !next_is("to");
    }
    if lower.eq_str("pressed") && next_any(&["against", "closely", "hard", "so"]) {
        return true;
    }
    if lower.eq_str("printed") && next_is("the") {
        return indices.get(pos + 2).is_some_and(|&idx| {
            normalized_word(&sentence[idx], source)
                .eq_any_ignore_ascii_case_str(&["name", "title", "word", "words"])
        });
    }
    if lower.eq_str("relieved") {
        return !next_is("of");
    }
    if lower.eq_str("required") && next_is("to") {
        return true;
    }
    if lower.eq_str("seen") && next_is("in") {
        let Some(tail) = indices.get(pos + 2..pos + 5) else {
            return false;
        };
        if has_hard_boundary(&sentence[indices[pos] + 1..tail[2]]) {
            return false;
        }
        return normalized_word(&sentence[tail[0]], source).eq_str("a")
            && normalized_word(&sentence[tail[1]], source).eq_any_ignore_ascii_case_str(&[
                "bad",
                "favorable",
                "favourable",
                "good",
                "negative",
                "positive",
            ])
            && normalized_word(&sentence[tail[2]], source).eq_str("light");
    }
    if lower.eq_str("tied") && next_is("up") {
        return indices.iter().skip(pos + 2).take(6).any(|&idx| {
            normalized_word(&sentence[idx], source)
                .eq_any_ignore_ascii_case_str(&["business", "meeting", "meetings", "work"])
        });
    }
    if lower.eq_str("used") && next_is("up") {
        return true;
    }
    if lower.eq_str("used") && next_is("to") {
        return indices.get(pos + 2).is_some_and(|&idx| {
            let token = &sentence[idx];
            !token.kind.is_upos(UPOS::VERB)
                && !token.kind.is_upos(UPOS::AUX)
                && !is_be_form(&normalized_word(token, source))
        });
    }
    if lower.eq_str("worn") && next_is("out") {
        return true;
    }
    false
}

fn allows_retained_object(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&[
        "asked", "awarded", "denied", "given", "offered", "paid", "promised", "sent", "shown",
        "taught", "told",
    ])
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
        && !next_lower.eq_any_ignore_ascii_case_str(&["today", "tomorrow", "yesterday"])
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
        if lower.eq_any_ignore_ascii_case_str(&[
            "being",
            "deliberately",
            "getting",
            "intentionally",
            "just",
            "newly",
            "recently",
        ]) {
            return true;
        }
        if !is_gap_modifier(&sentence[idx], &lower) && !is_be_form(&lower) {
            break;
        }
    }
    let mut previous: Option<Cow<[char]>> = None;
    for &idx in indices.iter().skip(pos + 1).take(4) {
        if has_hard_boundary(&sentence[candidate + 1..idx]) {
            break;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        if lower.eq_any_ignore_ascii_case_str(&[
            "earlier",
            "every",
            "overnight",
            "recently",
            "today",
            "tomorrow",
            "tonight",
            "yesterday",
        ]) || (previous.as_ref().is_some_and(|p| p.eq_str("on")) && lower.eq_str("election"))
            || (previous
                .as_ref()
                .is_some_and(|p| p.eq_any_ignore_ascii_case_str(&["at", "in", "last", "on"]))
                && is_time_word(&lower))
        {
            return true;
        }
        if token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX) {
            break;
        }
        previous = Some(lower);
    }
    false
}

fn has_clear_done_passive(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
) -> bool {
    if !normalized_word(&sentence[indices[pos]], source).eq_str("done") {
        return false;
    }
    let subject = indices[..pos]
        .iter()
        .rev()
        .take(8)
        .find(|&&idx| is_contextual_noun(&sentence[idx]) || sentence[idx].kind.is_pronoun());
    if subject.is_some_and(|&idx| {
        normalized_word(&sentence[idx], source)
            .eq_any_ignore_ascii_case_str(&["he", "i", "she", "they", "we", "you"])
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
        if lower.eq_str("being") {
            return true;
        }
        if lower.eq_str("been") {
            return before
                .is_some_and(|word| word.eq_any_ignore_ascii_case_str(&["had", "has", "have"]));
        }
        if lower.eq_str("be") {
            return before.is_some_and(|word| {
                word.eq_any_ignore_ascii_case_str(&[
                    "can",
                    "can't",
                    "cannot",
                    "could",
                    "couldn't",
                    "may",
                    "might",
                    "must",
                    "mustn't",
                    "shall",
                    "should",
                    "shouldn't",
                    "to",
                    "will",
                    "won't",
                    "would",
                    "wouldn't",
                ])
            });
        }
        return false;
    }
    false
}

fn should_suppress_adjectival(
    token: &Token,
    source: &[char],
    by_agent: bool,
    passive_source: PassiveSource,
    event_evidence: bool,
    technical_subject: bool,
) -> bool {
    if by_agent {
        return false;
    }

    let lower = normalized_word(token, source);
    // "Get started" commonly means "begin", including tutorial headings.
    if lower.eq_str("started") && passive_source == PassiveSource::Get {
        return true;
    }

    // Perfect aspect and degree modifiers do not make an emotional state
    // eventive: "I've been tired" and "I'm just worried" remain copular.
    if lower.eq_any_ignore_ascii_case_str(&[
        "accustomed",
        "annoyed",
        "appreciated",
        "astounded",
        "bored",
        "concerned",
        "confused",
        "delighted",
        "disappointed",
        "embarrassed",
        "engrossed",
        "excited",
        "flattered",
        "frightened",
        "interested",
        "offended",
        "paralyzed",
        "puzzled",
        "related",
        "satisfied",
        "scared",
        "surprised",
        "terrified",
        "tired",
        "worried",
    ]) {
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

    // Documentation commonly describes configurations, capabilities, and
    // states with agentless passives ("the feature is enabled", "files can be
    // found"). These read as result states rather than dynamic events, so keep
    // them out of the style warning unless an event or agent is present.
    if technical_state_participle(&lower) && technical_subject && !event_evidence {
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

fn is_gap_modifier(token: &Token, lower: &[char]) -> bool {
    token.kind.is_upos(UPOS::ADV)
        || token
            .kind
            .as_word()
            .and_then(|m| m.as_ref())
            .is_some_and(|m| m.pos_tag.is_none() && m.is_adverb())
        || lower.eq_any_ignore_ascii_case_str(&[
            "allegedly",
            "almost",
            "already",
            "also",
            "automatically",
            "carefully",
            "commonly",
            "completely",
            "currently",
            "deliberately",
            "directly",
            "eventually",
            "finally",
            "fully",
            "generally",
            "highly",
            "immediately",
            "intentionally",
            "just",
            "manually",
            "never",
            "newly",
            "not",
            "now",
            "only",
            "originally",
            "partially",
            "partly",
            "previously",
            "recently",
            "reportedly",
            "slightly",
            "still",
            "subsequently",
            "successfully",
            "then",
            "well",
            "widely",
        ])
}

fn is_be_form(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&[
        "am", "are", "aren't", "be", "been", "being", "is", "isn't", "was", "wasn't", "were",
        "weren't",
    ])
}

fn is_unambiguous_be_contraction(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&["i'm", "they're", "we're", "you're"])
}

fn is_get_form(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&["get", "gets", "getting", "got", "gotten"])
}

fn is_become_form(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&["became", "become", "becomes", "becoming"])
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
        if text.eq_str("by") {
            return !by_phrase_is_nonagentive(sentence, idx, source);
        }
        // An adjunct may intervene between the participle and its agent:
        // "guaranteed on any input by a finite state-space". Do not let the
        // search cross a new finite clause. A coordinated participle can
        // share the same agent: "was stunned and exhausted by the speech".
        if text.eq_any_ignore_ascii_case_str(&["and", "but", "or", "yet"]) {
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
            || text.eq_any_ignore_ascii_case_str(&["at", "for", "from", "in", "on", "to", "with"])
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
                is_time_word(&lower) || lower.eq_str("by")
            })
            .unwrap_or(true);
    }

    let mut lower = normalized_word(next, source);
    // Bare communication/transport phrases describe means, not actors. Keep
    // determiner-led phrases available as agents (e.g. "hit by a train").
    if lower.eq_any_ignore_ascii_case_str(&[
        "air",
        "bus",
        "caesarean",
        "candlelight",
        "cesarean",
        "email",
        "fax",
        "phone",
        "plane",
        "sea",
        "train",
    ]) {
        return true;
    }
    // "by <gerund>" almost always names a means or method rather than an actor:
    // "disabled by setting a flag", "found by searching", "ranked by counting".
    // Proper nouns and a few ordinary nouns that merely end in -ing are excluded.
    if lower.ends_with_ignore_ascii_case_str("ing")
        && !next.kind.is_proper_noun()
        && !lower.eq_any_ignore_ascii_case_str(&[
            "evening", "king", "morning", "ring", "spring", "string", "thing",
        ])
    {
        return true;
    }
    if lower.eq_any_ignore_ascii_case_str(&["a", "an", "the"]) {
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
                    is_time_word(&lower) || lower.eq_str("by")
                })
                .unwrap_or(true);
        }
        lower = normalized_word(next, source);
    }

    if lower.eq_any_ignore_ascii_case_str(&[
        "early",
        "following",
        "last",
        "late",
        "next",
        "previous",
        "same",
    ]) {
        return iter
            .take_while(|tok| !tok.kind.is_chunk_terminator())
            .find(|tok| tok.kind.is_word_like())
            .is_some_and(|tok| is_time_word(&normalized_word(tok, source)));
    }
    is_time_word(&lower)
        || lower.eq_any_ignore_ascii_case_str(&[
            "accident",
            "beach",
            "chance",
            "coast",
            "default",
            "definition",
            "design",
            "door",
            "hand",
            "harbor",
            "island",
            "lake",
            "mistake",
            "ocean",
            "river",
            "road",
            "shore",
            "stairs",
            "window",
        ])
}

fn is_time_word(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&[
        "a.m.",
        "afternoon",
        "am",
        "april",
        "august",
        "dawn",
        "day",
        "days",
        "deadline",
        "december",
        "dusk",
        "end",
        "evening",
        "february",
        "friday",
        "hour",
        "hours",
        "january",
        "july",
        "june",
        "march",
        "may",
        "midnight",
        "minute",
        "minutes",
        "monday",
        "month",
        "months",
        "morning",
        "night",
        "noon",
        "november",
        "now",
        "o'clock",
        "october",
        "p.m.",
        "pm",
        "saturday",
        "september",
        "start",
        "sunday",
        "then",
        "thursday",
        "time",
        "today",
        "tomorrow",
        "tuesday",
        "wednesday",
        "week",
        "weeks",
        "year",
        "years",
        "yesterday",
    ])
}

fn passivepy_ambiguous_participle(lower: &[char]) -> bool {
    // Surface forms corresponding to PassivePy's ambiguity lemmas. PassivePy
    // generally requires explicit "by" evidence for these to avoid adjective
    // readings such as "I was exhausted" and "I am involved".
    lower.eq_any_ignore_ascii_case_str(&[
        "associated",
        "based",
        "born",
        "borne",
        "complicated",
        "exhausted",
        "filled",
        "heated",
        "involved",
        "led",
        "overrated",
        "reserved",
        "screwed",
        "stunned",
    ])
}

fn lexicalized_nonpassive_state(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&["done", "drunk", "fainted", "gone"])
}

/// Restrict documentation exceptions to a local technical subject, rather than
/// treating common action verbs as states everywhere ("he was given a medal").
/// Subjectless excerpts cannot supply that context and retain the conservative
/// reading used by the review corpus. Clause boundaries and lexical verbs stop
/// the scan, so a technical noun in a containing clause cannot hide a warning.
fn has_technical_subject(
    sentence: &[Token],
    indices: &[usize],
    pos: usize,
    source: &[char],
) -> bool {
    let candidate = indices[pos];
    for &idx in indices[..pos].iter().rev().take(12) {
        if has_hard_boundary(&sentence[idx + 1..candidate]) {
            break;
        }
        let token = &sentence[idx];
        let lower = normalized_word(token, source);
        if is_be_form(&lower)
            || is_get_form(&lower)
            || is_become_form(&lower)
            || is_unambiguous_be_contraction(&lower)
            || is_gap_modifier(token, &lower)
            || (token.kind.is_auxiliary_verb()
                && !is_contextual_noun(token)
                && !token.kind.is_upos(UPOS::VERB))
        {
            continue;
        }
        if lower.eq_any_ignore_ascii_case_str(&["that", "which"]) {
            continue;
        }
        if token.kind.is_pronoun() {
            return false;
        }
        if is_contextual_noun(token) || token.kind.is_upos(UPOS::VERB) {
            return lower.eq_any_ignore_ascii_case_str(&[
                "api",
                "app",
                "application",
                "applications",
                "apps",
                "binaries",
                "binary",
                "build",
                "builds",
                "code",
                "config",
                "configs",
                "configuration",
                "configurations",
                "crash",
                "crashes",
                "data",
                "database",
                "databases",
                "dependencies",
                "dependency",
                "directories",
                "directory",
                "docker",
                "document",
                "documents",
                "example",
                "examples",
                "exception",
                "exceptions",
                "feature",
                "features",
                "file",
                "files",
                "interface",
                "interfaces",
                "interpreter",
                "interpreters",
                "key",
                "keys",
                "list",
                "lists",
                "makefile",
                "module",
                "modules",
                "network",
                "networks",
                "option",
                "options",
                "package",
                "packages",
                "path",
                "paths",
                "pattern",
                "patterns",
                "process",
                "processes",
                "program",
                "programs",
                "project",
                "projects",
                "protocol",
                "protocols",
                "record",
                "records",
                "release",
                "releases",
                "repo",
                "repos",
                "repositories",
                "repository",
                "resistor",
                "resistors",
                "script",
                "scripts",
                "setting",
                "settings",
                "shell",
                "software",
                "task",
                "tasks",
                "theme",
                "themes",
                "tool",
                "tools",
                "ui",
            ]);
        }
    }
    true
}

/// Participles that overwhelmingly describe technical configuration, capability,
/// or documentation states with technical subjects. Agentive or eventive uses
/// stay in scope, as do action readings outside documentation.
fn technical_state_participle(lower: &[char]) -> bool {
    lower.eq_any_ignore_ascii_case_str(&[
        "allowed",
        "built",
        "configured",
        "connected",
        "defined",
        "deprecated",
        "derived",
        "described",
        "disabled",
        "displayed",
        "documented",
        "enabled",
        "found",
        "generated",
        "given",
        "ignored",
        "included",
        "installed",
        "labeled",
        "labelled",
        "listed",
        "mentioned",
        "modeled",
        "modelled",
        "noted",
        "permitted",
        "pressed",
        "priced",
        "printed",
        "provided",
        "recorded",
        "required",
        "rotated",
        "sandboxed",
        "shared",
        "shown",
        "specified",
        "stopped",
        "stored",
        "summed",
        "supported",
        "tagged",
        "used",
        "weighted",
    ])
}

fn likely_participial_adjective(lower: &[char]) -> bool {
    // Additional high-frequency result/state participles. Unlike the PassivePy
    // ambiguity set above, these require additional event or agent evidence.
    lower.eq_any_ignore_ascii_case_str(&[
        "accustomed",
        "annoyed",
        "bored",
        "broken",
        "closed",
        "concerned",
        "confused",
        "descended",
        "disappointed",
        "divorced",
        "dressed",
        "engaged",
        "excited",
        "finished",
        "fit",
        "forgotten",
        "frightened",
        "haunted",
        "interested",
        "labeled",
        "labelled",
        "left",
        "located",
        "lost",
        "marked",
        "married",
        "ornamented",
        "prepared",
        "priced",
        "ready",
        "related",
        "retired",
        "satisfied",
        "scared",
        "seated",
        "set",
        "settled",
        "shut",
        "situated",
        "stained",
        "terrified",
        "thatched",
        "tired",
        "worried",
    ])
}

fn looks_like_participle_surface(lower: &[char]) -> bool {
    lower.ends_with_ignore_ascii_case_str("ed")
        || lower.eq_any_ignore_ascii_case_str(&[
            "beat",
            "beaten",
            "bitten",
            "born",
            "bought",
            "broken",
            "brought",
            "built",
            "caught",
            "chosen",
            "dealt",
            "done",
            "drawn",
            "driven",
            "drunk",
            "eaten",
            "fallen",
            "felt",
            "forbidden",
            "forgiven",
            "forgotten",
            "found",
            "frozen",
            "given",
            "gone",
            "grown",
            "held",
            "hidden",
            "kept",
            "known",
            "led",
            "left",
            "lost",
            "made",
            "meant",
            "paid",
            "read",
            "ridden",
            "risen",
            "run",
            "said",
            "seen",
            "sent",
            "shaken",
            "shown",
            "sold",
            "spent",
            "spoken",
            "stolen",
            "stuck",
            "taken",
            "taught",
            "thought",
            "thrown",
            "told",
            "understood",
            "woken",
            "won",
            "worn",
            "woven",
            "written",
        ])
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
    #[test]
    fn comparisons_preserve_case_and_unicode_contractions() {
        for text in [
            "The report WAS or Was WRITTEN by Alice.",
            "The report Was or WAS WRITTEN by Alice.",
            "They’re being questioned by police.",
            "THEY'RE being questioned by police.",
        ] {
            passive(text);
        }
        for text in [
            "You’re not supposed to say that.",
            "The alloy IS COMPOSED OF copper.",
        ] {
            active(text);
        }
    }

    use super::PassiveVoice;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    fn passive(text: &str) {
        assert_lint_count(text, PassiveVoice, 1);
    }

    fn active(text: &str) {
        assert_no_lints(text, PassiveVoice);
    }

    #[test]
    fn auxiliary_metadata_does_not_hide_nominal_subjects() {
        passive("The can was given to Alice.");
        passive("The will was given to Alice.");
        active("The feature may have been given a new name.");
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
    fn ignores_reduced_relatives_with_agents() {
        active("Arrested by the police, he never thought this would be his end.");
        active("The report written by Alice was useful.");
        active("Resources exhausted by humans can recover slowly.");
    }

    #[test]
    fn ignores_agentless_reduced_relatives() {
        active("The report written yesterday contains several errors.");
        active("The man seen near the station called police.");
        active("There was no change detected in her behavior.");
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
        active("Reduced by 10 percent, the rate became manageable.");
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
            ("There were people caught stealing.", 0),
            ("She has written the report by hand.", 0),
            ("They had completed the task by noon.", 0),
            ("He walked by the river.", 0),
            ("The vessel sailed by the coast.", 0),
            ("The hikers walked by the beach.", 0),
            ("The committee approved the proposal by a wide margin.", 0),
            ("She read by the window.", 0),
            ("He is tired and walks by the river.", 0),
            ("The exhausted hikers walked by the lake.", 0),
            ("The report written by Alice was published by Bob.", 1),
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
            ("She was exhausted by running.", 0),
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
            ("The issues discussed by the team are complex.", 0),
            ("The report carefully reviewed by Alice was published.", 1),
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
            ("The tool is used to cut paper.", 0),
            ("The things get used up.", 0),
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
            ("There were files detected on the disk.", 0),
            ("The runners run fast.", 0),
            ("Several cars come quickly.", 0),
            ("Reports read well.", 0),
            ("The company run by Alice was successful.", 0),
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
    fn preserves_passives_that_can_be_appropriate_in_context() {
        for text in [
            "The video game industry is being sucked into a crisis.",
            "The project is being actively developed.",
            "He got fired illegally.",
            "Some of the words have got altered.",
            "The trend isn't really driven by consumers.",
        ] {
            passive(text);
        }
    }

    #[test]
    fn suppresses_agentless_technical_states() {
        for text in [
            "The script is stopped and run again.",
            "Patterns get documented.",
            "The list is generated using a script.",
            "The feature can be disabled by setting a flag.",
            "Docker must be installed.",
            "The feature is enabled by default.",
            "The applications given here are examples.",
            "The resistors are connected up.",
            "Unsupported options are ignored.",
            "The process is sandboxed.",
            "The jar was labelled.",
            "The networks are modeled.",
            "The boxes were competitively priced.",
            "Both retrieval paths are weighted and summed.",
            "You're not supposed to say that.",
            "The alloy is composed of copper and tin.",
            "Your donation is always appreciated.",
        ] {
            active(text);
        }
        // An agent or a dated event brings the same verbs back into scope.
        passive("The feature was enabled by the administrator.");
        passive("The document was generated yesterday.");
        passive("The document was generated overnight.");
        passive("The document was generated today.");
        passive("The document will be generated tomorrow.");
        passive("The document will be generated tonight.");
        passive("The award was given by the committee.");
        passive("The label was pressed by the machine.");
    }

    #[test]
    fn retains_action_passives_outside_technical_descriptions() {
        for text in [
            "He was given a medal.",
            "The patient was given a dose of insulin.",
            "The victim was found under the bridge.",
            "The victim was found in the house.",
            "The prisoner was allowed a visitor.",
            "The child was ignored.",
            "The meal was provided.",
            "The car was stopped.",
            "The toy was shared.",
            "The letter was printed.",
            "The project says the victim was found under the bridge.",
            "The document says he was given a medal.",
            "She was a good deal frightened by this very sudden change.",
            "She was a great deal surprised by the news.",
        ] {
            passive(text);
        }
        active("Blizzard was still seen in a positive light.");
        active("The proposal was seen in a negative light.");
        passive("The suspect was seen in a positive light by witnesses.");
        passive("The suspect was seen in a brightly lit room.");
        // Ordinary nominal complements must not be treated as degree phrases.
        active("She was a good deal broker.");
        active("She was happy; a good deal frightened nobody.");
    }

    #[test]
    fn avoids_review_reported_states_and_intention_phrases() {
        for text in [
            "If just one of those is set incorrectly, the program fails.",
            "MockAPI is meant to be a test service.",
            "This tool is intended for use with audio files.",
            "Who is left to save us?",
            "All hope is not lost.",
            "Alice was soon left alone.",
            "There was a table set out under a tree.",
            "The roof was thatched with fur.",
            "The achievement was forgotten.",
            "The work bench was stained where he stood.",
            "I'd be damned if I'd go in; I'd had enough of all of them.",
            "There is an error flagged at this location.",
            "There's not a single pro to be found.",
            "There was not a moment to be lost.",
            "The releases will also be located in this directory.",
        ] {
            active(text);
        }
    }

    #[test]
    fn excludes_licensing_declarations_locally() {
        for text in [
            "This project is licensed under MIT.",
            "The packages are licensed under Apache-2.0.",
            "The project is released under the BSD license.",
            "This software can be made available under the GPL license.",
            "Use of the SDK is governed by the terms of service.",
        ] {
            active(text);
        }
        passive("The prisoner was released under police supervision.");
        passive("The files were made available under the counter.");
        passive("The project is licensed under MIT. The report was reviewed.");
    }

    #[test]
    fn does_not_infer_an_auxiliary_from_chains_of_modifiers() {
        for text in [
            "The report written by Alice and published by Bob arrived.",
            "The file reviewed by Ana, translated by Bo, and posted by Cy is here.",
            "Curated by Numman Ali.",
            "Run by wasmer.",
            "An application written in Rust.",
            "Instant reloading powered by Vite.",
            "He'd written the report by hand.",
            "He’s written the report by hand.",
        ] {
            active(text);
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

use harper_brill::UPOS;

use crate::linting::expr_linter::{Chunk, followed_by_word, preceded_by_word};
use crate::{
    Token,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct MissingTo {
    map: ExprMap<usize>,
}

impl MissingTo {
    fn strict_controller_words() -> WordSet {
        WordSet::new(&[
            "eager",
            "fail",
            "failed",
            "failing",
            "fails",
            "incline",
            "inclined",
            "inclines",
            "inclining",
            "manage",
            "managed",
            "manages",
            "managing",
            "ready",
        ])
    }

    fn permissive_controller_words() -> WordSet {
        WordSet::new(&[
            "aim",
            "aimed",
            "aiming",
            "aims",
            "agree",
            "agreed",
            "agreeing",
            "agrees",
            "arrange",
            "arranged",
            "arranges",
            "arranging",
            "aspire",
            "aspired",
            "aspires",
            "aspiring",
            "attempt",
            "attempted",
            "attempting",
            "attempts",
            "decide",
            "decided",
            "decides",
            "deciding",
            "endeavor",
            "endeavored",
            "endeavoring",
            "endeavors",
            "endeavour",
            "endeavoured",
            "endeavouring",
            "endeavours",
            "eager",
            "expect",
            "expected",
            "expecting",
            "expects",
            "forget",
            "forgot",
            "forgotten",
            "forgetting",
            "forgets",
            "hope",
            "hoped",
            "hopes",
            "hoping",
            "intend",
            "intended",
            "intending",
            "intends",
            "learn",
            "learned",
            "learning",
            "learns",
            "learnt",
            "long",
            "longed",
            "longing",
            "longs",
            "mean",
            "means",
            "meant",
            "need",
            "needed",
            "needing",
            "needs",
            "neglect",
            "neglected",
            "neglecting",
            "neglects",
            "prepare",
            "prepared",
            "prepares",
            "preparing",
            "refuse",
            "refused",
            "refuses",
            "refusing",
            "resolve",
            "resolved",
            "resolves",
            "resolving",
            "struggle",
            "struggled",
            "struggles",
            "struggling",
            "try",
            "tried",
            "trying",
            "tries",
            "want",
            "wanted",
            "wanting",
            "wants",
        ])
    }

    fn previous_word_with_span(source: &[char], start: usize) -> Option<(String, usize)> {
        let mut cursor = start;

        while cursor > 0 && source[cursor - 1].is_whitespace() {
            cursor -= 1;
        }

        if cursor == 0 {
            return None;
        }

        let end = cursor;

        while cursor > 0 {
            let ch = source[cursor - 1];
            if ch.is_alphabetic() || ch == '\'' {
                cursor -= 1;
            } else {
                break;
            }
        }

        if cursor == end {
            return None;
        }

        Some((
            source[cursor..end]
                .iter()
                .collect::<String>()
                .to_ascii_lowercase(),
            cursor,
        ))
    }

    fn previous_word(source: &[char], start: usize) -> Option<String> {
        Self::previous_word_with_span(source, start).map(|(word, _)| word)
    }

    fn previous_non_whitespace_char(source: &[char], start: usize) -> Option<char> {
        let mut cursor = start;

        while cursor > 0 {
            cursor -= 1;
            let ch = source[cursor];
            if !ch.is_whitespace() {
                return Some(ch);
            }
        }

        None
    }

    fn next_non_whitespace_char(source: &[char], start: usize) -> Option<char> {
        let mut cursor = start;

        while cursor < source.len() {
            let ch = source[cursor];
            if !ch.is_whitespace() {
                return Some(ch);
            }
            cursor += 1;
        }

        None
    }

    fn determiner_within_three(source: &[char], controller_span_start: usize) -> bool {
        let mut determiner_scan_cursor = controller_span_start;

        for _ in 0..3 {
            let Some((word, start)) = Self::previous_word_with_span(source, determiner_scan_cursor)
            else {
                break;
            };
            let word = word.as_str();

            if matches!(word, "and" | "or" | "but") {
                determiner_scan_cursor = start;
                continue;
            }

            if matches!(
                word,
                "a" | "an" | "the" | "this" | "that" | "these" | "those"
            ) {
                return true;
            }

            determiner_scan_cursor = start;
        }

        false
    }
}

impl Default for MissingTo {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let strict_pattern = SequenceExpr::with(Self::strict_controller_words())
            .t_ws()
            .then_kind_where(|kind| kind.is_upos(UPOS::VERB));
        map.insert(strict_pattern, 0);

        let permissive_pattern = SequenceExpr::with(Self::permissive_controller_words())
            .t_ws()
            .then_kind_where(|kind| kind.is_verb_lemma());
        map.insert(permissive_pattern, 0);

        Self { map }
    }
}

impl ExprLinter for MissingTo {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.map
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let offending_idx = *self.map.lookup(0, matched_tokens, source)?;
        let controller = &matched_tokens[offending_idx];
        let span = controller.span;

        let controller_text = controller.get_str(source).to_lowercase();
        let controller_text = controller_text.as_str();

        if controller.kind.is_verb()
            && controller.kind.is_adjective()
            && preceded_by_word(context, |tok| tok.kind.is_nominal())
            && followed_by_word(context, |tok| {
                tok.kind.is_auxiliary_verb() || tok.kind.is_adverb()
            })
        {
            return None;
        }

        let is_adjective_controller = matches!(controller_text, "eager" | "inclined" | "ready");

        if controller.kind.is_upos(UPOS::ADJ) && !is_adjective_controller {
            return None;
        }

        if !controller.kind.is_upos(UPOS::VERB) && !is_adjective_controller {
            return None;
        }

        let previous_word_info = Self::previous_word_with_span(source, span.start);
        let previous_word = previous_word_info.as_ref().map(|(word, _)| word.as_str());

        if matches!(
            previous_word,
            Some("a" | "an" | "the" | "this" | "that" | "these" | "those")
                | Some("very" | "so" | "too" | "quite" | "rather")
        ) {
            return None;
        }

        let controller_text_ends_with_d_or_en =
            controller_text.ends_with('d') || controller_text.ends_with("en");

        if previous_word == Some("of") && controller_text_ends_with_d_or_en {
            return None;
        }

        if previous_word.is_some_and(|word| word.ends_with("ly"))
            && controller_text_ends_with_d_or_en
        {
            return None;
        }

        if controller_text.starts_with("hope") && previous_word == Some("of") {
            return None;
        }

        if controller_text == "needs" && previous_word == Some("must") {
            return None;
        }

        let prev_non_whitespace_char = Self::previous_non_whitespace_char(source, span.start);

        if controller_text == "prepare"
            && matches!(prev_non_whitespace_char, None | Some('.' | '!' | '?'))
        {
            return None;
        }

        let next_token = matched_tokens
            .iter()
            .skip(offending_idx + 1)
            .find(|tok| !tok.kind.is_whitespace())?;

        let next_text = next_token.get_str(source).to_lowercase();

        if controller_text.starts_with("try") && next_text == "and" {
            return None;
        }

        if next_text.ends_with("ing") {
            return None;
        }

        // Ugly workaround since `Option::flatten` doesn't work with `Option<&Option<...>>`.
        let next_upos = next_token
            .kind
            .as_word()
            .and_then(Option::as_ref)
            .and_then(|word| word.pos_tag);

        let next_is_verb = next_upos == Some(UPOS::VERB);
        let next_is_noun = matches!(next_upos, Some(UPOS::NOUN | UPOS::PROPN | UPOS::ADJ));

        let determiner_within_three = Self::determiner_within_three(source, span.start);

        if next_token.kind.is_np_member()
            && !next_is_verb
            && (previous_word == Some("to") || determiner_within_three)
        {
            return None;
        }

        if !next_is_verb
            && matches!(
                next_upos,
                Some(UPOS::ADV | UPOS::ADJ | UPOS::ADP | UPOS::SCONJ | UPOS::CCONJ)
            )
        {
            return None;
        }

        let next_is_noun_but_not_verb = next_is_noun && !next_is_verb;

        if matches!(
            controller_text,
            "learn" | "learned" | "learning" | "learns" | "learnt" | "mean" | "means" | "meant"
        ) && next_is_noun_but_not_verb
        {
            return None;
        }

        if matches!(controller_text, "hope" | "hoped" | "hopes" | "hoping")
            && (next_is_noun_but_not_verb || next_upos == Some(UPOS::AUX))
        {
            return None;
        }

        let next_non_whitespace_char = Self::next_non_whitespace_char(source, next_token.span.end);

        if matches!(controller_text, "need" | "needed" | "needing" | "needs")
            && (next_is_noun_but_not_verb
                || next_text == "help"
                || next_non_whitespace_char == Some('-'))
        {
            return None;
        }

        if next_upos == Some(UPOS::PROPN)
            && matches!(
                prev_non_whitespace_char,
                Some('"' | '\'' | '”' | '’' | '!' | '?' | ',')
            )
        {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::InsertAfter(" to".chars().collect())],
            message: "Insert `to` to complete the infinitive (e.g., `need to talk`).".to_string(),
            priority: 62,
        })
    }

    fn description(&self) -> &str {
        "Flags verbs and adjectives like `need`, `want`, or `ready` that are missing `to` before an infinitive."
    }
}

#[cfg(test)]
mod tests {
    use super::MissingTo;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn inserts_to_after_meant() {
        assert_suggestion_result(
            "I meant call you last night.",
            MissingTo::default(),
            "I meant to call you last night.",
        );
    }

    #[test]
    fn inserts_to_after_wants() {
        assert_suggestion_result(
            "She wants finish early.",
            MissingTo::default(),
            "She wants to finish early.",
        );
    }

    #[test]
    fn inserts_to_after_need() {
        assert_suggestion_result(
            "We need talk about pricing.",
            MissingTo::default(),
            "We need to talk about pricing.",
        );
    }

    #[test]
    fn inserts_to_after_agreed() {
        assert_suggestion_result(
            "They agreed meet at dawn.",
            MissingTo::default(),
            "They agreed to meet at dawn.",
        );
    }

    #[test]
    fn inserts_to_after_forgot() {
        assert_suggestion_result(
            "He forgot send the file.",
            MissingTo::default(),
            "He forgot to send the file.",
        );
    }

    #[test]
    fn inserts_to_after_trying() {
        assert_suggestion_result(
            "I'm trying get better at chess.",
            MissingTo::default(),
            "I'm trying to get better at chess.",
        );
    }

    #[test]
    fn inserts_to_after_refused() {
        assert_suggestion_result(
            "She refused answer the question.",
            MissingTo::default(),
            "She refused to answer the question.",
        );
    }

    #[test]
    fn inserts_to_after_ready() {
        assert_suggestion_result(
            "We're ready start the meeting.",
            MissingTo::default(),
            "We're ready to start the meeting.",
        );
    }

    #[test]
    fn inserts_to_after_eager() {
        assert_suggestion_result(
            "I'm eager see the results.",
            MissingTo::default(),
            "I'm eager to see the results.",
        );
    }

    #[test]
    fn inserts_to_after_inclined() {
        assert_suggestion_result(
            "I'm inclined believe you.",
            MissingTo::default(),
            "I'm inclined to believe you.",
        );
    }

    #[test]
    fn inserts_to_after_resolved() {
        assert_suggestion_result(
            "She resolved solve the case.",
            MissingTo::default(),
            "She resolved to solve the case.",
        );
    }

    #[test]
    fn no_lint_when_to_present() {
        assert_lint_count("She wants to finish early.", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_with_noun_after_controller() {
        assert_lint_count("They arranged a meeting at noon.", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_needs_follow_up_appointments() {
        assert_lint_count(
            "Gus is recovering well, though he needs follow-up appointments.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_delays_meant_decisions() {
        assert_lint_count(
            "The delays meant decisions were often made on outdated information, hindering agility and potentially impacting return on investment.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_reduced_relative_clause_after_participle() {
        assert_lint_count(
            "The techniques learned would probably not change much with resolution so 480i would seem almost as usable for educational use as 8K video.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_bouquet_of_roses() {
        assert_lint_count(
            "I made a note to request a small bouquet of roses for his room, a simple gesture that I hoped would bring a moment of solace.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_for_intended_word_phrase() {
        assert_lint_count(
            "Detects incorrect usage of `peak` when the intended word is `pique`.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_long_passage() {
        assert_lint_count(
            "Before her was another long passage illuminated by lamps.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_long_island_sound() {
        assert_lint_count(
            "The sailboat drifted along Long Island Sound at sunrise.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_learn_tag_probabilities() {
        assert_lint_count(
            "These models learn tag probabilities from annotated corpora.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_standard_feature_nominal_phrase() {
        assert_lint_count(
            "This is a standard and expected feature for any e-commerce site selling visually-driven products.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_mixing_bowl_nominal_phrase() {
        assert_lint_count(
            "This is a 2-quart mixing bowl, ideal for everything from whipping cream to preparing cake batter.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_try_and_say() {
        assert_lint_count(
            "I'll try and say hello before I leave.",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_failed_edit_attempts() {
        assert_lint_count("failed edit attempts", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_ready_work() {
        assert_lint_count("ready work", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_bad_at_managing_side_effects() {
        assert_lint_count("Bad at managing side-effects", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_fully_resolved_conflict() {
        assert_lint_count("a fully resolved conflict", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_resolved_configuration() {
        assert_lint_count("A resolved configuration", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_fully_resolved_configuration() {
        assert_lint_count("A fully resolved configuration", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_resolved_set_of_configuration() {
        assert_lint_count("A resolved set of configuration", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_fully_resolved_set_of_configuration() {
        assert_lint_count(
            "A fully resolved set of configuration",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_system_produced_a_fully_resolved_set_of_dependencies() {
        assert_lint_count(
            "System produced a fully resolved set of dependencies",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_a_resolved_list_of_parameters() {
        assert_lint_count("A resolved list of parameters", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_fully_resolved_list_of_parameters() {
        assert_lint_count(
            "A fully resolved list of parameters",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_a_prepared_stranger() {
        assert_lint_count("A prepared stranger", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_fully_prepared_stranger() {
        assert_lint_count("A fully prepared stranger", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_prepared_group_of_strangers() {
        assert_lint_count("A prepared group of strangers", MissingTo::default(), 0);
    }

    #[test]
    fn no_lint_a_fully_prepared_group_of_strangers() {
        assert_lint_count(
            "A fully prepared group of strangers",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_a_nicely_arranged_set_of_flowers() {
        assert_lint_count(
            "A nicely arranged bunch of flowers",
            MissingTo::default(),
            0,
        );
    }

    #[test]
    fn no_lint_a_recently_forgotten_list_of_names() {
        assert_lint_count(
            "A recently forgotten list of names",
            MissingTo::default(),
            0,
        );
    }
}

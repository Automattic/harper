use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_index_matching},
    },
    patterns::WordSet,
};

const COMPOSE: &[&str] = &["compose", "composed", "composes", "composing"];
const COMPOST: &[&str] = &["compost", "composted", "composting", "composts"];

const COMPOST_CONTEXT: &[&str] = &[
    "aerate",
    "aerating",
    "aeration",
    "amendment",
    "bark",
    "bin",
    "bins",
    "bioactivator",
    "biochar",
    "biodegradable",
    "bokashi",
    "cardboard",
    "compostable",
    "decompose",
    "decomposing",
    "decomposition",
    "fertilizer",
    "food",
    "garden",
    "gardener",
    "gardeners",
    "gardening",
    "heap",
    "heaps",
    "herbicide",
    "herbicides",
    "landfill",
    "landfills",
    "leaf",
    "leaves",
    "manure",
    "microbe",
    "microbes",
    "microbial",
    "mulch",
    "organic",
    "peat",
    "peel",
    "peels",
    "pile",
    "piles",
    "rot",
    "rotting",
    "scrap",
    "scraps",
    "sewage",
    "sludge",
    "soil",
    "straw",
    "toilet",
    "toilets",
    "tumbler",
    "tumblers",
    "vermicompost",
    "waste",
    "wastes",
    "weed",
    "weeds",
    "worm",
    "worms",
];

const COMPOSE_CONTEXT: &[&str] = &[
    "album",
    "albums",
    "app",
    "apps",
    "arrangement",
    "article",
    "articles",
    "chord",
    "chords",
    "component",
    "components",
    "container",
    "containers",
    "docker",
    "email",
    "emails",
    "essay",
    "essays",
    "frame",
    "framing",
    "function",
    "functions",
    "harmony",
    "image",
    "images",
    "jetpack",
    "kotlin",
    "layout",
    "layouts",
    "letter",
    "letters",
    "lyrical",
    "lyrics",
    "melody",
    "melodies",
    "message",
    "messages",
    "music",
    "musical",
    "musician",
    "note",
    "notes",
    "orchestra",
    "orchestrate",
    "paragraph",
    "paragraphs",
    "photo",
    "photograph",
    "photography",
    "piano",
    "piece",
    "poem",
    "poems",
    "poetry",
    "prose",
    "render",
    "rendering",
    "screen",
    "screens",
    "sentence",
    "sentences",
    "shot",
    "shots",
    "song",
    "songs",
    "sonnet",
    "symphony",
    "text",
    "theme",
    "theming",
    "transparent",
    "tune",
    "tunes",
    "ui",
    "verse",
    "verses",
    "view",
    "views",
    "window",
    "windows",
];

pub struct ComposeCompost {
    expr: FirstMatchOf,
}

impl Default for ComposeCompost {
    fn default() -> Self {
        let compose_compost = WordSet::new(COMPOSE.iter().chain(COMPOST.iter()));

        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::with(compose_compost.clone())
                        .t_ws()
                        .then_any_word(),
                ),
                Box::new(SequenceExpr::any_word().t_ws().then(compose_compost)),
            ]),
        }
    }
}

enum Verb {
    Compose,
    Compost,
}
use Verb::*;

enum V {
    Root,
    Ed,
    Ing,
    S,
}

impl ExprLinter for ComposeCompost {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // verb index will be either 0 or 1 - we need context on at least one side for an error to even be possible
        let vi = find_the_only_token_index_matching(toks, src, |t, s| {
            t.get_ch(s).eq_any_ignore_ascii_case_str(COMPOSE)
                || t.get_ch(s).eq_any_ignore_ascii_case_str(COMPOST)
        })?;
        let vch = toks[vi].get_ch(src);

        let vk = if vch.iter().any(|&c| matches!(c, 't' | 'T')) {
            Compost
        } else {
            Compose
        };

        // Now we examine the context to see if this verb is likely to be the wrong one
        let (before, after) = ctx?;

        // Reconstruct the full left and right sides around `toks[vi]`
        // because `FirstMatchOf` consumed one neighbor word into `toks`.
        let (left_inner, right_inner) = (&toks[..vi], &toks[vi + 1..]);
        let all_before = before.iter().chain(left_inner.iter());
        let all_after = right_inner.iter().chain(after.iter());

        // 1. Skip mid-chunk capitalized proper nouns (e.g., "Jetpack Compose", "Docker Compose")
        let is_cap = vch.first().is_some_and(|c| c.is_uppercase());
        let has_prev_word = before
            .iter()
            .chain(left_inner.iter())
            .any(|t| t.kind.is_word());
        if is_cap && has_prev_word {
            return None;
        }

        // 2. Protect "composed of" / "composes of" (e.g., "solid waste composed of food waste")
        let next_is_of = right_inner
            .iter()
            .chain(after.iter())
            .find(|t| t.kind.is_word())
            .is_some_and(|t| t.get_ch(src).eq_str("of"));
        if next_is_of {
            return None;
        }

        // 3. Tally domain cues across the entire chunk (excluding the target verb itself)
        let mut compost_hits = 0usize;
        let mut compose_hits = 0usize;

        for t in all_before.chain(all_after) {
            if !t.kind.is_word() {
                continue;
            }
            let w = t.get_ch(src);
            if w.eq_any_ignore_ascii_case_str(COMPOST_CONTEXT) {
                compost_hits += 1;
            }
            if w.eq_any_ignore_ascii_case_str(COMPOSE_CONTEXT) {
                compose_hits += 1;
            }
        }

        let is_wrong = match vk {
            Compose => compost_hits > 0 && compose_hits == 0,
            Compost => compose_hits > 0 && compost_hits == 0,
        };

        if !is_wrong {
            return None;
        }

        let vf = match vch.last()? {
            'd' | 'D' => V::Ed,
            'g' | 'G' => V::Ing,
            's' | 'S' => V::S,
            _ => V::Root,
        };

        let other = match (vk, vf) {
            (Compose, V::Root) => "compost",
            (Compost, V::Root) => "compose",
            (Compose, V::Ed) => "composted",
            (Compost, V::Ed) => "composed",
            (Compose, V::Ing) => "composting",
            (Compost, V::Ing) => "composing",
            (Compose, V::S) => "composts",
            (Compost, V::S) => "composes",
        };

        let span = toks[vi].span;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            other,
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Are you perhaps confusing `compose` (layout) and `compost` (organic matter)?"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects mistakes mixing up `compose` and `compost`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ComposeCompost;

    #[test]
    fn fix_with_windows() {
        assert_suggestion_result(
            "Composting with transparent windows",
            ComposeCompost::default(),
            "Composing with transparent windows",
        );
    }
}

use std::collections::VecDeque;

use harper_core::{CharStringExt, Mask, Span};

#[derive(Debug, Default)]
pub struct Masker {}

impl harper_core::Masker for Masker {
    fn create_mask(&self, source: &[char]) -> Mask {
        let mut cursor = 0;
        let mut mask = Mask::new_blank();

        let mut cur_mask_start = 0;
        let mut actions = VecDeque::new();

        loop {
            if cursor >= source.len() {
                break;
            }

            let c = source[cursor];

            if let Some(ws) = whitespace_before_comment_at_cursor(cursor, source) {
                actions.push_back(CursorAction::PushMaskAndIncBy(ws));
            } else if matches!(c, '%') {
                actions.push_back(CursorAction::PushMaskAndIncBy(1));
            } else if let Some(ws) = newline_whitespace_at_cursor(cursor, source) {
                actions.push_back(CursorAction::PushMaskAndIncBy(ws));
            } else if let Some(s) = display_math_at_cursor(cursor, source) {
                push_mask_math_region(cursor, cursor + s, source, &mut actions);
            } else if let Some(s) = inline_math_at_cursor(cursor, source) {
                push_mask_math_region(cursor, cursor + s, source, &mut actions);
            } else if let Some(s) = escaped_delim_math(cursor, source, '(', ')') {
                push_mask_math_region(cursor, cursor + s, source, &mut actions);
            } else if let Some(s) = escaped_delim_math(cursor, source, '[', ']') {
                push_mask_math_region(cursor, cursor + s, source, &mut actions);
            } else if let Some(s) = equation_at_cursor(cursor, source) {
                push_mask_math_region(cursor, cursor + s, source, &mut actions);
            } else if let Some(s) = verbatim_env_at_cursor(cursor, source) {
                actions.push_back(CursorAction::PushMaskAndIncBy(s));
            } else if let Some(s) = verb_command_at_cursor(cursor, source) {
                actions.push_back(CursorAction::PushMaskAndIncBy(s));
            } else if !command_at_cursor(cursor, source, &mut actions) {
                actions.push_back(CursorAction::IncBy(1));
            }

            while let Some(action) = actions.pop_front() {
                match action {
                    CursorAction::IncBy(n) => cursor = (cursor + n).min(source.len()),
                    CursorAction::PushMaskAndIncBy(mut n) => {
                        if cur_mask_start != cursor {
                            mask.push_allowed(Span::new(cur_mask_start, cursor));
                        }

                        n = (cursor + n).min(source.len());

                        cursor = n;
                        cur_mask_start = n;
                    }
                }
            }
        }

        if cur_mask_start != cursor {
            mask.push_allowed(Span::new(cur_mask_start, cursor));
        }

        mask
    }
}

/// If the cursor is at horizontal whitespace immediately followed by `%` on the same line,
/// return the length of the whitespace run so it can be masked along with the comment.
fn whitespace_before_comment_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    let c = *source.get(cursor)?;
    if c == '\n' || !c.is_whitespace() {
        return None;
    }
    let ws_len = source[cursor..]
        .iter()
        .take_while(|&&ch| ch.is_whitespace() && ch != '\n')
        .count();
    if source.get(cursor + ws_len) == Some(&'%') {
        Some(ws_len)
    } else {
        None
    }
}

/// If the cursor is at a newline, mask the newline and any following whitespace (indentation).
/// This prevents indentation at the start of lines from being grammar-checked.
fn newline_whitespace_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    if source.get(cursor) != Some(&'\n') {
        return None;
    }
    let ws_len = source[cursor..]
        .iter()
        .take_while(|&&c| c.is_whitespace())
        .count();
    // Only mask if there is actual indentation after the newline, not a bare line break.
    // A bare \n between sentences should remain visible to preserve sentence boundaries.
    if ws_len > 1 { Some(ws_len) } else { None }
}

/// Display math `$$ ... $$`. Only a contiguous, unescaped `$$` closes.
/// A single `$` inside does NOT close. Unclosed runs mask to EOF.
fn display_math_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    if source.get(cursor) != Some(&'$') || source.get(cursor + 1) != Some(&'$') {
        return None;
    }

    let mut i = cursor + 2;
    while i + 1 < source.len() {
        if source[i] == '\\' {
            i += 2;
            continue;
        }
        if source[i] == '$' && source[i + 1] == '$' {
            return Some(i + 2 - cursor);
        }
        i += 1;
    }

    Some(source.len() - cursor)
}

/// Inline math `$ ... $`. Closer is the next unescaped `$`, even if it is
/// part of a `$$` run (TeX `$`-closes-`$`). Unclosed runs mask to EOF.
fn inline_math_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    if source.get(cursor) != Some(&'$') {
        return None;
    }
    // Display takes precedence; callers check it first, but guard anyway.
    if source.get(cursor + 1) == Some(&'$') {
        return None;
    }

    let mut i = cursor + 1;
    while i < source.len() {
        if source[i] == '\\' {
            i += 2;
            continue;
        }
        if source[i] == '$' {
            return Some(i + 1 - cursor);
        }
        i += 1;
    }

    Some(source.len() - cursor)
}

/// Math with an escaped delimiter pair: `\( ... \)` inline or `\[ ... \]`
/// display. Unclosed runs mask to EOF.
fn escaped_delim_math(cursor: usize, source: &[char], open: char, close: char) -> Option<usize> {
    if source.get(cursor) != Some(&'\\') || source.get(cursor + 1) != Some(&open) {
        return None;
    }

    let mut i = cursor + 2;
    while i + 1 < source.len() {
        if source[i] == '\\' {
            if source[i + 1] == close {
                return Some(i + 2 - cursor);
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    Some(source.len() - cursor)
}

/// Mask `\verb` and `\lstinline` constructs wholly (delimiters, options and
/// content). Returns the masked length, or `None` when the cursor is not at
/// such a construct (the generic command path then handles it as before).
fn verb_command_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    let rest = source.get(cursor..)?;
    if rest.first() != Some(&'\\') {
        return None;
    }

    // Read the maximal ASCII-alphabetic command name.
    let name_len = rest
        .iter()
        .skip(1)
        .take_while(|t| t.is_ascii_alphabetic())
        .count();
    let name: &[char] = rest.get(1..1 + name_len)?;
    let mut i = 1 + name_len;

    let is_verb = name.iter().copied().eq("verb".chars());
    let is_lstinline = name.iter().copied().eq("lstinline".chars());
    if !is_verb && !is_lstinline {
        return None;
    }

    // `\verb*` renders spaces explicitly; the `*` is part of the command.
    if is_verb && rest.get(i) == Some(&'*') {
        i += 1;
    }

    // `\lstinline[opts]...`: skip the options like the generic path does.
    if is_lstinline && rest.get(i) == Some(&'[') {
        let opts_len = rest.iter().skip(i + 1).position(|t| *t == ']')?;
        i += opts_len + 2;
    }

    let delim = *rest.get(i)?;
    if delim == '{' {
        // Brace form: content runs to the first `}` (braces cannot nest in
        // verbatim arguments). Unclosed runs mask to EOF.
        let end = rest.iter().skip(i + 1).position(|t| *t == '}');
        return Some(match end {
            Some(len) => i + 1 + len + 1,
            None => rest.len(),
        });
    }
    if delim.is_ascii_alphanumeric() || delim.is_whitespace() || delim.is_control() {
        // Not a delimiter (e.g. `\verb` followed by prose): leave this to the
        // generic command path so existing behavior is preserved.
        return None;
    }

    // Delimiter form: backslash is literal inside, so the next identical
    // character closes. Unclosed runs mask to EOF.
    let end = rest.iter().skip(i + 1).position(|t| *t == delim);
    Some(match end {
        Some(len) => i + 1 + len + 1,
        None => rest.len(),
    })
}

/// Mask the math region `[region_start, region_end)`, except that `\text{...}`
/// interiors stay visible: `\text` is prose even inside math (issue #4481,
/// goal 4). Nested braces are counted and `\{`/`\}` do not affect depth.
/// A `\text{` without a closing brace masks through the region end.
fn push_mask_math_region(
    region_start: usize,
    region_end: usize,
    source: &[char],
    actions: &mut VecDeque<CursorAction>,
) {
    let mut pos = region_start;
    while pos < region_end {
        let Some((cmd_start, brace_open)) = find_text_open(source, pos, region_end) else {
            actions.push_back(CursorAction::PushMaskAndIncBy(region_end - pos));
            return;
        };

        if cmd_start > pos {
            actions.push_back(CursorAction::PushMaskAndIncBy(cmd_start - pos));
        }
        // Mask `\text`, any whitespace, and the opening brace.
        actions.push_back(CursorAction::PushMaskAndIncBy(brace_open + 1 - cmd_start));

        match find_matching_close(source, brace_open + 1, region_end) {
            Some(close) => {
                actions.push_back(CursorAction::IncBy(close - (brace_open + 1)));
                actions.push_back(CursorAction::PushMaskAndIncBy(1));
                pos = close + 1;
            }
            None => {
                actions.push_back(CursorAction::PushMaskAndIncBy(
                    region_end - (brace_open + 1),
                ));
                return;
            }
        }
    }
}

/// Find `\text` (+ optional whitespace + `{`) in `source[start..end)`.
/// Returns the command start and the brace index. A trailing-letter check
/// keeps e.g. `\textarea` from matching, and backslash skipping keeps
/// `\\text` (newline command + prose) from matching.
fn find_text_open(source: &[char], start: usize, end: usize) -> Option<(usize, usize)> {
    let mut i = start;
    while i < end {
        if source[i] == '\\'
            && source.get(i + 1) == Some(&'t')
            && source.get(i + 2) == Some(&'e')
            && source.get(i + 3) == Some(&'x')
            && source.get(i + 4) == Some(&'t')
            && !source.get(i + 5).is_some_and(|c| c.is_ascii_alphabetic())
        {
            let mut j = i + 5;
            while j < end && (source[j] == ' ' || source[j] == '\t' || source[j] == '\n') {
                j += 1;
            }
            if j < end && source[j] == '{' {
                return Some((i, j));
            }
        }
        // Skip the escapee so `\\text` (newline + prose) never matches.
        i += if source[i] == '\\' { 2 } else { 1 };
    }
    None
}

/// Find the `}` closing the brace opened before `start`, counting nested
/// braces and treating `\{`/`\}` as literals. Searches `source[start..end)`.
fn find_matching_close(source: &[char], start: usize, end: usize) -> Option<usize> {
    let mut depth = 1;
    let mut i = start;
    while i < end {
        if source[i] == '\\' {
            i += 2;
            continue;
        }
        if source[i] == '{' {
            depth += 1;
        } else if source[i] == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Check whether there is a command at the current cursor. If so, this function will update the action queue to mask out the hidden elements.
/// Returns whether the action queue was modified.
fn command_at_cursor(cursor: usize, source: &[char], actions: &mut VecDeque<CursorAction>) -> bool {
    let Some(CommandComponents {
        name,
        square_content,
        curly_content,
    }) = deconstruct_command(&source[cursor..])
    else {
        return false;
    };

    let content_commands = [
        "section",
        "title",
        "subsection",
        "subsubsection",
        "textbf",
        "textit",
        "emph",
        "author",
        "part",
        "chapter",
        "caption",
    ];
    let is_content_command = content_commands
        .iter()
        .any(|c| name.iter().copied().eq(c.chars()));

    let diff = 1 + name.len() + square_content.map(|c| c.len() + 2).unwrap_or_default();

    if let Some(curly_content) = curly_content {
        if is_content_command {
            actions.push_back(CursorAction::PushMaskAndIncBy(diff + 1));
            actions.push_back(CursorAction::IncBy(curly_content.len()));
            actions.push_back(CursorAction::PushMaskAndIncBy(1));
            true
        } else {
            actions.push_back(CursorAction::PushMaskAndIncBy(
                curly_content.len() + diff + 1,
            ));
            true
        }
    } else {
        actions.push_back(CursorAction::PushMaskAndIncBy(diff));
        true
    }
}

fn is_math_env(env: &[char]) -> bool {
    let math_envs = [
        "equation",
        "equation*",
        "align",
        "align*",
        "gather",
        "gather*",
        "multline",
        "multline*",
        "flalign",
        "flalign*",
        "alignat",
        "alignat*",
        "eqnarray",
        "eqnarray*",
        "IEEEeqnarray",
        "IEEEeqnarray*",
        "subequations",
        "subequations*",
        "math",
        "displaymath",
    ];
    // NOTE: `eq_str` requires a lowercase-ASCII right-hand side, so (like the
    // content-command check above) this compares case-exactly via `eq`.
    math_envs.iter().any(|e| env.iter().copied().eq(e.chars()))
}

fn equation_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    environment_at_cursor(cursor, source, is_math_env)
}

/// Mask `\begin{<verbatim env>} ... \end{<verbatim env>}` wholly, including
/// any `%`, `$` or commands inside (verbatim content is literal).
/// Unclosed environments mask to EOF.
fn verbatim_env_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    environment_at_cursor(cursor, source, is_verbatim_env)
}

fn is_verbatim_env(env: &[char]) -> bool {
    let verbatim_envs = ["verbatim", "verbatim*", "lstlisting"];
    verbatim_envs
        .iter()
        .any(|e| env.iter().copied().eq(e.chars()))
}

/// Scan a `\begin{env} ... \end{env}` region for environments matching
/// `is_target`. Returns the masked length (through the matching `\end`, or
/// to EOF when unclosed).
fn environment_at_cursor(
    cursor: usize,
    source: &[char],
    is_target: impl Fn(&[char]) -> bool,
) -> Option<usize> {
    let CommandComponents {
        name,
        square_content,
        curly_content,
    } = deconstruct_command(&source[cursor..])?;

    if name.eq_str("begin") && curly_content.is_some_and(is_target) {
        let env_content = curly_content.unwrap();
        let mut diff = 1
            + name.len()
            + env_content.len()
            + square_content.map(|sc| sc.len()).unwrap_or_default();

        loop {
            if cursor + diff >= source.len() {
                return Some(source.len() - cursor);
            }

            if let Some(CommandComponents {
                name,
                curly_content,
                ..
            }) = deconstruct_command(&source[cursor + diff..])
                && name.eq_str("end")
                && curly_content.is_some_and(|cc| cc == env_content)
            {
                break;
            }

            diff += 1;
        }

        Some(diff)
    } else {
        None
    }
}

struct CommandComponents<'a> {
    /// The command's name.
    pub name: &'a [char],
    /// The content of the command's square bracket arguments.
    pub square_content: Option<&'a [char]>,
    /// The content of the command's curly bracket arguments.
    pub curly_content: Option<&'a [char]>,
}

/// Deconstruct a command into its constituent components.
/// Assumes the command is at the beginning of the slice.
/// Returns `None` if not command is present at the expected position.
fn deconstruct_command<'a>(source: &'a [char]) -> Option<CommandComponents<'a>> {
    let mut cursor = 0;

    if source.get(cursor) != Some(&'\\') {
        return None;
    }

    cursor += 1;

    // The name of the command. A command requires at least one character after the
    // leading backslash; otherwise a trailing `\` is malformed rather than a
    // command.
    source.get(cursor)?;
    // In (La)TeX, `\` + non-letter is a single-char command (e.g. `\(`, `\$`).
    // It must NOT swallow following letters into the name.
    // TeX control-word letters are ASCII, so a non-ASCII char also ends the name.
    let first = source[cursor];
    if !first.is_ascii_alphabetic() {
        let name = source.get(cursor..cursor + 1)?;
        cursor += 1;

        // Single-char commands take no arguments, except `\\` which takes an
        // optional `[..]` length (e.g. `\\[2pt]`). Do not swallow `[..]`/`{..}`
        // after e.g. `\,`, `\&`.
        let square_content = if first == '\\' && source.get(cursor) == Some(&'[') {
            cursor += 1;

            let brace_len = source.iter().skip(cursor).position(|t| *t == ']')?;
            let content = source.get(cursor..cursor + brace_len)?;

            //cursor += brace_len + 1;
            Some(content)
        } else {
            None
        };

        return Some(CommandComponents {
            name,
            square_content,
            curly_content: None,
        });
    }
    let name_len = source
        .iter()
        .skip(cursor + 1)
        .take_while(|t| t.is_ascii_alphabetic())
        .count();
    let name_end = cursor + 1 + name_len;
    let name = source.get(cursor..name_end)?;

    cursor = name_end;

    // The optional square braces
    let square_content = if source.get(cursor) == Some(&'[') {
        cursor += 1;

        let brace_len = source.iter().skip(cursor).position(|t| *t == ']')?;
        let content = source.get(cursor..cursor + brace_len)?;

        cursor += brace_len + 1;
        Some(content)
    } else {
        None
    };

    // The optional curly braces
    let curly_content = if source.get(cursor) == Some(&'{') {
        cursor += 1;

        let brace_len = source.iter().skip(cursor).position(|t| *t == '}')?;
        let content = source.get(cursor..cursor + brace_len)?;
        Some(content)
    } else {
        None
    };

    Some(CommandComponents {
        name,
        square_content,
        curly_content,
    })
}

#[derive(Debug)]
enum CursorAction {
    IncBy(usize),
    PushMaskAndIncBy(usize),
}

#[cfg(test)]
mod tests {
    use harper_core::Masker as _;

    use crate::masker::CommandComponents;

    use super::{Masker, deconstruct_command};

    #[test]
    fn does_not_mask_spaces_within_sentence() {
        // Spaces between words (not before %) must remain in allowed spans
        // so that lints like double-space detection can still fire.
        let source: Vec<_> = "word  word".chars().collect();
        let mask = Masker::default().create_mask(&source);
        let allowed: String = mask
            .iter_allowed(&source)
            .flat_map(|(_, chars)| chars.iter().copied())
            .collect();
        assert_eq!(allowed, "word  word");
    }

    #[test]
    fn masks_whitespace_before_comment() {
        let source: Vec<_> = "some text   % aligned comment".chars().collect();
        let mask = Masker::default().create_mask(&source);
        let allowed: Vec<String> = mask
            .iter_allowed(&source)
            .map(|(_, chars)| chars.iter().collect::<String>())
            .collect();
        for span in &allowed {
            assert!(
                !span.ends_with(' '),
                "Trailing space before comment leaked through: {span:?}"
            );
        }
    }

    #[test]
    fn ignores_many_comment_signs() {
        let source: Vec<_> = "%%%".chars().collect();
        let mask = Masker::default().create_mask(&source);

        assert_eq!(mask.iter_allowed(&source).next(), None)
    }

    #[test]
    fn ignores_single_comment_sign() {
        let source: Vec<_> = "%".chars().collect();
        let mask = Masker::default().create_mask(&source);

        assert_eq!(mask.iter_allowed(&source).next(), None)
    }

    #[test]
    fn ignores_single_comment_sign_in_phrase() {
        let source: Vec<_> = "this is a comment: % here it is!".chars().collect();
        let mask = Masker::default().create_mask(&source);

        assert_eq!(mask.iter_allowed(&source).count(), 2)
    }

    #[test]
    fn ignores_latex_command() {
        let source: Vec<_> = r"this is a command: \LaTeX there it was!".chars().collect();
        let mask = Masker::default().create_mask(&source);

        assert_eq!(mask.iter_allowed(&source).count(), 2)
    }

    fn masks_math_env(env: &str) {
        let source: Vec<_> =
            format!("This is text. \\begin{{{env}}} x^2 + y^2 \\end{{{env}}} More text.")
                .chars()
                .collect();
        let mask = Masker::default().create_mask(&source);
        let allowed: Vec<String> = mask
            .iter_allowed(&source)
            .map(|(_, chars)| chars.iter().collect::<String>())
            .collect();
        for chunk in &allowed {
            assert!(
                !chunk.contains("x^2"),
                "Math content leaked through for env '{env}': {chunk:?}"
            );
        }
    }

    #[test]
    fn masks_equation_star_env() {
        masks_math_env("equation*");
    }

    #[test]
    fn masks_align_env() {
        masks_math_env("align");
    }

    #[test]
    fn masks_ieee_eqnarray_env_with_arg() {
        // Goal 2: `\begin{IEEEeqnarray*}{c}` (with mandatory arg) is masked whole.
        let out = allowed_text(
            "Hello world. \\begin{IEEEeqnarray*}{c} ax + b & \\text{if} x \\in [L, H] \\\\ \\end{IEEEeqnarray*} Goodbye world.",
        );
        assert!(
            !out.contains("ax"),
            "IEEEeqnarray content leaked through: {out:?}"
        );
        assert!(
            out.contains("Hello world.") && out.contains("Goodbye world."),
            "IEEEeqnarray hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn masks_subequations_env() {
        masks_math_env("subequations");
    }

    #[test]
    fn masks_verbatim_env() {
        // Goal 3: verbatim content is literal; `%` and `$` inside must not
        // act as comment/math, and the code itself stays hidden.
        let out = allowed_text(
            "Hello world. \\begin{verbatim} 100% $x$ & \\end{verbatim} Goodbye world.",
        );
        assert!(
            !out.contains("100%") && !out.contains("$x$"),
            "Verbatim content leaked through: {out:?}"
        );
        assert!(
            out.contains("Hello world.") && out.contains("Goodbye world."),
            "Verbatim hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn masks_lstlisting_env_with_options() {
        // Goal 3: `\begin{lstlisting}[language=C]` masks wholly.
        let out = allowed_text(
            "Hello world. \\begin{lstlisting}[language=C] int QQQZZZ = 0; \\end{lstlisting} Goodbye world.",
        );
        assert!(
            !out.contains("QQQZZZ"),
            "Lstlisting content leaked through: {out:?}"
        );
        assert!(
            out.contains("Hello world.") && out.contains("Goodbye world."),
            "Lstlisting hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn masks_verb_command() {
        // Goal 3, issue example: `\lstinline|void* malloc(size_t size)|`
        // and `\verb` must hide code like `malloc`.
        let out = allowed_text("See \\verb|void* malloc(size_t size)| here.");
        assert!(
            !out.contains("malloc"),
            "Verb content leaked through: {out:?}"
        );
        assert!(
            out.contains("See") && out.contains("here."),
            "Verb hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn verb_dollar_does_not_start_math() {
        // Goal 3: the `$` inside `\verb|$|` is literal: it must neither leak
        // nor mask the rest of the file as unclosed math.
        assert_eq!(allowed_text("A \\verb|$| B"), "A  B");
    }

    #[test]
    fn masks_lstinline_command() {
        assert_eq!(allowed_text("A \\lstinline|void* malloc| B"), "A  B");
        assert_eq!(allowed_text("A \\lstinline{void* malloc} B"), "A  B");
    }

    #[test]
    fn masks_verb_star_variant() {
        assert_eq!(allowed_text("A \\verb*|a b| C"), "A  C");
    }

    #[test]
    fn unclosed_verb_masks_to_eof() {
        assert_eq!(allowed_text("A \\verb|abc"), "A ");
    }

    #[test]
    fn unclosed_verbatim_env_masks_to_eof() {
        assert_eq!(allowed_text("A \\begin{verbatim} x^2"), "A ");
    }

    #[test]
    fn text_inside_paren_math_stays_visible() {
        // Goal 4, issue example 1: `max` is prose, the math around it is not.
        assert_eq!(
            allowed_text("See \\(L_\\text{max} = \\max_{i \\in [n]}\\) here."),
            "See max here."
        );
    }

    #[test]
    fn text_inside_dollar_math_stays_visible() {
        let out = allowed_text("A $QQQ \\text{AAA} RRR \\text{BBB} SSS$ B");
        assert!(out.contains("AAA") && out.contains("BBB"), "{out:?}");
        assert!(
            !out.contains("QQQ") && !out.contains("RRR") && !out.contains("SSS"),
            "Math content leaked through: {out:?}"
        );
    }

    #[test]
    fn text_inside_display_math_stays_visible() {
        let out = allowed_text("A $$\\text{AAA} QQQ$$ B");
        assert!(out.contains("AAA"), "{out:?}");
        assert!(!out.contains("QQQ"), "Display math leaked through: {out:?}");
    }

    #[test]
    fn text_inside_math_env_stays_visible() {
        let out = allowed_text("A \\begin{equation} QQQ \\text{AAA} RRR \\end{equation} B");
        assert!(out.contains("AAA"), "{out:?}");
        assert!(
            !out.contains("QQQ") && !out.contains("RRR"),
            "Env math leaked through: {out:?}"
        );
    }

    #[test]
    fn text_with_nested_braces() {
        assert!(allowed_text("A $\\text{a{b}c}$ B").contains("a{b}c"));
    }

    #[test]
    fn unclosed_text_masks_through_math_end() {
        // No `}` before the math closes: the interior stays hidden.
        let out = allowed_text("A $x \\text{abc$ y} B");
        assert!(!out.contains("abc"), "\\text interior leaked: {out:?}");
        assert!(out.contains("y"), "Prose after math hidden: {out:?}");
    }

    #[test]
    fn text_outside_math_stays_hidden() {
        // Locked decision: `\text` outside math keeps the previous behavior
        // (fully masked) since it is not valid LaTeX there.
        assert!(!allowed_text("A \\text{QQQ} B").contains("QQQ"));
    }

    fn allowed_text(src: &str) -> String {
        let source: Vec<_> = src.chars().collect();
        let mask = Masker::default().create_mask(&source);
        mask.iter_allowed(&source)
            .flat_map(|(_, chars)| chars.iter().copied())
            .collect()
    }

    #[test]
    fn masks_display_double_dollar_math() {
        // Goal 1: `$$ ... $$` should be ignored, surrounding prose kept.
        // Use a distinctive marker so surrounding "text" (which contains 'x')
        // cannot cause false passes.
        let out = allowed_text("Hello world. $$QQQZZZ$$ Goodbye world.");
        assert!(
            !out.contains("QQQZZZ"),
            "Display math content leaked through: {out:?}"
        );
        assert!(
            out.contains("Hello world.") && out.contains("Goodbye world."),
            "Display math hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn masks_paren_inline_math() {
        // Goal 1: `\( ... \)` should be ignored, surrounding prose kept.
        let out = allowed_text("Hello world. \\( QQQZZZ \\) Goodbye world.");
        assert!(
            !out.contains("QQQZZZ"),
            "Paren math content leaked through: {out:?}"
        );
        assert!(
            out.contains("Hello world.") && out.contains("Goodbye world."),
            "Paren math hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn masks_bracket_display_math() {
        // Goal 1: `\[ ... \]` should be ignored, surrounding prose kept.
        let out = allowed_text("Hello world. \\[ QQQZZZ \\] Goodbye world.");
        assert!(
            !out.contains("QQQZZZ"),
            "Bracket math content leaked through: {out:?}"
        );
        assert!(
            out.contains("Hello world.") && out.contains("Goodbye world."),
            "Bracket math hid surrounding prose: {out:?}"
        );
    }

    #[test]
    fn empty_display_dollars_mask_all() {
        // `$$$$` = empty display math: all 4 masked, prose preserved.
        let source: Vec<_> = "Hello $$$$ World".chars().collect();
        let mask = Masker::default().create_mask(&source);
        let allowed: String = mask
            .iter_allowed(&source)
            .flat_map(|(_, chars)| chars.iter().copied())
            .collect();
        assert_eq!(allowed, "Hello  World");
    }

    #[test]
    fn five_dollars_mask_all() {
        // `$$$$$` alone: greedy `$$`+`$$` pair + dangling `$`->EOF, all masked.
        let source: Vec<_> = "$$$$$".chars().collect();
        let mask = Masker::default().create_mask(&source);
        assert_eq!(mask.iter_allowed(&source).next(), None);
    }

    #[test]
    fn escaped_dollar_does_not_start_math() {
        // `\$` is a command, not math: content must stay visible.
        let out = allowed_text("Price \\$QQQZZZ\\$ end");
        assert!(
            out.contains("QQQZZZ"),
            "Escaped dollars hid content: {out:?}"
        );
    }

    #[test]
    fn inline_dollar_masks() {
        assert_eq!(allowed_text("A $x$ B"), "A  B");
    }

    #[test]
    fn single_dollar_in_display() {
        assert_eq!(allowed_text("A $$ a $ b $$ B"), "A  B");
    }

    #[test]
    fn dollar_closes_dollar_run() {
        assert_eq!(allowed_text("A $x$$y$ B"), "A  B");
    }

    #[test]
    fn unclosed_display_to_eof() {
        assert_eq!(allowed_text("A $$ b c"), "A ");
    }

    #[test]
    fn unclosed_paren_to_eof() {
        assert_eq!(allowed_text("A \\( b c"), "A ");
    }

    #[test]
    fn paren_no_space() {
        // No leading space: single-char `deconstruct_command` fix (fixed in
        // this PR) must not swallow the content into the command name.
        assert_eq!(allowed_text("A \\(x\\) B"), "A  B");
    }

    #[test]
    fn escaped_dollar_inside_math() {
        assert_eq!(allowed_text("A $a\\$b$ B"), "A  B");
    }

    #[test]
    fn masks_indentation_before_item() {
        let source: Vec<_> = "\\begin{itemize}\n  \\item Hello world.\n\\end{itemize}"
            .chars()
            .collect();
        let mask = Masker::default().create_mask(&source);
        let allowed: Vec<String> = mask
            .iter_allowed(&source)
            .map(|(_, chars)| chars.iter().collect::<String>())
            .collect();
        for span in &allowed {
            let trimmed = span.trim();
            assert!(
                !trimmed.is_empty() || span.len() <= 1,
                "Whitespace-only span with multiple spaces leaked through: {span:?}"
            );
        }
    }

    #[test]
    fn trailing_backslash_is_not_a_command() {
        let source: Vec<_> = r"\".chars().collect();

        assert!(deconstruct_command(&source).is_none());
    }

    #[test]
    fn create_mask_does_not_panic_on_trailing_backslash() {
        for input in [r"\", r"Text ending with \"] {
            let source: Vec<_> = input.chars().collect();
            Masker::default().create_mask(&source);
        }
    }

    #[test]
    fn emits_all_command_components_correctly() {
        let source: Vec<_> = r"\begin[some]{math}".chars().collect();
        let CommandComponents {
            name,
            square_content,
            curly_content,
        } = deconstruct_command(&source).unwrap();

        assert_eq!(name.iter().collect::<String>(), "begin");
        assert_eq!(square_content.unwrap().iter().collect::<String>(), "some");
        assert_eq!(curly_content.unwrap().iter().collect::<String>(), "math");
    }

    #[test]
    fn unterminated_math_environment_masks_through_eof() {
        let source: Vec<_> = r"Text \begin{equation} x^2 + y^2".chars().collect();
        let mask = Masker::default().create_mask(&source);
        let allowed: String = mask
            .iter_allowed(&source)
            .flat_map(|(_, chars)| chars.iter().copied())
            .collect();

        assert_eq!(allowed, "Text ");
    }

    #[test]
    fn rejects_unterminated_command_arguments() {
        let square_source: Vec<_> = r"\begin[some".chars().collect();
        let curly_source: Vec<_> = r"\section{Energy and Environment".chars().collect();

        assert!(deconstruct_command(&square_source).is_none());
        assert!(deconstruct_command(&curly_source).is_none());
    }

    #[test]
    fn emits_command_curly_component_correctly() {
        let source: Vec<_> = r"\begin{math}".chars().collect();
        let CommandComponents {
            name,
            square_content,
            curly_content,
        } = deconstruct_command(&source).unwrap();

        assert_eq!(name.iter().collect::<String>(), "begin");
        assert_eq!(square_content, None);
        assert_eq!(curly_content.unwrap().iter().collect::<String>(), "math");
    }

    #[test]
    fn emits_command_square_component_correctly() {
        let source: Vec<_> = r"\begin[some]".chars().collect();
        let CommandComponents {
            name,
            square_content,
            curly_content,
        } = deconstruct_command(&source).unwrap();

        assert_eq!(name.iter().collect::<String>(), "begin");
        assert_eq!(square_content.unwrap().iter().collect::<String>(), "some");
        assert_eq!(curly_content, None);
    }

    #[test]
    fn emits_section_correctly() {
        let source: Vec<_> = r"\section{Energy and Environment}".chars().collect();
        let CommandComponents {
            name,
            square_content,
            curly_content,
        } = deconstruct_command(&source).unwrap();

        assert_eq!(name.iter().collect::<String>(), "section");
        assert_eq!(square_content, None);
        assert_eq!(
            curly_content.unwrap().iter().collect::<String>(),
            "Energy and Environment"
        );
    }
}

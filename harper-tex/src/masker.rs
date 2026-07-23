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

            if matches!(c, '%') || is_explicit_space_at_cursor(cursor, source) {
                actions.push_back(CursorAction::PushMaskAndIncBy(1));
            } else if let Some(ws_actions) = whitespace_actions_at_cursor(cursor, source) {
                actions.extend(ws_actions);
            } else if let Some(s) = math_mode_at_cursor(cursor, source) {
                actions.push_back(CursorAction::PushMaskAndIncBy(s));
            } else if let Some(s) = equation_at_cursor(cursor, source) {
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

fn is_blank(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

fn is_explicit_space_at_cursor(cursor: usize, source: &[char]) -> bool {
    source.get(cursor) == Some(&'\\') && source.get(cursor + 1) == Some(&' ')
}

/// Collapses TeX whitespace while retaining one separator.
fn whitespace_actions_at_cursor(cursor: usize, source: &[char]) -> Option<Vec<CursorAction>> {
    let first = *source.get(cursor)?;
    if !is_blank(first) && first != '\n' {
        return None;
    }

    if cursor > 0 && source.get(cursor - 1) == Some(&'\\') {
        return None;
    }

    let whitespace_len = source[cursor..]
        .iter()
        .take_while(|&&c| is_blank(c) || c == '\n')
        .count();
    let whitespace = &source[cursor..cursor + whitespace_len];
    let newline_count = whitespace.iter().filter(|&&c| c == '\n').count();
    let separator = if newline_count >= 2 {
        whitespace.iter().rposition(|&c| c == '\n')?
    } else {
        whitespace
            .iter()
            .position(|&c| c == ' ')
            .or_else(|| whitespace.iter().position(|&c| c == '\t'))
            .or_else(|| whitespace.iter().position(|&c| c == '\n'))?
    };

    let mut actions = Vec::new();
    if separator > 0 {
        actions.push(CursorAction::PushMaskAndIncBy(separator));
    }

    actions.push(CursorAction::IncBy(1));

    let trailing_whitespace = whitespace_len - separator - 1;
    if trailing_whitespace > 0 {
        actions.push(CursorAction::PushMaskAndIncBy(trailing_whitespace));
    }

    Some(actions)
}

/// Check whether there is a math mode block at the current cursor. If so, this function will return the amount cursor needs to be incremented by in order to escape the block.
fn math_mode_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    if *source.get(cursor)? != '$' {
        return None;
    }

    Some(
        source
            .iter()
            .skip(cursor + 1)
            .take_while(|t| **t != '$')
            .count()
            + 2,
    )
}

/// Check whether there is a command at the current cursor. If so, this function will update the action queue to mask out the hidden elements.
/// Returns whether the action queue was modified.
fn command_at_cursor(cursor: usize, source: &[char], actions: &mut VecDeque<CursorAction>) -> bool {
    // `\\` is intentionally deferred: a span-only mask cannot turn it into a visible
    // separator without a source whitespace character to retain.
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
        "math",
        "displaymath",
    ];
    math_envs.iter().any(|e| env.eq_str(e))
}

fn equation_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    let CommandComponents {
        name,
        square_content,
        curly_content,
    } = deconstruct_command(&source[cursor..])?;

    if name.eq_str("begin") && curly_content.is_some_and(is_math_env) {
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
    let name_len = source
        .iter()
        .skip(cursor + 1)
        .take_while(|t| t.is_alphabetic())
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
    use harper_core::{Masker as _, TokenKind, parsers::Parser};

    use crate::{TeX, masker::CommandComponents};

    use super::{Masker, deconstruct_command};

    fn allowed_text(source: &[char]) -> String {
        Masker::default()
            .create_mask(source)
            .iter_allowed(source)
            .flat_map(|(_, chars)| chars.iter().copied())
            .collect()
    }

    fn paragraph_break_count(source: &str) -> usize {
        TeX::default()
            .parse(&source.chars().collect::<Vec<_>>())
            .iter()
            .filter(|token| matches!(&token.kind, TokenKind::ParagraphBreak))
            .count()
    }

    #[test]
    fn collapses_spaces_within_sentence() {
        let source: Vec<_> = "word  word".chars().collect();

        assert_eq!(allowed_text(&source), "word word");
    }

    #[test]
    fn collapses_whitespace_within_sentence() {
        let source: Vec<_> = "word\t \t\t     \n\t\t  word".chars().collect();

        assert_eq!(allowed_text(&source), "word word");
    }

    #[test]
    fn keeps_separator_for_empty_line() {
        let source: Vec<_> = "word\n\nword".chars().collect();

        assert_eq!(allowed_text(&source), "word\nword");
        assert_eq!(paragraph_break_count("word\n\nword"), 1);
    }

    #[test]
    fn keeps_separator_for_blank_line() {
        let source: Vec<_> = "word\n\t \t \nword".chars().collect();

        assert_eq!(allowed_text(&source), "word\nword");
    }

    #[test]
    fn collapses_blank_lines() {
        let source: Vec<_> = "word\t  \n\t \t \n  \n\n\n\t\n  \tword".chars().collect();

        assert_eq!(allowed_text(&source), "word\nword");
    }

    #[test]
    fn keeps_explicit_and_non_breaking_spaces_visible() {
        let source: Vec<_> = "word\\ word~word".chars().collect();

        assert_eq!(allowed_text(&source), "word word~word");
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

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

            if matches!(c, '%') {
                actions.push_back(CursorAction::PushMaskAndIncBy(1));
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

fn equation_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    let CommandComponents {
        name,
        square_content,
        curly_content,
    } = deconstruct_command(&source[cursor..])?;

    if name.eq_str("begin") && curly_content.is_some_and(|cc| cc.eq_str("equation")) {
        let mut diff = 1
            + name.len()
            + curly_content.unwrap().len()
            + square_content.map(|sc| sc.len()).unwrap_or_default();

        loop {
            if let Some(CommandComponents {
                name,
                curly_content,
                ..
            }) = deconstruct_command(&source[cursor + diff..])
                && name.eq_str("end")
                && curly_content.is_some_and(|cc| cc.eq_str("equation"))
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

    // The name of the command
    let name_len = source
        .iter()
        .skip(cursor + 1)
        .take_while(|t| t.is_alphabetic())
        .count();
    let name = &source[cursor..cursor + 1 + name_len];

    cursor += name_len + 1;

    // The optional square braces
    let square_content = if source.get(cursor) == Some(&'[') {
        cursor += 1;

        let brace_len = source
            .iter()
            .skip(cursor)
            .take_while(|t| **t != ']')
            .count();

        let content = &source[cursor..cursor + brace_len];

        cursor += brace_len + 1;
        Some(content)
    } else {
        None
    };

    // The optional square braces
    let curly_content = if source.get(cursor) == Some(&'{') {
        cursor += 1;

        let brace_len = source
            .iter()
            .skip(cursor)
            .take_while(|t| **t != '}')
            .count();

        let content = &source[cursor..cursor + brace_len];
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

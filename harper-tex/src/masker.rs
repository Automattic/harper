use std::collections::VecDeque;

use harper_core::{Mask, Span};

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
            } else if !command_at_cursor(cursor, source, &mut actions)
                && let Some(s) = math_mode_at_cursor(cursor, source)
            {
                actions.push_back(CursorAction::PushMaskAndIncBy(s));
            } else {
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

/// Check whether there is a math mode block at the current cursor. If so, this function will return the
/// index of the next non-math-block index.
fn math_mode_at_cursor(cursor: usize, source: &[char]) -> Option<usize> {
    if *source.get(cursor)? != '$' {
        return None;
    }

    Some(
        source
            .iter()
            .skip(cursor)
            .take_while(|t| **t != '$')
            .count()
            + cursor
            + 1,
    )
}

/// Check whether there is a command at the current cursor. If so, this function will update the action queue to mask out the hidden elements.
/// Returns whether the action queue was modified.
fn command_at_cursor(
    mut cursor: usize,
    source: &[char],
    actions: &mut VecDeque<CursorAction>,
) -> bool {
    let orig_cursor = cursor;

    if source.get(cursor) != Some(&'\\') {
        return false;
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
    if source.get(cursor) == Some(&'[') {
        cursor += source
            .iter()
            .skip(cursor)
            .take_while(|t| **t != ']')
            .count()
            + 1;
    }

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

    // The optional curly braces
    if source.get(cursor) == Some(&'{') {
        let brace_len = source
            .iter()
            .skip(cursor)
            .take_while(|t| **t != '}')
            .count();

        if is_content_command {
            actions.push_back(CursorAction::PushMaskAndIncBy(cursor - orig_cursor));
            actions.push_back(CursorAction::IncBy(brace_len));
            actions.push_back(CursorAction::PushMaskAndIncBy(1));
            true
        } else {
            actions.push_back(CursorAction::PushMaskAndIncBy(
                brace_len + cursor + 1 - orig_cursor,
            ));
            true
        }
    } else {
        actions.push_back(CursorAction::PushMaskAndIncBy(cursor - orig_cursor));
        true
    }
}

#[derive(Debug)]
enum CursorAction {
    IncBy(usize),
    PushMaskAndIncBy(usize),
}

#[cfg(test)]
mod tests {
    use harper_core::Masker as _;

    use super::Masker;

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
}

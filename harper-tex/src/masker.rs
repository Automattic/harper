use harper_core::{Mask, Span};

#[derive(Debug, Default)]
pub struct Masker {}

impl harper_core::Masker for Masker {
    fn create_mask(&self, source: &[char]) -> Mask {
        let mut cursor = 0;
        let mut mask = Mask::new_blank();

        let mut cur_mask_start = 0;

        loop {
            if cursor >= source.len() {
                break;
            }

            let c = source[cursor];
            let mut action = CursorAction::Inc;

            if matches!(c, '%') {
                action = CursorAction::PushMaskAndIncBy(1)
            };

            if let Some(s) = command_at_cursor(cursor, source) {
                action = CursorAction::PushMaskAndIncBy(s)
            }

            if let Some(s) = math_mode_at_cursor(cursor, source) {
                action = CursorAction::PushMaskAndIncBy(s)
            }

            match action {
                CursorAction::Inc => cursor += 1,
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

        if cur_mask_start != cursor {
            mask.push_allowed(Span::new(cur_mask_start, cursor));
        }

        mask
    }
}

/// Check whether there is a math mode block at the current cursor. If so, this function will return the
/// index of the next non-math-block index.
fn math_mode_at_cursor(mut cursor: usize, source: &[char]) -> Option<usize> {
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

/// Check whether there is a command at the current cursor. If so, this function will return the
/// index of the next non-command index.
fn command_at_cursor(mut cursor: usize, source: &[char]) -> Option<usize> {
    let orig_cursor = cursor;

    if *source.get(cursor)? != '\\' {
        return None;
    }

    // The name of the command
    cursor += source
        .iter()
        .skip(cursor + 1)
        .take_while(|t| t.is_alphabetic())
        .count()
        + 1;

    // The optional square braces
    if source.get(cursor) == Some(&'[') {
        cursor += source
            .iter()
            .skip(cursor)
            .take_while(|t| **t != ']')
            .count()
            + 1;
    }

    dbg!(source.get(cursor));

    // The optional curly braces
    if source.get(cursor) == Some(&'{') {
        Some(
            source
                .iter()
                .skip(cursor)
                .take_while(|t| **t != '}')
                .count()
                + cursor
                + 1
                - orig_cursor,
        )
    } else {
        Some(cursor - orig_cursor)
    }
}

enum CursorAction {
    Inc,
    PushMaskAndIncBy(usize),
}

#[cfg(test)]
mod tests {
    use harper_core::Masker as _;

    use crate::masker::command_at_cursor;

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

    #[test]
    fn detects_latex_as_command() {
        assert_eq!(
            command_at_cursor(0, &"\\LaTeX".chars().collect::<Vec<_>>()),
            Some(6)
        )
    }

    #[test]
    fn detects_latex_with_braces_as_command() {
        assert_eq!(
            command_at_cursor(0, &"\\LaTeX{}".chars().collect::<Vec<_>>()),
            Some(8)
        )
    }

    #[test]
    fn detects_usepackage_as_command() {
        assert_eq!(
            command_at_cursor(0, &"\\usepackage{amsmath}".chars().collect::<Vec<_>>()),
            Some(20)
        )
    }

    #[test]
    fn detect_usepackage_with_square_braces_as_command() {
        assert_eq!(
            command_at_cursor(
                0,
                &"\\usepackage[utf8]{inputenc}".chars().collect::<Vec<_>>()
            ),
            Some(27)
        )
    }
}

use harper_core::{CharStringExt, Mask, Masker, Span};

pub struct LiterateHaskellMasker {
    text: bool,
    code: bool,
}

impl LiterateHaskellMasker {
    pub fn text_only() -> Self {
        Self {
            text: true,
            code: false,
        }
    }

    pub fn code_only() -> Self {
        Self {
            text: false,
            code: true,
        }
    }
}

impl Masker for LiterateHaskellMasker {
    fn create_mask(&mut self, source: &[char]) -> harper_core::Mask {
        let mut mask = Mask::new_blank();

        let mut location = 0;
        let mut in_code_env = false;
        let mut last_line_blank = false;

        for line in source.split(|c| *c == '\n') {
            let string_form = line.to_string();
            let trimmed = string_form.trim();
            let line_is_bird = line.first().map_or(false, |c| *c == '>');

            // Code fencing
            let latex_style = matches!(trimmed, r"\begin{code}" | r"\end{code}");
            let code_start = trimmed == r"\begin{code}" || (last_line_blank && line_is_bird);
            let code_end = trimmed == r"\end{code}" || trimmed.is_empty();

            // Toggle on fence
            if (!in_code_env && code_start) || (in_code_env && code_end) {
                in_code_env = !in_code_env;

                // Exclude latex-style fence
                if latex_style {
                    location += line.len() + 1; // +1 for the newline split on
                    last_line_blank = trimmed.is_empty();
                    continue;
                }

                // Exclude newline after code for bird style
                if trimmed.is_empty() {
                    location += line.len() + 1; // +1 for the newline split on
                    last_line_blank = true;
                    continue;
                }
            }

            let end_loc = location + line.len();
            if (!in_code_env && self.text) || (in_code_env && self.code) {
                let start_loc = if line_is_bird { location + 2 } else { location };
                mask.push_allowed(Span::new(start_loc, end_loc));
            }

            location = end_loc + 1; // +1 for the newline split on
            last_line_blank = trimmed.is_empty();
        }

        mask.merge_whitespace_sep(source);
        mask
    }
}

#[cfg(test)]
mod tests {
    use harper_core::{Masker, Span};
    use itertools::Itertools;

    use super::LiterateHaskellMasker;

    #[test]
    fn bird_format() {
        let source = r"Text here

> fact :: Integer -> Integer
> fact 0 = 1
> fact n = n * fact (n-1)

Text here
"
        .chars()
        .collect_vec();

        let text_mask = LiterateHaskellMasker::text_only().create_mask(&source);
        assert_eq!(
            text_mask
                .iter_allowed(&source)
                .map(|(s, _)| s)
                .collect_vec(),
            vec![Span::new(0, 10), Span::new(80, 90)],
        );

        let code_mask = LiterateHaskellMasker::code_only().create_mask(&source);
        assert_eq!(
            code_mask
                .iter_allowed(&source)
                .map(|(s, _)| s)
                .collect_vec(),
            vec![Span::new(13, 39), Span::new(42, 52), Span::new(55, 78)],
        );
    }

    #[test]
    fn latex_format() {
        let source = r#"Text here
\begin{code}
main :: IO ()
main = print "just an example"
\end{code}
Text here
"#
        .chars()
        .collect_vec();

        let text_mask = LiterateHaskellMasker::text_only().create_mask(&source);
        assert_eq!(
            text_mask
                .iter_allowed(&source)
                .map(|(s, _)| s)
                .collect_vec(),
            vec![Span::new(0, 9), Span::new(79, 89)],
        );

        let code_mask = LiterateHaskellMasker::code_only().create_mask(&source);
        assert_eq!(
            code_mask
                .iter_allowed(&source)
                .map(|(s, _)| s)
                .collect_vec(),
            vec![Span::new(23, 67)],
        );
    }
}

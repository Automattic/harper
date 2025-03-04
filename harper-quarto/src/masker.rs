use harper_core::{CharStringExt, Mask, Masker, Span};
use itertools::Itertools;

pub struct QuartoMasker;

fn isolate_document(source: &[char]) -> Option<Vec<Span>> {
    let doc = todo!();

    Some(
        doc.split(|(_, c)| **c == '\n')
            .filter_map(|l| Some(Span::new(l.first()?.0, l.last()?.0 + 1)))
            .collect_vec(),
    )
}

impl Masker for QuartoMasker {
    fn create_mask(&self, source: &[char]) -> harper_core::Mask {
        let mut mask = Mask::new_blank();

        if let Some(lines) = isolate_document(source) {
            lines.into_iter().for_each(|span| {
                mask.push_allowed(span);
            });
        }

        mask.merge_whitespace_sep(source);
        mask
    }
}

#[cfg(test)]
mod tests {
    use harper_core::Masker;
    use itertools::Itertools;

    use super::QuartoMasker;

    #[test]
    fn parse_no_header() {
        let src = "# Line
---
line
line
";
        let src = src.chars().collect_vec();

        let masker = QuartoMasker;
        let mask = masker.create_mask(&src);

        let allowed = mask.iter_allowed(&src).next().expect("No allowed blocks").1;
        assert_eq!(allowed, &src);
    }
}

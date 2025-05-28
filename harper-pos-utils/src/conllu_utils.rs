use std::{fs::File, path::Path};

use rs_conllu::{Sentence, parse_file};

/// Produce an iterator over the sentences in a `.conllu` file.
/// Will panic on error, so this should not be used outside of training.
pub fn iter_sentences_in_conllu(path: impl AsRef<Path>) -> impl Iterator<Item = Sentence> {
    let file = File::open(path).unwrap();
    let doc = parse_file(file);

    doc.map(|v| v.unwrap())
}

use std::borrow::Cow;

use smallvec::SmallVec;

/// A char sequence that improves cache locality.
/// Most English words are fewer than 12 characters.
pub type CharString = SmallVec<[char; 16]>;

/// Extensions to character sequences that make them easier to wrangle.
pub trait CharStringExt {
    fn to_lower(&self) -> Cow<[char]>;
    fn normalized(&self) -> Cow<[char]>;
    fn to_string(&self) -> String;
}

impl CharStringExt for [char] {
    fn to_lower(&self) -> Cow<[char]> {
        if self.iter().all(|c| c.is_lowercase()) {
            return Cow::Borrowed(self);
        }

        let mut out = CharString::with_capacity(self.len());

        out.extend(self.iter().flat_map(|v| v.to_lowercase()));

        Cow::Owned(out.to_vec())
    }

    fn to_string(&self) -> String {
        self.iter().collect()
    }

    /// Convert a given character sequence to the standard character set
    /// the dictionary is in.
    fn normalized(&self) -> Cow<[char]> {
        if self.as_ref().iter().any(|c| char_to_normalized(*c) != *c) {
            Cow::Owned(
                self.as_ref()
                    .iter()
                    .copied()
                    .map(char_to_normalized)
                    .collect(),
            )
        } else {
            Cow::Borrowed(self)
        }
    }
}

fn char_to_normalized(c: char) -> char {
    match c {
        '’' => '\'',
        '‘' => '\'',
        '＇' => '\'',
        _ => c,
    }
}

macro_rules! char_string {
    ($string:literal) => {{
        use crate::char_string::CharString;

        $string.chars().collect::<CharString>()
    }};
}

pub(crate) use char_string;

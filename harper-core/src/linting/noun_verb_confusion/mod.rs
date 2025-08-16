use super::merge_linters::merge_linters;

mod noun_instead_of_verb;
mod verb_instead_of_noun;

// Common noun-verb pairs that are often confused
// See also [`NounInsteadOfVerb``]
pub(crate) const NOUN_VERB_PAIRS: &[(&str, &str)] = &[
    ("advice", "advise"),
    ("belief", "believe"),
    ("breath", "breathe"),
    ("effect", "affect"), // "Effect" is also a verb meaning "to bring about". "Affect" is a noun in psychology.
    ("emphasis", "emphasize"), // TODO how to handle "emphasise" as well as "emphasize"?
    ("intent", "intend"),
    // ("proof", "prove"),  // "Proof" is also a verb, a synonym of "proofread".
    // Add more pairs here as needed
];

pub use noun_instead_of_verb::NounInsteadOfVerb;
pub use verb_instead_of_noun::VerbInsteadOfNoun;

// Merge the two linters with a descriptive message
merge_linters! {
    NounVerbConfusion =>
        NounInsteadOfVerb,
        VerbInsteadOfNoun
        => "Handles common confusions between related nouns and verbs (e.g., 'advice/advise', 'breath/breathe')"
}

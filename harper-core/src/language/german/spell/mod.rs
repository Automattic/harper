//! German spell checking support.

pub use self::german_dict::{
    annotated_german_dictionary, base_german_dictionary, base_german_dictionary_fst,
    combined_german_dictionary, compound_aware_german_dictionary,
    compound_aware_german_fst_dictionary, curated_german_dictionary, german_compound_checker,
    german_dictionary, mutable_german_dictionary,
};

pub mod compound;
pub mod compound_aware_dict;
pub mod compound_checker;
pub mod german_dict;

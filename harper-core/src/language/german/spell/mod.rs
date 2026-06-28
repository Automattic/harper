//! German spell checking support.

pub use self::german_dict::{
    annotated_german_dictionary, curated_german_dictionary, german_dictionary,
    mutable_german_dictionary,
};

pub mod german_dict;

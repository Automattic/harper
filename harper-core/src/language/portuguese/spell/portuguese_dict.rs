//! Portuguese dictionary support.
//!
//! This follows the same pattern as the English dictionary implementation.

use std::sync::{Arc, LazyLock};

use crate::spell::{FstDictionary, MutableDictionary};

#[cfg(feature = "pt")]
fn load_portuguese_dict() -> Arc<MutableDictionary> {
    MutableDictionary::from_rune_files(
        include_str!("../dictionary.dict"),
        include_str!("../annotations.json"),
    )
    .map(Arc::new)
    .unwrap_or_else(|e| panic!("Failed to load Portuguese dictionary: {}", e))
}

#[cfg(not(feature = "pt"))]
fn load_portuguese_dict() -> Arc<MutableDictionary> {
    Arc::new(MutableDictionary::new())
}

static DICT: LazyLock<Arc<MutableDictionary>> = LazyLock::new(load_portuguese_dict);

/// Returns a shared reference to the Portuguese FstDictionary.
///
/// This is the main curated Portuguese dictionary, equivalent to the English curated dictionary.
pub fn portuguese_dictionary() -> Arc<FstDictionary> {
    // Convert the MutableDictionary to FstDictionary
    Arc::new((**DICT).clone().into())
}

/// Alias for the main dictionary, following English naming conventions.
pub fn curated_portuguese_dictionary() -> Arc<FstDictionary> {
    portuguese_dictionary()
}

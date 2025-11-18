use serde::{Deserialize, Serialize};
use std::default::Default;
use strum_macros::{Display, EnumCount, EnumIter, EnumString, VariantArray};

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    EnumCount,
    EnumString,
    EnumIter,
    Display,
    VariantArray,
)]
pub enum Language {
    #[default]
    English,
    Portuguese,
}

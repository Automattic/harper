use serde::{Deserialize, Serialize};

use super::Error;
use super::affix_replacement::{AffixReplacement, HumanReadableAffixReplacement};
use crate::DictWordMetadata;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AffixEntryKind {
    Suffix,
    Prefix,
}

/// Defines how a word can be transformed and what metadata to apply
#[derive(Debug, Clone)]
pub struct Expansion {
    /// Whether this is a prefix or suffix expansion
    pub kind: AffixEntryKind,
    /// If true, allows this expansion to be combined with others (e.g., both prefix and suffix)
    pub cross_product: bool,
    /// The replacement rules that define how to modify the word
    pub replacements: Vec<AffixReplacement>,
    /// Metadata to apply to the transformed word
    pub target: Vec<MetadataExpansion>,
    /// Metadata to apply to the base word when this expansion is applied
    pub base_metadata: DictWordMetadata,
}

impl Expansion {
    pub fn into_human_readable(self) -> HumanReadableExpansion {
        HumanReadableExpansion {
            kind: self.kind,
            cross_product: self.cross_product,
            replacements: self
                .replacements
                .iter()
                .map(AffixReplacement::to_human_readable)
                .collect(),
            target: self.target,
            base_metadata: self.base_metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataExpansion {
    pub metadata: DictWordMetadata,
    pub if_base: Option<DictWordMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableExpansion {
    pub kind: AffixEntryKind,
    pub cross_product: bool,
    pub replacements: Vec<HumanReadableAffixReplacement>,
    pub target: Vec<MetadataExpansion>,
    pub base_metadata: DictWordMetadata,
}

impl HumanReadableExpansion {
    pub fn into_normal(self) -> Result<Expansion, Error> {
        let mut replacements = Vec::with_capacity(self.replacements.len());

        for replacement in &self.replacements {
            replacements.push(replacement.to_normal()?);
        }

        Ok(Expansion {
            kind: self.kind,
            cross_product: self.cross_product,
            replacements,
            target: self.target,
            base_metadata: self.base_metadata,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    /// Whether the metadata will propagate to all derived words.
    #[serde(default)]
    pub propagate: bool,
    /// The metadata applied to the word.
    pub metadata: DictWordMetadata,
}

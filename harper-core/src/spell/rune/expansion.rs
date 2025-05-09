use serde::{Deserialize, Serialize};

use super::Error;
use super::affix_replacement::{AffixReplacement, HumanReadableAffixReplacement};
use crate::WordMetadata;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AffixEntryKind {
    Property,
    Suffix,
    Prefix,
}

#[derive(Debug, Clone)]
pub struct Expansion {
    /// If `!true`, this is a prefix
    /// But if `true` it may be a prefix but may be a property only
    pub kind: AffixEntryKind,
    pub cross_product: bool,
    pub replacements: Vec<AffixReplacement>,
    /// When the expansion is applied, the resulting word will have this
    /// metadata appended to it.
    pub target: Vec<MetadataExpansion>,
    /// When the expansion is applied, the __parent__ word will have this
    /// metadata appended to it.
    pub base_metadata: WordMetadata,
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
    pub metadata: WordMetadata,
    pub if_base: Option<WordMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableExpansion {
    pub kind: AffixEntryKind,
    pub cross_product: bool,
    pub replacements: Vec<HumanReadableAffixReplacement>,
    pub target: Vec<MetadataExpansion>,
    pub base_metadata: WordMetadata,
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

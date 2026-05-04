use harper_core::{Dialect, IgnoredLints, linting::FlatConfig, spell::MutableDictionary};

/// User-controlled linting state needed to construct and apply a Harper lint group.
pub struct Config {
    pub mutable_dictionary: MutableDictionary,
    pub dialect: Dialect,
    pub ignored_lints: IgnoredLints,
    pub lint_config: FlatConfig,
}

impl Config {
    pub fn new() -> Self {
        Self {
            mutable_dictionary: MutableDictionary::new(),
            dialect: Dialect::American,
            ignored_lints: IgnoredLints::new(),
            lint_config: FlatConfig::new_curated(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

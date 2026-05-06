mod human_readable_structured_config;

use super::FlatConfig;
use serde_json;

pub use human_readable_structured_config::{HumanReadableSetting, HumanReadableStructuredConfig};

/// A structure for defining which rules to be enabled or disabled in a
/// [`LintGroup`](super::LintGroup).
///
/// So named because it represents a more structured view for organizing rules.
/// Designed to be something that can be converted _into_ a [`FlatConfig`] at runtime before
/// being passed to an actual [`LintGroup`](super::LintGroup`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredConfig {
    pub settings: Vec<Setting>,
}

impl StructuredConfig {
    /// Build the curated structured config, including labels.
    pub fn curated() -> Self {
        let human: HumanReadableStructuredConfig = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/default_config.json"
        )))
        .unwrap();

        human.to_structured_config().unwrap()
    }

    /// Validate that the structure of the settings is valid.
    /// Returns `true` if it is valid and `false` otherwise.
    pub fn validate(&self) -> bool {
        self.settings.iter().all(|s| s.validate())
    }

    /// Creates a [`FlatConfig`] that represents these settings.
    /// Will return `None` if `self` is invalid.
    pub fn to_flat_config(&self) -> Option<FlatConfig> {
        if !self.validate() {
            return None;
        }

        let mut config = FlatConfig::default();

        for setting in &self.settings {
            match setting {
                Setting::Bool { name, state } => config.set_rule_enabled(name, *state),
                Setting::OneOfMany { names, choice, .. } => {
                    if let Some(choice) = choice {
                        config.set_rule_enabled(&names[*choice], true);
                    }
                }
                Setting::Group { child, .. } => {
                    let flat = child.to_flat_config()?;
                    config.merge_from(flat);
                }
            }
        }

        Some(config)
    }

    /// Fills in the relevant values from a [`FlatConfig`] according to the relevant `name`
    /// fields.
    pub fn copy_from_flat_config(&mut self, config: &FlatConfig) {
        for setting in self.settings.iter_mut() {
            match setting {
                Setting::Bool { name, state } => *state = config.is_rule_enabled(name),
                Setting::OneOfMany { names, choice, .. } => {
                    *choice = names.iter().position(|n| config.is_rule_enabled(n))
                }
                Setting::Group { child, .. } => child.copy_from_flat_config(config),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    Bool {
        name: String,
        state: bool,
    },
    /// Selects one of many rules.
    /// Think of it as a dropdown field, where you can select one or no of several options.
    /// The labels are separate from the rule names, for readability.
    OneOfMany {
        /// The names of the linters we can select from.
        names: Vec<String>,
        labels: Vec<String>,
        choice: Option<usize>,
    },
    Group {
        label: String,
        description: String,
        child: StructuredConfig,
    },
}

impl Setting {
    /// Validate that the internal structure of the enum is valid.
    /// Returns `true` if it is valid and `false` otherwise.
    pub fn validate(&self) -> bool {
        match self {
            Setting::Bool { .. } => true,
            Setting::OneOfMany {
                names,
                labels,
                choice,
            } => labels.len() == names.len() && choice.is_none_or(|v| v < names.len()),
            Setting::Group { child, .. } => child.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::Dialect;
    use crate::linting::FlatConfig;
    use crate::linting::LintGroup;
    use crate::spell::MutableDictionary;

    use super::{Setting, StructuredConfig};

    fn collect_rule_names(config: &StructuredConfig) -> BTreeSet<String> {
        let mut out = BTreeSet::new();

        for setting in &config.settings {
            match setting {
                Setting::Bool { name, .. } => {
                    out.insert(name.clone());
                }
                Setting::OneOfMany { names, .. } => {
                    out.extend(names.iter().cloned());
                }
                Setting::Group { child, .. } => {
                    out.extend(collect_rule_names(child));
                }
            }
        }

        out
    }

    #[test]
    fn validates_bool_true() {
        assert!(
            Setting::Bool {
                name: "A".to_owned(),
                state: true,
            }
            .validate()
        )
    }

    #[test]
    fn validates_bool_false() {
        assert!(
            Setting::Bool {
                name: "A".to_owned(),
                state: false,
            }
            .validate()
        )
    }

    #[test]
    fn validates_valid_one_of_many_some() {
        assert!(
            Setting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                labels: vec!["C".to_owned(), "D".to_owned()],
                choice: Some(1)
            }
            .validate()
        )
    }

    #[test]
    fn validates_valid_one_of_many_none() {
        assert!(
            Setting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                labels: vec!["C".to_owned(), "D".to_owned()],
                choice: None
            }
            .validate()
        )
    }

    #[test]
    fn validates_invalid_one_of_many_some_too_large() {
        assert!(
            !Setting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                labels: vec!["C".to_owned(), "D".to_owned()],
                choice: Some(2)
            }
            .validate()
        )
    }

    #[test]
    fn validates_invalid_one_of_many_inconsistent_len() {
        assert!(
            !Setting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                labels: vec!["C".to_owned(), "D".to_owned(), "E".to_owned()],
                choice: None
            }
            .validate()
        )
    }

    #[test]
    fn converts_only_bools() {
        let settings = StructuredConfig {
            settings: vec![
                Setting::Bool {
                    name: "A".to_owned(),
                    state: false,
                },
                Setting::Bool {
                    name: "B".to_owned(),
                    state: true,
                },
                Setting::Bool {
                    name: "C".to_owned(),
                    state: false,
                },
            ],
        };

        let config = settings.to_flat_config().unwrap();

        assert!(!config.is_rule_enabled("A"));
        assert!(config.is_rule_enabled("B"));
        assert!(!config.is_rule_enabled("C"));
    }

    #[test]
    fn converts_only_one_of_many() {
        let settings = StructuredConfig {
            settings: vec![Setting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                labels: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                choice: Some(2),
            }],
        };

        let config = settings.to_flat_config().unwrap();

        assert!(!config.is_rule_enabled("A"));
        assert!(!config.is_rule_enabled("B"));
        assert!(config.is_rule_enabled("C"));
    }

    #[test]
    fn can_pull_simple_config_from_flat_config() {
        let mut settings = StructuredConfig {
            settings: vec![Setting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                labels: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                choice: None,
            }],
        };

        let mut lgc = FlatConfig::default();
        lgc.set_rule_enabled("A", false);
        lgc.set_rule_enabled("B", false);
        lgc.set_rule_enabled("C", true);

        settings.copy_from_flat_config(&lgc);

        assert_eq!(
            settings,
            StructuredConfig {
                settings: vec![Setting::OneOfMany {
                    names: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                    labels: vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
                    choice: Some(2),
                }],
            }
        )
    }

    #[test]
    fn validates_group_with_label() {
        assert!(
            Setting::Group {
                label: "Group".to_owned(),
                description: "Description".to_owned(),
                child: StructuredConfig {
                    settings: vec![Setting::Bool {
                        name: "A".to_owned(),
                        state: true,
                    }],
                },
            }
            .validate()
        );
    }

    #[test]
    fn curated_is_valid() {
        assert!(StructuredConfig::curated().validate());
    }

    #[test]
    fn curated_default_config_lists_every_registered_rule() {
        let curated = StructuredConfig::curated();
        let curated_rule_names = collect_rule_names(&curated);

        let runtime_rule_names =
            LintGroup::new_curated(MutableDictionary::new().into(), Dialect::American)
                .iter_keys()
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();

        let missing_from_default_config = runtime_rule_names
            .difference(&curated_rule_names)
            .cloned()
            .collect::<Vec<_>>();
        let extra_in_default_config = curated_rule_names
            .difference(&runtime_rule_names)
            .cloned()
            .collect::<Vec<_>>();

        assert!(
            missing_from_default_config.is_empty() && extra_in_default_config.is_empty(),
            "default_config.json drifted from the registered rule set\nmissing from default_config.json: {:?}\nextra in default_config.json: {:?}\nDid you forget to add the missing rule(s) to default_config.json?",
            missing_from_default_config,
            extra_in_default_config,
        );
    }
}

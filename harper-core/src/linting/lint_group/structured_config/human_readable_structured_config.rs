use serde::{Deserialize, Serialize};

use super::{Setting, StructuredConfig};

/// A human-readable mirror of [`super::StructuredConfig`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanReadableStructuredConfig {
    pub settings: Vec<HumanReadableSetting>,
}

impl HumanReadableStructuredConfig {
    /// Validate that the structure of the settings is valid.
    /// Returns `true` if it is valid and `false` otherwise.
    pub fn validate(&self) -> bool {
        self.settings.iter().all(|setting| setting.validate())
    }

    /// Convert this human-readable structure into a runtime [`StructuredConfig`].
    ///
    /// Human-facing metadata is preserved where the runtime structure supports it.
    pub fn to_structured_config(&self) -> Option<StructuredConfig> {
        if !self.validate() {
            return None;
        }

        Some(StructuredConfig {
            settings: self
                .settings
                .iter()
                .map(HumanReadableSetting::to_setting)
                .collect::<Option<_>>()?,
        })
    }

    /// Build a human-readable structure from a runtime [`StructuredConfig`].
    pub fn from_structured_config(config: &StructuredConfig) -> Self {
        Self {
            settings: config
                .settings
                .iter()
                .map(HumanReadableSetting::from_setting)
                .collect(),
        }
    }
}

/// A human-readable mirror of [`super::Setting`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanReadableSetting {
    Bool {
        name: String,
        state: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// Selects one of many rules.
    OneOfMany {
        /// The names of the linters we can select from.
        names: Vec<String>,
        /// The selected linter name, if any.
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        labels: Option<Vec<String>>,
    },
    Group {
        label: Option<String>,
        description: String,
        child: HumanReadableStructuredConfig,
    },
}

impl HumanReadableSetting {
    /// Validate that the internal structure of the enum is valid.
    /// Returns `true` if it is valid and `false` otherwise.
    pub fn validate(&self) -> bool {
        match self {
            Self::Bool { .. } => true,
            Self::OneOfMany {
                names,
                name,
                labels,
                ..
            } => {
                labels
                    .as_ref()
                    .is_none_or(|labels| labels.len() == names.len())
                    && name
                        .as_ref()
                        .is_none_or(|selected| names.iter().any(|name| name == selected))
            }
            Self::Group { child, .. } => child.validate(),
        }
    }

    fn to_setting(&self) -> Option<Setting> {
        match self {
            Self::Bool { name, state, .. } => Some(Setting::Bool {
                name: name.clone(),
                state: *state,
            }),
            Self::OneOfMany {
                names,
                name,
                labels,
                ..
            } => {
                let choice = match name {
                    Some(selected) => Some(names.iter().position(|name| name == selected)?),
                    None => None,
                };

                Some(Setting::OneOfMany {
                    names: names.clone(),
                    labels: labels.clone().unwrap_or_else(|| names.clone()),
                    choice,
                })
            }
            Self::Group {
                label,
                description,
                child,
            } => Some(Setting::Group {
                label: label.clone().unwrap_or_default(),
                description: description.clone(),
                child: child.to_structured_config()?,
            }),
        }
    }

    fn from_setting(setting: &Setting) -> Self {
        match setting {
            Setting::Bool { name, state } => Self::Bool {
                name: name.clone(),
                state: *state,
                label: None,
            },
            Setting::OneOfMany {
                names,
                labels,
                choice,
            } => Self::OneOfMany {
                names: names.clone(),
                name: choice.map(|choice| names[choice].clone()),
                labels: (labels != names).then(|| labels.clone()),
            },
            Setting::Group {
                label,
                description,
                child,
            } => Self::Group {
                label: Some(label.clone()),
                description: description.clone(),
                child: HumanReadableStructuredConfig::from_structured_config(child),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HumanReadableSetting, HumanReadableStructuredConfig};
    use crate::linting::FlatConfig;
    use crate::linting::lint_group::structured_config::{Setting, StructuredConfig};

    fn collect_rule_names(config: &HumanReadableStructuredConfig) -> Vec<&str> {
        let mut out = Vec::new();

        for setting in &config.settings {
            match setting {
                HumanReadableSetting::Bool { name, .. } => out.push(name.as_str()),
                HumanReadableSetting::OneOfMany { names, .. } => {
                    out.extend(names.iter().map(|name| name.as_str()));
                }
                HumanReadableSetting::Group { child, .. } => out.extend(collect_rule_names(child)),
            }
        }

        out
    }

    #[test]
    fn human_readable_config_round_trips_json() {
        let settings = HumanReadableStructuredConfig {
            settings: vec![
                HumanReadableSetting::Bool {
                    name: "A".to_owned(),
                    state: true,
                    label: Some("Rule A".to_owned()),
                },
                HumanReadableSetting::OneOfMany {
                    names: vec!["B".to_owned(), "C".to_owned()],
                    name: Some("C".to_owned()),
                    labels: Some(vec!["Rule B".to_owned(), "Rule C".to_owned()]),
                },
                HumanReadableSetting::Group {
                    label: Some("Group D".to_owned()),
                    description: "Description D".to_owned(),
                    child: HumanReadableStructuredConfig {
                        settings: vec![HumanReadableSetting::Bool {
                            name: "D".to_owned(),
                            state: false,
                            label: None,
                        }],
                    },
                },
            ],
        };

        let json = serde_json::to_string(&settings).unwrap();
        let decoded: HumanReadableStructuredConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, settings);
    }

    #[test]
    fn converts_to_structured_config() {
        let settings = HumanReadableStructuredConfig {
            settings: vec![
                HumanReadableSetting::Bool {
                    name: "A".to_owned(),
                    state: true,
                    label: Some("Rule A".to_owned()),
                },
                HumanReadableSetting::OneOfMany {
                    names: vec!["B".to_owned(), "C".to_owned()],
                    name: Some("C".to_owned()),
                    labels: Some(vec!["Rule B".to_owned(), "Rule C".to_owned()]),
                },
                HumanReadableSetting::Group {
                    label: Some("Group D".to_owned()),
                    description: "Description D".to_owned(),
                    child: HumanReadableStructuredConfig {
                        settings: vec![HumanReadableSetting::Bool {
                            name: "D".to_owned(),
                            state: false,
                            label: None,
                        }],
                    },
                },
            ],
        };

        let structured = settings.to_structured_config().unwrap();

        assert_eq!(
            structured,
            StructuredConfig {
                settings: vec![
                    Setting::Bool {
                        name: "A".to_owned(),
                        state: true,
                    },
                    Setting::OneOfMany {
                        names: vec!["B".to_owned(), "C".to_owned()],
                        labels: vec!["Rule B".to_owned(), "Rule C".to_owned()],
                        choice: Some(1),
                    },
                    Setting::Group {
                        label: "Group D".to_owned(),
                        description: "Description D".to_owned(),
                        child: StructuredConfig {
                            settings: vec![Setting::Bool {
                                name: "D".to_owned(),
                                state: false,
                            }],
                        },
                    },
                ],
            }
        );
    }

    #[test]
    fn converts_from_structured_config() {
        let structured = StructuredConfig {
            settings: vec![
                Setting::Bool {
                    name: "A".to_owned(),
                    state: true,
                },
                Setting::OneOfMany {
                    names: vec!["B".to_owned(), "C".to_owned()],
                    labels: vec!["Rule B".to_owned(), "Rule C".to_owned()],
                    choice: Some(1),
                },
                Setting::Group {
                    label: "Group D".to_owned(),
                    description: "Description D".to_owned(),
                    child: StructuredConfig {
                        settings: vec![Setting::Bool {
                            name: "D".to_owned(),
                            state: false,
                        }],
                    },
                },
            ],
        };

        let human = HumanReadableStructuredConfig::from_structured_config(&structured);

        assert_eq!(
            human,
            HumanReadableStructuredConfig {
                settings: vec![
                    HumanReadableSetting::Bool {
                        name: "A".to_owned(),
                        state: true,
                        label: None,
                    },
                    HumanReadableSetting::OneOfMany {
                        names: vec!["B".to_owned(), "C".to_owned()],
                        name: Some("C".to_owned()),
                        labels: Some(vec!["Rule B".to_owned(), "Rule C".to_owned()]),
                    },
                    HumanReadableSetting::Group {
                        label: Some("Group D".to_owned()),
                        description: "Description D".to_owned(),
                        child: HumanReadableStructuredConfig {
                            settings: vec![HumanReadableSetting::Bool {
                                name: "D".to_owned(),
                                state: false,
                                label: None,
                            }],
                        },
                    },
                ],
            }
        );
    }

    #[test]
    fn rejects_invalid_one_of_many_selection() {
        let settings = HumanReadableStructuredConfig {
            settings: vec![HumanReadableSetting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                name: Some("C".to_owned()),
                labels: None,
            }],
        };

        assert!(!settings.validate());
        assert!(settings.to_structured_config().is_none());
    }

    #[test]
    fn rejects_invalid_one_of_many_labels() {
        let settings = HumanReadableStructuredConfig {
            settings: vec![HumanReadableSetting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                name: Some("B".to_owned()),
                labels: Some(vec!["Only one".to_owned()]),
            }],
        };

        assert!(!settings.validate());
        assert!(settings.to_structured_config().is_none());
    }

    #[test]
    fn uses_rule_names_as_labels_when_none_are_provided() {
        let settings = HumanReadableStructuredConfig {
            settings: vec![HumanReadableSetting::OneOfMany {
                names: vec!["A".to_owned(), "B".to_owned()],
                name: Some("B".to_owned()),
                labels: None,
            }],
        };

        assert_eq!(
            settings.to_structured_config().unwrap(),
            StructuredConfig {
                settings: vec![Setting::OneOfMany {
                    names: vec!["A".to_owned(), "B".to_owned()],
                    labels: vec!["A".to_owned(), "B".to_owned()],
                    choice: Some(1),
                }],
            }
        );
    }

    #[test]
    fn default_config_matches_curated_flat_config() {
        let settings = StructuredConfig::curated();
        let actual = settings.to_flat_config().unwrap();
        let curated = FlatConfig::new_curated();

        assert!(settings.validate());

        let human = HumanReadableStructuredConfig::from_structured_config(&settings);
        let rule_names = collect_rule_names(&human);
        assert_eq!(rule_names.len(), 696);

        for rule_name in rule_names {
            assert_eq!(
                actual.is_rule_enabled(rule_name),
                curated.is_rule_enabled(rule_name),
                "mismatch for rule {rule_name}"
            );
        }
    }
}

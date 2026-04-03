use serde::{Deserialize, Serialize};

/// A human-readable mirror of [`super::StructuredConfig`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanReadableStructuredConfig {
    pub settings: Vec<HumanReadableSetting>,
}

/// A human-readable mirror of [`super::Setting`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanReadableSetting {
    Bool {
        name: String,
        state: bool,
    },
    /// Selects one of many rules.
    OneOfMany {
        /// The names of the linters we can select from.
        names: Vec<String>,
        /// The selected linter name, if any.
        name: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::{HumanReadableSetting, HumanReadableStructuredConfig};

    #[test]
    fn human_readable_config_round_trips_json() {
        let settings = HumanReadableStructuredConfig {
            settings: vec![
                HumanReadableSetting::Bool {
                    name: "A".to_owned(),
                    state: true,
                },
                HumanReadableSetting::OneOfMany {
                    names: vec!["B".to_owned(), "C".to_owned()],
                    name: Some("C".to_owned()),
                },
            ],
        };

        let json = serde_json::to_string(&settings).unwrap();
        let decoded: HumanReadableStructuredConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, settings);
    }
}

use super::{Lint, LintKind, Suggestion};
use super::{LintGroup, PatternLinter};
use crate::patterns::{EitherPattern, IsNotTitleCase, Pattern, SequencePattern, WordSet};
use crate::{Dictionary, make_title_case};
use crate::{Token, TokenStringExt};
use std::sync::Arc;

pub struct ProperNominalCapitalizationLinter<D: Dictionary + 'static> {
    pattern: Box<dyn Pattern>,
    description: String,
    dictionary: Arc<D>,
}

impl<D: Dictionary + 'static> ProperNominalCapitalizationLinter<D> {
    pub fn new(
        search_for: impl Pattern + 'static,
        description: impl ToString,
        dictionary: D,
    ) -> Self {
        let dictionary = Arc::new(dictionary);

        Self {
            pattern: Box::new(IsNotTitleCase::new(
                Box::new(search_for),
                dictionary.clone(),
            )),
            dictionary: dictionary.clone(),
            description: description.to_string(),
        }
    }
}

impl<D: Dictionary + 'static> PatternLinter for ProperNominalCapitalizationLinter<D> {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let proper = make_title_case(matched_tokens, source, &self.dictionary);

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Capitalization,
            suggestions: vec![Suggestion::ReplaceWith(proper)],
            message: self.description.to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }
}

pub fn lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
    let mut group = LintGroup::empty();

    group.add(
    "Americas",
    Box::new(ProperNominalCapitalizationLinter::new(
    SequencePattern::default()
        .then(WordSet::new(&["South", "North", "Central"]))
        .then_whitespace()
        .t_aco("America"),
    "When referring to North, Central, and South America, make sure to treat them as a proper noun.",
    dictionary.clone()))
);

    group.add(
        "Australia",
        Box::new(ProperNominalCapitalizationLinter::new(
            EitherPattern::new(vec![
                // the states and territories
                Box::new(
                    SequencePattern::default()
                        .t_aco("Australian")
                        .then_whitespace()
                        .t_aco("Capital")
                        .then_whitespace()
                        .t_aco("Territory"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("New")
                        .then_whitespace()
                        .t_aco("South")
                        .then_whitespace()
                        .t_aco("Wales"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Northern")
                        .then_whitespace()
                        .t_aco("Territory"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("South")
                        .then_whitespace()
                        .t_aco("Australia"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Western")
                        .then_whitespace()
                        .t_aco("Australia"),
                ),
                // major cities
                Box::new(
                    SequencePattern::default()
                        .t_aco("Alice")
                        .then_whitespace()
                        .t_aco("Springs"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Gold")
                        .then_whitespace()
                        .t_aco("Coast"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Sunshine")
                        .then_whitespace()
                        .t_aco("Coast"),
                ),
            ]),
            "When referring to the states of Australia, make sure to treat them as a proper noun.",
            dictionary.clone(),
        )),
    );

    group.add(
    "OceansAndSeas",
    Box::new(ProperNominalCapitalizationLinter::new(
        EitherPattern::new(vec![
            Box::new(
                SequencePattern::default()
                    .then(WordSet::new(&[
                        "Atlantic",
                        "Pacific",
                        "Indian",
                        "Southern",
                        "Arctic",
                    ]))
                    .then_whitespace()
                    .t_aco("Ocean")
            ),
            Box::new(
                SequencePattern::default()
                    .then(WordSet::new(&[
                        "Mediterranean",
                        "Caribbean",
                        "Baltic",
                        "Red",
                        "Black",
                        "Caspian",
                        "Coral",
                        "Bering",
                        "North",
                    ]))
                    .then_whitespace()
                    .t_aco("Sea")
            ),
            Box::new(
                SequencePattern::default()
                    .t_aco("South")
                    .then_whitespace()
                    .t_aco("China")
                    .then_whitespace()
                    .t_aco("Sea")
            ),
        ]),
        "When referring to the world's oceans and seas, ensure they are treated as proper nouns.",
        dictionary.clone()
    ))
);

    group.add(
        "Canada",
        Box::new(ProperNominalCapitalizationLinter::new(
            EitherPattern::new(vec![
                // the provinces and territories
                Box::new(
                    SequencePattern::default()
                        .t_aco("British")
                        .then_whitespace()
                        .t_aco("Columbia"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("New")
                        .then_whitespace()
                        .t_aco("Brunswick"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Northwest")
                        .then_whitespace()
                        .t_aco("Territories"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Nova")
                        .then_whitespace()
                        .t_aco("Scotia"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Prince")
                        .then_whitespace()
                        .t_aco("Edward")
                        .then_whitespace()
                        .t_aco("Island"),
                ),
                // major cities
                Box::new(
                    SequencePattern::default()
                        .t_aco("Quebec")
                        .then_whitespace()
                        .t_aco("City"),
                ),
            ]),
            "When referring to the provinces of Canada, make sure to treat them as a proper noun.",
            dictionary.clone(),
        )),
    );

    group.add(
        "Koreas",
        Box::new(ProperNominalCapitalizationLinter::new(
            SequencePattern::default()
                .then(WordSet::new(&["South", "North"]))
                .then_whitespace()
                .t_aco("Korea"),
            "When referring to the nations, make sure to treat them as a proper noun.",
            dictionary.clone(),
        )),
    );

    group.add(
    "Malaysia",
    Box::new(ProperNominalCapitalizationLinter::new(
    EitherPattern::new(vec![
        // multi-word states
        Box::new(SequencePattern::default()
            .t_aco("Negeri")
            .then_whitespace()
            .t_aco("Sembilan")
        ),
        // multi-word state capitals
        Box::new(SequencePattern::default()
            .t_aco("Alor")
            .then_whitespace()
            .t_aco("Setar")
        ),
        Box::new(SequencePattern::default()
            .t_aco("George")
            .then_whitespace()
            .t_aco("Town")
        ),
        Box::new(SequencePattern::default()
            .then(EitherPattern::new(vec![
                Box::new(WordSet::new(&[
                    "Johor",
                    "Kota"
                ])),
            ]))
            .then_whitespace()
            .t_aco("Bahru")
        ),
        Box::new(SequencePattern::default()
            .t_aco("Kota")
            .then_whitespace()
            .t_aco("Kinabalu")
        ),
        Box::new(SequencePattern::default()
            .t_aco("Kuala")
            .then_whitespace()
            .then(EitherPattern::new(vec![
                Box::new(WordSet::new(&[
                    "Lumpur",
                    "Terengganu"
                ])),
            ]))
        ),
        Box::new(SequencePattern::default()
            .t_aco("Shah")
            .then_whitespace()
            .t_aco("Alam")
        )
    ]),
    "When referring to the states of Malaysia and their capitals, make sure to treat them as a proper noun.",
    dictionary.clone()))
);

    group.add(
        "ChineseCommunistParty",
        Box::new(ProperNominalCapitalizationLinter::new(
            SequencePattern::aco("Chinese")
                .then_whitespace()
                .t_aco("Communist")
                .then_whitespace()
                .t_aco("Party"),
            "When referring to the political party, make sure to treat them as a proper noun.",
            dictionary.clone(),
        )),
    );

    group.add(
    "UnitedOrganizations",
    Box::new(ProperNominalCapitalizationLinter::new(
    SequencePattern::default()
        .t_aco("United")
        .then_whitespace()
        .then(EitherPattern::new(vec![
            Box::new(SequencePattern::aco("Nations")),
            Box::new(SequencePattern::aco("States")),
            Box::new(SequencePattern::aco("Kingdom")),
            Box::new(SequencePattern::aco("Airlines")),
            Box::new(
                SequencePattern::default()
                    .t_aco("Arab")
                    .then_whitespace()
                    .t_aco("Emirates")
            )
        ])),
    "When referring to national or international organizations, make sure to treat them as a proper noun.",
    dictionary.clone()))
);

    group.add(
        "Holidays",
        Box::new(ProperNominalCapitalizationLinter::new(
            EitherPattern::new(vec![
                Box::new(
                    SequencePattern::default()
                        .then(EitherPattern::new(vec![
                            Box::new(WordSet::new(&[
                                "Absolution",
                                "Admission",
                                "Alaska",
                                "Anzac",
                                "ANZAC",
                                "Arbor",
                                "Armistice",
                                "Ascension",
                                "Australia",
                                "Ayurveda",
                                "Bastille",
                                "Bonifacio",
                                "Boxing",
                                "Canada",
                                "Career",
                                "Chewidden",
                                "Christmas",
                                "Class",
                                "Columbus",
                                "Commonwealth",
                                "D",
                                "Darwin",
                                "Discovery",
                                "Distaff",
                                "Dominion",
                                "Earth",
                                "Easter",
                                "Election",
                                "Emancipation",
                                "Empire",
                                "Evolution",
                                "Family",
                                "Father's",
                                "Fathers'",
                                "Flag",
                                "Forefathers'",
                                "Foundation",
                                "Freedom",
                                "Galentine's",
                                "Groundhog",
                                "Gypsy",
                                "Halloween",
                                "Independence",
                                "Invasion",
                                "Ivy",
                                "Jamhuri",
                                "Jubilee",
                                "Kamehameha",
                                "Kenyatta",
                                "Labor",
                                "Labour",
                                "Lady",
                                "Land",
                                "Lei",
                                "Madaraka",
                                "Mashujaa",
                                "May",
                                "Memorial",
                                "Merdeka",
                                "Midsummer",
                                "Midsummer's",
                                "Mother's",
                                "Mothers'",
                                "Nakba",
                                "Nevada",
                                "Occupation",
                                "Parents",
                                "Patrick's",
                                "Patriots'",
                                "Pi",
                                "Picrous",
                                "Pioneer",
                                "Presidents'",
                                "Remembrance",
                                "Republic",
                                "Restoration",
                                "Rizal",
                                "Roc",
                                "Rock",
                                "Seward's",
                                "Singles'",
                                "Statehood",
                                "Tax",
                                "Thanksgiving",
                                "Treason",
                                "Ulster",
                                "Valentine's",
                                "VE",
                                "VJ",
                                "VP",
                                "Veterans",
                                "Victoria",
                                "Victory",
                                "Waffle",
                                "Waitangi",
                                "Wattle",
                                "White",
                                "Wren",
                                "Years",
                                "Year's",
                                "Youth",
                            ])),
                            Box::new(
                                SequencePattern::default()
                                    .t_aco("National")
                                    .then_whitespace()
                                    .t_aco("Freedom"),
                            ),
                            Box::new(
                                SequencePattern::default()
                                    .t_aco("All")
                                    .then_whitespace()
                                    .t_aco("Saints"),
                            ),
                            Box::new(
                                SequencePattern::default()
                                    .t_aco("All")
                                    .then_whitespace()
                                    .t_aco("Souls"),
                            ),
                        ]))
                        .then_whitespace()
                        .t_aco("Day"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Black")
                        .then_whitespace()
                        .t_aco("Friday"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Cyber")
                        .then_whitespace()
                        .t_aco("Monday"),
                ),
            ]),
            "When referring to holidays, make sure to treat them as a proper noun.",
            dictionary.clone(),
        )),
    );

    group.add(
    "AmazonNames",
    Box::new(ProperNominalCapitalizationLinter::new(
    SequencePattern::default()
    .t_aco("Amazon")
    .then_whitespace()
    .then(EitherPattern::new(vec![
        Box::new(
            SequencePattern::default()
                .t_aco("Shopping")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("Web")
                    .then_whitespace()
                .t_aco("Services")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("Lambda")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("RDS")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("DynamoDB")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("SageMaker")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("Rekognition")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("CloudFront")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("ECS")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("EKS")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("CloudWatch")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("IAM")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("Prime")
        ),
        Box::new(
            SequencePattern::default()
                .t_aco("Kindle")
        )
    ])),
    "When referring to the various products of Amazon.com, make sure to treat them as a proper noun.",
    dictionary.clone()))
);

    group.add(
        "GoogleNames",
        Box::new(ProperNominalCapitalizationLinter::new(
        SequencePattern::default()
            .t_aco("Google")
            .then_whitespace()
            .then(WordSet::new(&[
                "Search",
                "Cloud",
                "Maps",
                "Docs",
                "Sheets",
                "Slides",
                "Drive",
                "Meet",
                "Gmail",
                "Calendar",
                "Chrome",
                "ChromeOS",
                "Android",
                "Play",
                "Bard",
                "Gemini",
                "YouTube",
                "Photos",
                "Analytics",
                "AdSense",
                "Pixel",
                "Nest",
                "Workspace",
            ])),
        "When referring to Google products and services, make sure to treat them as proper nouns."
            ,dictionary.clone()))
    );

    group.add(
        "AzureNames",
        Box::new(ProperNominalCapitalizationLinter::new(
            SequencePattern::default()
                .t_aco("Azure")
                .then_whitespace()
                .then(EitherPattern::new(vec![
                    Box::new(SequencePattern::aco("DevOps")),
                    Box::new(SequencePattern::aco("Functions")),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Cosmos")
                            .then_whitespace()
                            .t_aco("DB"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("SQL")
                            .then_whitespace()
                            .t_aco("Database"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Kubernetes")
                            .then_whitespace()
                            .t_aco("Service"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Virtual")
                            .then_whitespace()
                            .t_aco("Machines"),
                    ),
                    Box::new(SequencePattern::aco("Monitor")),
                    Box::new(SequencePattern::aco("Storage")),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Active")
                            .then_whitespace()
                            .t_aco("Directory"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("App")
                            .then_whitespace()
                            .t_aco("Service"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Key")
                            .then_whitespace()
                            .t_aco("Vault"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Cognitive")
                            .then_whitespace()
                            .t_aco("Services"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Service")
                            .then_whitespace()
                            .t_aco("Bus"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Event")
                            .then_whitespace()
                            .t_aco("Hub"),
                    ),
                ])),
            "When referring to Azure cloud services, make sure to treat them as proper nouns.",
            dictionary.clone(),
        )),
    );

    group.add(
    "MicrosoftNames",
    Box::new(ProperNominalCapitalizationLinter::new(
    SequencePattern::default()
        .t_aco("Microsoft")
        .then_whitespace()
        .then(EitherPattern::new(vec![
            Box::new(WordSet::new(&[
                "Windows",
                "Office",
                "Teams",
                "Excel",
                "PowerPoint",
                "Word",
                "Outlook",
                "OneDrive",
                "SharePoint",
                "Xbox",
                "Surface",
                "Edge",
                "Bing",
                "Dynamics",
            ])),
            Box::new(
                SequencePattern::default()
                    .t_aco("Visual")
                    .then_whitespace()
                    .t_aco("Studio")
            )
        ])),
    "When referring to Microsoft products and services, make sure to treat them as proper nouns.",
    dictionary.clone()))
);

    group.add(
        "AppleNames",
        Box::new(ProperNominalCapitalizationLinter::new(
        SequencePattern::default()
            .t_aco("Apple")
            .then_whitespace()
            .then(EitherPattern::new(vec![
                Box::new(WordSet::new(&[
                    "iPhone", "iPad", "iMac", "MacBook", "Watch", "TV", "Music", "Arcade",
                    "iCloud", "Safari", "HomeKit", "CarPlay",
                ])),
                Box::new(
                    SequencePattern::aco("MacBook")
                        .then_whitespace()
                        .t_aco("Pro")
                ),
                Box::new(
                    SequencePattern::aco("MacBook")
                        .then_whitespace()
                        .t_aco("Air")
                ),
                Box::new(SequencePattern::aco("Mac").then_whitespace().t_aco("Pro")),
                Box::new(SequencePattern::aco("Mac").then_whitespace().t_aco("Mini")),
                Box::new(SequencePattern::aco("AirPods")),
                Box::new(
                    SequencePattern::aco("AirPods")
                        .then_whitespace()
                        .t_aco("Pro")
                ),
                Box::new(
                    SequencePattern::aco("AirPods")
                        .then_whitespace()
                        .t_aco("Max")
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Vision")
                        .then_whitespace()
                        .t_aco("Pro")
                )
            ])),
        "When referring to Apple products and services, make sure to treat them as proper nouns.",
        dictionary.clone()))

    );

    group.add(
        "MetaNames",
        Box::new(ProperNominalCapitalizationLinter::new(SequencePattern::aco("Meta")
            .then_whitespace()
            .then(EitherPattern::new(vec![
                Box::new(WordSet::new(&[
                    "Oculus", "Portals", "Quest", "Gaming", "Horizon",
                ])),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Reality")
                        .then_whitespace()
                        .t_aco("Labs")
                ),
            ])),
        "When referring to Meta products and services, make sure to treat them as proper nouns."
        , dictionary.clone()
        ))
    );

    group.add(
        "JetpackNames",
        Box::new(ProperNominalCapitalizationLinter::new(
            SequencePattern::default()
                .t_aco("Jetpack")
                .then_whitespace()
                .then(EitherPattern::new(vec![
                    Box::new(
                        SequencePattern::default()
                            .t_aco("VaultPress")
                            .then_whitespace()
                            .t_aco("Backup"),
                    ),
                    Box::new(SequencePattern::default().t_aco("VaultPress")),
                    Box::new(SequencePattern::default().t_aco("Scan")),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Akismet")
                            .then_whitespace()
                            .t_aco("Anti-spam"),
                    ),
                    Box::new(SequencePattern::default().t_aco("Stats")),
                    Box::new(SequencePattern::default().t_aco("Social")),
                    Box::new(SequencePattern::default().t_aco("Blaze")),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("AI")
                            .then_whitespace()
                            .t_aco("Assistant"),
                    ),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("Site")
                            .then_whitespace()
                            .t_aco("Search"),
                    ),
                    Box::new(SequencePattern::default().t_aco("Boost")),
                    Box::new(SequencePattern::default().t_aco("VideoPress")),
                    Box::new(
                        SequencePattern::default()
                            .t_aco("For")
                            .then_whitespace()
                            .t_aco("Agencies"),
                    ),
                    Box::new(SequencePattern::default().t_aco("CRM")),
                ])),
            "Ensure proper capitalization of Jetpack-related terms.",
            dictionary.clone(),
        )),
    );

    group.add(
        "TumblrNames",
        Box::new(ProperNominalCapitalizationLinter::new(
            SequencePattern::default()
                .t_aco("Tumblr")
                .then_whitespace()
                .then(EitherPattern::new(vec![
                    Box::new(SequencePattern::default().t_aco("Blaze")),
                    Box::new(SequencePattern::default().t_aco("Pro")),
                    Box::new(SequencePattern::default().t_aco("Live")),
                    Box::new(SequencePattern::default().t_aco("Ads")),
                    Box::new(SequencePattern::default().t_aco("Communities")),
                    Box::new(SequencePattern::default().t_aco("Shop")),
                    Box::new(SequencePattern::default().t_aco("Dashboard")),
                ])),
            "Ensure proper capitalization of Tumblr-related terms.",
            dictionary.clone(),
        )),
    );

    group.add(
        "PocketCastsNames",
        Box::new(ProperNominalCapitalizationLinter::new(
            EitherPattern::new(vec![
                Box::new(
                    SequencePattern::default()
                        .t_aco("Pocket")
                        .then_whitespace()
                        .t_aco("Casts"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Pocket")
                        .then_whitespace()
                        .t_aco("Casts")
                        .then_whitespace()
                        .t_aco("Plus"),
                ),
            ]),
            "Ensure proper capitalization of Pocket Casts and Pocket Casts Plus as brand names.",
            dictionary.clone(),
        )),
    );

    group.add(
        "DayOneNames",
        Box::new(ProperNominalCapitalizationLinter::new(
            EitherPattern::new(vec![
                Box::new(
                    SequencePattern::default()
                        .t_aco("Day")
                        .then_whitespace()
                        .t_aco("One"),
                ),
                Box::new(
                    SequencePattern::default()
                        .t_aco("Day")
                        .then_whitespace()
                        .t_aco("One")
                        .then_whitespace()
                        .t_aco("Premium"),
                ),
            ]),
            "Ensure proper capitalization of Day One and Day One Premium as brand names.",
            dictionary.clone(),
        )),
    );

    group.set_all_rules_to(Some(true));

    group
}

#[cfg(test)]
mod tests {
    use crate::{
        FstDictionary,
        linting::tests::{assert_lint_count, assert_suggestion_result},
    };

    use super::lint_group;

    #[test]
    fn americas_lowercase() {
        assert_suggestion_result(
            "south america",
            lint_group(FstDictionary::curated()),
            "South America",
        );
        assert_suggestion_result(
            "north america",
            lint_group(FstDictionary::curated()),
            "North America",
        );
    }

    #[test]
    fn americas_uppercase() {
        assert_suggestion_result(
            "SOUTH AMERICA",
            lint_group(FstDictionary::curated()),
            "South America",
        );
        assert_suggestion_result(
            "NORTH AMERICA",
            lint_group(FstDictionary::curated()),
            "North America",
        );
    }

    #[test]
    fn americas_allow_correct() {
        assert_lint_count("South America", lint_group(FstDictionary::curated()), 0);
        assert_lint_count("North America", lint_group(FstDictionary::curated()), 0);
    }

    #[test]
    fn united_nations_uppercase() {
        assert_suggestion_result(
            "UNITED NATIONS",
            lint_group(FstDictionary::curated()),
            "United Nations",
        );
    }

    #[test]
    fn united_arab_emirates_lowercase() {
        assert_suggestion_result(
            "UNITED ARAB EMIRATES",
            lint_group(FstDictionary::curated()),
            "United Arab Emirates",
        );
    }

    #[test]
    fn united_nations_allow_correct() {
        assert_lint_count("United Nations", lint_group(FstDictionary::curated()), 0);
    }

    #[test]
    fn meta_allow_correct() {
        assert_lint_count("Meta Quest", lint_group(FstDictionary::curated()), 0);
    }

    #[test]
    fn microsoft_lowercase() {
        assert_suggestion_result(
            "microsoft visual studio",
            lint_group(FstDictionary::curated()),
            "Microsoft Visual Studio",
        );
    }

    #[test]
    fn microsoft_first_word_is_correct() {
        assert_suggestion_result(
            "Microsoft visual studio",
            lint_group(FstDictionary::curated()),
            "Microsoft Visual Studio",
        );
    }

    #[test]
    fn test_atlantic_ocean_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("atlantic ocean", group, "Atlantic Ocean");
    }

    #[test]
    fn test_pacific_ocean_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("pacific ocean", group, "Pacific Ocean");
    }

    #[test]
    fn test_indian_ocean_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("indian ocean", group, "Indian Ocean");
    }

    #[test]
    fn test_southern_ocean_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("southern ocean", group, "Southern Ocean");
    }

    #[test]
    fn test_arctic_ocean_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("arctic ocean", group, "Arctic Ocean");
    }

    // Lowercase tests for seas

    #[test]
    fn test_mediterranean_sea_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("mediterranean sea", group, "Mediterranean Sea");
    }

    #[test]
    fn test_caribbean_sea_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("caribbean sea", group, "Caribbean Sea");
    }

    #[test]
    fn test_south_china_sea_lowercase() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_suggestion_result("south china sea", group, "South China Sea");
    }

    // Tests that allow correctly capitalized names

    #[test]
    fn test_atlantic_ocean_correct() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_lint_count("Atlantic Ocean", group, 0);
    }

    #[test]
    fn test_pacific_ocean_correct() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_lint_count("Pacific Ocean", group, 0);
    }

    #[test]
    fn test_indian_ocean_correct() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_lint_count("Indian Ocean", group, 0);
    }

    #[test]
    fn test_mediterranean_sea_correct() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_lint_count("Mediterranean Sea", group, 0);
    }

    #[test]
    fn test_south_china_sea_correct() {
        let dictionary = FstDictionary::curated();
        let group = lint_group(dictionary);
        assert_lint_count("South China Sea", group, 0);
    }
}

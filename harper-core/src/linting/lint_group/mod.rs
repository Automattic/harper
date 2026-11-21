mod flat_config;
mod structured_config;

use std::{collections::BTreeMap, hash::BuildHasher, num::NonZero, sync::Arc};

use {foldhash::quality::RandomState, hashbrown::HashMap, lru::LruCache};

use crate::{
    linting::{
        expr_linter::{Chunk, Sentence, run_on_chunk},
        {ExprLinter, HtmlDescriptionLinter, Lint, Linter},
    },
    spell::Dictionary,
    {Document, EnglishDialect, Language, Span, TokenStringExt},
};

pub use flat_config::FlatConfig;
pub use structured_config::{
    HumanReadableSetting, HumanReadableStructuredConfig, StructuredConfig,
};

/// A struct for collecting the output of a number of individual [Linter]s.
/// Each child can be toggled via the public, mutable `Self::config` object.
pub struct LintGroup {
    pub config: FlatConfig,
    /// We use a binary map here so the ordering is stable.
    linters: BTreeMap<String, Box<dyn Linter>>,
    /// We use a binary map here so the ordering is stable.
    chunk_expr_linters: BTreeMap<String, Box<dyn ExprLinter<Unit = Chunk>>>,
    /// We use a binary map here so the ordering is stable.
    sentence_expr_linters: BTreeMap<String, Box<dyn ExprLinter<Unit = Sentence>>>,
    /// Since [`ExprLinter`]s operate on a chunk-basis, we can store a
    /// mapping of `Chunk -> Lint` and only rerun the expr linters
    /// when a chunk changes.
    ///
    /// Since the expr linter results also depend on the configuration, we hash it and pass it as part
    /// of the key.
    chunk_expr_cache: LruCache<(u64, u64), BTreeMap<String, Vec<Lint>>>,
    sentence_expr_cache: LruCache<(u64, u64), BTreeMap<String, Vec<Lint>>>,
    hasher_builder: RandomState,
    clashing_linter_names: Option<Vec<String>>,
}

impl LintGroup {
    // Constructor methods

    pub fn empty() -> Self {
        Self {
            config: FlatConfig::default(),
            linters: BTreeMap::new(),
            chunk_expr_linters: BTreeMap::new(),
            sentence_expr_linters: BTreeMap::new(),
            chunk_expr_cache: LruCache::new(NonZero::new(1000).unwrap()),
            sentence_expr_cache: LruCache::new(NonZero::new(1000).unwrap()),
            hasher_builder: RandomState::default(),
            clashing_linter_names: None,
        }
    }

    /// Check if the group already contains a linter with a given name.
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.linters.contains_key(name.as_ref())
            || self.chunk_expr_linters.contains_key(name.as_ref())
            || self.sentence_expr_linters.contains_key(name.as_ref())
    }

    /// Add a [`Linter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    pub fn add(&mut self, name: impl AsRef<str>, linter: impl Linter + 'static) -> bool {
        if self.contains_key(&name) {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![name.as_ref().to_string()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(name.as_ref().to_string());
            }
            false
        } else {
            self.linters
                .insert(name.as_ref().to_string(), Box::new(linter));
            true
        }
    }

    /// Add a chunk-based [`ExprLinter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    ///
    /// This function is not significantly different from [`Self::add`], but allows us to take
    /// advantage of some properties of chunk-based [`ExprLinter`]s for cache optimization.
    pub fn add_chunk_expr_linter(
        &mut self,
        name: impl AsRef<str>,
        linter: impl ExprLinter<Unit = Chunk> + 'static,
    ) -> bool {
        if self.contains_key(&name) {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![name.as_ref().to_string()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(name.as_ref().to_string());
            }
            false
        } else {
            self.chunk_expr_linters
                .insert(name.as_ref().to_string(), Box::new(linter) as _);
            true
        }
    }

    /// Add a sentence-based [`ExprLinter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    pub fn add_sentence_expr_linter(
        &mut self,
        name: impl AsRef<str>,
        linter: impl ExprLinter<Unit = Sentence> + 'static,
    ) -> bool {
        if self.contains_key(&name) {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![name.as_ref().to_string()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(name.as_ref().to_string());
            }
            false
        } else {
            self.sentence_expr_linters
                .insert(name.as_ref().to_string(), Box::new(linter) as _);
            true
        }
    }

    /// Merge the contents of another [`LintGroup`] into this one.
    pub fn merge_from(&mut self, other: LintGroup) {
        self.config.merge_from(other.config);

        if let Some((conflicting_key, _)) = other.linters.iter().find(|(k, _)| self.contains_key(k))
        {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![conflicting_key.clone()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(conflicting_key.clone());
            }
        }
        self.linters.extend(other.linters);

        if let Some((conflicting_key, _)) = other
            .chunk_expr_linters
            .iter()
            .find(|(k, _)| self.contains_key(k))
        {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![conflicting_key.clone()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(conflicting_key.clone());
            }
        }
        self.chunk_expr_linters.extend(other.chunk_expr_linters);

        if let Some((conflicting_key, _)) = other
            .sentence_expr_linters
            .iter()
            .find(|(k, _)| self.contains_key(k))
        {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![conflicting_key.clone()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(conflicting_key.clone());
            }
        }
        self.sentence_expr_linters
            .extend(other.sentence_expr_linters);
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &str> {
        self.linters
            .keys()
            .chain(self.chunk_expr_linters.keys())
            .chain(self.sentence_expr_linters.keys())
            .map(|v| v.as_str())
    }

    /// Set all contained rules to a specific value.
    /// Passing `None` will unset that rule, allowing it to assume its default state.
    pub fn set_all_rules_to(&mut self, enabled: Option<bool>) {
        let keys = self.iter_keys().map(|v| v.to_string()).collect::<Vec<_>>();

        for key in keys {
            match enabled {
                Some(v) => self.config.set_rule_enabled(key, v),
                None => self.config.unset_rule_enabled(key),
            }
        }
    }

    /// Get map from each contained linter's name to its associated description.
    pub fn all_descriptions(&self) -> HashMap<&str, &str> {
        self.linters
            .iter()
            .map(|(key, value)| (key.as_str(), value.description()))
            .chain(
                self.chunk_expr_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), ExprLinter::description(value))),
            )
            .chain(
                self.sentence_expr_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), ExprLinter::description(value))),
            )
            .collect()
    }

    /// Get map from each contained linter's name to its associated description, rendered to HTML.
    pub fn all_descriptions_html(&self) -> HashMap<&str, String> {
        self.linters
            .iter()
            .map(|(key, value)| (key.as_str(), value.description_html()))
            .chain(
                self.chunk_expr_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.description_html())),
            )
            .chain(
                self.sentence_expr_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.description_html())),
            )
            .collect()
    }

    /// Swap out [`Self::config`] with another [`FlatConfig`].
    pub fn with_lint_config(mut self, config: FlatConfig) -> Self {
        self.config = config;
        self
    }

    /// Create the curated group of rules for `language`.
    pub fn new_curated_for_language(
        dictionary: Arc<impl Dictionary + 'static>,
        language: Language,
    ) -> Self {
        match language {
            Language::English(dialect) => Self::new_curated(dictionary, dialect),
        }
    }

    pub fn new_curated(
        dictionary: Arc<impl Dictionary + 'static>,
        dialect: EnglishDialect,
    ) -> Self {
        super::en::new_curated(dictionary, dialect)
    }

    /// Create a new curated group with all config values cleared out.
    pub fn new_curated_empty_config(
        dictionary: Arc<impl Dictionary + 'static>,
        dialect: EnglishDialect,
    ) -> Self {
        let mut group = Self::new_curated(dictionary, dialect);
        group.config.clear();
        group
    }

    pub fn organized_lints(&mut self, document: &Document) -> BTreeMap<String, Vec<Lint>> {
        let mut results = BTreeMap::new();

        // Normal linters
        for (key, linter) in &mut self.linters {
            if self.config.is_rule_enabled(key) {
                results.insert(key.to_owned(), linter.lint(document));
            }
        }

        let mut insert_items_to_results =
            |items: &BTreeMap<String, Vec<Lint>>, chunk_span: Span<char>| {
                for (key, vec) in items.iter() {
                    results
                        .entry(key.to_owned())
                        .or_default()
                        .extend(vec.iter().cloned().map(|mut lint| {
                            // Bring the spans back into document-space
                            lint.span.push_by(chunk_span.start);
                            lint
                        }));
                }
            };

        // Expr linters
        for chunk in document.iter_chunks() {
            let Some(chunk_span) = chunk.span() else {
                continue;
            };

            let chunk_chars = document.get_span_content(&chunk_span);
            let config_hash = self.hasher_builder.hash_one(&self.config);
            let char_hash = self.hasher_builder.hash_one(chunk_chars);
            let cache_key = (char_hash, config_hash);

            if let Some(hit) = self.chunk_expr_cache.get(&cache_key) {
                insert_items_to_results(hit, chunk_span);
            } else {
                let mut pattern_lints = BTreeMap::new();

                for (key, linter) in &mut self.chunk_expr_linters {
                    if self.config.is_rule_enabled(key) {
                        let lints =
                            run_on_chunk(linter, chunk, document.get_source()).map(|mut l| {
                                l.span.pull_by(chunk_span.start);
                                l
                            });

                        pattern_lints.insert(key.clone(), lints.collect());
                    }
                }

                self.chunk_expr_cache.put(cache_key, pattern_lints.clone());
                insert_items_to_results(&pattern_lints, chunk_span);
            }
        }

        // Sentence Expr linters
        for sentence in document.iter_sentences() {
            let Some(sentence_span) = sentence.span() else {
                continue;
            };

            let sentence_chars = document.get_span_content(&sentence_span);
            let config_hash = self.hasher_builder.hash_one(&self.config);
            let char_hash = self.hasher_builder.hash_one(sentence_chars);
            let cache_key = (char_hash, config_hash);

            if let Some(hit) = self.sentence_expr_cache.get(&cache_key) {
                insert_items_to_results(hit, sentence_span);
            } else {
                let mut pattern_lints = BTreeMap::new();

                for (key, linter) in &mut self.sentence_expr_linters {
                    if self.config.is_rule_enabled(key) {
                        let lints =
                            run_on_chunk(linter, sentence, document.get_source()).map(|mut l| {
                                l.span.pull_by(sentence_span.start);
                                l
                            });

                        pattern_lints.insert(key.clone(), lints.collect());
                    }
                }

                self.sentence_expr_cache
                    .put(cache_key, pattern_lints.clone());
                insert_items_to_results(&pattern_lints, sentence_span);
            }
        }

        results
    }
}

impl Default for LintGroup {
    fn default() -> Self {
        Self::empty()
    }
}

impl Linter for LintGroup {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.organized_lints(document)
            .into_values()
            .flatten()
            .collect()
    }

    fn description(&self) -> &str {
        "A collection of linters that can be run as one."
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::LintGroup;
    use crate::linting::LintKind;
    use crate::linting::pooled_linter::for_tests::create_test_pool;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};
    use crate::spell::{FstDictionary, MutableDictionary};
    use crate::weir::WeirLinter;
    use crate::{Document, EnglishDialect, Language, linting::Linter};

    create_test_pool!(
        LintGroup,
        LintGroup,
        LintGroup::new_curated(FstDictionary::curated(), EnglishDialect::American)
    );

    #[test]
    fn clean_interjection() {
        assert_no_lints(
            "Although I only saw the need to interject once, I still saw it.",
            test_linter(),
        );
    }

    #[test]
    fn clean_consensus() {
        assert_no_lints("But there is less consensus on this.", test_linter());
    }

    #[test]
    fn ive_corrects_to_single_word() {
        assert_suggestion_result(
            "ive never seen that before",
            test_linter(),
            "I've never seen that before",
        );
    }

    #[test]
    fn worthchecking_is_split() {
        assert_suggestion_result("It is worthchecking", test_linter(), "It is worth checking");
    }

    #[test]
    fn its_not_perfect_keeps_apostrophe() {
        assert_no_lints("It's not perfect", test_linter());
    }

    #[test]
    fn corrects_extention() {
        let group = test_linter();
        let document = Document::new_plain_english_curated("I love this extention!");
        let organized = group.run_with_inner(|l| l.organized_lints(&document));

        let spellcheck_lints = organized
            .get("SpellCheck")
            .expect("SpellCheck should produce a lint for extention");
        assert_eq!(spellcheck_lints.len(), 1);
        assert!(
            spellcheck_lints[0]
                .suggestions
                .iter()
                .any(|suggestion| suggestion.to_string() == "Replace with: “extension”")
        );

        assert!(
            organized.get("SplitWords").is_none_or(Vec::is_empty),
            "expected no lints from SplitWords, but found {:?}",
            organized.get("SplitWords")
        );
    }

    #[test]
    fn ok_becomes_okay() {
        assert_suggestion_result("This is ok.", test_linter(), "This is okay.");
    }

    #[test]
    fn weir_linter_uses_configured_sentence_scope() {
        let source = r#"
            expr main one**two
            let message "Use three."
            let description "Test sentence-scoped Weir."
            let kind "Miscellaneous"
            let becomes "three"
            let strategy "Exact"
            let scope "Sentence"
        "#;

        let mut group = LintGroup::empty();
        group.add_sentence_expr_linter(
            "TestSentenceWeir",
            WeirLinter::new(source)
                .unwrap()
                .into_sentence_linter()
                .unwrap_or_else(|_| unreachable!()),
        );
        group.config.set_rule_enabled("TestSentenceWeir", true);

        assert_suggestion_result("one, two.", group, "three.");
    }

    #[test]
    fn can_get_all_descriptions() {
        let group = LintGroup::new_curated(
            Arc::new(MutableDictionary::default()),
            EnglishDialect::American,
        );
        group.all_descriptions();
    }

    #[test]
    fn can_get_all_descriptions_as_html() {
        let group = LintGroup::new_curated(
            Arc::new(MutableDictionary::default()),
            EnglishDialect::American,
        );
        group.all_descriptions_html();
    }

    #[test]
    fn dont_flag_low_hanging_fruit_msg() {
        assert_no_lints(
            "The standard form is low-hanging fruit with a hyphen and singular form.",
            test_linter(),
        );
    }

    #[test]
    fn dont_flag_low_hanging_fruit_desc() {
        assert_no_lints(
            "Corrects nonstandard variants of low-hanging fruit.",
            test_linter(),
        );
    }

    #[test]
    fn fix_stoped() {
        assert_suggestion_result("He stoped to think.", test_linter(), "He stopped to think.");
    }

    #[test]
    fn allow_playthrough() {
        assert_no_lints("I recently did a playthrough.", test_linter());
    }

    #[test]
    fn fixes_politicans() {
        assert_suggestion_result(
            "I guess politicans like to complain.",
            test_linter(),
            "I guess politicians like to complain.",
        );
    }

    #[test]
    fn fixes_benefitting() {
        assert_suggestion_result(
            "Who is really benefitting from it?",
            test_linter(),
            "Who is really benefiting from it?",
        );
    }

    #[test]
    fn fixes_extention_correctly() {
        assert_suggestion_result(
            "There is no extention quite like this one.",
            test_linter(),
            "There is no extention quite like this one.",
        );
    }

    #[test]
    fn fixes_easir() {
        assert_suggestion_result(
            "It makes it easir to select it.",
            test_linter(),
            "It makes it easier to select it.",
        );
    }

    #[test]
    fn fixes_breakfest() {
        assert_suggestion_result(
            "Ice cream for breakfest?",
            test_linter(),
            "Ice cream for breakfast?",
        );
    }

    #[test]
    fn fixes_buget() {
        assert_suggestion_result(
            "It was in their buget range.",
            test_linter(),
            "It was in their budget range.",
        );
    }

    #[test]
    fn allows_chest_compressions() {
        assert_no_lints("Please continue chest compressions.", test_linter());
    }

    /// Tests that no linters' descriptions contain errors handled by other linters.
    ///
    /// This test verifies that the description of each linter (which is written in natural language)
    /// doesn't trigger any other linter's rules, with the exception of certain linters that
    /// suggest mere alternatives rather than flagging actual errors.
    ///
    /// For example, we disable the "MoreAdjective" linter since some comparative and superlative
    /// adjectives can be more awkward than their two-word counterparts, even if technically correct.
    ///
    /// If this test fails, it means either:
    /// 1. A linter's description contains an actual error that should be fixed, or
    /// 2. A linter is being too aggressive in flagging text that is actually correct English
    ///    in the context of another linter's description.
    #[test]
    fn lint_descriptions_are_clean() {
        let lints_to_check =
            LintGroup::new_curated(FstDictionary::curated(), EnglishDialect::American);

        let name_description_pairs: Vec<_> = lints_to_check
            .all_descriptions()
            .into_iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect();

        name_description_pairs
            .iter()
            .for_each(|(lint_name, description)| {
                let doc = Document::new_markdown_default_curated(&description);
                eprintln!("{lint_name}: {description}");

                let mut lints = test_linter().lint(&doc);

                // Remove ones related to style
                lints.retain(|l| l.lint_kind != LintKind::Style);

                if !lints.is_empty() {
                    panic!("🚨 Lint description for '{}' contains errors", lint_name);
                }
            });
    }

    #[test]
    fn no_linter_names_clash() {
        let group = LintGroup::new_curated(
            Arc::new(MutableDictionary::default()),
            EnglishDialect::American,
        );

        if let Some(names) = &group.clashing_linter_names {
            if !names.is_empty() {
                panic!(
                    "⚠️ Found {} clashing linter names: {}",
                    names.len(),
                    names.join(", ")
                );
            }
        }
    }

    #[test]
    fn spellcheck_beats_split_words_for_advices_4346() {
        assert_suggestion_result(
            "IMO these kind of advices never matters.",
            test_linter(),
            "IMO these kind of advice never matters.",
        );
    }

    #[test]
    fn english_group_matches_new_curated() {
        let dict = FstDictionary::curated();
        let english = LintGroup::new_curated_for_language(
            dict.clone(),
            Language::English(EnglishDialect::British),
        );
        let curated = LintGroup::new_curated(dict, EnglishDialect::British);
        assert_eq!(english.config, curated.config);
    }
}

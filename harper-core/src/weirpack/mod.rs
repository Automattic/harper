use std::io::{Read, Write};
use std::path::Path;

use hashbrown::HashMap;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::linting::LintGroup;
use crate::weir::{TestResult, WeirLinter};

mod error;
mod manifest;

pub use error::Error;
pub use manifest::WeirpackManifest;

/// A Weirpack, which carries within itself one or more rules to be used for grammar checking.
/// These rules are written in Weir.
#[derive(Debug, Clone, Default)]
pub struct Weirpack {
    pub rules: HashMap<String, String>,
    pub manifest: WeirpackManifest,
}

impl Weirpack {
    /// Create an empty Weirpack.
    pub fn new(manifest: WeirpackManifest) -> Self {
        Self {
            rules: HashMap::new(),
            manifest,
        }
    }

    /// Add a rule to this Weirpack. Does not compile to test the rule.
    pub fn add_rule(&mut self, name: impl Into<String>, rule: impl Into<String>) -> Option<String> {
        self.rules.insert(name.into(), rule.into())
    }

    /// Remove a rule from this Weirpack.
    pub fn remove_rule(&mut self, name: &str) -> Option<String> {
        self.rules.remove(name)
    }

    /// Run all the tests within all the Weir rules in this Weirpack.
    pub fn run_tests(&self) -> Result<HashMap<String, Vec<TestResult>>, Error> {
        let mut failures = HashMap::new();

        for (name, rule) in &self.rules {
            let mut linter = WeirLinter::new(rule)?;
            let failing_tests = linter.run_tests();
            if !failing_tests.is_empty() {
                failures.insert(name.to_string(), failing_tests);
            }
        }

        Ok(failures)
    }

    /// Parse and optimize the Weir rules in the pack, converting the set into a single [`LintGroup`].
    /// Does not run tests.
    pub fn to_lint_group(&self) -> Result<LintGroup, Error> {
        let mut group = LintGroup::default();

        for (name, rule) in &self.rules {
            let linter = WeirLinter::new(rule)?;
            group.add_chunk_expr_linter(name, linter);
            group.config.set_rule_enabled(name, true);
        }

        Ok(group)
    }

    /// Load a Weirpack from bytes.
    pub fn from_reader(mut reader: impl Read) -> Result<Self, Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    /// Write the Weirpack to bytes.
    pub fn write_to(&self, mut writer: impl Write) -> Result<(), Error> {
        let bytes = self.to_bytes()?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Load a Weirpack from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let mut manifest = None;
        let mut rules = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.is_dir() {
                continue;
            }

            let name = file.name().to_string();
            if name == "manifest.json" {
                if manifest.is_some() {
                    return Err(Error::DuplicateManifest("manifest.json"));
                }
                let manifest_data = WeirpackManifest::from_reader(&mut file)?;
                manifest = Some(manifest_data);
                continue;
            }

            if !name.ends_with(".weir") {
                continue;
            }

            let path = Path::new(&name);
            let file_name = path
                .file_name()
                .and_then(|segment| segment.to_str())
                .ok_or_else(|| Error::InvalidRuleFileName(name.clone()))?;
            let rule_name = Path::new(file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| Error::InvalidRuleFileName(name.clone()))?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            rules.insert(rule_name.to_string(), contents);
        }

        let manifest = manifest.ok_or(Error::MissingManifest("manifest.json"))?;

        Ok(Self { rules, manifest })
    }

    /// Write a Weirpack into bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut zip = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

        let mut manifest_bytes = Vec::new();
        self.manifest.write_to(&mut manifest_bytes)?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(&manifest_bytes)?;

        let mut rule_names: Vec<_> = self.rules.keys().collect();
        rule_names.sort();

        for rule_name in rule_names {
            let file_name = format!("{rule_name}.weir");
            zip.start_file(file_name, options)?;
            if let Some(rule) = self.rules.get(rule_name) {
                zip.write_all(rule.as_bytes())?;
            }
        }

        let cursor = zip.finish()?;
        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::{Weirpack, WeirpackManifest};

    #[test]
    fn round_trip_weirpack_bytes() {
        let mut manifest = WeirpackManifest::new();
        manifest.set_author("Test Author");
        manifest.set_version("0.1.0");
        manifest.set_description("Test pack");
        manifest.set_license("MIT");

        let mut pack = Weirpack::new(manifest);
        pack.add_rule("ExampleRule", "expr main test");

        let bytes = pack.to_bytes().expect("serialize weirpack");
        let parsed = Weirpack::from_bytes(&bytes).expect("deserialize weirpack");

        assert_eq!(parsed.manifest.author().unwrap(), "Test Author");
        assert_eq!(parsed.manifest.version().unwrap(), "0.1.0");
        assert_eq!(parsed.manifest.description().unwrap(), "Test pack");
        assert_eq!(parsed.manifest.license().unwrap(), "MIT");
        assert_eq!(parsed.rules.get("ExampleRule").unwrap(), "expr main test");
    }
}

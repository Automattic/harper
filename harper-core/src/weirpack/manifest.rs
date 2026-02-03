use std::io::{Read, Write};

use hashbrown::HashMap;
use serde_json::Value;

use super::WeirpackError;

#[derive(Debug, Clone, Default)]
pub struct WeirpackManifest {
    data: HashMap<String, Value>,
}

impl WeirpackManifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_reader(reader: impl Read) -> Result<Self, WeirpackError> {
        let data: HashMap<String, Value> = serde_json::from_reader(reader)?;
        let manifest = Self { data };
        manifest.validate_required()?;
        Ok(manifest)
    }

    pub fn write_to(&self, writer: impl Write) -> Result<(), WeirpackError> {
        self.validate_required()?;
        serde_json::to_writer_pretty(writer, &self.data)?;
        Ok(())
    }

    pub fn get_field(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn set_field(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
    }

    pub fn author(&self) -> Result<&str, WeirpackError> {
        self.required_str("author")
    }

    pub fn set_author(&mut self, value: impl Into<String>) {
        self.set_field("author", Value::String(value.into()));
    }

    pub fn version(&self) -> Result<&str, WeirpackError> {
        self.required_str("version")
    }

    pub fn set_version(&mut self, value: impl Into<String>) {
        self.set_field("version", Value::String(value.into()));
    }

    pub fn description(&self) -> Result<&str, WeirpackError> {
        self.required_str("description")
    }

    pub fn set_description(&mut self, value: impl Into<String>) {
        self.set_field("description", Value::String(value.into()));
    }

    pub fn license(&self) -> Result<&str, WeirpackError> {
        self.required_str("license")
    }

    pub fn set_license(&mut self, value: impl Into<String>) {
        self.set_field("license", Value::String(value.into()));
    }

    fn required_str(&self, key: &'static str) -> Result<&str, WeirpackError> {
        match self.data.get(key) {
            Some(Value::String(value)) => Ok(value),
            Some(_) => Err(WeirpackError::InvalidManifestFieldType(key)),
            None => Err(WeirpackError::MissingManifestField(key)),
        }
    }

    fn validate_required(&self) -> Result<(), WeirpackError> {
        self.author()?;
        self.version()?;
        self.description()?;
        self.license()?;
        Ok(())
    }
}

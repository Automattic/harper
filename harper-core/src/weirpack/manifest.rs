use std::io::{Read, Write};

use hashbrown::HashMap;
use paste::paste;
use serde_json::Value;

use super::Error;

#[derive(Debug, Clone, Default)]
pub struct WeirpackManifest {
    data: HashMap<String, Value>,
}

macro_rules! gen_fns {
    ($field:ident) => {
        paste! {
            #[doc = concat!("Get the ", stringify!($field), " field from the manifest.")]
            pub fn $field(&self) -> Result<&str, Error>{
                self.required_str(stringify!($field))
            }

            #[doc = concat!("Set the ", stringify!($field), " field in the manifest.")]
            pub fn [< set_ $field >](&mut self, value: impl Into<String>) {
                self.set_field(stringify!($field), Value::String(value.into()));
            }
        }
    };
}

impl WeirpackManifest {
    /// Create a new, empty manifest.
    /// Note that an empty manifest is an invalid manifest.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a manifest from some bytes.
    pub fn from_reader(reader: impl Read) -> Result<Self, Error> {
        let data: HashMap<String, Value> = serde_json::from_reader(reader)?;
        let manifest = Self { data };
        manifest.validate_required()?;
        Ok(manifest)
    }

    /// Write a manifest to some bytes.
    pub fn write_to(&self, writer: impl Write) -> Result<(), Error> {
        self.validate_required()?;
        serde_json::to_writer_pretty(writer, &self.data)?;
        Ok(())
    }

    /// Get an arbitrary field from the manifest.
    pub fn get_field(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Set an arbitrary field from the manifest.
    pub fn set_field(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
    }

    gen_fns!(author);
    gen_fns!(version);
    gen_fns!(description);
    gen_fns!(license);

    fn required_str(&self, key: &'static str) -> Result<&str, Error> {
        match self.data.get(key) {
            Some(Value::String(value)) => Ok(value),
            Some(_) => Err(Error::InvalidManifestFieldType(key)),
            None => Err(Error::MissingManifestField(key)),
        }
    }

    fn validate_required(&self) -> Result<(), Error> {
        self.author()?;
        self.version()?;
        self.description()?;
        self.license()?;
        Ok(())
    }
}

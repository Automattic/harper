use std::{borrow::Cow, path::PathBuf};

use enum_dispatch::enum_dispatch;
use strum_macros::EnumTryAs;

use crate::input::single_input::{FileInput, SingleInput};

use super::InputTrait;

#[enum_dispatch]
pub(crate) trait MultiInputTrait: InputTrait {
    /// Get an iterator of [`SingleInput`] from this `MultiInput`.
    ///
    /// For instance, if this is a directory input, the returned inputs might correspond to the
    /// files inside that directory.
    #[allow(dead_code)]
    fn iter_inputs(&self) -> anyhow::Result<impl Iterator<Item = SingleInput>>;
}

#[derive(Clone, EnumTryAs)]
#[enum_dispatch(MultiInputTrait)]
pub(crate) enum MultiInput {
    /// A directory.
    Dir(DirInput),
}
impl MultiInput {
    /// Try to parse a `MultiInput` from the provided string. This might fail if the provided
    /// string cannot be parsed as a supported `MultiInput`.
    pub(crate) fn try_parse_string(input_string: &str) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(input_string);
        if metadata?.is_dir() {
            // Input is a valid directory path.
            Ok(Self::Dir(DirInput {
                path: input_string.into(),
            }))
        } else {
            anyhow::bail!(
                "Unsupported input '{}' for {}",
                input_string,
                std::any::type_name::<Self>()
            )
        }
    }
}

impl InputTrait for MultiInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        match self {
            MultiInput::Dir(input) => input.get_identifier(),
        }
    }
}

/// A directory.
#[derive(Clone)]
pub(crate) struct DirInput {
    /// The path pointing to the directory.
    path: PathBuf,
}
impl DirInput {
    /// An iterator of the files inside the directory, as [`FileInput`].
    pub(crate) fn iter_files(&self) -> anyhow::Result<impl Iterator<Item = FileInput>> {
        Ok(std::fs::read_dir(&self.path)?.filter_map(|dir_entry| {
            if let Ok(dir_entry) = dir_entry
                && let Ok(file) = FileInput::try_from_path(&dir_entry.path())
            {
                Some(file)
            } else {
                None
            }
        }))
    }
}
impl MultiInputTrait for DirInput {
    fn iter_inputs(&self) -> anyhow::Result<impl Iterator<Item = SingleInput>> {
        Ok(self.iter_files()?.map(|file| file.into()))
    }
}
impl InputTrait for DirInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        self.path
            .file_name()
            .map_or(Cow::from("<dir>"), |dir_name| dir_name.to_string_lossy())
    }
}

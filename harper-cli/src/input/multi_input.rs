use std::{borrow::Cow, path::PathBuf};

use enum_dispatch::enum_dispatch;
use strum_macros::EnumTryAs;

use crate::input::single_input::{FileInput, SingleInput};

use super::InputTrait;

#[enum_dispatch]
pub(crate) trait MultiInputTrait: InputTrait {
    #[allow(dead_code)]
    fn iter_inputs(&self) -> anyhow::Result<Box<dyn Iterator<Item = SingleInput> + '_>>;
}

#[derive(Clone, EnumTryAs)]
#[enum_dispatch(MultiInputTrait)]
pub(crate) enum MultiInput {
    Dir(DirInput),
}
impl MultiInput {
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

#[derive(Clone)]
pub(crate) struct DirInput {
    path: PathBuf,
}
impl DirInput {
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
    fn iter_inputs(&self) -> anyhow::Result<Box<dyn Iterator<Item = SingleInput> + '_>> {
        Ok(Box::new(self.iter_files()?.map(|file| file.into())))
    }
}
impl InputTrait for DirInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        self.path
            .file_name()
            .map_or(Cow::from("<dir>"), |dir_name| dir_name.to_string_lossy())
    }
}

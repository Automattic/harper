use std::path::Path;
use std::{borrow::Cow, io::Read, path::PathBuf};

use enum_dispatch::enum_dispatch;
use strum_macros::EnumTryAs;

use harper_comments::CommentParser;
use harper_core::parsers::{Markdown, OrgMode, Parser};
use harper_core::spell::Dictionary;
use harper_core::{
    Document,
    parsers::{MarkdownOptions, PlainEnglish},
};
use harper_ink::InkParser;
use harper_literate_haskell::LiterateHaskellParser;
use harper_python::PythonParser;

use super::InputTrait;

/// An input of a single source. This would not include a directory for instance, which may have
/// multiple (file) Sources.
#[enum_dispatch]
pub(crate) trait SingleInputTrait: InputTrait {
    /// Loads the contained file/string into a conventional format. Returns a `Result` containing
    /// a tuple of a `Document` and its corresponding source text as a string.
    fn load(
        &self,
        markdown_options: MarkdownOptions,
        dictionary: &dyn Dictionary,
    ) -> anyhow::Result<(Document, Cow<'_, str>)> {
        let text = self.get_content()?;
        let parser = self.get_parser(markdown_options);
        Ok((Document::new(&text, &parser, &dictionary), text))
    }

    fn get_parser(&self, _markdown_options: MarkdownOptions) -> Box<dyn Parser> {
        Box::new(PlainEnglish)
    }

    fn get_content(&self) -> anyhow::Result<Cow<'_, str>>;
}

#[derive(Clone, EnumTryAs)]
#[enum_dispatch(SingleInputTrait)]
pub(crate) enum SingleInput {
    File(FileInput),
    Text(TextInput),
    Stdin(StdinInput),
}
impl SingleInput {
    pub(crate) fn parse_string(input_string: &str) -> Self {
        if let Ok(file) = FileInput::try_from_path(Path::new(input_string)) {
            // Input is a valid filepath.
            Self::File(file)
        } else {
            // Input is not a valid filepath, we assume it's intended to be a string.
            Self::Text(TextInput {
                text: input_string.to_owned(),
            })
        }
    }
}
impl InputTrait for SingleInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        match self {
            SingleInput::File(input) => input.get_identifier(),
            SingleInput::Text(input) => input.get_identifier(),
            SingleInput::Stdin(input) => input.get_identifier(),
        }
    }
}

pub trait SingleInputOptionExt {
    fn unwrap_or_read_from_stdin(self) -> SingleInput;
}
impl SingleInputOptionExt for Option<SingleInput> {
    fn unwrap_or_read_from_stdin(self) -> SingleInput {
        self.unwrap_or_else(|| SingleInput::from(StdinInput))
    }
}

/// File (path) input.
#[derive(Clone)]
pub(crate) struct FileInput {
    path: PathBuf,
}
impl FileInput {
    pub(crate) fn path(&self) -> &PathBuf {
        &self.path
    }
    pub(crate) fn try_from_path(path: &Path) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(path);
        if metadata?.is_file() {
            Ok(Self::from_path_unchecked(path))
        } else {
            anyhow::bail!(
                "Failed to parse '{}' as {}",
                path.to_string_lossy(),
                std::any::type_name::<Self>()
            )
        }
    }
    pub(crate) fn from_path_unchecked(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}
impl SingleInputTrait for FileInput {
    fn get_content(&self) -> anyhow::Result<Cow<'_, str>> {
        Ok(std::fs::read_to_string(&self.path)?.into())
    }
    fn get_parser(&self, _markdown_options: MarkdownOptions) -> Box<dyn Parser> {
        match self.path.extension().map(|ext| ext.to_str().unwrap()) {
            Some("md") => Box::new(Markdown::default()),
            Some("ink") => Box::new(InkParser::default()),
            Some("lhs") => Box::new(LiterateHaskellParser::new_markdown(
                MarkdownOptions::default(),
            )),
            Some("org") => Box::new(OrgMode),
            Some("typ") => Box::new(harper_typst::Typst),
            Some("py") | Some("pyi") => Box::new(PythonParser::default()),
            Some("txt") => Box::new(PlainEnglish),
            _ => {
                if let Some(comment_parser) =
                    CommentParser::new_from_filename(&self.path, _markdown_options)
                {
                    Box::new(comment_parser)
                } else {
                    eprintln!(
                        "{}: Warning: Could not detect language ID; falling back to PlainEnglish parser.",
                        self.get_identifier()
                    );
                    Box::new(PlainEnglish)
                }
            }
        }
    }
}
impl InputTrait for FileInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        self.path
            .file_name()
            .map_or(Cow::from("<file>"), |file_name| file_name.to_string_lossy())
    }
}

/// Direct text input via the command line.
#[derive(Clone)]
pub(crate) struct TextInput {
    text: String,
}
impl SingleInputTrait for TextInput {
    fn get_content(&self) -> anyhow::Result<Cow<'_, str>> {
        Ok(Cow::from(&self.text))
    }
}
impl InputTrait for TextInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        Cow::from("<text>")
    }
}

/// Standard input (stdin).
#[derive(Clone)]
pub(crate) struct StdinInput;
impl SingleInputTrait for StdinInput {
    fn get_content(&self) -> anyhow::Result<Cow<'_, str>> {
        let mut buf = String::new();
        std::io::stdin().lock().read_to_string(&mut buf)?;
        Ok(Cow::from(buf))
    }
}
impl InputTrait for StdinInput {
    fn get_identifier(&self) -> Cow<'_, str> {
        Cow::from("<stdin>")
    }
}

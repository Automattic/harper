use std::path::Path;
use std::{borrow::Cow, io::Read, path::PathBuf};

use harper_comments::CommentParser;
use harper_core::parsers::{Markdown, OrgMode};
use harper_core::spell::Dictionary;
use harper_core::{
    Document,
    parsers::{MarkdownOptions, PlainEnglish},
};
use harper_ink::InkParser;
use harper_literate_haskell::LiterateHaskellParser;
use harper_python::PythonParser;

/// Represents an input/source passed via the command line. For example, this can be a file, or
/// text passed via the command line directly.
#[derive(Clone, Debug)]
pub(super) enum Input {
    /// File (path) input.
    File(PathBuf),
    /// Directory (path) input.
    Dir(PathBuf),
    /// Direct text input, via the command line.
    Text(String),
    /// Standard input (stdin).
    Stdin(String),
}
impl Input {
    /// Loads the contained file/string into a conventional format. Returns a `Result` containing
    /// a tuple of a `Document` and its corresponding source text as a string.
    pub(super) fn load(
        &self,
        batch_mode: bool,
        markdown_options: MarkdownOptions,
        dictionary: &impl Dictionary,
    ) -> anyhow::Result<(Option<Document>, String)> {
        match self {
            Input::File(file) => Self::load_file(
                file,
                Some(&self.get_identifier()),
                batch_mode,
                markdown_options,
                dictionary,
            ),
            Input::Dir(dir) => Ok((None, dir.display().to_string())),
            Input::Text(s) => Ok((Some(Document::new(s, &PlainEnglish, dictionary)), s.clone())),
            Input::Stdin(s) => Ok((Some(Document::new(s, &PlainEnglish, dictionary)), s.clone())),
        }
    }

    /// Gets a human-readable identifier for the input. For example, this can be a filename, or
    /// simply the string `"<input>"`.
    #[must_use]
    pub(super) fn get_identifier(&'_ self) -> Cow<'_, str> {
        match self {
            Input::File(file) => file
                .file_name()
                .map_or(Cow::from("<file>"), |file_name| file_name.to_string_lossy()),
            Input::Dir(dir) => dir
                .file_name()
                .map_or(Cow::from("<dir>"), |dir_name| dir_name.to_string_lossy()),
            Input::Text(_) => Cow::from("<text>"),
            Input::Stdin(_) => Cow::from("<stdin>"),
        }
    }

    /// Tries to construct an `Input` by reading standard input. This will fail if the standard
    /// input cannot be read.
    pub(super) fn try_from_stdin() -> anyhow::Result<Self> {
        let mut buf = String::new();
        std::io::stdin().lock().read_to_string(&mut buf)?;
        Ok(Self::Stdin(buf))
    }

    fn load_file(
        file: &Path,
        input_identifier: Option<&str>,
        batch_mode: bool,
        markdown_options: MarkdownOptions,
        dictionary: &impl Dictionary,
    ) -> anyhow::Result<(Option<Document>, String)> {
        let source = std::fs::read_to_string(file)?;

        let parser: Box<dyn harper_core::parsers::Parser> =
            match file.extension().map(|v| v.to_str().unwrap()) {
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
                        CommentParser::new_from_filename(file, markdown_options)
                    {
                        Box::new(comment_parser)
                    } else {
                        eprintln!(
                            "{}Warning: Could not detect language ID; {}",
                            input_identifier
                                .map(|id| format!("{}: ", id))
                                .unwrap_or_default(),
                            if batch_mode {
                                "skipping file."
                            } else {
                                "falling back to PlainEnglish parser."
                            }
                        );
                        if batch_mode {
                            return Ok((None, source));
                        } else {
                            Box::new(PlainEnglish)
                        }
                    }
                }
            };

        Ok((Some(Document::new(&source, &parser, dictionary)), source))
    }
}
// This allows this type to be directly used with clap as an argument.
// https://docs.rs/clap/latest/clap/macro.value_parser.html
impl From<String> for Input {
    /// Converts the given string into an `Input`. `Input` is automatically set to the correct variant
    /// depending on whether `input_string` is a valid filepath or not.
    fn from(input_string: String) -> Self {
        let metadata = std::fs::metadata(&input_string);
        let metadata = metadata.as_ref();
        if metadata.is_ok_and(|m| m.is_file()) {
            // Input is a valid filepath.
            Self::File(input_string.into())
        } else if metadata.is_ok_and(|m| m.is_dir()) {
            // Input is a valid directory path.
            Self::Dir(input_string.into())
        } else {
            // Input is not a valid filepath, we assume it's intended to be a string.
            Self::Text(input_string)
        }
    }
}

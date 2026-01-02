use std::borrow::Cow;

use enum_dispatch::enum_dispatch;

pub mod single_input;
use single_input::SingleInput;

pub mod multi_input;
use multi_input::MultiInput;
use strum_macros::EnumTryAs;

#[enum_dispatch]
pub(crate) trait InputTrait {
    /// Gets a human-readable identifier for the input. For example, this can be a filename, or
    /// simply the string `"<input>"`.
    fn get_identifier(&self) -> Cow<'_, str>;
}

/// Represents an input/source passed via the command line. For example, this can be a file,
/// a directory, or text passed via the command line directly.
#[enum_dispatch(InputTrait)]
#[derive(Clone, EnumTryAs)]
pub(super) enum AnyInput {
    /// An input of a single source. For instance, a specific file, or input from standard input.
    Single(SingleInput),
    /// An input of multiple sources. For instance, a path to a directory.
    Multi(MultiInput),
}

// This allows this type to be directly used with clap as an argument.
// https://docs.rs/clap/latest/clap/macro.value_parser.html
impl From<String> for AnyInput {
    /// Converts the given string into an `Input`. `Input` is automatically set to the correct variant
    /// depending on whether `input_string` is a valid filepath or not.
    fn from(input_string: String) -> Self {
        if let Ok(multi_input) = MultiInput::try_parse_string(&input_string) {
            Self::Multi(multi_input)
        } else {
            Self::Single(SingleInput::parse_string(&input_string))
        }
    }
}

// This allows this type to be directly used with clap as an argument.
// It can be used in place of AnyInput if the command should only accept single-inputs
// (e.g. a file).
impl From<String> for SingleInput {
    fn from(input_string: String) -> Self {
        SingleInput::parse_string(&input_string)
    }
}

// This allows this type to be directly used with clap as an argument.
// It can be used in place of AnyInput if the command should only accept multi-inputs,
// (e.g. directories).
impl From<String> for MultiInput {
    fn from(input_string: String) -> Self {
        MultiInput::try_parse_string(&input_string).unwrap()
    }
}

use crate::input::{
    AnyInput, InputTrait,
    multi_input::MultiInput,
    single_input::{SingleInput, StdinInput},
};

/// Metadata about an expanded input.
#[derive(Clone)]
pub struct ExpandedInput {
    /// The actual input to process.
    pub input: AnyInput,
    /// Parent directory identifier (empty if not from a directory).
    pub parent_id: String,
    /// Whether this input is part of a batch (from a directory).
    pub is_batch: bool,
}

/// Expand inputs into a flat list, handling directories and stdin default.
///
/// - If `inputs` is empty, defaults to stdin
/// - Directories are expanded to their sorted file contents
/// - Non-directory inputs are passed through as-is
pub fn expand_inputs(inputs: Vec<AnyInput>) -> anyhow::Result<Vec<ExpandedInput>> {
    let inputs = if inputs.is_empty() {
        vec![SingleInput::from(StdinInput).into()]
    } else {
        inputs
    };

    let mut expanded = Vec::new();
    for input in inputs {
        if let Some(dir) = input
            .try_as_multi_ref()
            .and_then(MultiInput::try_as_dir_ref)
        {
            let mut files: Vec<_> = dir.iter_files()?.collect();
            files.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));
            let parent_id = input.get_identifier().to_string();
            for file in files {
                expanded.push(ExpandedInput {
                    input: SingleInput::from(file).into(),
                    parent_id: parent_id.clone(),
                    is_batch: true,
                });
            }
        } else {
            expanded.push(ExpandedInput {
                input,
                parent_id: String::new(),
                is_batch: false,
            });
        }
    }

    Ok(expanded)
}

/// Process expanded inputs with a callback function.
///
/// The callback receives the `ExpandedInput` and should return a `Result`.
/// Errors are printed to stderr but don't stop processing of other inputs.
pub fn process_inputs<F>(
    inputs: Vec<ExpandedInput>,
    mut f: F,
) -> anyhow::Result<()>
where
    F: FnMut(&ExpandedInput) -> anyhow::Result<()>,
{
    for input in inputs {
        if let Err(e) = f(&input) {
            eprintln!("{}", e);
        }
    }
    Ok(())
}

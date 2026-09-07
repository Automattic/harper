use rayon::prelude::*;

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

/// Process inputs and collect results, with optional parallelism.
///
/// This combines input expansion and processing into a single step.
/// The callback receives an `ExpandedInput` and returns a `Result<T>`.
/// All results are collected into a `Vec<Result<T>>`.
///
/// # Arguments
///
/// * `inputs` - The raw inputs to process
/// * `parallel` - If true, use rayon for parallel processing (when >1 input)
/// * `f` - Function to process each input, returning a result
///
/// # Returns
///
/// A vector of results, one per input. Errors in individual inputs
/// are returned as `Err` values in the vector, not propagated.
pub fn process_inputs_collect<T, F>(
    inputs: Vec<AnyInput>,
    parallel: bool,
    f: F,
) -> Vec<anyhow::Result<T>>
where
    T: Send,
    F: Fn(ExpandedInput) -> anyhow::Result<T> + Send + Sync,
{
    let expanded = match expand_inputs(inputs) {
        Ok(e) => e,
        Err(e) => return vec![Err(e)],
    };

    if parallel && expanded.len() > 1 {
        expanded.into_par_iter().map(f).collect()
    } else {
        expanded.into_iter().map(f).collect()
    }
}

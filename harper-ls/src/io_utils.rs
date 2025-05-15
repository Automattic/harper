use anyhow::anyhow;
use std::path::{Component, PathBuf};

use tower_lsp::lsp_types::Url;

/// Rewrites a path to a filename using the same conventions as
/// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
pub fn fileify_path(url: &Url) -> anyhow::Result<PathBuf> {
    let mut rewritten = String::new();

    // We assume all URLs are local files and have a base.
    for seg in url
        .to_file_path()
        .map_err(|_| anyhow!("Unable to convert URL to file path."))?
        .components()
    {
        if !matches!(seg, Component::RootDir) {
            rewritten.push_str(&seg.as_os_str().to_string_lossy());
            rewritten.push('%');
        }
    }

    Ok(rewritten.into())
}

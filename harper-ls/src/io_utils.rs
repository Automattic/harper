use anyhow::anyhow;
use std::path::{Component, PathBuf};

use tower_lsp_server::{UriExt, lsp_types::Uri};

/// Rewrites a path to a filename using the same conventions as
/// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
pub fn fileify_path(uri: &Uri) -> anyhow::Result<PathBuf> {
    let mut rewritten = String::new();

    // We assume all URLs are local files and have a base.
    for seg in uri
        .to_file_path()
        .ok_or_else(|| anyhow!("Unable to convert URI to file path."))?
        .components()
    {
        if !matches!(seg, Component::RootDir) {
            rewritten.push_str(&seg.as_os_str().to_string_lossy());
            rewritten.push('%');
        }
    }

    Ok(rewritten.into())
}

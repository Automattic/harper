use anyhow::anyhow;
use itertools::Itertools;
use std::path::{Component, Path, PathBuf};

use harper_core::{Dictionary, MutableDictionary, WordMetadata};
use tokio::fs::{self, File};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, Result};
use tower_lsp::lsp_types::Url;

/// Save the contents of a dictionary to a file.
/// Ensures that the path to the destination exists.
pub async fn save_dict(path: impl AsRef<Path>, dict: impl Dictionary) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).await?;
    }

    let file = File::create(path.as_ref()).await?;
    let mut write = BufWriter::new(file);

    write_word_list(dict, &mut write).await?;
    write.flush().await?;

    Ok(())
}

/// Write a dictionary somewhere.
async fn write_word_list(dict: impl Dictionary, mut w: impl AsyncWrite + Unpin) -> Result<()> {
    let mut cur_str = String::new();

    for word in dict.words_iter().sorted() {
        cur_str.clear();
        cur_str.extend(word);

        w.write_all(cur_str.as_bytes()).await?;
        w.write_all(b"\n").await?;
    }

    Ok(())
}

/// Load a dictionary from a file on disk.
pub async fn load_dict(path: impl AsRef<Path>) -> Result<MutableDictionary> {
    let file = File::open(path.as_ref()).await?;
    let read = BufReader::new(file);

    dict_from_word_list(read).await
}

/// Load a dictionary from a list of words.
/// It could definitely be optimized to use less memory.
/// Right now it isn't an issue.
async fn dict_from_word_list(mut r: impl AsyncRead + Unpin) -> Result<MutableDictionary> {
    let mut str = String::new();

    r.read_to_string(&mut str).await?;

    let mut dict = MutableDictionary::new();
    dict.extend_words(
        str.lines()
            .map(|l| (l.chars().collect::<Vec<char>>(), WordMetadata::default())),
    );

    Ok(dict)
}

/// Rewrites a path to a filename using the same conventions as
/// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
pub fn file_dict_name(url: &Url) -> anyhow::Result<PathBuf> {
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

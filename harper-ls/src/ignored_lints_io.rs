use std::path::Path;

use anyhow::Result;
use harper_core::IgnoredLints;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

/// Save the contents of a dictionary to a file.
/// Ensures that the path to the destination exists.
pub async fn save_ignored_lints(
    path: impl AsRef<Path>,
    ignored_lints: &IgnoredLints,
) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).await?;
    }

    let file = File::create(path.as_ref()).await?;
    let mut write = BufWriter::new(file);

    let json = serde_json::to_string_pretty(ignored_lints)?;
    write.write_all(json.as_bytes()).await?;

    write.flush().await?;

    Ok(())
}

/// Load ignored lints from a file on disk.
pub async fn load_ignored_lints(path: impl AsRef<Path>) -> Result<IgnoredLints> {
    let file = File::open(path.as_ref()).await?;
    let mut read = BufReader::new(file);

    let mut buf = String::new();
    read.read_to_string(&mut buf).await?;

    Ok(serde_json::from_str(&buf)?)
}

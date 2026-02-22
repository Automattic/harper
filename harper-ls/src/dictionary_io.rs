use std::path::Path;

use itertools::Itertools;
use tokio::fs::{self, File};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, Result};

use harper_core::{
    CharString, Dialect, DialectFlags, DictWordMetadata,
    spell::{CommonDictFuncs, MutableDictionary, WordMap, WordMapEntry},
};

/// Save the contents of a dictionary to a file.
/// Ensures that the path to the destination exists.
pub async fn save_dict(path: impl AsRef<Path>, dict: &WordMap) -> Result<()> {
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
async fn write_word_list(dict: &WordMap, mut w: impl AsyncWrite + Unpin) -> Result<()> {
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
pub async fn load_dict(path: impl AsRef<Path>, dialect: Dialect) -> Result<MutableDictionary> {
    let file = File::open(path.as_ref()).await?;
    let read = BufReader::new(file);

    dict_from_word_list(read, dialect).await
}

/// Load a dictionary from a list of words.
/// It could definitely be optimized to use less memory.
/// Right now it isn't an issue.
async fn dict_from_word_list(
    mut r: impl AsyncRead + Unpin,
    dialect: Dialect,
) -> Result<MutableDictionary> {
    let mut str = String::new();

    r.read_to_string(&mut str).await?;

    let mut dict = MutableDictionary::new();
    dict.extend(str.lines().map(|l| {
        WordMapEntry::new(l.chars().collect::<CharString>()).with_md(DictWordMetadata {
            dialects: DialectFlags::from_dialect(dialect),
            ..Default::default()
        })
    }));

    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::spell::{Dictionary, MutableDictionary};
    use std::io::Cursor;

    const TEST_UNSORTED_WORDS: [&str; 10] = [
        "peafowl",
        "housebroken",
        "blackjack",
        "Žižek",
        "BMX",
        "icebox",
        "stetting",
        "ツ",
        "ASCII",
        "link",
    ];
    const TEST_SORTED_WORDS: [&str; 10] = [
        "ASCII",
        "BMX",
        "blackjack",
        "housebroken",
        "icebox",
        "link",
        "peafowl",
        "stetting",
        "Žižek",
        "ツ",
    ];

    /// Creates an unsorted `MutableDictionary` for testing.
    fn get_test_unsorted_dict() -> MutableDictionary {
        let mut test_unsorted_dict = MutableDictionary::new();
        test_unsorted_dict.extend(
            TEST_UNSORTED_WORDS.map(|w| WordMapEntry::new(w.chars().collect::<CharString>())),
        );
        test_unsorted_dict
    }

    #[tokio::test]
    async fn writes_sorted_word_list() {
        let test_unsorted_dict = get_test_unsorted_dict();
        let mut test_writer = Cursor::new(Vec::new());
        write_word_list(test_unsorted_dict.get_word_map(), &mut test_writer)
            .await
            .expect("writing to Vec<u8> should not fail. (Unless OOM?)");
        assert_eq!(
            // Append trailing newline to match write_word_list output format.
            TEST_SORTED_WORDS.join("\n") + "\n",
            String::from_utf8_lossy(&test_writer.into_inner())
        );
    }
}

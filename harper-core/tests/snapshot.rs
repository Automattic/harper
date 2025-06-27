use std::{
    marker::Sync,
    path::{Path, PathBuf},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn get_tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}
fn get_text_dir() -> PathBuf {
    get_tests_dir().join("text")
}

pub fn get_text_files() -> Vec<PathBuf> {
    let mut files = vec![];
    for entry in std::fs::read_dir(get_text_dir())
        .unwrap()
        .filter_map(|f| f.ok())
        .filter(|f| f.metadata().unwrap().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        if matches!(ext.as_str(), "txt" | "md") {
            files.push(entry.path());
        }
    }
    files
}

fn tag_file(
    text_file: &Path,
    snapshot_file: &Path,
    create_snapshot: impl Fn(&str) -> String,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(text_file)?.replace("\r\n", "\n");
    let tagged = create_snapshot(source.trim_end());

    // compare with snapshot
    let has_snapshot = snapshot_file.exists();
    if has_snapshot {
        let snapshot = std::fs::read_to_string(snapshot_file)?;
        if tagged == snapshot {
            return Ok(());
        }
    }

    // write snapshot
    std::fs::write(snapshot_file, tagged)?;

    Err(if has_snapshot {
        "Snapshot mismatches!".into()
    } else {
        "No snapshot!".into()
    })
}
fn get_snapshot_file(text_file: &Path, snapshot_dir: &Path, ext: &str) -> PathBuf {
    let snapshot_name = text_file.file_stem().unwrap().to_string_lossy().to_string() + ext;
    snapshot_dir.join(snapshot_name)
}
#[allow(dead_code)]
pub fn snapshot_all_text_files(
    out_dir: &str,
    snapshot_ext: &str,
    create_snapshot: impl Copy + Fn(&str) -> String + 'static + Sync,
) {
    let snapshot_dir = get_text_dir().join(out_dir);
    std::fs::create_dir_all(&snapshot_dir).expect("Failed to create snapshot directory");

    let errors: u64 = get_text_files()
        .par_iter()
        .map(|text_file| {
            println!("Processing {}", text_file.display());
            let snapshot_file = get_snapshot_file(text_file, &snapshot_dir, snapshot_ext);
            if let Err(e) = tag_file(text_file, &snapshot_file, create_snapshot) {
                eprintln!("Error processing {}: {}", text_file.display(), e);
                1
            } else {
                0
            }
        })
        .sum();

    if errors > 0 {
        panic!("{errors} errors occurred while processing files");
    }
}

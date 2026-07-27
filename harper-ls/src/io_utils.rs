use anyhow::anyhow;
use std::path::{Component, Path, PathBuf};

use tower_lsp_server::{UriExt, lsp_types::Uri};

/// Rewrites a path to a filename using the same conventions as
/// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
/// Windows path prefixes are sanitized into one segment, so the result is always
/// relative and can be safely joined to the config directory.
pub fn fileify_path(uri: &Uri) -> anyhow::Result<PathBuf> {
    // We assume all URLs are local files and have a base.
    let path = uri
        .to_file_path()
        .ok_or_else(|| anyhow!("Unable to convert URI to file path."))?;
    Ok(fileify_file_path(&path).into())
}

fn fileify_file_path(path: &Path) -> String {
    fileify_path_segments(path.components().filter_map(|seg| match seg {
        Component::RootDir => None,
        Component::Prefix(p) => Some(sanitize_windows_prefix(&p.as_os_str().to_string_lossy())),
        other => Some(other.as_os_str().to_string_lossy().into_owned()),
    }))
}

fn fileify_path_segments<I, S>(segments: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut rewritten = String::new();

    for segment in segments {
        let segment = segment.as_ref();

        rewritten.push_str(segment);
        rewritten.push('%');
    }

    rewritten
}

/// Windows path prefixes (`C:`, `\\server\share`, `\\?\C:`) contain characters that are
/// illegal in a filename, which made the joined config path escape its base (#3667).
/// Strip them so the flattened name stays relative while drives/shares stay distinct.
/// Unreachable on Unix, where `Component::Prefix` never occurs.
fn sanitize_windows_prefix(prefix: &str) -> String {
    prefix
        .chars()
        .filter(|c| !matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect()
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::fileify_file_path;
    use super::{fileify_path_segments, sanitize_windows_prefix};
    use std::path::Path;

    #[test]
    fn preserves_unix_path_format() {
        let rewritten = fileify_path_segments(["home", "user", "proj", "README.md"]);

        assert_eq!(rewritten, "home%user%proj%README.md%");
    }

    #[test]
    fn fileified_path_stays_under_base_when_joined() {
        let rewritten = fileify_path_segments([
            sanitize_windows_prefix("C:"),
            "Users".into(),
            "proj".into(),
            "README.md".into(),
        ]);

        assert_eq!(rewritten, "C%Users%proj%README.md%");
        assert!(Path::new(&rewritten).is_relative());
    }

    #[test]
    fn preserves_colons_and_backslashes_in_unix_segments() {
        let rewritten = fileify_path_segments(["home", "user", "notes:draft.md", r"back\slash.md"]);

        assert_eq!(rewritten, r"home%user%notes:draft.md%back\slash.md%");
    }

    #[cfg(windows)]
    #[test]
    fn fileifies_windows_paths() {
        assert_ne!(
            fileify_file_path(Path::new(r"C:\proj\a.md")),
            fileify_file_path(Path::new(r"D:\proj\a.md"))
        );
        assert_eq!(
            fileify_file_path(Path::new(r"C:\Users\x\proj\README.md")),
            "C%Users%x%proj%README.md%"
        );
        assert_eq!(
            fileify_file_path(Path::new(r"\\server\share\proj\a.md")),
            "servershare%proj%a.md%"
        );
    }
}

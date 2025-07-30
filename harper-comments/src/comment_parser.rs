use std::path::Path;

use comment_parsers::{Go, JavaDoc, JsDoc, Solidity, Unit};
use harper_core::parsers::{self, MarkdownOptions, Parser};
use harper_core::{MutableDictionary, Token};
use tree_sitter::Node;

use crate::comment_parsers;
use crate::masker::CommentMasker;

pub struct CommentParser {
    inner: parsers::Mask<CommentMasker, Box<dyn Parser>>,
}

impl CommentParser {
    pub fn create_ident_dict(&self, source: &[char]) -> Option<MutableDictionary> {
        self.inner.masker.create_ident_dict(source)
    }

    pub fn new_from_language_id(
        language_id: &str,
        markdown_options: MarkdownOptions,
    ) -> Option<Self> {
        let language = match language_id {
            "cmake" => tree_sitter_cmake::LANGUAGE,
            "cpp" => tree_sitter_cpp::LANGUAGE,
            "csharp" => tree_sitter_c_sharp::LANGUAGE,
            "c" => tree_sitter_c::LANGUAGE,
            "dart" => harper_tree_sitter_dart::LANGUAGE,
            "go" => tree_sitter_go::LANGUAGE,
            "haskell" => tree_sitter_haskell::LANGUAGE,
            "javascriptreact" => tree_sitter_typescript::LANGUAGE_TSX,
            "javascript" => tree_sitter_javascript::LANGUAGE,
            "java" => tree_sitter_java::LANGUAGE,
            "kotlin" => tree_sitter_kotlin_ng::LANGUAGE,
            "lua" => tree_sitter_lua::LANGUAGE,
            "nix" => tree_sitter_nix::LANGUAGE,
            "php" => tree_sitter_php::LANGUAGE_PHP,
            "python" => tree_sitter_python::LANGUAGE,
            "ruby" => tree_sitter_ruby::LANGUAGE,
            "rust" => tree_sitter_rust::LANGUAGE,
            "scala" => tree_sitter_scala::LANGUAGE,
            "shellscript" => tree_sitter_bash::LANGUAGE,
            "solidity" => tree_sitter_solidity::LANGUAGE,
            "swift" => tree_sitter_swift::LANGUAGE,
            "toml" => tree_sitter_toml_ng::LANGUAGE,
            "typescriptreact" => tree_sitter_typescript::LANGUAGE_TSX,
            "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            "clojure" => tree_sitter_clojure::LANGUAGE,
            _ => return None,
        };

        let comment_parser: Box<dyn Parser> = match language_id {
            "go" => Box::new(Go::new_markdown(markdown_options)),
            "java" => Box::new(JavaDoc::default()),
            "javascriptreact" | "typescript" | "typescriptreact" | "javascript" => {
                Box::new(JsDoc::new_markdown(markdown_options))
            }
            "solidity" => Box::new(Solidity::new_markdown(markdown_options)),
            _ => Box::new(Unit::new_markdown(markdown_options)),
        };

        Some(Self {
            inner: parsers::Mask::new(
                CommentMasker::new(language.into(), Self::node_condition),
                comment_parser,
            ),
        })
    }

    /// Infer the programming language from a provided filename.
    pub fn new_from_filename(filename: &Path, markdown_options: MarkdownOptions) -> Option<Self> {
        Self::new_from_language_id(Self::filename_to_filetype(filename)?, markdown_options)
    }

    /// Convert a provided path to a corresponding Language Server Protocol file
    /// type.
    ///
    /// Note to contributors: try to keep this in sync with
    /// [`Self::new_from_language_id`]
    fn filename_to_filetype(path: &Path) -> Option<&'static str> {
        Some(match path.extension()?.to_str()? {
            "bash" => "shellscript",
            "c" => "c",
            "cmake" => "cmake",
            "cpp" => "cpp",
            "cs" => "csharp",
            "dart" => "dart",
            "go" => "go",
            "h" => "cpp",
            "hs" => "haskell",
            "java" => "java",
            "js" => "javascript",
            "jsx" => "javascriptreact",
            "kt" | "kts" => "kotlin",
            "lua" => "lua",
            "nix" => "nix",
            "php" => "php",
            "py" => "python",
            "rb" => "ruby",
            "rs" => "rust",
            "scala" | "sbt" | "mill" => "scala",
            "sh" => "shellscript",
            "sol" => "solidity",
            "swift" => "swift",
            "toml" => "toml",
            "ts" => "typescript",
            "tsx" => "typescriptreact",
            "clj" | "cljs" | "cljc" | "bb" => "clojure",
            _ => return None,
        })
    }

    fn node_condition(n: &Node) -> bool {
        n.kind().contains("comment")
    }
}

impl Parser for CommentParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

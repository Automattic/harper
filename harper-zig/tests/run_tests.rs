#[cfg(test)]
mod tests {
    use harper_core::parsers::Parser;
    use harper_zig::ZigParser;

    #[test]
    fn test_zig_parser() {
        let parser = ZigParser::new();
        let source = r#"
        /// This is a comment with a mispelling: "teh".
        pub fn hello() void {
            // This is a singel line comment
        }
        "#;

        let tokens = parser.parse(&source.chars().collect::<Vec<_>>());

        // Should find some tokens from the comments
        assert!(
            !tokens.is_empty(),
            "Parser should find tokens in Zig comments"
        );
    }

    #[test]
    fn test_comment_parsing() {
        let parser = ZigParser::new();
        let source = "// This is a comment with spelling errors like teh and singel";

        let tokens = parser.parse(&source.chars().collect::<Vec<_>>());

        // Should parse the comment content
        assert!(!tokens.is_empty(), "Parser should tokenize comments");
    }
}

use ammonia::clean;
use markdown::{CompileOptions, Options, to_html_with_options};

/// The standard Markdown rendering function for the crate.
/// Do not call the `markdown` crate directly. Use this.
pub fn render_markdown(markdown: &str) -> String {
    let options = Options {
        compile: CompileOptions {
            // Raw HTML is sanitized by `ammonia` below.
            allow_dangerous_html: true,
            ..CompileOptions::gfm()
        },
        ..Options::gfm()
    };

    let html = to_html_with_options(markdown, &options).unwrap_or_else(|_| markdown.to_string());
    clean(&html)
}

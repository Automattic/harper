use ammonia::clean;
use pulldown_cmark::{Parser, html};

/// The standard Markdown rendering function for the crate.
/// Do not call `pulldown_cmark` directly. Use this.
pub fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html = String::new();
    html::push_html(&mut html, parser);
    clean(&html)
}

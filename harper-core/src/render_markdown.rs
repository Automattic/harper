/// The standard Markdown rendering function for the crate.
/// Do not call `pulldown_cmark` directly. Use this.
pub fn render_markdown(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

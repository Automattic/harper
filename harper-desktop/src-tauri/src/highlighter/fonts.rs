//! Native overlay fonts shared by highlighter windows.

use egui::{Context, FontData, FontDefinitions, FontFamily, FontId};

const ATKINSON_REGULAR: &[u8] =
    include_bytes!("../../../static/fonts/atkinson/AtkinsonHyperlegible-Regular.ttf");
const ATKINSON_BOLD: &[u8] =
    include_bytes!("../../../static/fonts/atkinson/AtkinsonHyperlegible-Bold.ttf");
const REGULAR_FONT: &str = "Harper-Regular";
const BOLD_FAMILY: &str = "Harper Bold";
const BOLD_FONT: &str = "Harper-Bold";

/// Loads macOS's installed San Francisco font without redistributing Apple's font files.
/// Uses its text optical size for the small popup UI. Other platforms, or Macs where the system
/// font cannot be read, use the bundled Atkinson Hyperlegible regular and bold faces instead.
fn platform_fonts() -> (FontData, FontData) {
    #[cfg(target_os = "macos")]
    {
        const SYSTEM_FONT: &str = "/System/Library/Fonts/SFNS.ttf";
        match std::fs::read(SYSTEM_FONT) {
            Ok(bytes) => {
                let mut font = FontData::from_owned(bytes);
                font.tweak.coords.push(b"opsz", 17.0);
                font.tweak.coords.push(b"wght", 400.0);
                let mut bold = font.clone();
                bold.tweak.coords.push(b"wght", 700.0);
                return (font, bold);
            }
            Err(error) => {
                eprintln!("Could not load {SYSTEM_FONT}; using Atkinson Hyperlegible: {error}")
            }
        }
    }

    (
        FontData::from_static(ATKINSON_REGULAR),
        FontData::from_static(ATKINSON_BOLD),
    )
}

/// Creates an independent overlay context using San Francisco on macOS and Atkinson elsewhere.
///
/// Both default families use the platform face, including text styled as monospace in markdown.
/// The named bold family uses San Francisco's weight 700 or Atkinson's separate bold face.
/// Hack and egui's emoji fonts remain symbol fallbacks. Keep a separate context per window so each GPU gets its own atlas.
pub fn context() -> Context {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.remove("Ubuntu-Light");
    let (regular, bold) = platform_fonts();
    fonts
        .font_data
        .insert(REGULAR_FONT.to_owned(), regular.into());
    fonts.font_data.insert(BOLD_FONT.to_owned(), bold.into());

    let family = |primary: &str| {
        vec![
            primary.to_owned(),
            "Hack".to_owned(),
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ]
    };
    fonts
        .families
        .insert(FontFamily::Proportional, family(REGULAR_FONT));
    fonts
        .families
        .insert(FontFamily::Monospace, family(REGULAR_FONT));
    fonts
        .families
        .insert(FontFamily::Name(BOLD_FAMILY.into()), family(BOLD_FONT));

    let context = Context::default();
    context.set_fonts(fonts);
    context
}

/// Selects the platform face at weight 700, rather than egui's color-only `RichText::strong` styling.
pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD_FAMILY.into()))
}

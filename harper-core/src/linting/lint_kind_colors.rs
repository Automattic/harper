use super::LintKind;

/// RGB color tuple for a lint kind.
pub struct LintKindColor {
    pub kind: LintKind,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub hex: &'static str,
}

/// Get all lint kind colors in a consistent order.
pub fn lint_kind_colors() -> &'static [LintKindColor] {
    &[
        LintKindColor {
            kind: LintKind::Agreement,
            r: 0x22,
            g: 0x8B,
            b: 0x22,
            hex: "#228B22",
        },
        LintKindColor {
            kind: LintKind::BoundaryError,
            r: 0x8B,
            g: 0x45,
            b: 0x13,
            hex: "#8B4513",
        },
        LintKindColor {
            kind: LintKind::Capitalization,
            r: 0x54,
            g: 0x0D,
            b: 0x6E,
            hex: "#540D6E",
        },
        LintKindColor {
            kind: LintKind::Eggcorn,
            r: 0xFF,
            g: 0x8C,
            b: 0x00,
            hex: "#FF8C00",
        },
        LintKindColor {
            kind: LintKind::Enhancement,
            r: 0x0E,
            g: 0xAD,
            b: 0x69,
            hex: "#0EAD69",
        },
        LintKindColor {
            kind: LintKind::Formatting,
            r: 0x7D,
            g: 0x3C,
            b: 0x98,
            hex: "#7D3C98",
        },
        LintKindColor {
            kind: LintKind::Grammar,
            r: 0x9B,
            g: 0x59,
            b: 0xB6,
            hex: "#9B59B6",
        },
        LintKindColor {
            kind: LintKind::Malapropism,
            r: 0xC7,
            g: 0x15,
            b: 0x85,
            hex: "#C71585",
        },
        LintKindColor {
            kind: LintKind::Miscellaneous,
            r: 0x3B,
            g: 0xCE,
            b: 0xAC,
            hex: "#3BCEAC",
        },
        LintKindColor {
            kind: LintKind::Nonstandard,
            r: 0x00,
            g: 0x8B,
            b: 0x8B,
            hex: "#008B8B",
        },
        LintKindColor {
            kind: LintKind::Punctuation,
            r: 0xD4,
            g: 0x85,
            b: 0x0F,
            hex: "#D4850F",
        },
        LintKindColor {
            kind: LintKind::Readability,
            r: 0x2E,
            g: 0x8B,
            b: 0x57,
            hex: "#2E8B57",
        },
        LintKindColor {
            kind: LintKind::Redundancy,
            r: 0x46,
            g: 0x82,
            b: 0xB4,
            hex: "#4682B4",
        },
        LintKindColor {
            kind: LintKind::Regionalism,
            r: 0xC0,
            g: 0x61,
            b: 0xCB,
            hex: "#C061CB",
        },
        LintKindColor {
            kind: LintKind::Repetition,
            r: 0x00,
            g: 0xA6,
            b: 0x7C,
            hex: "#00A67C",
        },
        LintKindColor {
            kind: LintKind::Spelling,
            r: 0xEE,
            g: 0x42,
            b: 0x66,
            hex: "#EE4266",
        },
        LintKindColor {
            kind: LintKind::Style,
            r: 0xFF,
            g: 0xD2,
            b: 0x3F,
            hex: "#FFD23F",
        },
        LintKindColor {
            kind: LintKind::Typo,
            r: 0xFF,
            g: 0x6B,
            b: 0x35,
            hex: "#FF6B35",
        },
        LintKindColor {
            kind: LintKind::Usage,
            r: 0x1E,
            g: 0x90,
            b: 0xFF,
            hex: "#1E90FF",
        },
        LintKindColor {
            kind: LintKind::WordChoice,
            r: 0x22,
            g: 0x8B,
            b: 0x22,
            hex: "#228B22",
        },
    ]
}

/// Get the RGB color for a specific lint kind.
pub fn rgb_for_lint_kind(kind: LintKind) -> (u8, u8, u8) {
    lint_kind_colors()
        .iter()
        .find(|c| c.kind == kind)
        .map(|c| (c.r, c.g, c.b))
        .unwrap_or((0, 0, 0))
}

/// Get the hex color for a specific lint kind.
pub fn hex_for_lint_kind(kind: LintKind) -> &'static str {
    lint_kind_colors()
        .iter()
        .find(|c| c.kind == kind)
        .map(|c| c.hex)
        .unwrap_or("#000000")
}

use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
    patterns::WordSet,
};

const TRIGGERS: &[&str] = &[
    "128",
    "32bit",
    "64",
    "64bit",
    "68k",
    "aircraft",
    "algorithm",
    "analyse",
    "analysis",
    "analyze",
    "arch",
    "architecture",
    "arduino",
    "asm",
    "assemble",
    "assembled",
    "assembler",
    "assembly",
    "atari",
    "binary",
    "buffer",
    "bypass",
    "byte",
    "bytes",
    "cable",
    "cables",
    "cheat",
    "code",
    "compiler",
    "component",
    "components",
    "computer",
    "contraption",
    "controller",
    "controls",
    "core",
    "crash",
    "debug",
    "debugger",
    "decompile",
    "decompiled",
    "decompiler",
    "dex",
    "disassemble",
    "disassembler",
    "disassembly",
    "dismantle",
    "dll",
    "drive",
    "dump",
    "dumped",
    "electronic",
    "electronics",
    "enclosure",
    "engine",
    "engineer",
    "engineered",
    "engineering",
    "exe",
    "executable",
    "executables",
    "execute",
    "exploit",
    "exploiting",
    "firewall",
    "firmware",
    "function",
    "functions",
    "furniture",
    "game",
    "games",
    "gdb",
    "ghidra",
    "hack",
    "hacking",
    "hardware",
    "hex",
    "ida",
    "instruction",
    "instructions",
    "kit",
    "lib",
    "library",
    "load",
    "logic",
    "metadata",
    "motor",
    "nintendo",
    "opcode",
    "opcodes",
    "operation",
    "operations",
    "oscilloscope",
    "parameter",
    "parameters",
    "penetration",
    "pic",
    "processor",
    "protocol",
    "pseudocode",
    "register",
    "reverse",
    "rom",
    "runtime",
    "sata",
    "save",
    "sector",
    "sega",
    "server",
    "smali",
    "software",
    "stack",
    "switches",
    "toolkit",
    "usb",
    "windbg",
    "wires",
    "x64",
    "x86",
];

pub struct DissembleDisassemble {
    expr: WordSet,
}

impl Default for DissembleDisassemble {
    fn default() -> Self {
        Self {
            expr: WordSet::new([
                "dissemble",
                "dissembled",
                "dissembles",
                "dissembling",
                "dissembler",
                "dissemblers",
                "dissembly",
                "dissemblies",
            ]),
        }
    }
}

impl ExprLinter for DissembleDisassemble {
    type Unit = Sentence;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let mut token_iter = ctx
            .into_iter()
            .flat_map(|(prev, next)| prev.iter().chain(next.iter()));

        if !token_iter.any(|t| t.get_ch(src).eq_any_ignore_ascii_case_str(TRIGGERS)) {
            return None;
        }

        let mut word = toks.first()?.get_ch(src).to_vec();

        // Use the case of the first 'e' to get the right case for the 'a'
        word.splice(2..2, [word[2], (word[4] as u8 - 4) as char]);

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(word)],
            message: "Did you confuse `dissemble` for `disassemble?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Tries to detect `dissemble` used instead of `disassemble` by mistake."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::DissembleDisassemble;

    // Correct sentences with certain relevant words in them

    #[test]
    fn fix_dissemble_slash_assemble() {
        assert_suggestion_result(
            "I think it should be implemented in a way that keeps the games in the gme file because there's no real need to dissemble/assemble the whole file",
            DissembleDisassemble::default(),
            "I think it should be implemented in a way that keeps the games in the gme file because there's no real need to disassemble/assemble the whole file",
        );
    }

    #[test]
    fn fix_dissemble_slash_assemble_2() {
        assert_good_and_bad_suggestions(
            "I think it should be implemented in a way that keeps the games in the gme file because there's no real need to dissemble/assemble the whole file",
            DissembleDisassemble::default(),
            &[
                "I think it should be implemented in a way that keeps the games in the gme file because there's no real need to disassemble/assemble the whole file",
            ],
            &[],
        );
    }

    #[test]
    fn fix_decompile_reverse_engineer_dissemble() {
        assert_good_and_bad_suggestions(
            "decompile, reverse-engineer, dissemble, or attempt to derive any source code from the Software",
            DissembleDisassemble::default(),
            &[
                "decompile, reverse-engineer, disassemble, or attempt to derive any source code from the Software",
            ],
            &[],
        );
    }

    #[test]
    fn fix_dissembled_then_assembled() {
        assert_good_and_bad_suggestions(
            "In MosaicKD, these local patterns are first dissembled from OOD data and then assembled to synthesize in-domain data, making OOD-KD feasible.",
            DissembleDisassemble::default(),
            &[
                "In MosaicKD, these local patterns are first disassembled from OOD data and then assembled to synthesize in-domain data, making OOD-KD feasible.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_dissembled_radio() {
        assert_good_and_bad_suggestions(
            "Disregard this issue, I dissembled the radio only to find out that the ribbon cable going to the trim switches and other controls was dislodged.",
            DissembleDisassemble::default(),
            &[
                "Disregard this issue, I disassembled the radio only to find out that the ribbon cable going to the trim switches and other controls was dislodged.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_dissembling_contraption() {
        assert_good_and_bad_suggestions(
            "I was dissembling a minecart contraption, when my game crashed.",
            DissembleDisassemble::default(),
            &["I was disassembling a minecart contraption, when my game crashed."],
            &[],
        );
    }

    #[test]
    fn fix_dissemble_sata_usb_drive_enclosure() {
        assert_good_and_bad_suggestions(
            "I have try to dissemble the drive and bought a generic SATA to USB enclosure.",
            DissembleDisassemble::default(),
            &["I have try to disassemble the drive and bought a generic SATA to USB enclosure."],
            &[],
        );
    }

    #[test]
    fn fix_dissemble_hex() {
        assert_good_and_bad_suggestions(
            "Hex view can be kept in another tab at the bottom, also dissemble tab might be helpful here.",
            DissembleDisassemble::default(),
            &[
                "Hex view can be kept in another tab at the bottom, also disassemble tab might be helpful here.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_title_case_dissemble_all_caps_asm() {
        assert_good_and_bad_suggestions(
            "I am looking at the Dissembly window in Visual Studio 2012 and I have the setting for interlacing C++ and generated ASM turned on.",
            DissembleDisassemble::default(),
            &[
                "I am looking at the Disassembly window in Visual Studio 2012 and I have the setting for interlacing C++ and generated ASM turned on.",
            ],
            &[],
        );
    }

    // Avoid false positives in sentences without relevant words

    #[test]
    fn dont_flag_no_tech_words() {
        assert_no_lints(
            "Do not dissemble, we each have our own identity. When we say \"ambassador\", we do not mean a populist but a decent person, with a knack for communication",
            DissembleDisassemble::default(),
        );
    }

    #[test]
    fn dont_flag_no_apparent_tech_words() {
        assert_no_lints(
            "Dissembling parachute on Kerbin yields 2k material kits & on Mun it yields 10k.",
            DissembleDisassemble::default(),
        );
    }
}

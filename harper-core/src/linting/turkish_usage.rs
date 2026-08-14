use std::collections::HashMap;

use crate::expr::{Expr, SequenceExpr};
use crate::linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenKind};

use super::turkish_redundancy::{turkish_lower, turkish_match_case};

/// Türkçe'de bitişik yazılan ama TDK'ya göre ayrı yazılması gereken kalıplar,
/// "de/da" bağlacı (yalnızca `benimde` gibi açık birleşik biçimler) ve
/// "mi/mı/mu/mü" soru eki ayrı yazım hataları.
///
/// Circumflex homographs (`kar`/`kâr`, `hala`/`hâlâ`, `hakim`/`hâkim`) and
/// locatives (`bende`, `sende`) are valid words on their own and are **not**
/// rewritten without context.
const USAGE_PAIRS: &[(&str, &str)] = &[
    // Bitişik yazılan ama ayrı yazılması gereken kalıplar
    ("birşey", "bir şey"),
    ("birşeyler", "bir şeyler"),
    ("herşey", "her şey"),
    ("hiçbirşey", "hiçbir şey"),
    ("herkez", "herkes"),
    ("yalnış", "yanlış"),
    ("yanlız", "yalnız"),
    ("malesef", "maalesef"),
    ("yada", "ya da"),
    ("arasıra", "ara sıra"),
    ("bazan", "bazen"),
    ("herzaman", "her zaman"),
    ("hergün", "her gün"),
    ("birgün", "bir gün"),
    ("heryer", "her yer"),
    ("heryerde", "her yerde"),
    ("birkez", "bir kez"),
    ("şuan", "şu an"),
    ("şuanda", "şu anda"),
    ("hiçbirzaman", "hiçbir zaman"),
    ("okadar", "o kadar"),
    ("bukadar", "bu kadar"),
    ("şukadar", "şu kadar"),
    ("nekadar", "ne kadar"),
    ("herhangibir", "herhangi bir"),
    ("bişey", "bir şey"),
    ("bişi", "bir şey"),
    // Circumflex eksik
    ("eger", "eğer"),
    ("gercek", "gerçek"),
    ("gercekten", "gerçekten"),
    // Yaygın yazım hataları (kaynak: Denomas/Turkce-yazim-denetimi, MIT)
    ("süpriz", "sürpriz"),
    ("şarz", "şarj"),
    ("espiri", "espri"),
    ("insiyatif", "inisiyatif"),
    ("teşekürler", "teşekkürler"),
    ("teşekür", "teşekkür"),
    ("diğil", "değil"),
    ("deyil", "değil"),
    ("yokki", "yok ki"),
    ("varki", "var ki"),
    // Konuşma dilindeki kısaltılmış gelecek zaman ekleri
    ("gelicem", "geleceğim"),
    ("gidicem", "gideceğim"),
    ("yapıcam", "yapacağım"),
    ("edicek", "edecek"),
    // "de/da" bağlacı: ayrı yazılır
    ("benimde", "benim de"),
    ("seninde", "senin de"),
    ("onunda", "onun da"),
    ("bizimde", "bizim de"),
    ("sizinde", "sizin de"),
    ("onuda", "onu da"),
    ("bunuda", "bunu da"),
    ("şunuda", "şunu da"),
    ("kendide", "kendi de"),
    ("kendiside", "kendisi de"),
    // "ki" bağlacı: yalnızca ayrı yazılması gereken kapalı liste (benimki/halbuki değil)
    ("demekki", "demek ki"),
    ("öyleki", "öyle ki"),
    ("taaki", "ta ki"),
    ("yeterki", "yeter ki"),
    ("gördümki", "gördüm ki"),
    ("dedimki", "dedim ki"),
    ("eminki", "emin ki"),
    ("açıkki", "açık ki"),
    ("yazıkki", "yazık ki"),
    ("belliki", "belli ki"),
];

pub struct TurkishUsage {
    expr: SequenceExpr,
    replacements: HashMap<String, &'static str>,
}

impl Default for TurkishUsage {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_word(),
            replacements: USAGE_PAIRS
                .iter()
                .map(|(wrong, right)| (turkish_lower(wrong), *right))
                .collect(),
        }
    }
}

impl ExprLinter for TurkishUsage {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let tok = matched_tokens.first()?;
        if !matches!(tok.kind, TokenKind::Word(_)) {
            return None;
        }

        let matched: String = source[tok.span.start..tok.span.end].iter().collect();
        let key = turkish_lower(&matched);
        let replacement = self.replacements.get(&key)?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith(turkish_match_case(
                &matched,
                replacement,
            ))],
            message: format!("\"{matched}\" yazımı hatalı, doğrusu \"{replacement}\"."),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects common Turkish spacing and clitic errors (e.g. `yanlız` -> `yalnız`)."
    }
}

#[cfg(test)]
mod tests {
    use super::TurkishUsage;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn detects_birsey() {
        assert_suggestion_result(
            "Bana birşey söyle.",
            TurkishUsage::default(),
            "Bana bir şey söyle.",
        );
    }

    #[test]
    fn detects_hersey() {
        assert_suggestion_result(
            "Herşey yolunda.",
            TurkishUsage::default(),
            "Her şey yolunda.",
        );
    }

    #[test]
    fn detects_hicbirsey() {
        assert_suggestion_result(
            "Hiçbirşey kalmadı.",
            TurkishUsage::default(),
            "Hiçbir şey kalmadı.",
        );
    }

    #[test]
    fn detects_yada() {
        assert_suggestion_result(
            "Çay yada kahve iç.",
            TurkishUsage::default(),
            "Çay ya da kahve iç.",
        );
    }

    #[test]
    fn detects_malesef() {
        assert_suggestion_result(
            "Malesef gelemedim.",
            TurkishUsage::default(),
            "Maalesef gelemedim.",
        );
    }

    #[test]
    fn detects_herkez() {
        assert_suggestion_result("Herkez geldi.", TurkishUsage::default(), "Herkes geldi.");
    }

    #[test]
    fn detects_suan() {
        assert_suggestion_result("Şuan gelecek.", TurkishUsage::default(), "Şu an gelecek.");
    }

    #[test]
    fn detects_de_da_conjunction() {
        assert_suggestion_result(
            "Onunda bir fikri var.",
            TurkishUsage::default(),
            "Onun da bir fikri var.",
        );
    }

    #[test]
    fn detects_benimde() {
        assert_suggestion_result(
            "Benimde bir sorum var.",
            TurkishUsage::default(),
            "Benim de bir sorum var.",
        );
    }

    #[test]
    fn no_lint_kar_snow() {
        assert_no_lints("Kar yağdı.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_hala_aunt_or_still() {
        assert_no_lints("Hala geldi.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_locative_bende() {
        assert_no_lints("Bende para yok.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_locative_sende() {
        assert_no_lints("Sende kalmış.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_hakim_judge() {
        assert_no_lints("Hakim karar verdi.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_adet_count() {
        assert_no_lints("Üç adet aldım.", TurkishUsage::default());
    }

    #[test]
    fn detects_birsey_before_period() {
        assert_suggestion_result("Bana birşey.", TurkishUsage::default(), "Bana bir şey.");
    }

    #[test]
    fn detects_uppercase_birsey() {
        assert_suggestion_result(
            "BİRŞEY söyleme.",
            TurkishUsage::default(),
            "BİR ŞEY söyleme.",
        );
    }

    #[test]
    fn detects_dotted_i_herkez() {
        assert_suggestion_result(
            "Herkes değil, herkez yanlış.",
            TurkishUsage::default(),
            "Herkes değil, herkes yanlış.",
        );
    }

    #[test]
    fn no_false_positive_on_correct_text() {
        assert_no_lints(
            "Bugün hava çok güzel, dışarı çıkalım.",
            TurkishUsage::default(),
        );
    }

    #[test]
    fn no_lint_when_already_split() {
        assert_no_lints("Bana bir şey söyle.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_on_locative_evde() {
        assert_no_lints("Evde kaldım.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_on_separate_question_particle() {
        assert_no_lints("Yapar mı?", TurkishUsage::default());
    }

    #[test]
    fn splits_demekki() {
        assert_suggestion_result(
            "Demekki haklıymış.",
            TurkishUsage::default(),
            "Demek ki haklıymış.",
        );
    }

    #[test]
    fn splits_oyleki() {
        assert_suggestion_result(
            "Öyleki şaşırdım.",
            TurkishUsage::default(),
            "Öyle ki şaşırdım.",
        );
    }

    #[test]
    fn splits_gorumki() {
        assert_suggestion_result(
            "Gördümki gelmiş.",
            TurkishUsage::default(),
            "Gördüm ki gelmiş.",
        );
    }

    #[test]
    fn splits_taaki() {
        assert_suggestion_result("Taaki bitsin.", TurkishUsage::default(), "Ta ki bitsin.");
    }

    #[test]
    fn splits_yeterki() {
        assert_suggestion_result(
            "Yeterki gelsin.",
            TurkishUsage::default(),
            "Yeter ki gelsin.",
        );
    }

    #[test]
    fn no_lint_benimki() {
        assert_no_lints("Bu kalem benimki.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_halbuki() {
        assert_no_lints("Halbuki söylemiştim.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_evdeki() {
        assert_no_lints("Evdeki kitap.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_belki() {
        assert_no_lints("Belki gelir.", TurkishUsage::default());
    }

    #[test]
    fn detects_arasira() {
        assert_suggestion_result("Arasıra uğrar.", TurkishUsage::default(), "Ara sıra uğrar.");
    }

    #[test]
    fn detects_bazan() {
        assert_suggestion_result("Bazan gelir.", TurkishUsage::default(), "Bazen gelir.");
    }

    #[test]
    fn detects_herzaman() {
        assert_suggestion_result(
            "Herzaman haklı değil.",
            TurkishUsage::default(),
            "Her zaman haklı değil.",
        );
    }

    #[test]
    fn detects_yalnis() {
        assert_suggestion_result("Bu yalnış.", TurkishUsage::default(), "Bu yanlış.");
    }

    #[test]
    fn detects_suanda() {
        assert_suggestion_result(
            "Şuanda bekliyoruz.",
            TurkishUsage::default(),
            "Şu anda bekliyoruz.",
        );
    }

    #[test]
    fn no_lint_onunki() {
        assert_no_lints("Bu onunki.", TurkishUsage::default());
    }

    #[test]
    fn detects_hergun() {
        assert_suggestion_result(
            "Hergün çalışıyorum.",
            TurkishUsage::default(),
            "Her gün çalışıyorum.",
        );
    }

    #[test]
    fn detects_yanliz() {
        assert_suggestion_result("Yanlız kaldım.", TurkishUsage::default(), "Yalnız kaldım.");
    }

    #[test]
    fn detects_onuda() {
        assert_suggestion_result("Onuda al.", TurkishUsage::default(), "Onu da al.");
    }

    #[test]
    fn detects_bunuda() {
        assert_suggestion_result("Bunuda iste.", TurkishUsage::default(), "Bunu da iste.");
    }

    #[test]
    fn detects_kendide() {
        assert_suggestion_result(
            "Kendide bilmiyor.",
            TurkishUsage::default(),
            "Kendi de bilmiyor.",
        );
    }

    #[test]
    fn detects_kendiside() {
        assert_suggestion_result(
            "Kendiside geldi.",
            TurkishUsage::default(),
            "Kendisi de geldi.",
        );
    }

    #[test]
    fn detects_eger() {
        assert_suggestion_result("Eger gelirsen.", TurkishUsage::default(), "Eğer gelirsen.");
    }

    #[test]
    fn detects_gercek() {
        assert_suggestion_result("Bu gercek mi?", TurkishUsage::default(), "Bu gerçek mi?");
    }

    #[test]
    fn detects_gercekten() {
        assert_suggestion_result(
            "Gercekten güzel.",
            TurkishUsage::default(),
            "Gerçekten güzel.",
        );
    }

    #[test]
    fn detects_heryerde() {
        assert_suggestion_result(
            "Heryerde arıyorum.",
            TurkishUsage::default(),
            "Her yerde arıyorum.",
        );
    }

    #[test]
    fn no_lint_bunda_locative() {
        assert_no_lints("Bunda bir sorun yok.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_sunda_locative() {
        assert_no_lints("Şunda bir hata var.", TurkishUsage::default());
    }

    #[test]
    fn no_lint_birkac() {
        assert_no_lints("Birkaç kişi geldi.", TurkishUsage::default());
    }

    // Kaynak: Denomas/Turkce-yazim-denetimi (MIT) — bkz. turkish/KURALLAR.md
    #[test]
    fn detects_birgun() {
        assert_suggestion_result(
            "Birgün geleceğim.",
            TurkishUsage::default(),
            "Bir gün geleceğim.",
        );
    }

    #[test]
    fn detects_hicbirzaman() {
        assert_suggestion_result(
            "Hiçbirzaman unutmam.",
            TurkishUsage::default(),
            "Hiçbir zaman unutmam.",
        );
    }

    #[test]
    fn detects_okadar() {
        assert_suggestion_result(
            "Okadar yorgunum ki.",
            TurkishUsage::default(),
            "O kadar yorgunum ki.",
        );
    }

    #[test]
    fn detects_bukadar() {
        assert_suggestion_result("Bukadar basit.", TurkishUsage::default(), "Bu kadar basit.");
    }

    #[test]
    fn detects_sukadar() {
        assert_suggestion_result("Şukadar yeter.", TurkishUsage::default(), "Şu kadar yeter.");
    }

    #[test]
    fn detects_nekadar() {
        assert_suggestion_result("Nekadar sürer?", TurkishUsage::default(), "Ne kadar sürer?");
    }

    #[test]
    fn detects_herhangibir() {
        assert_suggestion_result(
            "Herhangibir sorun olursa ara.",
            TurkishUsage::default(),
            "Herhangi bir sorun olursa ara.",
        );
    }

    #[test]
    fn detects_bisey() {
        assert_suggestion_result("Bişey söyle.", TurkishUsage::default(), "Bir şey söyle.");
    }

    #[test]
    fn detects_bisi() {
        assert_suggestion_result("Bişi yedim.", TurkishUsage::default(), "Bir şey yedim.");
    }

    #[test]
    fn detects_supriz() {
        assert_suggestion_result("Ne süpriz ama!", TurkishUsage::default(), "Ne sürpriz ama!");
    }

    #[test]
    fn detects_sarz() {
        assert_suggestion_result(
            "Telefonu şarz et.",
            TurkishUsage::default(),
            "Telefonu şarj et.",
        );
    }

    #[test]
    fn detects_espiri() {
        assert_suggestion_result(
            "Kötü bir espiri yaptı.",
            TurkishUsage::default(),
            "Kötü bir espri yaptı.",
        );
    }

    #[test]
    fn detects_insiyatif() {
        assert_suggestion_result(
            "İnsiyatif almalısın.",
            TurkishUsage::default(),
            "İnisiyatif almalısın.",
        );
    }

    #[test]
    fn detects_tesekurler() {
        assert_suggestion_result(
            "Çok teşekürler.",
            TurkishUsage::default(),
            "Çok teşekkürler.",
        );
    }

    #[test]
    fn detects_diğil() {
        assert_suggestion_result("Bu diğil o.", TurkishUsage::default(), "Bu değil o.");
    }

    #[test]
    fn detects_deyil() {
        assert_suggestion_result("Doğru deyil.", TurkishUsage::default(), "Doğru değil.");
    }

    #[test]
    fn detects_yokki() {
        assert_suggestion_result(
            "Param yokki vereyim.",
            TurkishUsage::default(),
            "Param yok ki vereyim.",
        );
    }

    #[test]
    fn detects_varki() {
        assert_suggestion_result("Bir şey varki.", TurkishUsage::default(), "Bir şey var ki.");
    }

    #[test]
    fn detects_gelicem() {
        assert_suggestion_result(
            "Yarın gelicem.",
            TurkishUsage::default(),
            "Yarın geleceğim.",
        );
    }

    #[test]
    fn detects_gidicem() {
        assert_suggestion_result(
            "Şimdi gidicem.",
            TurkishUsage::default(),
            "Şimdi gideceğim.",
        );
    }

    #[test]
    fn detects_yapicam() {
        assert_suggestion_result("Onu yapıcam.", TurkishUsage::default(), "Onu yapacağım.");
    }

    #[test]
    fn detects_edicek() {
        assert_suggestion_result("Yardım edicek.", TurkishUsage::default(), "Yardım edecek.");
    }
}

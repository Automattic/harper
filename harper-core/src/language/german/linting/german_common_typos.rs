use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document};

/// Misspellings that Harper's German spell check cannot see, and their
/// corrections.
///
/// The keys are lower case and are looked up case-insensitively; the values
/// carry the spelling a capitalized occurrence should get. Sorted, so a new
/// entry goes where it belongs and `binary_search_by_key` can find it.
const COMMON_TYPOS: &[(&str, &str)] = &[
    ("addressiert", "adressiert"),
    ("adminstration", "Administration"),
    ("algorhitmus", "Algorithmus"),
    ("alstadt", "Altstadt"),
    ("amtsitz", "Amtssitz"),
    ("anderere", "andere"),
    ("andereren", "anderen"),
    ("andererer", "anderer"),
    ("aufenhalt", "Aufenthalt"),
    ("aufname", "Aufnahme"),
    ("auforderung", "Aufforderung"),
    ("aufsteig", "Aufstieg"),
    ("augenlied", "Augenlid"),
    ("auserdem", "außerdem"),
    ("ausgangpunkt", "Ausgangspunkt"),
    ("ausicht", "Aussicht"),
    ("ausprache", "Aussprache"),
    ("auspruch", "Ausspruch"),
    ("aussschließlich", "ausschließlich"),
    ("austellung", "Ausstellung"),
    ("austellungen", "Ausstellungen"),
    ("austerben", "Aussterben"),
    ("bauerhöfe", "Bauernhöfe"),
    ("bedürfniss", "Bedürfnis"),
    ("beindrucken", "beeindrucken"),
    ("beispielswiese", "beispielsweise"),
    ("beispielweise", "beispielsweise"),
    ("beitag", "Beitrag"),
    ("bespiel", "Beispiel"),
    ("bischofsitz", "Bischofssitz"),
    ("bundestraße", "Bundesstraße"),
    ("danaben", "daneben"),
    ("daüber", "darüber"),
    ("dollmetscher", "Dolmetscher"),
    ("durchschlagkraft", "Durchschlagskraft"),
    ("durschnitt", "Durchschnitt"),
    ("durschnittlich", "durchschnittlich"),
    ("durschnittliche", "durchschnittliche"),
    ("durschnittlichen", "durchschnittlichen"),
    ("ebenfall", "ebenfalls"),
    ("eingen", "einigen"),
    ("eingentlich", "eigentlich"),
    ("eingesetz", "eingesetzt"),
    ("eingesetzen", "eingesetzten"),
    ("engeneering", "Engineering"),
    ("entstandende", "entstandene"),
    ("erfolgslos", "erfolglos"),
    ("erstaustrahlung", "Erstausstrahlung"),
    ("erwachsenalter", "Erwachsenenalter"),
    ("festellen", "feststellen"),
    ("freimauerei", "Freimaurerei"),
    ("friedenschluss", "Friedensschluss"),
    ("fußballstadium", "Fußballstadion"),
    ("garnision", "Garnison"),
    ("gebähren", "gebären"),
    ("gegebenfalls", "gegebenenfalls"),
    ("gegnüber", "gegenüber"),
    ("gemeinsammen", "gemeinsamen"),
    ("gesetztlich", "gesetzlich"),
    ("gesichtpunkt", "Gesichtspunkt"),
    ("handelschiff", "Handelsschiff"),
    ("handies", "Handys"),
    ("herrausragend", "herausragend"),
    ("herschaft", "Herrschaft"),
    ("herscher", "Herrscher"),
    ("hervorragenste", "hervorragendste"),
    ("hingegegen", "hingegen"),
    ("hintegrund", "Hintergrund"),
    ("hochaus", "Hochhaus"),
    ("hochäuser", "Hochhäuser"),
    ("imbus", "Inbus"),
    ("inhaltstoff", "Inhaltsstoff"),
    ("intergration", "Integration"),
    ("inverstor", "Investor"),
    ("jahrhudert", "Jahrhundert"),
    ("jahundert", "Jahrhundert"),
    ("jahunderte", "Jahrhunderte"),
    ("jahunderts", "Jahrhunderts"),
    ("janur", "Januar"),
    ("kenntniss", "Kenntnis"),
    ("krankenaus", "Krankenhaus"),
    ("kriegschiff", "Kriegsschiff"),
    ("kunstoff", "Kunststoff"),
    ("landesaustellung", "Landesausstellung"),
    ("landesprache", "Landessprache"),
    ("landsmannschaftschaftliche", "landsmannschaftliche"),
    ("landwirschaft", "Landwirtschaft"),
    ("landwirschaftlich", "landwirtschaftlich"),
    ("langläufig", "landläufig"),
    ("mannschaf", "Mannschaft"),
    ("manschaft", "Mannschaft"),
    ("meerespiegel", "Meeresspiegel"),
    ("meeresspegel", "Meeresspiegel"),
    ("mehrtätige", "mehrtägige"),
    ("meistbesuchtesten", "meistbesuchten"),
    ("metereologe", "Meteorologe"),
    ("metereologisch", "meteorologisch"),
    ("mettal", "Metall"),
    ("mittelaterlich", "mittelalterlich"),
    ("mittlerweilen", "mittlerweile"),
    ("mobilar", "Mobiliar"),
    ("nahaufname", "Nahaufnahme"),
    ("namenlich", "namentlich"),
    ("namenslos", "namenlos"),
    ("neugründeten", "neugegründeten"),
    ("nordlich", "nördlich"),
    ("organistion", "Organisation"),
    ("ortteil", "Ortsteil"),
    ("rechtspruch", "Rechtsspruch"),
    ("rechtstaat", "Rechtsstaat"),
    ("rechtstaatlich", "rechtsstaatlich"),
    ("rechtwinklich", "rechtwinklig"),
    ("rechzeitig", "rechtzeitig"),
    ("resourcen", "Ressourcen"),
    ("russsische", "russische"),
    ("russsischen", "russischen"),
    ("rückrad", "Rückgrat"),
    ("rückrat", "Rückgrat"),
    ("schafte", "schaffte"),
    ("schwerpunktsmäßig", "schwerpunktmäßig"),
    ("seeman", "Seemann"),
    ("sehenwürdigkeit", "Sehenswürdigkeit"),
    ("sogenante", "sogenannte"),
    ("sogenanten", "sogenannten"),
    ("standarts", "Standards"),
    ("stehgreif", "Stegreif"),
    ("tolleranz", "Toleranz"),
    ("totkrank", "todkrank"),
    ("trotzdessen", "trotzdem"),
    ("turist", "Tourist"),
    ("umgangsprachlich", "umgangssprachlich"),
    ("umgangsprachliche", "umgangssprachliche"),
    ("umgangsprachlicher", "umgangssprachlicher"),
    ("vergleichweise", "vergleichsweise"),
    ("verwaltungsitz", "Verwaltungssitz"),
    ("vieleicht", "vielleicht"),
    ("vorausichtlich", "voraussichtlich"),
    ("vorgesetze", "Vorgesetzte"),
    ("vormachtsstellung", "Vormachtstellung"),
    ("wachholder", "Wacholder"),
    ("warscheinlich", "wahrscheinlich"),
    ("wehrmutstropfen", "Wermutstropfen"),
    ("weiterere", "weitere"),
    ("weitereren", "weiteren"),
    ("weitesgehend", "weitestgehend"),
    ("weißmachen", "weismachen"),
    ("wesendlich", "wesentlich"),
    ("widerstandkämpfer", "Widerstandskämpfer"),
    ("wiedersacher", "Widersacher"),
    ("wiedersprüche", "Widersprüche"),
    ("wirtschaflich", "wirtschaftlich"),
    ("wissenschaflich", "wissenschaftlich"),
    ("währendessen", "währenddessen"),
    ("wärend", "während"),
    ("überlicherweise", "üblicherweise"),
];

/// Corrects the misspellings people actually make, which the spell checker
/// lets through.
///
/// German compounds freely, so Harper's spell checker accepts any word it can
/// cut into dictionary pieces. That is what makes German spell checking work
/// at all, and it is also why the most ordinary German typos sail past it:
/// *Amtsitz* cuts into `Amt` + `Sitz`, *vieleicht* into `viel` + `eicht`,
/// *Landesprache* into `Landes` + `Sprache`. Every one of them is a legal cut
/// of illegal German.
///
/// The table cannot be derived, so it is sourced and filtered instead. Source:
/// `Wikipedia:Liste von Tippfehlern`, the list de-wiki's own correction bots
/// run on, which is the same shape as Microsoft Word's autocorrect table.
/// Rebuild it by taking the entries that are a single word on both sides and
/// keeping the ones where:
///
/// - Harper reports nothing, so the entry adds something; and
/// - **both** LanguageTool and `hunspell -d de_DE` reject the misspelling.
///
/// The second condition is the one that matters. Both of those also split
/// compounds, so a word all three accept is a real German word — that is how
/// `anderseits`, `wohlgesonnen` and `Nachkommens`, which the source list calls
/// errors, stayed out. Proper names are out of scope and are dropped by hand.
///
/// Case is the writer's: the suggestion copies the casing of what was typed,
/// so `austellung` becomes `ausstellung` and is left to
/// [`super::GermanNounCapitalization`] to capitalize. Fixing both here would
/// make one lint out of two independent mistakes.
#[derive(Default)]
pub struct GermanCommonTypos;

impl GermanCommonTypos {
    fn correction(word: &[char]) -> Option<&'static str> {
        let lower: String = word.iter().flat_map(|c| c.to_lowercase()).collect();

        COMMON_TYPOS
            .binary_search_by_key(&lower.as_str(), |(wrong, _)| wrong)
            .ok()
            .map(|index| COMMON_TYPOS[index].1)
    }
}

impl Linter for GermanCommonTypos {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let chars = document.get_span_content(&word.span);
            let Some(correction) = Self::correction(chars) else {
                continue;
            };

            let original: String = chars.iter().collect();
            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Spelling,
                suggestions: vec![Suggestion::replace_with_match_case_str(correction, chars)],
                message: format!("»{original}« ist ein häufiger Tippfehler."),
                priority: 19,
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "Korrigiert häufige deutsche Tippfehler, die der Rechtschreibprüfung entgehen."
    }
}

#[cfg(test)]
mod tests {
    use super::{COMMON_TYPOS, GermanCommonTypos};
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn document(text: &str) -> Document {
        Document::new(text, &PlainGerman, &combined_german_dictionary())
    }

    fn flagged(text: &str) -> Vec<String> {
        let document = document(text);
        GermanCommonTypos
            .lint(&document)
            .into_iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    fn correction(word: &str) -> Option<&'static str> {
        let chars: Vec<char> = word.chars().collect();
        GermanCommonTypos::correction(&chars)
    }

    /// `binary_search_by_key` compares `&str` by UTF-8 bytes. A table sorted
    /// any other way — by code point after NFD, or by a locale that files `ü`
    /// under `u` — still looks orderly and silently stops finding its umlaut
    /// entries.
    #[test]
    fn the_table_is_in_the_order_the_search_assumes() {
        for pair in COMMON_TYPOS.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "{:?} must sort before {:?}",
                pair[0].0,
                pair[1].0
            );
        }
    }

    /// A correction that is itself a key would leave the writer fixing the
    /// same word forever.
    #[test]
    fn no_correction_is_itself_a_typo() {
        for (wrong, right) in COMMON_TYPOS {
            assert_eq!(correction(right), None, "{wrong} -> {right} loops");
        }
    }

    #[test]
    fn catches_the_typos_the_compound_splitter_waves_through() {
        for (wrong, right) in [
            ("Amtsitz", "Amtssitz"),
            ("Bundestraße", "Bundesstraße"),
            ("Landesprache", "Landessprache"),
            ("vieleicht", "vielleicht"),
            ("wärend", "während"),
            ("Manschaft", "Mannschaft"),
            ("warscheinlich", "wahrscheinlich"),
            ("Kunstoff", "Kunststoff"),
        ] {
            assert_eq!(correction(wrong).as_deref(), Some(right), "{wrong}");
            assert_eq!(
                flagged(&format!("Hier steht {wrong} im Satz.")),
                vec![wrong]
            );
        }
    }

    /// The suggestion takes the writer's casing, not the table's. Capitalizing
    /// a lower-case noun is a second, separate mistake and belongs to
    /// `GermanNounCapitalization`.
    #[test]
    fn the_suggestion_follows_the_writers_case() {
        let document = document("Die austellung und die Austellung.");
        let lints = GermanCommonTypos.lint(&document);

        let suggestions: Vec<String> = lints
            .iter()
            .flat_map(|lint| &lint.suggestions)
            .map(|suggestion| format!("{suggestion:?}"))
            .collect();

        assert_eq!(lints.len(), 2);
        assert!(suggestions[0].contains("ausstellung"), "{suggestions:?}");
        assert!(suggestions[1].contains("Ausstellung"), "{suggestions:?}");
    }

    /// Every one of these is a word the source list calls an error and two
    /// independent spell checkers call German. They stay out of the table.
    #[test]
    fn leaves_real_german_words_alone() {
        for word in [
            "anderseits",
            "wohlgesonnen",
            "Nachkommens",
            "Autorenschaft",
            "Hilfsfonds",
            "Amtssitz",
            "Bundesstraße",
            "vielleicht",
            "Mannschaft",
            "Aufnahme",
            "Kenntnis",
            "Standard",
            "Rückgrat",
        ] {
            assert_eq!(correction(word), None, "{word} should be left alone");
        }
    }
}

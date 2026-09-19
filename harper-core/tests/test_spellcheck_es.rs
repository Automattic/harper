use harper_core::linting::{Linter, SpellCheck};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

#[test]
fn probar_spellcheck_espanol() {
    let dict = FstDictionary::curated_spanish();
    let mut spell_checker = SpellCheck::new(dict.clone(), Dialect::Spanish);

    // Texto con palabras bien escritas y palabras con faltas de ortografía
    let texto = "Hola estov haciendo una prueba con faltas de ortografia como abiamos hablado";
    let document = Document::new_plain_english_curated_spanish(texto);

    let lints = spell_checker.lint(&document);

    println!("\n========================================================");
    println!("   PRUEBA DE SPELLCHECK (VERIFICADOR ORTOGRÁFICO ESPAÑOL)");
    println!("========================================================");
    println!("Texto analizado: \"{}\"", texto);
    println!("Errores de ortografía encontrados: {}", lints.len());
    println!("--------------------------------------------------------");
    for (i, lint) in lints.iter().enumerate() {
        let sugerencias: Vec<String> = lint
            .suggestions
            .iter()
            .map(|s| match s {
                harper_core::linting::Suggestion::ReplaceWith(chars) => chars.iter().collect(),
                _ => format!("{:?}", s),
            })
            .collect();
        println!(
            "[Error {}] Texto: '{}' | Mensaje: {} | Sugerencias: {:?}",
            i + 1,
            document.get_span_content_str(&lint.span),
            lint.message,
            sugerencias
        );
    }
    println!("========================================================\n");
}

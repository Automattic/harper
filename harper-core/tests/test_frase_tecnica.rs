use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::MergedDictionary;
use harper_core::{Dialect, Document};
use std::sync::Arc;

#[test]
fn probar_frase_tecnica_completa() {
    let dict = Arc::new(MergedDictionary::curated_spanish_multilingual());
    let mut linter_group = LintGroup::new_curated(dict, Dialect::Spanish);

    let texto = "La biblioteca Hyphen posee características avanzadas para dar soporte a la separación silábica de palabras compuestas y reglas no estándares, además de admitir codificaciones de caracteres multibyte como UTF-8.";

    let document = Document::new_plain_english_curated_spanish(texto);
    let lints = linter_group.lint(&document);

    println!("\n========================================================");
    println!("   VERIFICACIÓN DE FRASE TÉCNICA MULTI-IDIOMA DE HARPER");
    println!("========================================================");
    println!("Texto: \"{}\"", texto);
    println!("Errores o sugerencias encontradas: {}", lints.len());
    println!("--------------------------------------------------------");
    for (i, lint) in lints.iter().enumerate() {
        println!(
            "[{}] Palabra con observación: '{}' | Mensaje: {}",
            i + 1,
            document.get_span_content_str(&lint.span),
            lint.message
        );
    }
    println!("========================================================\n");

    assert_eq!(
        lints.len(),
        0,
        "No debería haber errores en palabras válidas como compuestas, codificaciones, UTF-8 o Hyphen."
    );
}

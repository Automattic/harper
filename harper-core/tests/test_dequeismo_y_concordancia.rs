use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::MergedDictionary;
use harper_core::{Dialect, Document};
use std::sync::Arc;

#[test]
fn probar_dequeismo_y_concordancia_sujeto_verbo() {
    let dict = Arc::new(MergedDictionary::curated_spanish_multilingual());
    let mut linter = LintGroup::new_curated(dict, Dialect::Spanish);

    // Texto con Dequeísmo ("dijo de que"), Queísmo ("seguro que") y falta de concordancia ("los niños juega")
    let texto = "El profesor dijo de que los niños juega en el parque y estoy seguro que ganarán.";
    let doc = Document::new_plain_english_curated_spanish(texto);

    let lints = linter.lint(&doc);

    println!("\n========================================================");
    println!("   VERIFICACIÓN DE DEQUEÍSMO Y CONCORDANCIA EN HARPER");
    println!("========================================================");
    println!("Texto: \"{}\"", texto);
    println!("Errores encontrados: {}", lints.len());
    println!("--------------------------------------------------------");
    for (i, lint) in lints.iter().enumerate() {
        println!(
            "[{}] Texto: '{}' | Mensaje: {}",
            i + 1,
            doc.get_span_content_str(&lint.span),
            lint.message
        );
    }
    println!("========================================================\n");

    assert!(
        lints.len() >= 3,
        "Se esperaban al menos 3 observaciones de gramática."
    );
}

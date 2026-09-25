use harper_core::linting::{Linter, SpellCheck};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

#[test]
fn probar_regionalismos_hispanos() {
    let dict = FstDictionary::curated_spanish();
    let mut spell_checker = SpellCheck::new(dict.clone(), Dialect::Spanish);

    // Palabras / regionalismos de Colombia, Uruguay, Argentina y México
    let palabras = vec![
        "parce",   // Colombia
        "che",     // Uruguay / Argentina
        "bo",      // Uruguay
        "parcero", // Colombia
        "bacán",   // Colombia / Perú / Chile
        "wey",     // México
        "chido",   // México
        "charrúa", // Uruguay
    ];

    println!("\n========================================================");
    println!("   VERIFICACIÓN DE REGIONALISMOS HISPANOS EN HARPER");
    println!("========================================================");

    for palabra in palabras {
        let doc = Document::new_plain_english_curated_spanish(palabra);
        let lints = spell_checker.lint(&doc);
        let es_valida = lints.is_empty();
        println!(
            "Palabra: '{:10}' | ¿Reconocida por Harper?: {}",
            palabra,
            if es_valida {
                "✅ SÍ"
            } else {
                "❌ NO (Se añadirá)"
            }
        );
    }
    println!("========================================================\n");
}

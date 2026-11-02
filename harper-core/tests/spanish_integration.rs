use harper_core::Document;
use harper_core::linting::Linter;
use harper_core::linting::SpanishGenderAgreement;
use harper_core::spell::{Dictionary, FstDictionary};

#[test]
fn test_spanish_integration_paragraph() {
    let mut linter = SpanishGenderAgreement;

    let text = "¿Hola cómo estás? Tengo la problema con la foto y el mano en el mapa.";
    let document = Document::new_plain_english_curated_spanish(text);

    let lints = linter.lint(&document);

    assert_eq!(
        lints.len(),
        2,
        "Se esperaban 2 errores de concordancia de género."
    );

    println!("\n========================================================");
    println!("   PRUEBA DE INTEGRACIÓN DE CORRECCIÓN EN ESPAÑOL DE HARPER");
    println!("========================================================");
    println!("Texto analizado: \"{}\"", text);
    println!(
        "Palabras cargadas en diccionario español: {}",
        FstDictionary::curated_spanish().word_count()
    );
    println!("--------------------------------------------------------");
    for (i, lint) in lints.iter().enumerate() {
        println!("[Error {}] Mensaje: {}", i + 1, lint.message);
    }
    println!("========================================================\n");
}

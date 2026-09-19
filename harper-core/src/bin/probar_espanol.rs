use harper_core::Document;
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::{Dictionary, MergedDictionary};
use std::io::{self, Write};
use std::sync::Arc;

fn main() {
    println!("========================================================");
    println!("    🦙 HARPER ESPAÑOL - CONSOLA INTERACTIVA DE PRUEBA");
    println!("========================================================");
    println!("Cargando diccionario combinado (Español + Inglés técnico)...");

    let dict = Arc::new(MergedDictionary::curated_spanish_multilingual());
    println!(
        "¡Diccionario listo! ({} palabras cargadas en memoria)",
        dict.word_count()
    );
    println!("--------------------------------------------------------");
    println!("Escribe cualquier frase en español y presiona ENTER.");
    println!("Escribe 'salir' o 'exit' para terminar.\n");

    let mut linter_group = LintGroup::new_curated(dict.clone(), harper_core::Dialect::Spanish);

    loop {
        print!("👉 Frase > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let texto = input.trim();
        if texto.eq_ignore_ascii_case("salir") || texto.eq_ignore_ascii_case("exit") {
            println!("\n¡Hasta luego!");
            break;
        }

        if texto.is_empty() {
            continue;
        }

        let document = Document::new_plain_english_curated_spanish(texto);
        let lints = linter_group.lint(&document);

        if lints.is_empty() {
            println!("   ✅ No se encontraron errores gramaticales o de género.\n");
        } else {
            println!("   ⚠️ Se encontraron {} observacion(es):", lints.len());
            for (i, lint) in lints.iter().enumerate() {
                let sugerencia = if !lint.suggestions.is_empty() {
                    format!(" | Sugerencias: {:?}", lint.suggestions)
                } else {
                    String::new()
                };
                println!("      [{}] {}{}", i + 1, lint.message, sugerencia);
            }
            println!();
        }
    }
}

# Harper (Español) 🇪🇸

Soporte nativo y optimización del idioma **Español** para la herramienta de revisión gramatical y ortográfica **Harper**.

## 🚀 Características Implementadas

1. **Diccionario Integrado en Español (FST)**
   - Diccionario curado con más de **200,000 palabras** en español.
   - Soporte para variaciones morfológicas, plurales y terminaciones comunes (`-ción`, `-s`, `-es`).
   - Integración con términos técnicos (`UTF-8`, `multibyte`, `separación silábica`, `Hyphen`) y palabras compuestas.

2. **Reglas de Gramática Específicas del Español**
   - **`SpanishGenderAgreement`**: Validación de concordancia de género entre artículos/determinantes y sustantivos (ej. detecta *"la problema"* ➡️ *"el problema"*, *"el mano"* ➡️ *"la mano"*).
   - **`SpanishHomophones`**: Detector de homófonos comunes y errores ortográficos de palabras con pronunciación similar.

3. **Reconocimiento de Regionalismos Hispanos**
   - Soporte para regionalismos de Colombia, Uruguay, Argentina, México, Chile, Perú, etc. (ej. *parce*, *che*, *bo*, *parcero*, *bacán*, *wey*, *chido*, *charrúa*).

---

## 🧪 Ejecutar la Suite de Pruebas en Español

Puedes ejecutar todas las pruebas locales de español con el siguiente comando:

```bash
cargo test --package harper-core --test test_spellcheck_es --test test_regionalismos --test test_frase_tecnica --test spanish_integration -- --nocapture
```

---

## 🏗️ Uso en Código (Rust API)

```rust
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::MergedDictionary;
use harper_core::{Document, Dialect};
use std::sync::Arc;

fn main() {
    let dict = Arc::new(MergedDictionary::curated_spanish_multilingual());
    let mut linter = LintGroup::new_curated(dict, Dialect::Spanish);

    let texto = "Hola estov haciendo una prueba con la problema.";
    let doc = Document::new_plain_english_curated_spanish(texto);

    let lints = linter.lint(&doc);
    for lint in lints {
        println!("Error: {}", lint.message);
    }
}
```

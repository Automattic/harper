# Harper Language Support

## Quick Start for Senior Developers

### Adding a New Language

1. **Create module structure:**
   ```bash
   mkdir -p harper-core/src/language/<lang>/linting
   mkdir -p harper-core/src/language/<lang>/tests
   ```

2. **Implement required files** (use existing languages as templates):
   - `mod.rs` - module exports
   - `module.rs` - implement `LanguageModule` trait
   - `dialects.rs` - dialect definitions
   - `language_detection.rs` - language detector
   - `lexing.rs` - token lexing
   - `parsers.rs` - text parser
   - `spell/mod.rs` - dictionary
   - `linting/mod.rs` - linters
   - `annotations.json` - word formation rules
   - `dictionary.dict` - word list with POS tags
   - `tests/mod.rs` - tests

3. **Register the language:**
   - Add feature flag in `harper-core/Cargo.toml`:
     ```toml
     [features]
     <lang> = []
     ```
   - Add to `harper-core/src/language/registry.rs`:
     ```rust
     #[cfg(feature = "<lang>")]
     use super::<lang>::module::<Lang>Module;
     
     // In DETECTORS array:
     #[cfg(feature = "<lang>")]
     detectors.push((Box::new(<Lang>Module::detector()), <confidence>));
     
     // In ProseLanguage enum:
     #[cfg(feature = "<lang>")]
     <Lang>,
     ```
   - Add to `harper-core/src/language/languages.rs`:
     ```rust
     <Lang>(<Lang>Dialect),
     ```

4. **Test it:**
   ```bash
   just language-test-language language="<lang>"
   ```

### Rapid Iteration Without Recompilation

Use the language testing framework to test dictionary and annotations changes instantly:

```bash
# Test text with current dictionary and annotations (no rebuild needed)
just language-german-test "your test text here"

# For other languages, use:
./harper-core/src/language/testing_framework/target/release/harper-lang-test \
    --language <lang> \
    --dict harper-core/src/language/<lang>/dictionary.dict \
    --annotations harper-core/src/language/<lang>/annotations.json \
    --text "your text"

# Build the testing framework first if needed:
just language-build-lang-test
```

### Language Analysis

```bash
# Full analysis of a language (dictionary stats + coverage)
just language-<lang>-analysis

# Validate all dictionaries
just language-validate-dicts

# Test all languages
just language-test-all-languages

# Full language analysis (stats + validation + tests)
just language-full-analysis
```

### Architecture Principles

- Each language is **isolated** in its own module
- Each language implements the `LanguageModule` trait
- English is always included (no feature flag)
- Other languages use **feature flags** (`de`, `pt`, `sk`) for conditional compilation
- All language-specific code uses `#[cfg(feature = "<lang>")]`
- Dictionary and annotations can be tested without recompiling using the testing framework

### Existing Languages to Use as Templates

- **German (de):** Full implementation with linters, dictionary, annotations
- **Portuguese (pt):** Similar structure to German
- **Slovak (sk):** Similar structure to German
- **English:** Special case - always included, different structure

For new languages, copy the structure from German or Portuguese.

### Key Trait: LanguageModule

Every language must implement these methods:

```rust
pub trait LanguageModule {
    type Dialect;
    type Detector;
    
    fn default_dialect() -> Self::Dialect;
    fn detector() -> Self::Detector;
    fn lex_token(source: &[char]) -> FoundToken;
    fn plain_parser() -> impl Parser;
    fn dictionary() -> Arc<FstDictionary>;
    fn rust_lint_group(dictionary: Arc<impl Dictionary>) -> LintGroup;
    fn weir_lint_group() -> LintGroup;
    fn curated_lint_group(dialect: Self::Dialect) -> LintGroup;
}
```

### Directory Structure Example (German)

```
harper-core/src/language/german/
├── mod.rs                    # Module exports
├── module.rs                # LanguageModule implementation
├── dialects.rs               # Dialect definitions
├── language_detection.rs    # Language detector
├── lexing.rs                # Token lexing
├── parsers.rs               # Text parsers
├── spell/
│   └── mod.rs               # Dictionary
├── linting/
│   ├── mod.rs               # Linter group
│   ├── german_noun_capitalization.rs
│   ├── german_spell_check.rs
│   ├── german_sentence_capitalization.rs
│   └── german_filler_words.rs
├── annotations.json          # Word formation rules
├── dictionary.dict           # Word list with POS tags
└── tests/
    └── mod.rs               # Language tests
```

### Notes

- The testing framework (`harper-core/src/language/testing_framework/`) allows rapid iteration
- Once built, you can test dictionary and annotation changes without recompiling
- All language-specific tests go in the language's `tests/` directory
- Use `just language-test-all-languages` to ensure your changes don't break other languages

# Harper Language Support

## Quick Start for Senior Developers

### Special Case: English

**English is built into Harper core and does not use the standard language module structure.**
It uses an embedded dictionary (`FstDictionary::curated()`) and does not require feature flags.
All other languages (German, Portuguese, Slovak, etc.) follow the standard module structure.

### Adding a New Language (non-English)

1. **Create module structure:**
   ```bash
   mkdir -p harper-core/src/language/<lang>/linting
   mkdir -p harper-core/src/language/<lang>/tests
   ```

2. **Implement required files** (copy from German/Portuguese/Slovak):
   - `mod.rs` - module exports
   - `module.rs` - language module implementation
   - `dialects.rs` - dialect definitions
   - `language_detection.rs` - language detector
   - `lexing.rs` - token lexing
   - `parsers.rs` - text parser
   - `spell/mod.rs` - dictionary loading
   - `linting/mod.rs` - linter group
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
     
     #[cfg(feature = "<lang>")]
     detectors.push((Box::new(<Lang>Module::detector()), <confidence>));
     
     #[cfg(feature = "<lang>")]
     <Lang>,
     ```
   - Add to `harper-core/src/language/languages.rs`

4. **Build and test:**
   ```bash
   just language-test-language language="<lang>"
   ```

### Rapid Iteration Without Recompilation

The language testing framework allows you to test dictionary and annotation changes **without rebuilding**:

```bash
# For German (pre-built just recipe):
just language-german-test "your test text here"

# For any language (including new ones):
./harper-core/src/language/testing_framework/target/release/harper-lang-test \
    --language <lang> \
    --dict harper-core/src/language/<lang>/dictionary.dict \
    --annotations harper-core/src/language/<lang>/annotations.json \
    --text "your text"

# Build the framework first:
just language-build-lang-test
```

### Language Analysis Commands

```bash
# Full analysis (stats + coverage) - language specific
just language-<lang>-analysis

# Validate all dictionaries
just language-validate-dicts

# Test all languages
just language-test-all-languages

# Full analysis (all languages)
just language-full-analysis
```

### Template Languages

Use these as templates for new languages:
- **German (de):** Full implementation with linters, dictionary, annotations
- **Portuguese (pt):** Similar to German
- **Slovak (sk):** Similar to German

**Note:** English does not follow this structure - it's embedded in the core.

### Directory Structure (Standard Languages)

```
harper-core/src/language/<lang>/
├── mod.rs                    # Module exports
├── module.rs                # Language module implementation
├── dialects.rs               # Dialect definitions
├── language_detection.rs    # Language detector
├── lexing.rs                # Token lexing
├── parsers.rs               # Text parsers
├── spell/
│   └── mod.rs               # Dictionary loading
├── linting/
│   ├── mod.rs               # Linter group
│   └── <lang>_*.rs          # Language-specific linters
├── annotations.json          # Word formation rules
├── dictionary.dict           # Word list with POS tags
└── tests/
    └── mod.rs               # Language tests
```

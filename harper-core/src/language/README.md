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
   just language-test <lang> "test text"
   ```

### Rapid Iteration Without Recompilation

The language testing framework allows you to test dictionary and annotation changes **without rebuilding**:

```bash
# Test text for any standard language (german, portuguese, slovak, etc.)
just language-test <language> "your test text here"

# Examples:
just language-test german "die freiheit ist wichtig"
just language-test portuguese "a liberdade e importante"

# Build the framework first (only needed once):
just language-build
```

### Language Analysis Commands

```bash
# Full analysis for a specific language
just language-analyze <language>

# Validate all dictionaries
just language-validate-all

# Test all standard languages
just language-test-all

# Full check (validate + test + analyze all languages)
just language-full-check

# Clean build artifacts
just language-clean
```

### Legacy Recipe Names

For backward compatibility, these old names still work:
- `language-german-test` → use `language-test german` instead
- `language-build-lang-test` → use `language-build` instead

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

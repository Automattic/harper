# Harper Language Support

## Quick Start for Senior Developers

### Special Case: English

**English is built into Harper core and does not use the standard language module structure.**
It uses an embedded dictionary and does not require feature flags.
All other languages (German, Portuguese, Slovak, etc.) follow the standard module structure.

### Adding a New Language (non-English)

1. **Create module structure** (copy from German):
   ```bash
   harper-core/src/language/<lang>/
   ├── mod.rs              # Module exports
   ├── module.rs          # Language module implementation (implements LanguageModule)
   ├── dialects.rs         # Dialect definitions
   ├── language_detection.rs  # Language detector
   ├── lexing.rs          # Token lexing
   ├── parsers.rs         # Text parsers
   ├── spell/
   │   └── mod.rs         # Dictionary loading
   ├── linting/
   │   ├── mod.rs         # Linter group
   │   └── <lang>_*.rs    # Language-specific linters
   ├── annotations.json    # Word formation rules
   └── dictionary.dict     # Word list with POS tags
   ```

2. **Register the language:**
   - Add feature flag in `harper-core/Cargo.toml`: `<lang> = []`
   - Add to `harper-core/src/language/registry.rs` with `#[cfg(feature = "<lang>")]`
   - Add to `harper-core/src/language/languages.rs`

3. **Test:**
   ```bash
   just language-test <lang> "test text"
   ```

### Rapid Iteration Without Recompilation

Test dictionary and annotation changes without rebuilding:

```bash
# Test text for any standard language
just language-test german "die freiheit ist wichtig"
just language-test portuguese "a liberdade e importante"

# Build the framework first (only needed once):
just language-build
```

### Complete Language Development Toolkit

```bash
# Full analysis for a specific language
just language-analyze <language>

# Validate all language dictionaries
just language-validate-all

# Test all standard languages
just language-test-all

# Full check (validate + test + analyze all languages)
just language-full-check

# Clean build artifacts
just language-clean

# Run Rust unit tests for a language
just language-rust-test german

# Run all Rust tests for standard languages
just language-rust-test-all
```

### Legacy Recipe Names

For backward compatibility:
- `language-german-test` → use `language-test german` instead
- `language-build-lang-test` → use `language-build` instead

### Template Languages

Use these as templates for new languages:
- **German (de):** Full implementation with linters, dictionary, annotations
- **Portuguese (pt):** Similar to German
- **Slovak (sk):** Similar to German

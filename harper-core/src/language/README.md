# Harper Language Support Architecture

Compile-time plugin architecture. Each language implements the `LanguageModule` trait and integrates automatically via the build system.

## Architecture

- **LanguageModule Trait**: Core interface in `module.rs`
- **Build System**: Discovers languages via `config.toml`, generates all integration code
- **Feature Flags**: Optional languages use `#[cfg(feature)]` for conditional compilation
- **Generated Code**: `mod.rs`, `languages.rs`, `registry.rs`, `dialects/dialect_flags.rs`

## Configuration

Each language requires `harper-core/src/language/<lang>/config.toml`:
```toml
[language]
name = "LanguageName"    # e.g., "German"
dir_name = "language"   # directory name
feature = "lang"       # e.g., "de" (omit for English - always included)

[metadata]
aliases = ["lang", "language"]  # detection aliases
confidence = 0.95                # detection priority (0.0-1.0)

[[dialects]]
name = "Standard"
aliases = ["primary", "aliases"]

[weir]  # Optional
rules_subdirectory = "lang"  # e.g., "de" for weir rules
```

## Required Files

```
harper-core/src/language/<lang>/
├── config.toml          # Language metadata
├── module.rs           # LanguageModule trait implementation
├── dialects.rs         # Dialect definitions
├── language_detection.rs
├── lexing.rs
├── mod.rs
├── parsers/
├── spell/
│   └── <lang>_dict.rs  # Dictionary loading (uses include_str!)
├── linting/
├── dictionary.dict      # Base words with POS flags
├── annotations.json     # Word formation rules + POS mappings
└── test_sources/
```

Files are embedded via `include_str!()`. Structure must match exactly.

## Adding a New Language

1. Create `harper-core/src/language/<lang>/` with required files
2. Implement `LanguageModule` trait in `module.rs`
3. Create `config.toml` with language metadata
4. Add feature to `harper-core/Cargo.toml`: `<lang> = []`
5. Forward feature in dependent Cargo.toml files (harper-cli, harper-ls, harper-wasm) as needed

**Note**: Feature forwarding is intentional. Different binaries can enable different language sets (e.g., CLI with all languages, Chrome plugin with English only).

## Rapid Iteration Without Recompilation

```bash
# Build the testing framework (once):
just language-build

# Test spell checking for any language:
just language-test german "die freiheit ist wichtig"
just language-test portuguese "a liberdade e importante"

# Show metadata for a single word:
just language-meta german "Mondlandung"
just language-meta german "ist"

# Show metadata for all words in text:
just language-meta-text german "die mondlandung ist wieder fehlgeschlagen"

# Run basic dictionary tests:
just language-dict-test german
```

## Coverage and Efficiency Analysis

```bash
# Analyze coverage against expanded dictionary (requires *.dict.gz file)
just language-coverage german

# Analyze efficiency (base words vs expanded coverage)
just language-efficiency german

# Compare Harper with hunspell spell checking
just language-hunspell german "die mondlandung ist wieder fehlgeschlagen"

# Test all example texts in test_sources/ folder
just language-test-examples german
```

These recipes work for any standard language (german, portuguese, slovak, etc.).
Note: English is a special case - some recipes may not work for English.

## Complete Language Development Toolkit

```bash
# Build the testing framework
just language-build

# Test text for a language
just language-test <lang> "text to test"

# Show metadata for a single word
just language-meta <lang> "word"

# Show metadata for all words in text
just language-meta-text <lang> "text to test"

# Run basic dictionary tests
just language-dict-test <lang>

# Run Rust unit tests for a language
just language-rust-test german

# Run all Rust tests for standard languages
just language-rust-test-all

# Clean build artifacts
just language-clean
```

## Improving Dictionary and Annotations

### Step-by-step process:

1. **Add missing words** to `dictionary.dict` with correct POS flags
2. **Add properties** to `annotations.json` if needed for new flags
3. **Test** with `just language-meta <lang> "word"` or `just language-meta-text <lang> "sentence"`
4. **Verify** metadata is correctly applied

### Quick Examples:

Add a noun:
```
Mondlandung/~~NF  # feminine noun
```

Add a verb:
```
schreiben/~~V  # verb
```

Add an adverb:
```
wieder/~~r  # adverb (requires property in annotations.json)
```

Then add the property to annotations.json:
```json
"properties": {
  "r": {"metadata": {"adverb": {}}}
}
```

### Testing:
```bash
# Test single word
just language-meta german "Mondlandung"

# Test sentence
just language-meta-text german "die mondlandung ist wieder fehlgeschlagen"

# Run all tests
python3 harper-core/src/language/german/test_sources/test_german_noun_verification.py

# Compare with hunspell to find missing words
just language-hunspell german "example text with missing words"
```

## Improving Dictionary and Annotations with Example Texts

1. **Create example text files** in `harper-core/src/language/<lang>/test_sources/`
   - Each file should contain example sentences
   - Create a companion `.expected.md` file with expected Harper output

2. **Run example tests** to verify behavior:
   ```bash
   just language-test-examples german
   ```

3. **Compare with Hunspell** to find gaps:
   ```bash
   just language-hunspell german "your test text"
   ```

4. **Check coverage** against the full expanded dictionary:
   ```bash
   just language-coverage german
   ```

5. **Check efficiency** to see how well your rules generate words:
   ```bash
   just language-efficiency german
   ```

## Step-by-Step Improvement Process

### When Harper incorrectly identifies a word's POS:

1. **Identify the issue** with metadata inspection:
   ```bash
   just language-meta german "problemword"
   ```

2. **Fix the dictionary entry** in `dictionary.dict`:
   - Add correct POS flags (e.g., `fehlgeschlagen/~~g` for past participle)
   - Remove incorrect flags

3. **Add missing properties** to `annotations.json` if needed:
   ```json
   {
     "properties": {
       "g": {"metadata": {"verb": {"verb_form": "PAST_PARTICIPLE"}}}
     }
   }
   ```

4. **Test the fix**:
   ```bash
   just language-meta german "problemword"
   just language-test german "sentence with problemword"
   ```

5. **Verify with Hunspell** that the word is recognized:
   ```bash
   just language-hunspell german "problemword"
   ```

### When Harper doesn't recognize a valid word:

1. **Add the word** to `dictionary.dict` with appropriate flags
2. **Add necessary affix rules** to `annotations.json` if it's an inflected form
3. **Test** with the new word
4. **Update example texts** if this was a known gap

## Legacy Recipe Names

For backward compatibility:
- `language-german-test` → use `language-test german` instead
- `language-build-lang-test` → use `language-build` instead

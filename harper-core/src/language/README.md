# Harper Language Support - Architecture Guide

## Core Principle

**Each language uses exactly 2 files:**
- **`dictionary.dict`** - Base words with POS flags
- **`annotations.json`** - Word formation rules + POS metadata mappings

Both files are combined at runtime to create a comprehensive dictionary with metadata.

## File Structure

```
harper-core/src/language/<lang>/
├── mod.rs              # Exports
├── module.rs          # LanguageModule implementation
├── dialects.rs       # Dialect definitions
├── language_detection.rs
├── lexing.rs          # Token lexing
├── parsers/
├── spell/
├── linting/           # Language-specific linters
├── test_sources/      # Test files
├── annotations.json    # Word formation rules + POS mappings
└── dictionary.dict     # Base words with POS flags
```

## Dictionary Format

### dictionary.dict
- One word per line: `word/flags # comment`
- Flags are single characters (POS tags), separated by `~`:
  - `N` = noun, `V` = verb, `J` = adjective, `A` = adjective (alias)
  - `M` = masculine noun, `F` = feminine noun, `Z` = neuter noun, `P` = plural
  - `g` = past participle, `t` = past tense, `c` = comparative, `s` = superlative
  - `r` = adverb, `D` = determiner, `O` = preposition, `C` = conjunction

Example:
```
Mondlandung/~~NF    # compound noun, feminine
schreiben/~~V      # verb infinitive
fehlgeschlagen/~~g # past participle
wieder/~~r         # adverb
```

### annotations.json
- `affixes`: Word formation rules (generate inflected forms)
- `properties`: Maps flags to metadata (e.g., `N` → noun, `V` → verb)

Example:
```json
{
  "properties": {
    "N": {"metadata": {"noun": {}}},
    "V": {"metadata": {"verb": {}}},
    "g": {"metadata": {"verb": {"verb_form": "PAST_PARTICIPLE"}}},
    "r": {"metadata": {"adverb": {}}}
  }
}
```

## Special Case: English

**English is built into Harper core and does NOT use this structure.**
- Uses embedded files: `harper-core/dictionary.dict` + `harper-core/annotations.json`
- No feature flag needed
- All other languages follow the standard module structure above

## Adding a New Language

1. **Copy German** as template to `harper-core/src/language/<lang>/`
2. **Register** in 3 files:
   - `harper-core/Cargo.toml`: add `<lang> = []` feature
   - `harper-core/src/language/registry.rs`: add with `#[cfg(feature = "<lang>")]`
   - `harper-core/src/language/languages.rs`: add language enum
3. **Create** `dictionary.dict` + `annotations.json`
4. **Implement** `module.rs`

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
```

## Legacy Recipe Names

For backward compatibility:
- `language-german-test` → use `language-test german` instead
- `language-build-lang-test` → use `language-build` instead

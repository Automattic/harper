# Harper Language Support - Architecture Guide

## File Structure

Each language's functionality is in a subfolder `harper-core/src/language/<lang>`.
The core trait `LanguageModule` must be implemented. (defined in `harper-core/src/language/module.rs`).

```
harper-core/src/language/<lang>/
├── mod.rs                    # Exports language module
├── module.rs                # LanguageModule trait implementation (REQUIRED)
├── dialects.rs             # Dialect definitions for language variants
├── language_detection.rs   # Language detection logic
├── lexing.rs                # Token lexing rules
├── parsers/                 # Language-specific parser implementations
├── spell/                   # Spell checking module (REQUIRED for dictionary loading)
│   └── <lang>_dict.rs       # Dictionary loading - uses include_str!() for dictionary.dict and annotations.json
├── linting/                 # Language-specific linters
├── test_sources/            # Test files and expected outputs
├── annotations.json          # Word formation rules + POS mappings (REQUIRED - loaded via include_str!)
└── dictionary.dict           # Base words with POS flags (REQUIRED - loaded via include_str!)
```

**Important:** Harper's build system uses `include_str!()` to embed dictionary and annotation files directly into the binary. This means the file structure **must** follow this exact pattern. The spell module (`<lang>_dict.rs`) expects `dictionary.dict` and `annotations.json` in the parent directory.

## Dictionary and Annotations Structure

**Note:** The exact structure of `dictionary.dict` and `annotations.json` varies by language. Consult the language-specific implementations for details.

### dictionary.dict (General Guidelines)
- One word per line: `word/flags # comment`
- Flags are single characters (POS tags), separated by `~`
- Compound word handling varies by language (see language-specific READMEs). 

### annotations.json (General Guidelines)
- `affixes`: Word formation rules (generate inflected forms)
- `properties`: Maps flags to metadata (e.g., `N` → noun, `V` → verb)

For language-specific details, see the individual language directories.

## Special Case: English

**English does NOT use this structure.**
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

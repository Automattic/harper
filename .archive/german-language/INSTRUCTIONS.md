# German Language Support - Improvement Instructions

This file provides **guidance** on how to improve German language support in Harper. It should remain unchanged except for architectural updates. For current status, see `CURRENT_STATE.md`.

## Architecture Overview

Harper uses a **compile-time plugin architecture** with the `LanguageModule` trait. German support is enabled via the `de` feature flag under the `multilingual` umbrella feature in `harper-core/Cargo.toml`.

### Required File Structure
```
harper-core/src/language/german/
├── config.toml              # Language metadata
├── module.rs               # LanguageModule trait implementation
├── dialects.rs             # Dialect definitions
├── language_detection.rs
├── lexing.rs
├── mod.rs
├── parsers/
├── spell/
│   └── de_dict.rs          # Dictionary loading (uses include_str!)
├── linting/                # Language-specific linters
├── dictionary.dict          # Base words with POS flags
├── annotations.json         # Word formation rules + POS mappings
└── test_sources/            # Example texts for testing
```

**Key Principle**: Dictionary flags are **SINGLE CHARACTERS**. Compound flags like `~~JOQRSTUW` are parsed as individual characters: J, O, Q, R, S, T, U, W.

## Improvement Areas

### 1. Dictionary Enhancement

**Goal**: Improve coverage and efficiency by adding missing words and optimizing affix rule usage.

**Files to modify**:
- `dictionary.dict` - Add base words with correct POS flags
- `annotations.json` - Add/improve word formation rules

**Process**:
1. Identify missing words using hunspell comparison
2. Add words to `dictionary.dict` with appropriate single-character flags
3. Leverage existing affix rules to generate inflected forms automatically
4. Test with the provided `just` commands

**Current affix rules available**:
- **Noun pluralization**: X (-e), Y (-n/-en), a (-er+umlaut), b (-s)
- **Adjective declension**: J, O, Q, R, S, T, U, W
- **Verb conjugation**: c, d, e, f, h, i, j
- **Compound formation**: H, I, K, L, M, N

### 2. Compound Word Support

**Status**: Initial implementation completed for compound adjectives and nouns.

**Implementation completed**:
- Compound adjective support: Added `q` flag to 83 adjectives, enabling noun+adjective and adjective+noun compounds
- Compound noun support: Added compound flags (h,i,k,l,m,o) to 3,884 nouns, enabling noun+noun compounds
- The compound system in `compound.rs` automatically handles adjective compounds when words have the `q` flag

**Priority Implementation Order** (updated):
1. ✅ Add compound flags to adjectives in `dictionary.dict` - COMPLETED (83 adjectives)
2. ✅ Extend `compound.rs` to handle adjective compounds - ALREADY IMPLEMENTED
3. ✅ Implement noun + adjective and adjective + noun compounds - COMPLETED
4. ⏳ Add adjective + adjective and verb-based compounds - FUTURE WORK

**Examples of now-working patterns**:
- Compound nouns: `Bundesstaat`, `Mitteleuropa`, `Arbeitsplatz`, `Hauptstadt`
- Compound adjectives: `kindfreundlich`, `verwaltungstechnisch`, `arbeitslos`

**Examples of patterns still missing**:
- Adjective-only compounds: `rotgelb`, `sauerstoffarm`
- Verb-based compounds: `Schreibblock`, `Lesezeichen`
- Mixed POS: `pflichtbewusst`, `schreibfaehig`

**How to add more compound support**:
```bash
# Add q flag to specific adjective
python3 scripts/add_compound_q_to_top_adjectives.py --word adjective_name

# Add compound flag to specific noun  
python3 scripts/add_compound_flags_to_nouns.py --word noun_name --flag h

# Add q flags to top 100 adjectives
python3 scripts/add_compound_q_to_top_adjectives.py --count 100

# Analyze current state
python3 scripts/add_compound_q_to_top_adjectives.py --analyze
python3 scripts/add_compound_flags_to_nouns.py --analyze
```

### 3. Performance Optimization

**Main bottleneck**: `german_noun_capitalization.rs`

**Optimization strategies**:
- Consolidate string operations (single `to_lowercase()` per word)
- Reduce dictionary lookups (single-pass metadata collection)
- Implement early exit strategies
- Order checks by frequency (common cases first)

**Note**: When adding compound flags, be mindful of the O(n²) complexity of compound generation. Adding flags to thousands of words can significantly increase memory usage and processing time. Use targeted approaches.

### 4. Grammar Rule Development

**Current supported**:
- Noun capitalization detection
- Basic compound noun formation
- Affix-based word generation
- Basic spell checking

**To implement**:
- Compound adjective formation and declension
- Advanced compound word handling
- Verb conjugation validation

## Development Workflow

### Testing Commands
```bash
# Build the testing framework (required once)
just language-build

# Test specific text
just language-test german "die mondlandung ist wichtig"

# Show metadata for a single word
just language-meta german "Mondlandung"

# Show metadata for all words in text
just language-meta-text german "sentence here"

# Run dictionary tests
just language-dict-test german

# Analyze coverage against expanded dictionary
just language-coverage german

# Check efficiency (base words vs expanded coverage)
just language-efficiency german

# Compare with hunspell to find missing words
just language-hunspell german "text to test"

# Validate dictionary format, flags, duplicates, Unicode
just language-validate german

# Get dictionary statistics
just language-stats german

# Test all example texts in test_sources/
just language-test-examples german
```

### Adding New Words

1. **Identify missing words**:
   ```bash
   just language-hunspell german "text with missing words"
   ```

2. **Add to dictionary.dict**:
   ```
   # Format: word/~~flags # optional comment
   Mondlandung/~~NF  # feminine noun
   schreiben/~~V    # verb
   wieder/~~r      # adverb (requires property in annotations.json)
   ```

3. **Add properties to annotations.json** if needed:
   ```json
   {
     "properties": {
       "r": {"metadata": {"adverb": {}}}
     }
   }
   ```

4. **Test the changes**:
   ```bash
   just language-meta german "Mondlandung"
   just language-test german "sentence with new word"
   ```

### Creating Example Tests

1. Create text files in `harper-core/src/language/german/test_sources/`
2. Create companion `.expected.md` files with expected Harper output
3. Run tests with:
   ```bash
   just language-test-examples german
   ```

### Wikipedia Test Files

Real-world German Wikipedia articles have been added for testing:
- `wikipedia_deutschland.txt` - Deutschland article excerpt
- `wikipedia_berlin.txt` - Berlin article excerpt

To test with these:
```bash
just language-test german "$(cat harper-core/src/language/german/test_sources/wikipedia_deutschland.txt)"
```

### Wikipedia Scraping Scripts

For obtaining more real-world German texts:
- `scripts/scrape_german_wikipedia.py` - Full-featured scraper (requires requests, bs4)
- `scripts/scrape_german_wikipedia_simple.py` - Simple scraper (standard library only)

Usage:
```bash
# List available topics
python3 scripts/scrape_german_wikipedia_simple.py --list-topics

# Scrape specific topics
python3 scripts/scrape_german_wikipedia_simple.py --topics Deutschland Berlin München

# Analyze coverage
python3 scripts/scrape_german_wikipedia_simple.py --topics Deutschland --analyze
```

### Running Unit Tests
```bash
# Run German-specific unit tests
cargo test --package harper-core --features de --lib language::german
```

## Key Principles

1. **FST-first approach**: Use affix rules to generate inflected forms rather than listing them explicitly.

2. **Iterative testing**: Test after each batch of changes to ensure no regressions.

3. **Architecture preservation**: All changes must work within the existing `LanguageModule` trait and build system.

4. **Feature flags**: German support is gated by the `de` feature flag. Enable with `--features de` or `--features multilingual`.

## References

- `harper-core/src/language/README.md` - Complete language architecture documentation
- `harper-core/src/language/german/README.md` - German-specific development guide (if exists)
- Existing German implementation files - Working examples

## Template for Current State Updates

When updating `CURRENT_STATE.md`, include:
- Date of last update
- Current dictionary statistics (size, coverage, efficiency)
- Recent improvements with impact metrics
- Current gaps and priorities
- Next steps
- Success metrics table

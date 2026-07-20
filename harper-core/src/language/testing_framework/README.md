# Harper Language Testing Framework

This framework allows testing language dictionaries and annotations without recompiling the main Harper binary.

## Overview

The framework provides a way to:
1. Load dictionary and annotation files dynamically at runtime
2. Test spell checking functionality for any language
3. Run basic validation tests
4. Iterate rapidly during language development

## Files

- `Cargo.toml` - Project configuration
- `src/main.rs` - Main testing application
- `README.md` - This file

## Usage

### Build the testing binary

```bash
cd /home/konrad/gallery/harper/harper-core/src/language/testing_framework
cargo build --release
```

### Test a specific language

```bash
# For German
./target/release/harper-lang-test --language german --test

# For English
./target/release/harper-lang-test --language english --test

# For Portuguese
./target/release/harper-lang-test --language portuguese --test
```

### Spell check text with a specific language

```bash
./target/release/harper-lang-test --language german --text "Hallo Welt"
```

### Specify custom file paths (optional)

```bash
./target/release/harper-lang-test --language german --dict path/to/dictionary.dict --annotations path/to/annotations.json --test
```

## Language Support

The framework automatically looks for language files in:
- Dictionary: `../../language/{language}/dictionary.dict`
- Annotations: `../../language/{language}/annotations-{language}.json`

## Workflow

1. **Edit files**: Modify dictionary and annotation files in the language directory
2. **Run tests**: Use the testing binary to validate changes
3. **Analyze results**: Check which words are missing or have issues
4. **Iterate**: Add missing words and refine annotations
5. **Deploy**: When satisfied, the files are already in the correct location for Harper core

## Example

```bash
# Test current German dictionary
./target/release/harper-lang-test --language german --test

# Check specific German text
./target/release/harper-lang-test --language german --text "Der schnelle braune Fuchs springt über den faulen Hund"

# Test with custom files
./target/release/harper-lang-test --language german --dict ../../language/german/test_dict.dict --annotations ../../language/german/test_annotations.json --test
```

## Technical Details

The testing binary uses Harper's `MutableDictionary::from_rune_files()` method to load dictionaries dynamically from file content. This avoids the need to recompile the main Harper binary during development.

## Performance Note

Dynamic file loading is slower than the compiled-in dictionaries used in production. This is intentional - the testing framework prioritizes development flexibility over runtime performance.

## Adding Support for New Languages

To add support for a new language:

1. Create the language directory: `mkdir ../../language/{new_language}`
2. Add dictionary file: `dictionary.dict`
3. Add annotations file: `annotations-{new_language}.json`
4. Use the framework: `./target/release/harper-lang-test --language {new_language} --test`

## Future Enhancements

- Add more sophisticated grammar checking tests
- Support for testing affix rule application
- Integration with LanguageTool rules
- Performance benchmarking
- Batch processing of text files
- Multi-language comparison tests
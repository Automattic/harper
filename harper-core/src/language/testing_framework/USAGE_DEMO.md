# Harper Language Testing Framework - Usage Demonstration

This document demonstrates how to use the testing framework to develop and test language dictionaries without recompiling Harper.

## Framework Location

```
harper-core/src/language/testing_framework/
```

## Current Structure

```
harper-core/src/language/testing_framework/
├── Cargo.toml              # Rust project configuration
├── README.md              # Comprehensive documentation
├── USAGE_DEMO.md          # This file
├── src/
│   └── main.rs            # Main testing application (Rust source)
├── target/
│   └── release/
│       └── harper-lang-test  # Compiled testing binary
└── test_language.sh       # User-friendly wrapper script
```

## How It Works

The framework uses Harper's `MutableDictionary::from_rune_files()` method to load dictionaries and annotations dynamically from file content, avoiding the need to recompile the main Harper binary.

## Usage Examples

### 1. Test German Language (default)

```bash
cd harper-core/src/language/testing_framework
./test_language.sh --test
```

**Output:**
```
🌍 Harper Language Testing Framework
==================================================
📚 Testing language: german
📖 Dictionary: ../../language/german/dictionary.dict
📝 Annotations: ../../language/german/annotations-german.json
🧪 Running basic tests...
✅ Dictionary loaded successfully!
   Word count: 210
📊 Test Results:
   Found: 28/28 words
   Missing: 0/28 words
   ✅ All test words present!
```

### 2. Test Specific German Text

```bash
./test_language.sh --text "Der Hund spielt im Garten"
```

**Output:**
```
🌍 Harper Language Testing Framework
==================================================
📚 Testing language: german
📖 Dictionary: ../../language/german/dictionary.dict
📝 Annotations: ../../language/german/annotations-german.json
🔍 Spell checking text: "Der Hund spielt im Garten"
✅ Dictionary loaded successfully!
   Word count: 210
🔍 Spell checking text: "Der Hund spielt im Garten"
   ✅ All words recognized!
```

### 3. Test Portuguese Language

```bash
./test_language.sh --language portuguese --test
```

**Output:**
```
🌍 Harper Language Testing Framework
==================================================
📚 Testing language: portuguese
📖 Dictionary: ../../language/portuguese/dictionary.dict
📝 Annotations: ../../language/portuguese/annotations-portuguese.json
🧪 Running basic tests...
✅ Dictionary loaded successfully!
   Word count: 147
📊 Test Results:
   Found: 0/28 words  # (Expected - testing with German words)
   Missing: 28/28 words
```

### 4. Test with Custom File Paths

```bash
./test_language.sh --language german --dict /path/to/custom/dict.dict --annotations /path/to/custom/annotations.json --test
```

## Development Workflow

### Step 1: Edit Language Files

Edit the dictionary and annotations files directly in the language directory:

```bash
# Edit German dictionary
nano ../../language/german/dictionary.dict

# Edit German annotations
nano ../../language/german/annotations-german.json
```

### Step 2: Test Changes Immediately

```bash
# Run basic tests
./test_language.sh --test

# Test specific sentences
./test_language.sh --text "Your test sentence here"
```

### Step 3: Analyze Results

The framework will show you:
- ✅ Words that are recognized
- ❌ Words that are missing
- Total word count in the dictionary

### Step 4: Iterate

Add missing words to the dictionary with proper annotations, then test again.

### Step 5: Verify with Official Tests

Once satisfied, run the official Harper tests:

```bash
cd /home/konrad/gallery/harper
cargo test --package harper-core --lib german_annotations_test
```

## File Path Resolution

The framework automatically resolves file paths:

- **Dictionary**: `../../language/{language}/dictionary.dict`
- **Annotations**: `../../language/{language}/annotations-{language}.json`

For German:
- Dictionary: `../../language/german/dictionary.dict`
- Annotations: `../../language/german/annotations-german.json`

For Portuguese:
- Dictionary: `../../language/portuguese/dictionary.dict`
- Annotations: `../../language/portuguese/annotations-portuguese.json`

## Supported Languages

Currently supported languages (with dictionary files):
- `german` - German language support
- `portuguese` - Portuguese language support

## Adding New Languages

To add support for a new language:

1. **Create language directory**:
   ```bash
   mkdir ../../language/{new_language}
   ```

2. **Add dictionary file**: `dictionary.dict`

3. **Add annotations file**: `annotations-{new_language}.json`

4. **Use the framework**:
   ```bash
   ./test_language.sh --language {new_language} --test
   ```

## Performance Considerations

- **Development mode**: Dynamic file loading (slower but flexible)
- **Production mode**: Compiled-in dictionaries (fast)
- **Testing framework**: Prioritizes development flexibility over runtime performance

## Example: Adding Words to German Dictionary

### Current German Test
```bash
./test_language.sh --text "Das Wetter ist heute sehr schön"
```

If words are missing, add them to the dictionary:

```bash
# Edit the dictionary
nano ../../language/german/dictionary.dict

# Add missing words with annotations
Wetter/~N # neuter noun (weather)
heute/~R # adverb (today)
sehr/~R # adverb (very)
schön/~J # adjective (beautiful)
```

### Test Again
```bash
./test_language.sh --text "Das Wetter ist heute sehr schön"
# Should now show: ✅ All words recognized!
```

## Integration with Harper Tests

The framework is designed to work alongside the official Harper test suite:

1. **Rapid iteration**: Use the testing framework for quick development
2. **Final validation**: Run official Harper tests before committing
3. **No recompilation needed**: All changes are in the dictionary/annotation files

## Future Enhancements

The framework can be extended to support:
- Batch processing of text files
- Grammar rule testing
- Affix rule validation
- Performance benchmarking
- Multi-language comparison

## Summary

The Harper Language Testing Framework provides:

✅ **No recompilation needed** - Edit files and test immediately
✅ **Multi-language support** - Works with any language structure
✅ **Direct file access** - Uses files from the code directory
✅ **Comprehensive testing** - Basic tests and custom text checking
✅ **Development workflow** - Rapid iteration cycle
✅ **Production ready** - Files are in the correct location for Harper core

This framework significantly accelerates language development by eliminating the need to recompile Harper for every dictionary or annotation change.
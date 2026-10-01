#!/bin/bash

# Harper Language Testing Framework Wrapper
# This script demonstrates how to use the testing framework with language files from the code directory

LANGUAGE="german"
DICT_PATH="../../language/${LANGUAGE}/dictionary.dict"
ANNOTATIONS_PATH="../../language/${LANGUAGE}/annotations-${LANGUAGE}.json"

# Check if language argument is provided
if [ "$1" == "--language" ] && [ -n "$2" ]; then
    LANGUAGE="$2"
    DICT_PATH="../../language/${LANGUAGE}/dictionary.dict"
    ANNOTATIONS_PATH="../../language/${LANGUAGE}/annotations-${LANGUAGE}.json"
    shift 2
fi

# Check if dictionary file exists
if [ ! -f "${DICT_PATH}" ]; then
    echo "❌ Dictionary file not found: ${DICT_PATH}"
    echo "Available languages:"
    ls -d ../../language/*/ | sed 's|../../language/||' | sed 's|/||'
    exit 1
fi

# Check if annotations file exists
if [ ! -f "${ANNOTATIONS_PATH}" ]; then
    echo "❌ Annotations file not found: ${ANNOTATIONS_PATH}"
    echo "Looking for: annotations-${LANGUAGE}.json"
    exit 1
fi

echo "🌍 Harper Language Testing Framework"
echo "=================================================="
echo "📚 Testing language: ${LANGUAGE}"
echo "📖 Dictionary: ${DICT_PATH}"
echo "📝 Annotations: ${ANNOTATIONS_PATH}"

# Run the actual test binary with the correct paths
if [ "$1" == "--test" ]; then
    echo "🧪 Running basic tests..."
    ./target/release/harper-lang-test --test --dict "${DICT_PATH}" --annotations "${ANNOTATIONS_PATH}"
elif [ "$1" == "--text" ] && [ -n "$2" ]; then
    echo "🔍 Spell checking text: \"$2\""
    ./target/release/harper-lang-test --text "$2" --dict "${DICT_PATH}" --annotations "${ANNOTATIONS_PATH}"
else
    echo ""
    echo "💡 Usage:"
    echo "   ./test_language.sh --language german --test          Run basic tests for German"
    echo "   ./test_language.sh --language german --text \"text\"   Spell check German text"
    echo "   ./test_language.sh --test                             Test German (default)"
    echo ""
    echo "Available languages:"
    ls -d ../../language/*/ | sed 's|../../language/||' | sed 's|/||'
fi
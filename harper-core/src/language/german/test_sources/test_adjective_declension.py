#!/usr/bin/env python3
"""
German Adjective Declension Test Script

This script tests adjective declension forms, particularly testing words like
"unbekannt" and "bekannt" that should generate inflected forms.

Usage:
    python3 test_adjective_declension.py
    
Requirements:
    - Harper language testing framework must be built
    - Run from harper-core/src/language/german/test_sources/
"""

import subprocess
import json
import sys
import os

# Paths to testing framework and language files
# Relative to harper-core/src/language/german/test_sources/
TESTING_FRAMEWORK = "../../../language/testing_framework/target/release/harper-lang-test"
LANGUAGE = "german"
DICT_PATH = "../../../language/german/dictionary.dict"
ANNOTATIONS_PATH = "../../../language/german/annotations.json"

# Test cases for adjective declension
TEST_CASES = [
    {
        "name": "Original Problem Sentence",
        "sentence": "An den Wänden hingen Gemälde von bekannten und unbekannten Künstlern",
        "expected_results": {
            "all_words_recognized": True,
            "unknown_words": []
        },
        "description": "The original sentence that had issues with 'unbekannten'",
        "focus_words": ["unbekannten", "bekannten"]
    },
    {
        "name": "Adjective Base Forms",
        "sentence": "unbekannt bekannt gut schön",
        "expected_results": {
            "all_words_recognized": True,
            "unknown_words": []
        },
        "description": "Test base adjective forms",
        "focus_words": ["unbekannt", "bekannt", "gut", "schön"]
    },
    {
        "name": "Adjective Declension Forms",
        "sentence": "unbekannten bekannten guten schönen",
        "expected_results": {
            "all_words_recognized": True,
            "unknown_words": []
        },
        "description": "Test declension forms of adjectives",
        "focus_words": ["unbekannten", "bekannten", "guten", "schönen"]
    },
    {
        "name": "Mixed Context",
        "sentence": "Die unbekannten Künstler zeigen bekannte Werke in schönen Galerien",
        "expected_results": {
            "all_words_recognized": True,
            "unknown_words": []
        },
        "description": "Test adjectives in a complete sentence context",
        "focus_words": ["unbekannten", "bekannten", "schönen"]
    }
]


def run_spell_check(sentence):
    """Run spell check on a sentence and return unknown words."""
    try:
        # Build the command
        testing_framework_path = os.path.normpath(
            os.path.join(os.path.dirname(__file__), TESTING_FRAMEWORK)
        )
        dict_path = os.path.normpath(
            os.path.join(os.path.dirname(__file__), DICT_PATH)
        )
        annotations_path = os.path.normpath(
            os.path.join(os.path.dirname(__file__), ANNOTATIONS_PATH)
        )
        
        cmd = [
            testing_framework_path,
            "--language", LANGUAGE,
            "--dict", dict_path,
            "--annotations", annotations_path,
            "--text", sentence
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        
        if result.returncode != 0:
            print(f"Error running spell check: {result.stderr}")
            return None, None
        
        # Parse output to find unknown words
        unknown_words = []
        all_recognized = False
        
        for line in result.stdout.split('\n'):
            line = line.strip()
            if "✅ All words recognized!" in line:
                all_recognized = True
            elif "⚠️  Unknown words:" in line:
                all_recognized = False
            elif line.startswith("      - "):
                word = line[6:].strip()
                unknown_words.append(word)
        
        return all_recognized, unknown_words
        
    except Exception as e:
        print(f"Error: {e}")
        return None, None


def run_metadata_check(word):
    """Check metadata for a single word."""
    try:
        testing_framework_path = os.path.normpath(
            os.path.join(os.path.dirname(__file__), TESTING_FRAMEWORK)
        )
        dict_path = os.path.normpath(
            os.path.join(os.path.dirname(__file__), DICT_PATH)
        )
        annotations_path = os.path.normpath(
            os.path.join(os.path.dirname(__file__), ANNOTATIONS_PATH)
        )
        
        cmd = [
            testing_framework_path,
            "--language", LANGUAGE,
            "--dict", dict_path,
            "--annotations", annotations_path,
            "--word", word
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        
        if result.returncode != 0:
            print(f"Error running metadata check for '{word}': {result.stderr}")
            return None
        
        # Parse output to extract POS information
        metadata = {
            "found": False,
            "noun": False,
            "verb": False,
            "adjective": False,
            "adverb": False
        }
        
        for line in result.stdout.split('\n'):
            line = line.strip()
            if "✅ Found" in line:
                metadata["found"] = True
            elif "📚 Noun" in line:
                metadata["noun"] = True
            elif "✍️ Verb" in line:
                metadata["verb"] = True
            elif "🎨 Adjective" in line:
                metadata["adjective"] = True
            elif "💨 Adverb" in line:
                metadata["adverb"] = True
        
        return metadata
        
    except Exception as e:
        print(f"Error checking metadata for '{word}': {e}")
        return None


def check_test_case(test_case):
    """Check a single test case and return results."""
    print(f"\n{'='*70}")
    print(f"Test: {test_case['name']}")
    print(f"Sentence: {test_case['sentence']}")
    print(f"Description: {test_case['description']}")
    print("-" * 70)
    
    # Run spell check
    all_recognized, unknown_words = run_spell_check(test_case['sentence'])
    
    if all_recognized is None:
        print("❌ FAILED: Could not run spell check")
        return False
    
    # Check expected results
    expected = test_case['expected_results']
    all_passed = True
    
    # Check if all words recognized
    if expected['all_words_recognized'] and not all_recognized:
        print(f"  ❌ Expected all words to be recognized, but found unknown words: {unknown_words}")
        all_passed = False
    elif not expected['all_words_recognized'] and all_recognized:
        print(f"  ❌ Expected some unknown words, but all words were recognized")
        all_passed = False
    else:
        print(f"  ✅ Word recognition: {'All words recognized' if all_recognized else 'Expected unknown words found'}")
    
    # Check specific unknown words
    expected_unknown = expected['unknown_words']
    if unknown_words:
        print(f"  Unknown words found: {unknown_words}")
        for word in expected_unknown:
            if word in unknown_words:
                print(f"    ✅ Expected unknown word found: '{word}'")
            else:
                print(f"    ❌ Expected unknown word not found: '{word}'")
                all_passed = False
    
    # Check focus words metadata
    print(f"\n  Focus words analysis:")
    for word in test_case['focus_words']:
        metadata = run_metadata_check(word)
        if metadata and metadata['found']:
            pos_list = []
            if metadata['noun']:
                pos_list.append('Noun')
            if metadata['verb']:
                pos_list.append('Verb')
            if metadata['adjective']:
                pos_list.append('Adjective')
            if metadata['adverb']:
                pos_list.append('Adverb')
            
            if pos_list:
                print(f"    ✅ '{word}': {', '.join(pos_list)}")
            else:
                print(f"    ⚠️  '{word}': Found but no specific POS metadata")
        else:
            print(f"    ❌ '{word}': Not found in dictionary")
            all_passed = False
    
    return all_passed


def main():
    """Run all test cases and report results."""
    print("German Adjective Declension Tests")
    print("=" * 70)
    
    passed_count = 0
    failed_count = 0
    
    for test_case in TEST_CASES:
        if check_test_case(test_case):
            passed_count += 1
            print(f"\n✅ PASSED: {test_case['name']}")
        else:
            failed_count += 1
            print(f"\n❌ FAILED: {test_case['name']}")
    
    print("\n" + "=" * 70)
    print(f"Results: {passed_count} passed, {failed_count} failed")
    print("=" * 70)
    
    # Additional verification: test the specific problematic words
    print(f"\n🔍 Additional Verification:")
    print("Testing the specific words that were problematic...")
    
    problematic_words = ["unbekannt", "unbekannten", "bekannt", "bekannten"]
    all_good = True
    
    for word in problematic_words:
        metadata = run_metadata_check(word)
        if metadata and metadata['found']:
            print(f"  ✅ '{word}': Found in dictionary")
        else:
            print(f"  ❌ '{word}': NOT found in dictionary")
            all_good = False
    
    print(f"\n🎯 Final Status: {'✅ ALL TESTS PASSED' if all_good and failed_count == 0 else '❌ SOME TESTS FAILED'}")
    
    return 0 if (failed_count == 0 and all_good) else 1


if __name__ == "__main__":
    sys.exit(main())
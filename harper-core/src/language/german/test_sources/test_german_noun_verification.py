#!/usr/bin/env python3
"""
German Noun Capitalization Verification Test Script

This script tests the example sentences from german_noun_verification.md
and verifies that words are correctly identified with their POS tags.

Usage:
    python3 test_german_noun_verification.py
    
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

# Test cases from german_noun_verification.md
TEST_CASES = [
    {
        "name": "User's Original Example",
        "sentence": "die mondlandung ist wieder fehlgeschlagen",
        "expected_metadata": {
            "mondlandung": {"noun": True, "verb": False, "adjective": False, "adverb": False},
            "ist": {"noun": False, "verb": True, "adjective": False, "adverb": False},
            "wieder": {"noun": False, "verb": False, "adjective": False, "adverb": True},
            "fehlgeschlagen": {"noun": False, "verb": True, "adjective": False, "adverb": False},
        },
        "description": "Original example from user with Mondlandung, ist, wieder, fehlgeschlagen"
    },
    {
        "name": "Verb Conjugation",
        "sentence": "ich schreibe einen brief und du liest ein buch",
        "expected_metadata": {
            "schreibe": {"noun": False, "verb": True, "adjective": False, "adverb": False},
            "liest": {"noun": False, "verb": True, "adjective": False, "adverb": False},
            "brief": {"noun": True, "verb": False, "adjective": False, "adverb": False},
            "buch": {"noun": True, "verb": False, "adjective": False, "adverb": False},
        },
        "description": "Verb conjugation test with schreibe and liest"
    },
    {
        "name": "Mixed POS",
        "sentence": "die freiheit ist wichtig für die menschheit",
        "expected_metadata": {
            "freiheit": {"noun": True, "verb": False, "adjective": False, "adverb": False},
            "ist": {"noun": False, "verb": True, "adjective": False, "adverb": False},
            "wichtig": {"noun": False, "verb": False, "adjective": True, "adverb": False},
            "menschheit": {"noun": True, "verb": False, "adjective": False, "adverb": False},
        },
        "description": "Mixed parts of speech test"
    },
    {
        "name": "Verb Forms",
        "sentence": "die forschung hat ergebnisse erzielt",
        "expected_metadata": {
            "forschung": {"noun": True, "verb": False, "adjective": False, "adverb": False},
            "hat": {"noun": False, "verb": True, "adjective": False, "adverb": False},
            "ergebnisse": {"noun": True, "verb": False, "adjective": False, "adverb": False},
            "erzielt": {"noun": False, "verb": True, "adjective": False, "adverb": False},
        },
        "description": "Verb forms test with hat and erzielt"
    },
]


def run_test(sentence):
    """Run the language testing framework and return metadata for all words."""
    try:
        # Build the command
        # TESTING_FRAMEWORK is already relative to harper-core/src/language/german/test_sources/
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
            "--metadata",
            "--text", sentence
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        
        if result.returncode != 0:
            print(f"Error running test: {result.stderr}")
            return None
        
        # Parse the output to extract metadata
        # The output format is:
        # 🔹 Word: "word"
        #    ✅ Exact case:
        #       emoji Description
        #    ✅ Lowercase:
        #       emoji Description
        
        metadata = {}
        current_word = None
        
        for line in result.stdout.split('\n'):
            line = line.strip()
            if line.startswith('🔹 Word:'):
                # Extract word
                word = line.split('"')[1]
                current_word = word.lower()  # Normalize to lowercase
                metadata[current_word] = {
                    "noun": False,
                    "verb": False,
                    "adjective": False,
                    "adverb": False
                }
            elif line.startswith('📚 Noun') and current_word:
                metadata[current_word]["noun"] = True
            elif line.startswith('✍️ Verb') and current_word:
                metadata[current_word]["verb"] = True
            elif line.startswith('🎨 Adjective') and current_word:
                metadata[current_word]["adjective"] = True
            elif line.startswith('💨 Adverb') and current_word:
                metadata[current_word]["adverb"] = True
        
        return metadata
        
    except Exception as e:
        print(f"Error: {e}")
        return None


def check_test_case(test_case):
    """Check a single test case and return results."""
    print(f"\n{'='*60}")
    print(f"Test: {test_case['name']}")
    print(f"Sentence: {test_case['sentence']}")
    print(f"Description: {test_case['description']}")
    print("-" * 60)
    
    # Run the test
    actual_metadata = run_test(test_case['sentence'])
    
    if actual_metadata is None:
        print("❌ FAILED: Could not run test")
        return False
    
    # Check expected metadata
    expected = test_case['expected_metadata']
    all_passed = True
    
    for word, expected_pos in expected.items():
        if word not in actual_metadata:
            print(f"  ❌ Word '{word}' not found in results")
            all_passed = False
            continue
        
        actual_pos = actual_metadata[word]
        passed = True
        
        for pos, expected_value in expected_pos.items():
            actual_value = actual_pos.get(pos, False)
            if expected_value and not actual_value:
                # Expected to have this POS but doesn't
                passed = False
                all_passed = False
                print(f"  ❌ '{word}': should have {pos}, but does NOT have {pos}")
            elif not expected_value and actual_value:
                # Expected NOT to have this POS but does
                # Only flag as error if the primary expected POS is not present
                # This allows for words that have multiple POS tags
                primary_expected = [p for p, v in expected_pos.items() if v]
                has_primary = any(actual_pos.get(p, False) for p in primary_expected)
                if not has_primary:
                    passed = False
                    all_passed = False
                    print(f"  ⚠️  '{word}': has unexpected {pos} (but also missing primary POS)")
        
        if passed:
            pos_list = [pos for pos, val in actual_pos.items() if val]
            print(f"  ✅ '{word}': {', '.join(pos_list)}")
    
    return all_passed


def main():
    """Run all test cases and report results."""
    print("German Noun Capitalization Verification Tests")
    print("=" * 60)
    
    passed_count = 0
    failed_count = 0
    
    for test_case in TEST_CASES:
        if check_test_case(test_case):
            passed_count += 1
            print(f"\n✅ PASSED: {test_case['name']}")
        else:
            failed_count += 1
            print(f"\n❌ FAILED: {test_case['name']}")
    
    print("\n" + "=" * 60)
    print(f"Results: {passed_count} passed, {failed_count} failed")
    print("=" * 60)
    
    return 0 if failed_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

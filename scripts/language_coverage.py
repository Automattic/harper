#!/usr/bin/env python3
"""
Language Coverage Analysis Script
Analyzes how many words from the expanded Hunspell dictionary are recognized by Harper
Works for any language that has a compressed dictionary file (*.dict.gz)
"""

import gzip
import subprocess
import sys
import os
import random
from collections import Counter
from pathlib import Path

def analyze_language_coverage(language):
    """Analyze coverage of Harper dictionary against expanded benchmark for any language"""
    
    # Set random seed for reproducibility
    random.seed(42)
    
    print(f"🔍 {language.capitalize()} Coverage Analysis")
    print("=" * 50)
    
    # Paths - standardized for all languages
    base_path = f"harper-core/src/language/{language}"
    expanded_dict_path = f"{base_path}/{language}_dictionary.dict.gz"
    harper_dict_path = f"{base_path}/dictionary.dict"
    annotations_path = f"{base_path}/annotations.json"
    test_framework = "harper-core/src/language/testing_framework/target/release/harper-lang-test"
    
    # Check if files exist
    if not os.path.exists(expanded_dict_path):
        print(f"❌ Expanded dictionary not found: {expanded_dict_path}")
        print(f"   Expected: {language}_dictionary.dict.gz in {base_path}")
        return
    
    if not os.path.exists(harper_dict_path):
        print(f"❌ Harper dictionary not found: {harper_dict_path}")
        return
        
    if not os.path.exists(annotations_path):
        print(f"❌ Annotations not found: {annotations_path}")
        return
        
    if not os.path.exists(test_framework):
        print(f"❌ Test framework not found: {test_framework}")
        print("💡 Please build the test framework first:")
        print("   just language-build")
        return
    
    # Load expanded dictionary
    print("📖 Loading expanded dictionary...")
    with gzip.open(expanded_dict_path, 'rt', encoding='utf-8', errors='ignore') as f:
        expanded_words = [line.strip() for line in f if line.strip()]
    
    print(f"   Loaded {len(expanded_words):,} words from expanded dictionary")
    
    # Filter to reasonable test sample (remove proper nouns, abbreviations, etc.)
    filtered_words = []
    for word in expanded_words:
        # Skip words that start with hyphen or uppercase
        if word.startswith('-') or (word and word[0].isupper()):
            continue
        # Skip very long words (likely compounds we can't handle yet)
        if len(word) > 30:
            continue
        # Skip words that are too short
        if len(word) < 3:
            continue
        # Skip words with special characters we might not handle
        if any(c in word for c in ['/', '\\', '*', '?', '[', ']', '{', '}', '(', ')']):
            continue
        filtered_words.append(word)
    
    # Use all words if less than 10,000, otherwise sample 10,000
    test_word_count = min(10000, len(filtered_words))
    test_words = random.sample(filtered_words, test_word_count) if len(filtered_words) >= 10000 else filtered_words
    
    print(f"   Using {len(test_words):,} words for coverage testing")
    
    # Test words with Harper
    print("🧪 Testing words with Harper...")
    
    recognized = 0
    batch_size = 100
    
    for i in range(0, len(test_words), batch_size):
        batch = test_words[i:i+batch_size]
        text = ' '.join(batch)
        
        try:
            result = subprocess.run([
                test_framework,
                '--language', language,
                '--dict', harper_dict_path,
                '--annotations', annotations_path,
                '--text', text,
            ], capture_output=True, text=True, timeout=60)
            
            if "All words recognized!" in result.stdout:
                recognized += len(batch)
            elif "Unknown words:" in result.stdout:
                # Parse unknown words
                lines = result.stdout.split('\n')
                unknown_section = False
                unknown_count = 0
                for line in lines:
                    if "Unknown words:" in line:
                        unknown_section = True
                        continue
                    if unknown_section and line.strip().startswith('-'):
                        unknown_count += 1
                    elif unknown_section and not line.strip():
                        break
                recognized += (len(batch) - unknown_count)
            elif result.returncode != 0:
                # If there's an error, assume none recognized
                print(f"   ⚠️  Error in batch {i//batch_size + 1}: {result.stderr[:100]}")
                
        except subprocess.TimeoutExpired:
            print(f"   ⚠️  Timeout testing batch {i//batch_size + 1}")
            continue
        except Exception as e:
            print(f"   ❌ Error testing batch {i//batch_size + 1}: {e}")
            continue
    
    coverage_percentage = (recognized / len(test_words)) * 100 if test_words else 0
    
    print(f"\n📊 Coverage Results")
    print(f"   Words Tested: {len(test_words):,}")
    print(f"   Words Recognized: {recognized:,}")
    print(f"   Coverage: {coverage_percentage:.1f}%")
    
    # Dictionary statistics
    print(f"\n📚 Dictionary Statistics")
    
    # Get Harper dictionary word count from first line or count all non-comment lines
    harper_word_count = 0
    with open(harper_dict_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith('#') and '/' in line:
                harper_word_count += 1
    
    print(f"   Harper Dictionary Size: {harper_word_count:,} base words")
    print(f"   Expanded Dictionary Size: {len(expanded_words):,} words")
    
    if harper_word_count > 0 and len(expanded_words) > 0:
        size_ratio = (harper_word_count / len(expanded_words)) * 100
        print(f"   Size Ratio: {size_ratio:.2f}%")
    
    # Calculate efficiency: words covered per base word
    if harper_word_count > 0:
        efficiency = recognized / harper_word_count if recognized > 0 else 0
        print(f"\n🎯 Efficiency Metrics")
        print(f"   Base words: {harper_word_count:,}")
        print(f"   Words recognized: {recognized:,}")
        print(f"   Efficiency ratio: {efficiency:.2f} words per base word")
        
        # Compare to English if available
        print(f"\n   For reference:")
        print(f"   - English typically has ~1.5-2.0 words per base word")
        print(f"   - German should aim for >2.5 due to compounding")
    
    # Annotation statistics
    print(f"\n🏷️  Annotation Statistics")
    try:
        import json
        with open(annotations_path, 'r', encoding='utf-8') as f:
            annotations = json.load(f)
        
        affix_count = len(annotations.get('affixes', {}))
        property_count = len(annotations.get('properties', {}))
        
        print(f"   Affix Rules: {affix_count}")
        print(f"   Property Rules: {property_count}")
        print(f"   Total Rules: {affix_count + property_count}")
    except Exception as e:
        print(f"   ⚠️  Could not parse annotations: {e}")
    
    # Recommendations
    print(f"\n💡 Recommendations")
    if coverage_percentage < 30:
        print(f"   ⚠️  Low coverage ({coverage_percentage:.1f}%) - consider adding more root words")
    elif coverage_percentage < 60:
        print(f"   🟡 Moderate coverage ({coverage_percentage:.1f}%) - focus on common word patterns and affix rules")
    else:
        print(f"   ✅ Good coverage ({coverage_percentage:.1f}%) - focus on edge cases and compound words")
    
    if harper_word_count > 0 and len(expanded_words) > 0:
        target_coverage = 80.0
        words_needed_approx = int((len(expanded_words) * target_coverage / 100 - recognized) / efficiency) if efficiency > 0 else 0
        if coverage_percentage < target_coverage:
            print(f"   🎯 To reach {target_coverage}% coverage: approximately {words_needed_approx:,} more base words or improved rules")
    
    print(f"\n" + "=" * 50)
    print(f"📈 Summary: {coverage_percentage:.1f}% coverage with {harper_word_count:,} base words")
    print(f"   Efficiency: {efficiency:.2f} words per base word" if harper_word_count > 0 else "")
    print(f"=" * 50)

def analyze_all_languages():
    """Analyze coverage for all languages with available data"""
    
    # Find all languages with coverage data
    languages = []
    base_path = "harper-core/src/language"
    
    if os.path.exists(base_path):
        for lang_dir in sorted(os.listdir(base_path)):
            lang_path = os.path.join(base_path, lang_dir)
            if os.path.isdir(lang_path):
                # Check for compressed dictionary
                dict_gz = os.path.join(lang_path, f"{lang_dir}_dictionary.dict.gz")
                if os.path.exists(dict_gz):
                    languages.append(lang_dir)
    
    if not languages:
        print("❌ No languages with coverage data found")
        print("   Expected: harper-core/src/language/<lang>/{lang}_dictionary.dict.gz")
        return
    
    print(f"🌍 Language Coverage Analysis for: {', '.join(languages)}")
    print("=" * 70)
    
    results = []
    for language in languages:
        print()
        analyze_language_coverage(language)
        results.append(language)
    
    print(f"\n✅ Completed coverage analysis for {len(results)} languages")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Analyze specific language
        language = sys.argv[1].lower()
        analyze_language_coverage(language)
    else:
        # Analyze all languages
        analyze_all_languages()

#!/usr/bin/env python3
"""
German Coverage Analysis Script
Analyzes how many words from the expanded Hunspell dictionary are recognized by Harper
"""

import gzip
import subprocess
import sys
import os
import random
from collections import Counter

def analyze_german_coverage():
    """Analyze coverage of German dictionary against expanded benchmark"""
    
    # Set random seed for reproducibility
    random.seed(42)
    
    print("🔍 German Coverage Analysis")
    print("===========================")
    
    # Paths
    expanded_dict_path = "harper-core/src/language/german/german_dictionary.dict.gz"
    harper_dict_path = "harper-core/src/language/german/dictionary.dict"
    annotations_path = "harper-core/src/language/german/annotations-german.json"
    test_framework = "harper-core/src/language/testing_framework/target/release/harper-lang-test"
    
    # Check if files exist
    if not os.path.exists(expanded_dict_path):
        print(f"❌ Expanded dictionary not found: {expanded_dict_path}")
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
        print("   cd harper-core/src/language/testing_framework")
        print("   cargo build --release")
        return
    
    # Load expanded dictionary
    print("📖 Loading expanded dictionary...")
    with gzip.open(expanded_dict_path, 'rt', encoding='utf-8') as f:
        expanded_words = [line.strip() for line in f if line.strip()]
    
    print(f"   Loaded {len(expanded_words):,} words from expanded dictionary")
    
    # Filter to reasonable test sample (remove proper nouns, abbreviations, etc.)
    filtered_words = []
    for word in expanded_words:
        # Skip words that start with hyphen or uppercase
        if word.startswith('-') or word[0].isupper():
            continue
        # Skip very long words (likely compounds we can't handle yet)
        if len(word) > 20:
            continue
        # Skip words that are too short
        if len(word) < 3:
            continue
        filtered_words.append(word)
    
    # Randomly sample 10,000 words from the filtered list
    test_words = random.sample(filtered_words, min(10000, len(filtered_words)))
    
    print(f"   Using {len(test_words):,} randomly sampled words for coverage testing")
    
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
                '--dict', harper_dict_path,
                '--annotations', annotations_path,
                '--text', text,
                '--language', 'german'
            ], capture_output=True, text=True, timeout=30)
            
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
                
        except subprocess.TimeoutExpired:
            print(f"   ⚠️  Timeout testing batch {i//batch_size + 1}")
            continue
        except Exception as e:
            print(f"   ❌ Error testing batch {i//batch_size + 1}: {e}")
            continue
    
    coverage_percentage = (recognized / len(test_words)) * 100 if test_words else 0
    
    print(f"📊 Coverage Results")
    print(f"   Words Tested: {len(test_words):,}")
    print(f"   Words Recognized: {recognized:,}")
    print(f"   Coverage: {coverage_percentage:.1f}%")
    
    # Dictionary statistics
    print(f"\n📚 Dictionary Statistics")
    with open(harper_dict_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    # Parse dictionary header
    word_count = 0
    for line in lines:
        if line.strip().isdigit():
            word_count = int(line.strip())
            break
    
    # English dictionary size for efficiency calculation
    english_dict_size = 54705
    
    print(f"   Harper Dictionary Size: {word_count:,} words")
    print(f"   Expanded Dictionary Size: {len(expanded_words):,} words")
    print(f"   Size Ratio: {word_count/len(expanded_words)*100:.2f}%")
    
    # Efficiency calculation: (coverage / dict_size) / (1 / english_dict_size) * 100
    efficiency = (coverage_percentage / 100) * english_dict_size / word_count * 100
    print(f"   Efficiency: {efficiency:.2f}% (relative to English)")
    
    # Annotation statistics
    print(f"\n🏷️  Annotation Statistics")
    with open(annotations_path, 'r', encoding='utf-8') as f:
        import json
        annotations = json.load(f)
    
    affix_count = len(annotations.get('affixes', {}))
    property_count = len(annotations.get('properties', {}))
    
    print(f"   Affix Rules: {affix_count}")
    print(f"   Property Rules: {property_count}")
    print(f"   Total Rules: {affix_count + property_count}")
    
    # Target analysis
    print(f"\n🎯 Target Analysis")
    target_size = 50000
    print(f"   Target Dictionary Size: {target_size:,} words")
    print(f"   Current Size: {word_count:,} words")
    print(f"   Progress: {word_count/target_size*100:.1f}%")
    print(f"   Words Needed: {target_size - word_count:,}")
    
    # Recommendations
    print(f"\n💡 Recommendations")
    if coverage_percentage < 30:
        print("   ⚠️  Low coverage - consider adding more root words")
    elif coverage_percentage < 60:
        print("   🟡 Moderate coverage - focus on common word patterns")
    else:
        print("   ✅ Good coverage - focus on edge cases and compounds")
    
    if word_count < 40000:
        print(f"   🔄 Dictionary growth needed: {target_size - word_count:,} more words")
    
    print(f"\n===========================")
    print(f"📈 Summary: {coverage_percentage:.1f}% coverage with {word_count:,} words")
    print(f"   Efficiency: {efficiency:.2f}% (vs English baseline)")
    print(f"   Target: {target_size:,} words for comprehensive German support")

if __name__ == "__main__":
    analyze_german_coverage()

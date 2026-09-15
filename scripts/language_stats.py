#!/usr/bin/env python3
"""
Language Dictionary Statistics Script

Provides detailed statistics about dictionary composition for Harper language modules.
"""

import json
import os
import re
from collections import defaultdict
from pathlib import Path

# Regex patterns
COMMENT_PATTERN = re.compile(r'^\s*#')
EMPTY_LINE_PATTERN = re.compile(r'^\s*$')


def get_language_paths(language):
    """Get paths for dictionary and annotations files for a given language."""
    base_path = Path(f"harper-core/src/language/{language}")
    dict_path = base_path / "dictionary.dict"
    annotations_path = base_path / "annotations.json"
    return dict_path, annotations_path


def load_dictionary_data(dict_path):
    """Load and parse dictionary file, returning word data."""
    words = []
    word_lengths = []
    flag_usage = defaultdict(int)
    
    try:
        with open(dict_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for line in lines:
            line = line.rstrip('\n\r').strip()
            
            # Skip comments and empty lines
            if COMMENT_PATTERN.match(line) or EMPTY_LINE_PATTERN.match(line):
                continue
            
            # Skip numeric metadata lines
            if line.isdigit():
                continue
            
            # Parse word/flags format
            if '/' in line:
                entry_part = line.split('#')[0].strip() if '#' in line else line
                if entry_part and '/' in entry_part:
                    parts = entry_part.split('/')
                    word = parts[0].strip()
                    flags_part = '/'.join(parts[1:])
                    
                    if word:
                        words.append(word)
                        word_lengths.append(len(word))
                        
                        # Extract individual flags
                        for flag in flags_part.split('/'):
                            flag = flag.strip()
                            if flag:
                                # Remove ~~ prefix for Harper internal flags
                                clean_flag = flag[2:] if flag.startswith('~~') else flag
                                flag_usage[clean_flag] += 1
        
        return words, word_lengths, flag_usage
        
    except Exception as e:
        print(f"❌ Error loading dictionary: {e}")
        return [], [], defaultdict(int)


def load_annotations_data(annotations_path):
    """Load and parse annotations file."""
    try:
        with open(annotations_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if not content.strip():
            return {'properties': {}, 'affixes': {}}
        
        annotations = json.loads(content)
        return annotations
        
    except Exception as e:
        print(f"❌ Error loading annotations: {e}")
        return {'properties': {}, 'affixes': {}}


def analyze_word_lengths(word_lengths):
    """Analyze word length distribution."""
    if not word_lengths:
        return {}
    
    lengths = defaultdict(int)
    for length in word_lengths:
        lengths[length] += 1
    
    min_len = min(word_lengths)
    max_len = max(word_lengths)
    avg_len = sum(word_lengths) / len(word_lengths)
    
    # Create distribution by length ranges
    ranges = {
        '1-3 chars': 0,
        '4-6 chars': 0,
        '7-10 chars': 0,
        '11-15 chars': 0,
        '16+ chars': 0
    }
    
    for length in word_lengths:
        if length <= 3:
            ranges['1-3 chars'] += 1
        elif length <= 6:
            ranges['4-6 chars'] += 1
        elif length <= 10:
            ranges['7-10 chars'] += 1
        elif length <= 15:
            ranges['11-15 chars'] += 1
        else:
            ranges['16+ chars'] += 1
    
    return {
        'min': min_len,
        'max': max_len,
        'average': avg_len,
        'ranges': ranges
    }


def generate_language_stats(language):
    """Generate comprehensive statistics for a language."""
    print(f"\n📊 {language.capitalize()} Dictionary Statistics")
    print("=" * 60)
    
    dict_path, annotations_path = get_language_paths(language)
    
    # Check file existence
    if not dict_path.exists():
        print(f"❌ Dictionary file not found: {dict_path}")
        return
    
    if not annotations_path.exists():
        print(f"❌ Annotations file not found: {annotations_path}")
        return
    
    # Load data
    words, word_lengths, flag_usage = load_dictionary_data(dict_path)
    annotations = load_annotations_data(annotations_path)
    
    if not words:
        print("❌ No valid word entries found in dictionary")
        return
    
    # Basic statistics
    total_words = len(words)
    unique_words = len(set(words))
    unique_flags = len(flag_usage)
    total_flag_usage = sum(flag_usage.values())
    
    print(f"📚 Dictionary Overview")
    print(f"   Total entries: {total_words:,}")
    print(f"   Unique words: {unique_words:,}")
    if total_words > 0:
        duplicate_percentage = ((total_words - unique_words) / total_words) * 100
        print(f"   Duplicate entries: {total_words - unique_words:,} ({duplicate_percentage:.1f}%)")
    
    # Word length statistics
    if word_lengths:
        length_stats = analyze_word_lengths(word_lengths)
        print(f"\n📏 Word Length Statistics")
        print(f"   Minimum length: {length_stats['min']} characters")
        print(f"   Maximum length: {length_stats['max']} characters")
        print(f"   Average length: {length_stats['average']:.1f} characters")
        print(f"   Length distribution:")
        for range_name, count in length_stats['ranges'].items():
            percentage = (count / len(word_lengths)) * 100 if word_lengths else 0
            print(f"      {range_name}: {count:,} words ({percentage:.1f}%)")
    
    # Flag usage statistics
    print(f"\n🏷️  Flag Usage Statistics")
    print(f"   Total unique flags: {unique_flags}")
    print(f"   Total flag usages: {total_flag_usage:,}")
    
    if flag_usage and unique_flags > 0:
        print(f"   Top 10 most used flags:")
        sorted_flags = sorted(flag_usage.items(), key=lambda x: x[1], reverse=True)[:10]
        for flag, count in sorted_flags:
            percentage = (count / total_flag_usage) * 100
            print(f"      {flag}: {count:,} usages ({percentage:.1f}%)")
    
    # Annotations analysis
    annotations_properties = annotations.get('properties', {})
    annotations_affixes = annotations.get('affixes', {})
    
    print(f"\n📋 Annotations Analysis")
    print(f"   Properties defined: {len(annotations_properties)}")
    print(f"   Affix rules defined: {len(annotations_affixes)}")
    
    # Flag coverage
    defined_flags = set(annotations_properties.keys())
    used_flags = set(flag_usage.keys())
    
    flags_in_annotations_not_used = defined_flags - used_flags
    flags_used_not_in_annotations = used_flags - defined_flags
    
    print(f"\n🔍 Flag Coverage")
    print(f"   Flags defined in annotations: {len(defined_flags)}")
    print(f"   Flags used in dictionary: {len(used_flags)}")
    
    if flags_in_annotations_not_used:
        unused_count = len(flags_in_annotations_not_used)
        print(f"   ⚠️  Flags in annotations but not used: {unused_count}")
        if unused_count <= 10:
            print(f"      Unused flags: {', '.join(sorted(flags_in_annotations_not_used))}")
    
    if flags_used_not_in_annotations:
        undefined_count = len(flags_used_not_in_annotations)
        simple_undefined = {f for f in flags_used_not_in_annotations if len(f) == 1}
        compound_undefined = {f for f in flags_used_not_in_annotations if len(f) > 1}
        
        if simple_undefined:
            print(f"   ❌ Simple flags used but not defined: {len(simple_undefined)}")
            print(f"      Undefined flags: {', '.join(sorted(simple_undefined))}")
        
        if compound_undefined:
            print(f"   ⚠️  Compound flags used: {len(compound_undefined)}")
            if len(compound_undefined) <= 5:
                print(f"      Compound flags: {', '.join(sorted(compound_undefined))}")
    
    # Summary
    print(f"\n" + "=" * 60)
    print(f"📈 Summary for {language}")
    print(f"   Words: {total_words:,} ({unique_words:,} unique)")
    print(f"   Flags: {unique_flags} unique, {total_flag_usage:,} total usages")
    if word_lengths:
        print(f"   Avg word length: {length_stats['average']:.1f} characters")
    print(f"=" * 60)


def generate_all_languages_stats():
    """Generate statistics for all languages."""
    print("🌍 Language Dictionary Statistics for All Languages")
    print("=" * 70)
    
    language_dir = Path("harper-core/src/language")
    all_results = {}
    
    if not language_dir.exists():
        print("❌ Language directory not found")
        return
    
    for lang_dir in sorted(language_dir.iterdir()):
        if lang_dir.is_dir() and not lang_dir.name.startswith('_'):
            config_file = lang_dir / "config.toml"
            dict_file = lang_dir / "dictionary.dict"
            annotations_file = lang_dir / "annotations.json"
            
            if config_file.exists() and dict_file.exists() and annotations_file.exists():
                language = lang_dir.name
                print(f"\n📚 Processing {language}...")
                
                # Load basic data
                words, word_lengths, flag_usage = load_dictionary_data(dict_file)
                annotations = load_annotations_data(annotations_file)
                
                all_results[language] = {
                    'total_words': len(words),
                    'unique_words': len(set(words)),
                    'unique_flags': len(flag_usage),
                    'avg_word_length': sum(word_lengths) / len(word_lengths) if word_lengths else 0,
                    'properties_count': len(annotations.get('properties', {}))
                }
    
    # Summary table
    print(f"\n📊 SUMMARY TABLE")
    print(f"{'Language':<12} {'Total Words':>12} {'Unique Words':>12} {'Unique Flags':>12} {'Avg Length':>10} {'Properties':>10}")
    print("-" * 70)
    
    for language in sorted(all_results.keys()):
        data = all_results[language]
        print(f"{language:<12} {data['total_words']:>12,} {data['unique_words']:>12,} {data['unique_flags']:>12} {data['avg_word_length']:>10.1f} {data['properties_count']:>10}")
    
    # Overall statistics
    total_words_all = sum(data['total_words'] for data in all_results.values())
    total_unique_words_all = sum(data['unique_words'] for data in all_results.values())
    total_flags_all = sum(data['unique_flags'] for data in all_results.values())
    
    print(f"\n🎯 OVERALL STATISTICS")
    print(f"   Total words across all languages: {total_words_all:,}")
    print(f"   Total unique words across all languages: {total_unique_words_all:,}")
    print(f"   Total unique flags across all languages: {total_flags_all}")
    print(f"   Number of languages: {len(all_results)}")
    print(f"=" * 70)


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        # Statistics for specific language
        language = sys.argv[1].lower()
        generate_language_stats(language)
    else:
        # Statistics for all languages
        generate_all_languages_stats()


if __name__ == "__main__":
    import sys
    main()
#!/usr/bin/env python3
"""
Dictionary Diff Tool

Compares dictionaries between two Harper languages or versions.
Finds differences in words, flags, and other characteristics.
"""

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
    return dict_path


def load_dictionary_words(dict_path):
    """Load dictionary and return words with their flags."""
    words_with_flags = {}
    
    try:
        with open(dict_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for line_num, line in enumerate(lines, 1):
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
                        # Remove ~~ prefix from flags and clean up
                        clean_flags = []
                        for flag in flags_part.split('/'):
                            flag = flag.strip()
                            if flag:
                                clean_flag = flag[2:] if flag.startswith('~~') else flag
                                clean_flags.append(clean_flag)
                        
                        flag_set = frozenset(clean_flags) if clean_flags else frozenset()
                        words_with_flags[word] = flag_set
        
        return words_with_flags
        
    except Exception as e:
        print(f"❌ Error loading dictionary: {e}")
        return {}


def extract_clean_flag(flag):
    """Extract clean flag by removing ~~ prefix."""
    return flag[2:] if flag.startswith('~~') else flag


def compare_dictionaries(lang1, lang2):
    """Compare dictionaries between two languages."""
    print(f"\n🔄 Comparing dictionaries: {lang1} vs {lang2}")
    print("=" * 70)
    
    dict_path1 = get_language_paths(lang1)
    dict_path2 = get_language_paths(lang2)
    
    # Check file existence
    if not dict_path1.exists():
        print(f"❌ Dictionary file not found for {lang1}: {dict_path1}")
        return
    
    if not dict_path2.exists():
        print(f"❌ Dictionary file not found for {lang2}: {dict_path2}")
        return
    
    # Load dictionaries
    print(f"📖 Loading {lang1} dictionary...")
    dict1 = load_dictionary_words(dict_path1)
    print(f"   Loaded {len(dict1):,} words from {lang1}")
    
    print(f"📖 Loading {lang2} dictionary...")
    dict2 = load_dictionary_words(dict_path2)
    print(f"   Loaded {len(dict2):,} words from {lang2}")
    
    if not dict1 or not dict2:
        print("❌ Could not load dictionaries for comparison")
        return
    
    # Extract word sets
    words1 = set(dict1.keys())
    words2 = set(dict2.keys())
    
    # Find differences
    words_only_in_lang1 = words1 - words2
    words_only_in_lang2 = words2 - words1
    words_in_both = words1 & words2
    
    print(f"\n📊 Word Comparison")
    print(f"   Words in {lang1} only: {len(words_only_in_lang1):,}")
    print(f"   Words in {lang2} only: {len(words_only_in_lang2):,}")
    print(f"   Words in both: {len(words_in_both):,}")
    print(f"   Total unique words: {len(words1 | words2):,}")
    
    # Show some examples
    if words_only_in_lang1:
        print(f"\n   🟢 Words only in {lang1} (first 10):")
        for word in sorted(words_only_in_lang1)[:10]:
            print(f"      + {word}")
        if len(words_only_in_lang1) > 10:
            print(f"      ... and {len(words_only_in_lang1) - 10} more")
    
    if words_only_in_lang2:
        print(f"\n   🔴 Words only in {lang2} (first 10):")
        for word in sorted(words_only_in_lang2)[:10]:
            print(f"      - {word}")
        if len(words_only_in_lang2) > 10:
            print(f"      ... and {len(words_only_in_lang2) - 10} more")
    
    # Flag comparison for common words
    different_flags = []
    for word in sorted(words_in_both)[:100]:  # Limit to first 100 common words for performance
        flags1 = dict1[word]
        flags2 = dict2[word]
        if flags1 != flags2:
            different_flags.append((word, flags1, flags2))
    
    if different_flags:
        print(f"\n   🏷️  Words with different flags (first 10):")
        for word, flags1, flags2 in different_flags[:10]:
            print(f"      {word}:")
            print(f"         {lang1}: {sorted(flags1) if flags1 else 'No flags'}")
            print(f"         {lang2}: {sorted(flags2) if flags2 else 'No flags'}")
        if len(different_flags) > 10:
            print(f"      ... and {len(different_flags) - 10} more")
    else:
        print(f"\n   ✅ No flag differences found in common words")
    
    # Flag usage statistics
    all_flags1 = set()
    all_flags2 = set()
    
    for flags in dict1.values():
        all_flags1.update(flags)
    for flags in dict2.values():
        all_flags2.update(flags)
    
    flags_only_in_lang1 = all_flags1 - all_flags2
    flags_only_in_lang2 = all_flags2 - all_flags1
    flags_in_both = all_flags1 & all_flags2
    
    print(f"\n🏷️  Flag Comparison")
    print(f"   Flags in {lang1} only: {len(flags_only_in_lang1)}")
    print(f"   Flags in {lang2} only: {len(flags_only_in_lang2)}")
    print(f"   Flags in both: {len(flags_in_both)}")
    print(f"   Total unique flags: {len(all_flags1 | all_flags2)}")
    
    if flags_only_in_lang1:
        print(f"   🟢 Flags only in {lang1}: {sorted(flags_only_in_lang1)[:20]}" + 
              ("..." if len(flags_only_in_lang1) > 20 else ""))
    
    if flags_only_in_lang2:
        print(f"   🔴 Flags only in {lang2}: {sorted(flags_only_in_lang2)[:20]}" + 
              ("..." if len(flags_only_in_lang2) > 20 else ""))
    
    # Summary
    print(f"\n" + "=" * 70)
    print(f"📈 Comparison Summary: {lang1} vs {lang2}")
    print(f"   {lang1} words: {len(dict1):,}, {lang2} words: {len(dict2):,}")
    print(f"   Overlap: {len(words_in_both):,} words ({len(words_in_both) / max(len(dict1), len(dict2)) * 100:.1f}% of larger)")
    print(f"   Flag overlap: {len(flags_in_both)} flags ({len(flags_in_both) / max(len(all_flags1), len(all_flags2)) * 100:.1f}% of larger)")
    print(f"=" * 70)


def main():
    """Main entry point."""
    import sys
    
    if len(sys.argv) == 3:
        lang1 = sys.argv[1].lower()
        lang2 = sys.argv[2].lower()
        compare_dictionaries(lang1, lang2)
    else:
        print("Usage: python3 dictionary_diff.py <language1> <language2>")
        print("Example: python3 dictionary_diff.py german portuguese")
        sys.exit(1)


if __name__ == "__main__":
    main()
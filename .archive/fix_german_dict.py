#!/usr/bin/env python3
"""
Script to fix common German dictionary annotation errors.

This script identifies and corrects German words that are incorrectly marked as nouns (N)
when they should be marked as verbs (V) or adjectives (A).

The main issue is that the auto-generated dictionary has many verb forms and participles
incorrectly marked as nouns, which causes false positives in the German noun
capitalization linter.
"""

import re
from pathlib import Path

def load_dictionary(filepath):
    """Load the dictionary file."""
    with open(filepath, 'r', encoding='utf-8') as f:
        return f.readlines()

def save_dictionary(filepath, lines):
    """Save the dictionary file."""
    with open(filepath, 'w', encoding='utf-8') as f:
        f.writelines(lines)

def fix_german_annotations():
    """Fix common German annotation errors in the dictionary."""
    dict_file = Path('harper-core/src/language/german/dictionary.dict')
    
    if not dict_file.exists():
        print(f"Dictionary file not found: {dict_file}")
        return
    
    print(f"Loading dictionary from {dict_file}...")
    lines = load_dictionary(dict_file)
    
    # Track changes made
    changes_made = 0
    
    # Pattern for lines with annotations: word/annotation # comment
    line_pattern = re.compile(r'^(.+)/~~([A-Z]+)\s*(#.*)?$')
    
    # Common verb suffixes that should not be marked as nouns when they appear as standalone words
    verb_suffixes = [
        'e', 'st', 't', 'en',  # Present tense
        'te', 'ten',  # Preterite
        't', 'en',  # Participles
    ]
    
    # Common words that are often incorrectly marked as nouns
    # These are verb forms that should be marked as verbs (V)
    verb_forms_to_fix = {
        # schreiben conjugations
        'schreibe', 'schreibst', 'schreibt', 'schrieben', 'schriebt', 'schriebst',
        # andere Verben
        'beschreibe', 'beschreibst', 'beschreibt', 'beschrieben',
        'anschreibe', 'anschreibst', 'anschreibt', 'anschrieben',
        # fehlschlagen
        'fehlschlagen',
    }
    
    # Common participles that should be marked as adjectives (A)
    participles_to_fix = {
        'fehlgeschlagen',
        'geschlagen',
        'abgeschlagen',
        'umgeschlagen',
        'zugeschlagen',
        'aufgeschlagen',
        'ausgeschlagen',
        'eingeschlagen',
        'hingeschlagen',
        'losgeschlagen',
        'totgeschlagen',
        'vorgeschlagen',
        'weggeschlagen',
        'dazugeschlagen',
        'handgeschlagen',
        'hochgeschlagen',
        'kahlgeschlagen',
        'kaltgeschlagen',
    }
    
    print("Fixing verb forms...")
    for i, line in enumerate(lines):
        match = line_pattern.match(line.strip())
        if match:
            word, annotation, comment = match.groups()
            
            # Check if this is a verb form that should be V, not N
            if annotation == 'N' and word in verb_forms_to_fix:
                lines[i] = line.replace('/~~N', '/~~V')
                changes_made += 1
                print(f"  Fixed {word}: N -> V at line {i+1}")
            
            # Check if this is a participle that should be A, not N
            elif annotation == 'N' and word in participles_to_fix:
                lines[i] = line.replace('/~~N', '/~~A')
                changes_made += 1
                print(f"  Fixed {word}: N -> A at line {i+1}")
    
    print(f"\nFixing all verb conjugations ending with typical verb suffixes...")
    
    # More general approach: fix words ending with typical verb suffixes
    # that are marked as N but don't look like nouns
    verb_endings = ['e', 'st', 't', 'en']
    
    for i, line in enumerate(lines):
        match = line_pattern.match(line.strip())
        if match:
            word, annotation, comment = match.groups()
            
            # Skip if already fixed or not marked as N
            if annotation != 'N':
                continue
            
            # Skip if it has a compound marker (like -heit, -keit, -ung, etc.)
            if any(word.endswith(suffix) for suffix in ['heit', 'keit', 'ung', 'nis', 'tum', 'ion', 'tät', 'schaft']):
                continue
            
            # Check if word ends with verb suffixes and doesn't have noun-like characteristics
            if any(word.endswith(ending) for ending in verb_endings):
                # Additional checks to avoid false positives
                # - Not a known noun (we can't check this easily without a noun list)
                # - Not in the list of words we already fixed
                if word not in verb_forms_to_fix and word not in participles_to_fix:
                    # For now, only fix words that contain verb-like patterns
                    # This is a conservative approach to avoid breaking valid nouns
                    if any(verb in word for verb in ['schreib', 'fehl', 'geh', 'mach', 'sag', 'hab']):
                        lines[i] = line.replace('/~~N', '/~~V')
                        changes_made += 1
                        print(f"  Fixed {word}: N -> V (by pattern) at line {i+1}")
    
    print(f"\nTotal changes made: {changes_made}")
    
    if changes_made > 0:
        print(f"Saving changes to {dict_file}...")
        save_dictionary(dict_file, lines)
        print("Done!")
    else:
        print("No changes needed.")

if __name__ == '__main__':
    fix_german_annotations()

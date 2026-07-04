#!/usr/bin/env python3
"""
Conservative script to fix German dictionary annotation errors.

This script only fixes the most obvious errors to avoid breaking valid nouns.
The main focus is on:
1. Specific verb forms that are commonly used (like "schreibe", "fehlgeschlagen")
2. Participles of the form "*geschlagen" which should be adjectives
3. Conjugations of common verbs like "schreiben", "beschreiben", etc.
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
    """Fix German annotation errors in the dictionary."""
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
    
    # Specific verb forms that should be V, not N
    # These are commonly used words that cause false positives
    verb_forms_to_fix = {
        # schreiben and its conjugations
        'schreibe', 'schreibst', 'schreibt', 'schrieben', 'schriebt', 'schriebst',
        'beschreibe', 'beschreibst', 'beschreibt', 'beschrieben',
        'anschreibe', 'anschreibst', 'anschreibt', 'anschrieben',
        'abschreibe', 'abschreibst', 'abschreibt', 'abschrieben',
        'aufschreibe', 'aufschreibst', 'aufschreibt', 'aufschrieben',
        'ausschreibe', 'ausschreibst', 'ausschreibt', 'ausschrieben',
        'einschreibe', 'einschreibst', 'einschreibt', 'einschrieben',
        'gutschreibe', 'gutschreibst', 'gutschreibt', 'gutschrieben',
        'herschreibe', 'herschreibst', 'herschreibt', 'herschrieben',
        'umschreibe', 'umschreibst', 'umschreibt', 'umschrieben',
        'zuschreibe', 'zuschreibst', 'zuschreibt', 'zuschrieben',
        'fehlschlagen',
    }
    
    # Specific participles that should be A (adjectives), not N
    # These are all forms ending in "geschlagen" which are participles, not nouns
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
    
    print("Fixing specific verb forms...")
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
    
    print(f"\nTotal changes made: {changes_made}")
    
    if changes_made > 0:
        print(f"Saving changes to {dict_file}...")
        save_dictionary(dict_file, lines)
        print("Done!")
    else:
        print("No changes needed.")

if __name__ == '__main__':
    fix_german_annotations()

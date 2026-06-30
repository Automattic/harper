#!/usr/bin/env python3

import re
from collections import defaultdict

def analyze_verb_conjugation():
    dict_file = "harper-core/src/language/german/dictionary.dict"
    
    # Read dictionary
    with open(dict_file, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    # Parse dictionary entries: word/flags # comment
    entries = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#') or line == '134791':
            continue
        parts = line.split('/')
        if len(parts) >= 2:
            word = parts[0]
            flags_comment = parts[1:]
            flags = flags_comment[0].split('#')[0].strip() if flags_comment else ''
            entries.append((word, flags))
    
    print(f"Total dictionary entries: {len(entries)}")
    
    # Count entries by flag
    flag_counts = defaultdict(int)
    for word, flags in entries:
        flag_counts[flags] += 1
    
    print("\nFlag distribution:")
    for flags, count in sorted(flag_counts.items(), key=lambda x: x[1], reverse=True)[:10]:
        print(f"  {flags}: {count}")
    
    # Find verb bases - words that have multiple conjugated forms
    # Strategy: look for words ending in common verb endings
    verb_ending_patterns = [
        ('en', 'infinitive/plural'),
        ('e', 'present 1st person'), 
        ('st', 'present 2nd person'),
        ('t', 'present 3rd person'),
        ('te', 'preterite 1st/3rd'),
        ('ten', 'preterite plural'),
        ('test', 'preterite 2nd'),
        ('tet', 'preterite 2nd plural'),
        ('end', 'present participle'),
        ('nd', 'present participle'),
    ]
    
    # Find potential verb bases
    potential_verbs = defaultdict(list)
    
    for word, flags in entries:
        if 'V' in flags:  # Already marked as verb
            for ending, desc in verb_ending_patterns:
                if word.endswith(ending) and len(word) > len(ending):
                    base = word[:-len(ending)]
                    potential_verbs[base].append((word, ending, desc, flags))
                    break
    
    # Find bases with multiple forms
    multi_form_bases = {base: forms for base, forms in potential_verbs.items() if len(forms) >= 2}
    
    print(f"\nVerb bases with multiple conjugated forms: {len(multi_form_bases)}")
    
    # Show some examples
    for base, forms in sorted(multi_form_bases.items())[:10]:
        print(f"  {base}: {forms}")
    
    # Count total verb forms that could be converted
    total_convertible = sum(len(forms) for forms in multi_form_bases.values())
    print(f"\nTotal convertible verb forms: {total_convertible}")
    print(f"Unique bases: {len(multi_form_bases)}")
    print(f"Potential savings: {total_convertible - len(multi_form_bases)} words")
    
    return multi_form_bases

if __name__ == "__main__":
    analyze_verb_conjugation()
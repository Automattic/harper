#!/usr/bin/env python3

import re
from collections import defaultdict

def analyze_verb_patterns():
    dict_file = "harper-core/src/language/german/dictionary.dict"
    
    # Read dictionary
    with open(dict_file, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    # Extract words with verb flags
    verb_words = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#') or line == '134791':
            continue
        if '/~~V' in line:
            word = line.split('/')[0]
            verb_words.append(word)
    
    print(f"Total verb-marked words: {len(verb_words)}")
    
    # Analyze verb endings
    ending_counts = defaultdict(int)
    verb_base_forms = defaultdict(list)
    
    for word in verb_words:
        # Skip very short words
        if len(word) <= 2:
            continue
            
        # Find potential verb endings
        endings_to_check = [
            'en', 'e', 'st', 't', 'te', 'ten', 'test', 'tet', 
            'est', 'em', 'es', 'nd', 'end', 'endem', 'enden'
        ]
        
        for ending in endings_to_check:
            if word.endswith(ending):
                ending_counts[ending] += 1
                base = word[:-len(ending)]
                verb_base_forms[base].append(word)
                break  # Only count the longest matching ending
    
    print("\nTop verb endings:")
    for ending, count in sorted(ending_counts.items(), key=lambda x: x[1], reverse=True)[:20]:
        print(f"  {ending}: {count}")
    
    print(f"\nBase forms that could use affix rules:")
    # Find bases with multiple forms
    multi_form_bases = {base: forms for base, forms in verb_base_forms.items() if len(forms) >= 2}
    print(f"Bases with multiple forms: {len(multi_form_bases)}")
    
    for base, forms in sorted(multi_form_bases.items())[:10]:
        print(f"  {base}: {forms}")
    
    return verb_words, verb_base_forms

if __name__ == "__main__":
    analyze_verb_patterns()
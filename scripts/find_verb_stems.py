#!/usr/bin/env python3

import re
from collections import defaultdict

def find_verb_stems():
    dict_file = "harper-core/src/language/german/dictionary.dict"
    
    # Read dictionary
    with open(dict_file, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    # Parse dictionary entries
    entries = []
    for line in lines:
        line = line.strip()
        if not line or line.startswith('#') or line == '134791':
            continue
        parts = line.split('/')
        if len(parts) >= 2:
            word = parts[0]
            flags_part = parts[1]
            flags = flags_part.split('#')[0].strip()
            entries.append((word, flags))
    
    print(f"Total dictionary entries: {len(entries)}")
    
    # Group words by their potential verb stem
    # Common German verb conjugation patterns:
    # infinitive: -en (lernen)
    # present: -e (ich lerne), -st (du lernst), -t (er lernt), -en (wir lernen)
    # preterite: -te (ich lernte), -test (du lerntest), -te (er lernte), -ten (wir lernten)
    # participles: -end (lernend), -t (gelernt)
    
    verb_groups = defaultdict(list)
    
    # First, collect all words with ~~V flag (these are verb forms)
    verb_words = [(word, flags) for word, flags in entries if '~~V' in flags or 'V' in flags]
    
    print(f"Verb-marked words: {len(verb_words)}")
    
    # For each verb word, try to find its stem by removing common verb endings
    for word, flags in verb_words:
        # Skip very short words
        if len(word) <= 3:
            continue
        
        # Try to find the verb stem
        endings = [
            ('en', 'infinitive'),      # lernen -> lern
            ('e', 'present_1st'),       # lerne -> lern
            ('st', 'present_2nd'),      # lernst -> lern
            ('t', 'present_3rd'),       # lernt -> lern
            ('te', 'preterite_1st'),   # lernte -> lern
            ('test', 'preterite_2nd'), # lerntest -> lern
            ('ten', 'preterite_pl'),   # lernten -> lern
            ('tet', 'preterite_2ndpl'),# lerntet -> lern
            ('end', 'participle_pres'),# lernend -> lern
            ('nd', 'participle_pres'), # lernnd -> lern (unlikely but possible)
            ('t', 'participle_past'),  # gelernt -> gelernt (this is tricky)
        ]
        
        stem_found = False
        for ending, desc in endings:
            if word.endswith(ending) and len(word) > len(ending):
                stem = word[:-len(ending)]
                verb_groups[stem].append((word, ending, desc, flags))
                stem_found = True
                break
        
        if not stem_found:
            # No common ending found, treat the whole word as potential stem
            verb_groups[word].append((word, '', 'unknown', flags))
    
    # Filter to groups with multiple forms
    multi_form_groups = {stem: forms for stem, forms in verb_groups.items() if len(forms) >= 2}
    
    print(f"Stems with multiple verb forms: {len(multi_form_groups)}")
    
    # Show examples
    print("\nExamples of verb stems with multiple forms:")
    for stem, forms in sorted(multi_form_groups.items())[:15]:
        print(f"  {stem}: {forms}")
    
    # Count total forms and potential savings
    total_forms = sum(len(forms) for forms in multi_form_groups.values())
    potential_savings = total_forms - len(multi_form_groups)
    
    print(f"\nTotal verb forms in multi-form groups: {total_forms}")
    print(f"Unique stems: {len(multi_form_groups)}")
    print(f"Potential dictionary size reduction: {potential_savings} words")
    
    # Calculate efficiency improvement
    current_dict_size = 134791
    new_dict_size = current_dict_size - potential_savings
    current_coverage = 24.1  # from analysis
    new_efficiency = (current_coverage / 100) * 54705 / new_dict_size * 100
    old_efficiency = (current_coverage / 100) * 54705 / current_dict_size * 100
    
    print(f"\nEfficiency improvement:")
    print(f"  Current efficiency: {old_efficiency:.2f}%")
    print(f"  New efficiency: {new_efficiency:.2f}%")
    print(f"  Improvement: {new_efficiency - old_efficiency:.2f} percentage points")
    
    return multi_form_groups

if __name__ == "__main__":
    find_verb_stems()
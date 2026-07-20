#!/usr/bin/env python3

import re
from collections import defaultdict

def convert_verbs():
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
            flags_part = parts[1]
            flags = flags_part.split('#')[0].strip()
            comment = parts[2] if len(parts) > 2 else ''
            entries.append((word, flags, comment))
    
    print(f"Total dictionary entries: {len(entries)}")
    
    # Group verb entries by potential base form
    # Only consider entries with ~~V flag (pure verb forms, not nouns marked as verbs)
    verb_entries = [(word, flags, comment) for word, flags, comment in entries 
                   if flags.strip() == '~~V']
    
    print(f"Pure verb entries (~~V only): {len(verb_entries)}")
    
    # Find potential verb stems
    verb_stems = defaultdict(list)
    
    # Verb conjugation patterns to look for
    patterns = [
        ('en', 's'),   # infinitive/plural -> base + s (lern-en -> lern + s)
        ('e', 'p'),    # present 1st person -> base + p (lern-e -> lern + p)  
        ('st', 'q'),   # present 2nd person -> base + q (lern-st -> lern + q)
        ('t', 'r'),    # present 3rd person -> base + r (lernt -> lern + r)
        ('te', 'u'),   # preterite 1st/3rd -> base + u (lern-te -> lern + u)
        ('ten', 'v'),  # preterite plural -> base + v (lern-ten -> lern + v)
        ('test', 'w'), # preterite 2nd -> base + w (lern-test -> lern + w)
        ('tet', 'x'),  # preterite 2nd plural -> base + x (lern-tet -> lern + x)
    ]
    
    # Group verb forms by their base
    base_to_forms = defaultdict(list)
    
    for word, flags, comment in verb_entries:
        # Skip very short words
        if len(word) <= 3:
            continue
        
        for ending, affix_flag in patterns:
            if word.endswith(ending) and len(word) > len(ending):
                base = word[:-len(ending)]
                base_to_forms[base].append((word, ending, affix_flag, comment))
                break  # Take the first matching pattern
        else:
            # No pattern matched, keep as is
            base_to_forms[word].append((word, '', '', comment))
    
    # Filter to bases with multiple forms that can use affix rules
    convertible_bases = {}
    for base, forms in base_to_forms.items():
        # Only consider bases that have at least 2 forms with affix rules
        affix_forms = [(word, ending, affix_flag, comment) for word, ending, affix_flag, comment in forms if affix_flag]
        if len(affix_forms) >= 2:
            convertible_bases[base] = affix_forms
    
    print(f"Bases with multiple convertible verb forms: {len(convertible_bases)}")
    
    # Show examples
    print("\nExamples of convertible verb bases:")
    for base, forms in sorted(convertible_bases.items())[:10]:
        print(f"  {base}: {[(word, ending, affix_flag) for word, ending, affix_flag, _ in forms]}")
    
    # Count total savings
    total_original_forms = sum(len(forms) for forms in convertible_bases.values())
    total_base_forms = len(convertible_bases)
    potential_savings = total_original_forms - total_base_forms
    
    print(f"\nTotal convertible verb forms: {total_original_forms}")
    print(f"Unique bases: {total_base_forms}")
    print(f"Potential dictionary size reduction: {potential_savings} words")
    
    # Calculate efficiency improvement
    current_dict_size = 134791
    new_dict_size = current_dict_size - potential_savings
    current_coverage = 24.1  # from analysis
    old_efficiency = (current_coverage / 100) * 54705 / current_dict_size * 100
    new_efficiency = (current_coverage / 100) * 54705 / new_dict_size * 100
    
    print(f"\nEfficiency improvement:")
    print(f"  Current efficiency: {old_efficiency:.2f}%")
    print(f"  New efficiency: {new_efficiency:.2f}%")
    print(f"  Improvement: {new_efficiency - old_efficiency:.2f} percentage points")
    
    # Create the conversion mapping
    conversion_map = {}
    for base, forms in convertible_bases.items():
        # Collect all affix flags for this base
        affix_flags = set(affix_flag for _, _, affix_flag, _ in forms)
        combined_flags = ''.join(sorted(affix_flags))
        
        # The base form should be the infinitive (ending in 'en') or the shortest form
        base_word = base + 'en'  # typically the infinitive
        
        # All the individual forms that can be removed
        removable_forms = [word for word, _, _, _ in forms if word != base_word]
        
        conversion_map[base_word] = {
            'affix_flags': combined_flags,
            'removable_forms': removable_forms,
            'base': base
        }
    
    print(f"\nConversion candidates: {len(conversion_map)}")
    
    # Save the conversion mapping for later use
    import json
    with open('scripts/verb_conversion_mapping.json', 'w', encoding='utf-8') as f:
        json.dump(conversion_map, f, indent=2, ensure_ascii=False)
    
    print("Saved conversion mapping to scripts/verb_conversion_mapping.json")
    
    return conversion_map

if __name__ == "__main__":
    convert_verbs()
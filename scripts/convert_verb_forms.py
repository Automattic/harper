#!/usr/bin/env python3

import re
from collections import defaultdict

def convert_verb_forms():
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
            comment = parts[2] if len(parts) > 2 else ''
            entries.append((word, flags, comment))
    
    print(f"Total dictionary entries: {len(entries)}")
    
    # Identify verb forms that can be converted
    # Pattern: base verb ending in -en, with corresponding -ten and -enden forms
    
    # Group by base (removing common endings)
    base_groups = defaultdict(list)
    
    for word, flags, comment in entries:
        if '~~V' not in flags:
            continue  # Only process verb-marked entries
            
        # Skip compound forms for now - focus on simple verbs
        if any(prefix in word for prefix in ['ab', 'an', 'auf', 'aus', 'be', 'er', 'ge', 'um', 'zu', 'ver']):
            continue
        
        # Try to identify base verb stem
        if word.endswith('en') and len(word) > 3:
            base = word[:-2]
            base_groups[base].append((word, 'en', flags, comment))
        elif word.endswith('ten') and len(word) > 4:
            base = word[:-3]
            base_groups[base].append((word, 'ten', flags, comment))
        elif word.endswith('enden') and len(word) > 6:
            base = word[:-5]
            base_groups[base].append((word, 'enden', flags, comment))
    
    # Filter to bases with multiple forms
    multi_form_bases = {base: forms for base, forms in base_groups.items() if len(forms) >= 2}
    
    print(f"Bases with multiple verb forms: {len(multi_form_bases)}")
    
    # Show examples
    print("\nExamples of verb bases with multiple forms:")
    for base, forms in sorted(multi_form_bases.items())[:15]:
        endings = [ending for word, ending, _, _ in forms]
        print(f"  {base}: endings={endings}, forms={[word for word, _, _, _ in forms]}")
    
    # Count total savings potential
    total_forms = sum(len(forms) for forms in multi_form_bases.values())
    unique_bases = len(multi_form_bases)
    potential_savings = total_forms - unique_bases
    
    print(f"\nTotal forms: {total_forms}")
    print(f"Unique bases: {unique_bases}")
    print(f"Potential savings: {potential_savings} words")
    
    # Create the conversion mapping
    conversion_map = {}
    for base, forms in multi_form_bases.items():
        # Find the base form (usually the one ending in 'en')
        base_form = None
        other_forms = []
        
        for word, ending, flags, comment in forms:
            if ending == 'en':
                base_form = word
            else:
                other_forms.append(word)
        
        if base_form is None:
            # No base form found, skip
            continue
            
        # Determine which affix flags to use
        affix_flags = set()
        for word, ending, _, _ in forms:
            if ending == 'en':
                affix_flags.add('s')  # -en suffix
            elif ending == 'ten':
                affix_flags.add('v')  # -ten suffix  
            elif ending == 'enden':
                affix_flags.add('y')  # -enden suffix (need to add this rule)
        
        combined_flags = ''.join(sorted(affix_flags))
        
        conversion_map[base_form] = {
            'affix_flags': combined_flags,
            'removable_forms': other_forms,
            'base': base
        }
    
    print(f"\nConversion candidates: {len(conversion_map)}")
    
    # Save the conversion mapping
    import json
    with open('scripts/verb_conversion_candidates.json', 'w', encoding='utf-8') as f:
        json.dump(conversion_map, f, indent=2, ensure_ascii=False)
    
    print("Saved conversion candidates to scripts/verb_conversion_candidates.json")
    
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
    
    return conversion_map

if __name__ == "__main__":
    convert_verb_forms()
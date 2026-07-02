#!/usr/bin/env python3

import re
from collections import defaultdict

def convert_verbs_to_affixes():
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
    
    # Extract verb entries
    verb_entries = [(word, flags, comment) for word, flags, comment in entries if '~~V' in flags]
    print(f"Verb entries: {len(verb_entries)}")
    
    # Group verb entries by their potential base (removing common endings)
    base_groups = defaultdict(list)
    
    # Patterns to match: base + ending -> flag
    # Sort by length (longest first) to ensure proper matching
    # Use the new verb flags that Harper recognizes: c, d, e, f, h, i, j
    patterns = [
        ('enden', 'c'),   # present participle: lernenden -> base + c
        ('ten', 'e'),     # preterite plural: lernten -> base + e  
        ('te', 'd'),      # preterite 1st/3rd: lernte -> base + d
        ('en', 'j'),      # infinitive: lernen -> base + j
        ('st', 'h'),      # present 2nd: lernst -> base + h
        ('t', 'i'),       # present 3rd: lernt -> base + i
        ('e', 'f'),       # present 1st: lerne -> base + f
    ]
    
    # Group by base - collect all possible matches for each word
    for word, flags, comment in verb_entries:
        # Skip very short words
        if len(word) <= 3:
            continue
        
        # Find all matching patterns for this word
        matches = []
        for ending, affix_flag in patterns:
            if word.endswith(ending) and len(word) > len(ending):
                base = word[:-len(ending)]
                matches.append((base, ending, affix_flag))
        
        if matches:
            # Use the first match for grouping
            base, ending, affix_flag = matches[0]
            base_groups[base].append((word, ending, affix_flag, flags, comment, matches))
        else:
            # No pattern matched, add with empty ending
            base_groups[word].append((word, '', '', flags, comment, []))
    
    # Filter to bases with multiple verb forms that can use affix rules
    multi_form_bases = {}
    for base, forms in base_groups.items():
        # Only consider bases that have at least 2 forms with affix rules
        affix_forms = [(word, ending, affix_flag, flags, comment, matches) for word, ending, affix_flag, flags, comment, matches in forms if affix_flag]
        if len(affix_forms) >= 2:
            multi_form_bases[base] = affix_forms
    
    print(f"Bases with multiple convertible verb forms: {len(multi_form_bases)}")
    
    # Show examples
    print("\nExamples of verb bases with multiple forms:")
    for base, forms in sorted(multi_form_bases.items())[:15]:
        endings = [ending for word, ending, _, _, _, _ in forms]
        words = [word for word, _, _, _, _, _ in forms]
        print(f"  {base}: endings={endings}, words={words}")
    
    # Count total savings potential
    total_forms = sum(len(forms) for forms in multi_form_bases.values())
    unique_bases = len(multi_form_bases)
    potential_savings = total_forms - unique_bases
    
    print(f"\nTotal forms: {total_forms}")
    print(f"Unique bases: {unique_bases}")
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
    for base, forms in multi_form_bases.items():
        # Find the base form (prefer the one ending in 'en' - infinitive)
        base_form = None
        other_forms = []
        all_affix_flags = set()
        
        for word, ending, affix_flag, flags, comment, matches in forms:
            all_affix_flags.add(affix_flag)
            if ending == 'en':
                base_form = word
            else:
                other_forms.append(word)
        
        # If no -en form found, use the first form as base
        if base_form is None and forms:
            base_form = forms[0][0]  # First word
            other_forms = [word for word, _, _, _, _, _ in forms[1:]]
        
        combined_flags = ''.join(sorted(all_affix_flags))
        
        conversion_map[base_form] = {
            'base': base,
            'affix_flags': combined_flags,
            'removable_forms': other_forms,
            'original_entries': [(word, ending, flags, comment) for word, ending, _, flags, comment, _ in forms]
        }
    
    print(f"\nConversion candidates: {len(conversion_map)}")
    
    # Save the conversion mapping
    import json
    with open('scripts/verb_conversion_detailed.json', 'w', encoding='utf-8') as f:
        json.dump(conversion_map, f, indent=2, ensure_ascii=False)
    
    print("Saved conversion candidates to scripts/verb_conversion_detailed.json")
    
    return conversion_map

if __name__ == "__main__":
    convert_verbs_to_affixes()
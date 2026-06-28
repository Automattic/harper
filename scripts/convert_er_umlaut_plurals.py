#!/usr/bin/env python3
"""
Script to convert German nouns with -er plurals + umlaut to use the 'a' affix rule.
This targets words that form plurals with -er and umlaut changes.
"""

import re
from collections import defaultdict

# Paths
DICT_PATH = "harper-core/src/language/german/dictionary.dict"

def load_dict_entries():
    """Load dictionary entries into memory."""
    entries = {}
    lines = []
    
    with open(DICT_PATH, 'r', encoding='utf-8') as f:
        for line_num, line in enumerate(f, 1):
            line = line.strip()
            lines.append((line_num, line))
            
            if not line or line.startswith('#') or line == str(len(entries)):
                continue
                
            # Parse entry: word/annotations # comment
            if '/' in line:
                word_part, rest = line.split('/', 1)
                if ' ' in rest:
                    annotations, comment = rest.split(' ', 1)
                else:
                    annotations = rest
                    comment = ""
            else:
                continue
                
            entries[word_part] = {
                'annotations': annotations,
                'comment': comment.strip('#').strip(),
                'line_num': line_num,
                'line': line
            }
    
    return entries, lines

def is_noun_word(annotations):
    """Check if a word is primarily a noun."""
    noun_props = ['N', 'M', 'F', 'Z', 'P']
    for prop in noun_props:
        if f'~~{prop}' in annotations or f'~~{prop} ' in annotations:
            return True
    return False

def apply_umlaut(word):
    """Apply German umlaut to a word."""
    umlaut_map = {
        'a': 'ä', 'A': 'Ä',
        'o': 'ö', 'O': 'Ö',
        'u': 'ü', 'U': 'Ü',
        'au': 'äu', 'Au': 'Äu', 'AU': 'ÄU'
    }
    
    result = word
    # Try single character umlauts first
    for original, umlauted in umlaut_map.items():
        if len(original) == 1:
            if original in result:
                result = result.replace(original, umlauted)
                return result
    
    # Try multi-character umlauts
    for original, umlauted in umlaut_map.items():
        if len(original) == 2:
            if original in result:
                result = result.replace(original, umlauted)
                return result
    
    return None

def find_er_umlaut_plural_candidates(entries):
    """Find noun candidates for -er + umlaut plural conversion."""
    candidates = []
    
    # Look for words that are nouns and have plural forms ending in -er with umlaut
    for word, data in entries.items():
        if not is_noun_word(data['annotations']):
            continue
            
        # Try applying umlaut and adding -er
        umlauted_word = apply_umlaut(word)
        if umlauted_word:
            plural_word = umlauted_word + 'er'
            if plural_word in entries and is_noun_word(entries[plural_word]['annotations']):
                candidates.append((word, plural_word, umlauted_word))
    
    return candidates

def generate_conversion_script(candidates, entries, lines):
    """Generate and perform the conversions."""
    
    print(f"Found {len(candidates)} -er + umlaut plural candidates")
    
    # Filter to only high-confidence candidates
    filtered_candidates = []
    for base, plural, umlauted in candidates:
        # Skip if base word is too short
        if len(base) < 3:
            continue
            
        # Skip if base word looks like it's already an inflected form
        if base.endswith(('e', 'en', 'er', 'es', 'em', 'nd', 'ng', 'st', 'n')):
            continue
            
        # Skip if the plural word has different annotations that might be important
        base_annos = entries[base]['annotations']
        plural_annos = entries[plural]['annotations']
        
        # Only convert if both are simple nouns (not verbs, adjectives, etc.)
        if 'V' in base_annos or 'J' in base_annos or 'V' in plural_annos or 'J' in plural_annos:
            continue
            
        filtered_candidates.append((base, plural, umlauted))
    
    print(f"Filtered to {len(filtered_candidates)} high-confidence candidates")
    
    # Show examples
    print("\nExample candidates:")
    for i, (base, plural, umlauted) in enumerate(filtered_candidates[:10]):
        print(f"  {base} -> {umlauted} -> {plural}")
    
    # Perform the conversions
    conversion_count = 0
    modifications = []
    
    for base, plural, umlauted in filtered_candidates:  # Convert all candidates
        base_data = entries[base]
        plural_data = entries[plural]
        
        # Modify base to add 'a' suffix rule (for -er + umlaut plurals)
        base_annos = base_data['annotations']
        if not 'a' in base_annos:
            new_base_annos = base_annos + 'a'
            line_num = base_data['line_num']
            comment = base_data['comment']
            modifications.append((line_num, f"{base}/{new_base_annos} # {comment}"))
            
        # Comment out plural entry
        plural_line_num = plural_data['line_num']
        plural_line = plural_data['line']
        modifications.append((plural_line_num, f"# REMOVED: now generated by {base}/{new_base_annos}"))
        
        conversion_count += 1
    
    print(f"\nGenerated {conversion_count} conversions")
    
    # Return the modifications
    return filtered_candidates, modifications

def main():
    """Main function."""
    print("Loading German dictionary...")
    entries, lines = load_dict_entries()
    print(f"Loaded {len(entries)} dictionary entries")
    
    print("\nFinding -er + umlaut plural candidates...")
    candidates = find_er_umlaut_plural_candidates(entries)
    
    print("\nGenerating conversion script...")
    filtered_candidates, modifications = generate_conversion_script(candidates, entries, lines)
    
    # Apply the modifications to the dictionary file
    if modifications:
        print(f"\nApplying {len(modifications)} modifications to dictionary...")
        
        # Read the current file
        with open(DICT_PATH, 'r', encoding='utf-8') as f:
            file_lines = f.readlines()
        
        # Sort modifications by line number in reverse order so we can modify from bottom to top
        modifications.sort(key=lambda x: x[0], reverse=True)
        
        # Apply modifications
        for line_num, new_line in modifications:
            if line_num < len(file_lines):
                file_lines[line_num - 1] = new_line + '\n'
        
        # Write back to file
        with open(DICT_PATH, 'w', encoding='utf-8') as f:
            f.writelines(file_lines)
        
        print("Dictionary updated successfully!")
        
        # Update word count
        word_count_line = 0
        with open(DICT_PATH, 'r', encoding='utf-8') as f:
            for i, line in enumerate(f):
                line = line.strip()
                if line and line.isdigit():
                    word_count_line = i
                    current_count = int(line)
                    break
        
        if word_count_line > 0:
            # Count actual word entries (non-comment, non-empty lines)
            word_count = 0
            with open(DICT_PATH, 'r', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#') and '/' in line and not line.startswith(str(word_count)):
                        word_count += 1
            
            # Update the word count
            with open(DICT_PATH, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            lines[word_count_line] = str(word_count) + '\n'
            
            with open(DICT_PATH, 'w', encoding='utf-8') as f:
                f.writelines(lines)
            
            print(f"Updated word count to {word_count}")

if __name__ == "__main__":
    main()
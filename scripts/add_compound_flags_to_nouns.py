#!/usr/bin/env python3
"""
Script to add compound flags to German nouns that commonly appear in compound words.

This is a targeted approach to improve compound word recognition without causing
memory issues by adding too many combinations.
"""

import re
import os
import shutil
from pathlib import Path
import argparse
import time

def parse_dictionary_entry(line):
    """Parse a dictionary entry line into word, flags, and comment."""
    line = line.strip()
    if not line or line.startswith('#') or line == '233400':
        return None
    
    if '/' not in line:
        return None
    
    try:
        # Split on first /
        word_part, rest = line.split('/', 1)
        word = word_part.strip()
        
        # Handle both single tilde and double tilde formats
        if rest.startswith('~~'):
            rest_without_tilde = rest[2:]
            tilde_format = '~~'
        elif rest.startswith('~'):
            rest_without_tilde = rest[1:]
            tilde_format = '~'
        else:
            return None
        
        # Split comment
        if '#' in rest_without_tilde:
            flags_part, comment = rest_without_tilde.split('#', 1)
            flags = flags_part.strip()
            comment = comment.strip()
        else:
            flags = rest_without_tilde.strip()
            comment = ''
        
        return {
            'word': word,
            'flags': flags,
            'comment': comment,
            'original_line': line,
            'tilde_format': tilde_format
        }
    except Exception as e:
        print(f"Error parsing line: {line} - {e}")
        return None

def add_compound_flag_to_noun(dict_path, noun_word, compound_flag='h'):
    """Add a compound flag to a specific noun."""
    
    with open(dict_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    modified = False
    
    for i, line in enumerate(lines):
        entry = parse_dictionary_entry(line)
        if entry is None:
            continue
        
        # Check if this is the word we're looking for (case-insensitive)
        if entry['word'].lower() == noun_word.lower():
            flags = entry['flags']
            
            # Check if it's a noun (has N, M, F, Z flags)
            is_noun = any(flag in flags for flag in ['N', 'M', 'F', 'Z'])
            
            if is_noun and compound_flag not in flags:
                # Add the compound flag
                new_flags = flags + compound_flag
                tilde_format = entry['tilde_format']
                new_line = f"{entry['word']}/{tilde_format}{new_flags}"
                if entry['comment']:
                    new_line += f" #{entry['comment']}"
                new_line += '\n'
                lines[i] = new_line
                modified = True
                print(f"  Added {compound_flag} flag to: {entry['word']} (flags: {flags} -> {new_flags})")
                break
    
    if modified:
        # Write the modified dictionary
        # Note: No backup needed - dictionary is under Git version control
        
        with open(dict_path, 'w', encoding='utf-8') as f:
            f.writelines(lines)
        
        print(f"✅ Modified dictionary")
    else:
        print(f"❌ Could not find noun '{noun_word}' or it already has compound flags")
    
    return modified

def add_compound_flags_to_common_nouns(dict_path):
    """Add compound flags to common nouns that are missing them."""
    
    # List of common nouns that should have compound flags
    # These are nouns that commonly appear in compound words
    common_nouns = {
        'staat': 'h',      # Staat -> Bundesstaat, Rechtsstaat
        'land': 'h',      # Land -> Bundesland, Ausland  
        'stadt': 'h',      # Stadt -> Hauptstadt, Großstadt
        'haus': 'h',       # Haus -> Wohnhaus, Schulhaus
        'mann': 'h',       # Mann -> Businessmann, Feuerwehmann
        'frau': 'h',       # Frau -> Geschäftsfrau, Hausfrau
        'kind': 'h',       # Kind -> Schulkind, Wirtskind (already has h via Kind/~~Nh)
        'leben': 'h',      # Leben -> Arbeitsleben, Privatleben
        'arbeit': 'i',     # Arbeit -> Arbeitsplatz, Arbeitszeit (s interfix)
        'zeit': 'h',       # Zeit -> Arbeitszeit, Freizeit
        'platz': 'h',      # Platz -> Arbeitsplatz, Parkplatz
        'weg': 'h',        # Weg -> Fußweg, Radweg
        'teil': 'h',       # Teil -> Ersatztteil, Wertteil
        'punkt': 'h',      # Punkt -> Endpunkt, Ausgangspunkt
        'jahr': 'h',       # Jahr -> Lebensjahr, Schuljahr
        'tag': 'h',        # Tag -> Arbeitstag, Feiertag
        'woche': 'h',      # Woche -> Arbeitswoche, Schulwoche
        'monat': 'h',      # Monat -> Arbeitsmonat, Schulmonat
        'kraft': 'h',      # Kraft -> Arbeitskraft, Handkraft
        'recht': 'h',      # Recht -> Arbeitsrecht, Menschenrecht
        'system': 'h',     # System -> Betriebssystem, Computersystem
        'dienst': 'h',     # Dienst -> Postdienst, Zivildienst
        'wirtschaft': 'h', # Wirtschaft -> Planwirtschaft, Marktwirtschaft
        'schule': 'h',     # Schule -> Grundschule, Hauptschule
        'plus': 'h',       # Not a German word, but just in case
    }
    
    with open(dict_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    modified_count = 0
    
    for i, line in enumerate(lines):
        entry = parse_dictionary_entry(line)
        if entry is None:
            continue
        
        word_lower = entry['word'].lower()
        
        if word_lower in common_nouns:
            target_flag = common_nouns[word_lower]
            flags = entry['flags']
            
            # Check if it's a noun (has N, M, F, Z flags)
            is_noun = any(flag in flags for flag in ['N', 'M', 'F', 'Z'])
            
            if is_noun and target_flag not in flags:
                # Add the compound flag
                new_flags = flags + target_flag
                tilde_format = entry['tilde_format']
                new_line = f"{entry['word']}/{tilde_format}{new_flags}"
                if entry['comment']:
                    new_line += f" #{entry['comment']}"
                new_line += '\n'
                lines[i] = new_line
                modified_count += 1
                print(f"  Added {target_flag} flag to: {entry['word']} (flags: {flags} -> {new_flags})")
    
    if modified_count > 0:
        # Write the modified dictionary
        # Note: No backup needed - dictionary is under Git version control
        
        with open(dict_path, 'w', encoding='utf-8') as f:
            f.writelines(lines)
        
        print(f"✅ Added compound flags to {modified_count} common nouns")
    else:
        print("❌ No modifications made")
    
    return modified_count

def analyze_nouns_with_compound_flags(dict_path):
    """Analyze nouns that have compound flags."""
    compound_flags = ['h', 'i', 'k', 'l', 'm', 'o']
    
    with open(dict_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    total_nouns = 0
    with_compound_flags = 0
    compound_flag_counts = {flag: 0 for flag in compound_flags}
    
    for line in lines:
        entry = parse_dictionary_entry(line)
        if entry is None:
            continue
        
        flags = entry['flags']
        
        # Check if it's a noun
        is_noun = any(flag in flags for flag in ['N', 'M', 'F', 'Z'])
        
        if is_noun:
            total_nouns += 1
            
            for flag in compound_flags:
                if flag in flags:
                    with_compound_flags += 1
                    compound_flag_counts[flag] += 1
                    break  # Count each noun only once
    
    print(f"📊 Noun compound flag statistics:")
    print(f"  Total nouns: {total_nouns}")
    print(f"  Nouns with compound flags: {with_compound_flags}")
    print(f"  Percentage: {(with_compound_flags / total_nouns * 100):.1f}%" if total_nouns > 0 else "  Percentage: 0%")
    
    print(f"  Compound flag distribution:")
    for flag, count in compound_flag_counts.items():
        print(f"    {flag}: {count}")

def main():
    parser = argparse.ArgumentParser(description='Add compound flags to German nouns')
    parser.add_argument('--dict', type=str, 
                        default='/home/konrad/gallery/harper/harper-core/src/language/german/dictionary.dict',
                        help='Path to the German dictionary file')
    parser.add_argument('--analyze', action='store_true', 
                        help='Analyze nouns with compound flags')
    parser.add_argument('--add-common', action='store_true',
                        help='Add compound flags to common nouns')
    parser.add_argument('--word', type=str, 
                        help='Add compound flag to a specific noun')
    parser.add_argument('--flag', type=str, default='h',
                        help='Compound flag to add (h, i, k, l, m, o)')
    
    args = parser.parse_args()
    
    dict_path = args.dict
    
    if not os.path.exists(dict_path):
        print(f"❌ Dictionary file not found: {dict_path}")
        return
    
    if args.analyze:
        analyze_nouns_with_compound_flags(dict_path)
        return
    
    if args.add_common:
        add_compound_flags_to_common_nouns(dict_path)
        return
    
    if args.word:
        add_compound_flag_to_noun(dict_path, args.word, args.flag)
        return
    
    print("Please specify --analyze, --add-common, or --word")

if __name__ == '__main__':
    main()
#!/usr/bin/env python3
"""
Script to add compound adjective flags (q) to the most common German adjectives.

This is a conservative approach that adds the q flag to a limited number
of common adjectives to improve compound adjective support without causing
memory/performance issues.
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

def add_q_to_top_adjectives(dict_path, max_adjectives=200):
    """Add q flag to the most common adjectives."""
    
    # List of most common German adjectives (based on frequency data)
    # These are adjectives that are most likely to be used in compound formation
    top_adjectives = [
        # Basic descriptions
        'groß', 'klein', 'alt', 'neu', 'jung', 'gut', 'schlecht', 'hoch', 'niedrig',
        'lang', 'kurz', 'breit', 'schmal', 'stark', 'schwach', 'schnell', 'langsam',
        'leicht', 'schwer', 'einfach', 'kompliziert', 'wichtig', 'unwichtig',
        'teuer', 'günstig', 'reich', 'arm', 'voll', 'leer', 'offen', 'geschlossen',
        'sicher', 'gefährlich', 'sauber', 'schmutzig', 'trocken', 'nass',
        
        # Temperature and physical properties
        'warm', 'kalt', 'heiß', 'kühl', 'hell', 'dunkel', 'laut', 'leise',
        'hart', 'weich', 'glatt', 'rau', 'glänzend', 'matt', 'bunt', 'farbig',
        
        # Health and condition
        'gesund', 'krank', 'frisch', 'faul', 'fröhlich', 'traurig', 'müde', 'wach',
        'hungrig', 'durstig', 'satt', 'fertig', 'unfertig', 'bekannt', 'unbekannt',
        
        # Possibility and necessity
        'möglich', 'unmöglich', 'notwendig', 'unnötig', 'praktisch', 'theoretisch',
        
        # Natural vs artificial
        'natürlich', 'künstlich', 'automatisch', 'manuell', 'digital', 'analog',
        
        # Scientific and technical
        'elektrisch', 'mechanisch', 'chemisch', 'biologisch', 'physikalisch',
        'mathematisch', 'historisch', 'kulturell', 'politisch', 'wirtschaftlich',
        'sozial', 'technisch', 'wissenschaftlich', 'künstlerisch', 'musikalisch',
        
        # Other common adjectives
        'literarisch', 'philosophisch', 'psychologisch', 'medizinisch', 'juristisch',
        'ökonomisch', 'finanziell', 'industriell', 'agrarisch', 'maritim',
        'militärisch', 'zivil', 'öffentliche', 'privat', 'international',
        'national', 'lokal', 'regional', 'global', 'universell',
        'spezifisch', 'allgemein', 'individuell', 'kollektiv', 'persönlich',
        'offiziell', 'inoffiziell', 'formell', 'informell', 'direkt',
        'indirekt', 'positiv', 'negativ', 'neutral', 'objektiv', 'subjektiv',
        'aktiv', 'passiv', 'dynamisch', 'statisch', 'flexibel', 'starr',
        'modern', 'antik', 'klassisch', 'innovativ', 'traditionell',
        'effektiv', 'ineffektiv', 'produktiv', 'unproduktiv', 'kreativ',
        'original', 'kopiert', 'authentisch', 'falsch', 'echt',
        'simpliziert', 'komplex', 'homogen', 'heterogen', 'uniform',
        'vielfältig', 'einzigartig', 'normal', 'anormal', 'standard',
        'optimal', 'suboptimal', 'maximal', 'minimal', 'proportional',
        'relativ', 'absolut', 'theoretisch', 'praktisch', 'abstrakt',
        'konkret', 'exakt', 'ungefähre', 'genau', 'präzise',
        'vage', 'klar', 'deutlich', ' undeutlich', 'sichtbar',
        'unsichtbar', 'hörbar', 'unhörbar', 'spürbar', 'fühlbar'
    ]
    
    with open(dict_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    modified_count = 0
    top_adjectives_set = set(top_adjectives)
    
    for i, line in enumerate(lines):
        entry = parse_dictionary_entry(line)
        if entry is None:
            continue
        
        flags = entry['flags']
        word = entry['word'].lower()
        
        if word in top_adjectives_set:
            if 'q' not in flags:
                # Check if it has adjective flags
                if 'J' in flags or 'A' in flags:
                    new_flags = flags + 'q'
                    tilde_format = entry['tilde_format']
                    new_line = f"{entry['word']}/{tilde_format}{new_flags}"
                    if entry['comment']:
                        new_line += f" #{entry['comment']}"
                    new_line += '\n'
                    lines[i] = new_line
                    modified_count += 1
                    print(f"  Added q flag to: {entry['word']} (flags: {flags} -> {new_flags})")
        
        # Stop if we've modified enough
        if modified_count >= max_adjectives:
            break
    
    # Write the modified dictionary
    # Note: No backup needed - dictionary is under Git version control
    with open(dict_path, 'w', encoding='utf-8') as f:
        f.writelines(lines)
    
    print(f"✅ Added q flag to {modified_count} top adjectives")
    return modified_count

def analyze_current_state(dict_path):
    """Analyze current state of adjectives with q flag."""
    with open(dict_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    total_adjectives = 0
    with_q_flag = 0
    
    for line in lines:
        entry = parse_dictionary_entry(line)
        if entry is None:
            continue
        
        flags = entry['flags']
        
        if 'J' in flags or 'A' in flags:
            total_adjectives += 1
            if 'q' in flags:
                with_q_flag += 1
    
    print(f"📊 Current adjective statistics:")
    print(f"  Total adjectives: {total_adjectives}")
    print(f"  With q flag: {with_q_flag}")
    print(f"  Percentage: {(with_q_flag / total_adjectives * 100):.1f}%" if total_adjectives > 0 else "  Percentage: 0%")

def main():
    parser = argparse.ArgumentParser(description='Add compound q flag to top German adjectives')
    parser.add_argument('--dict', type=str, 
                        default='/home/konrad/gallery/harper/harper-core/src/language/german/dictionary.dict',
                        help='Path to the German dictionary file')
    parser.add_argument('--analyze', action='store_true', 
                        help='Analyze current adjective statistics')
    parser.add_argument('--count', type=int, default=200,
                        help='Maximum number of adjectives to modify')
    
    args = parser.parse_args()
    
    dict_path = args.dict
    
    if not os.path.exists(dict_path):
        print(f"❌ Dictionary file not found: {dict_path}")
        return
    
    if args.analyze:
        analyze_current_state(dict_path)
        return
    
    print(f"🔧 Adding q flag to top {args.count} adjectives...")
    count = add_q_to_top_adjectives(dict_path, args.count)
    print(f"✅ Processed {count} adjectives")

if __name__ == '__main__':
    main()
#!/usr/bin/env python3
"""
Validate flags against a language's annotations.json file.
"""

import json
import sys
import os


def get_valid_flags(language):
    """Get all valid flags from annotations.json for a language."""
    annotations_path = f"harper-core/src/language/{language}/annotations.json"
    
    if not os.path.exists(annotations_path):
        print(f"Error: Annotations file not found: {annotations_path}")
        return None
    
    try:
        with open(annotations_path, 'r', encoding='utf-8') as f:
            annotations = json.load(f)
    except Exception as e:
        print(f"Error reading annotations: {e}")
        return None
    
    valid_flags = set()
    
    # Get affix flags
    if 'affixes' in annotations:
        for flag in annotations['affixes'].keys():
            valid_flags.add(flag)
    
    # Get property flags
    if 'properties' in annotations:
        for flag in annotations['properties'].keys():
            valid_flags.add(flag)
    
    return valid_flags


def validate_flags(language, flags):
    """Validate that all flags are defined in the annotations."""
    valid_flags = get_valid_flags(language)
    
    if valid_flags is None:
        return False, []
    
    invalid_flags = []
    for flag in flags:
        if flag not in valid_flags:
            invalid_flags.append(flag)
    
    return len(invalid_flags) == 0, invalid_flags


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 validate_flags.py <language> <flags>")
        print("Example: python3 validate_flags.py german JOQRSTUW")
        sys.exit(1)
    
    language = sys.argv[1]
    flags = sys.argv[2]
    
    is_valid, invalid_flags = validate_flags(language, flags)
    
    if is_valid:
        print(f"✅ All flags '{flags}' are valid for {language}")
        sys.exit(0)
    else:
        print(f"❌ Invalid flags found: {', '.join(invalid_flags)}")
        print(f"   Valid flags are: {', '.join(sorted(get_valid_flags(language) or []))}")
        sys.exit(1)

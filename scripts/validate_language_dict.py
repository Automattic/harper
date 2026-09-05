#!/usr/bin/env python3
"""
Language Dictionary Validation Script

Validates dictionary.dict and annotations.json files for Harper language modules.
Checks for format errors, flag consistency, duplicates, and other issues.
"""

import json
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

# Regex patterns for validation
# Support both simple format: word/FLAG and Harper internal format: word/~~FLAGS # comment
WORD_FLAG_PATTERN = re.compile(r'^[a-zA-ZäöüßąćęłńóśźżáéíóúýàèìòùâêîôûãñõÄÖÜßÅÆÇØåæçø]+(/~~[A-Za-z0-9_]+)+(\s*#.*)?$')
SIMPLE_WORD_FLAG_PATTERN = re.compile(r'^([^/#]+)/([^/#]+)(\s*#.*)?$')
HARPER_INTERNAL_PATTERN = re.compile(r'^([^/#]+)/(~~[A-Za-z0-9_]+)(\s*#.*)?$')
COMMENT_PATTERN = re.compile(r'^\s*#')
EMPTY_LINE_PATTERN = re.compile(r'^\s*$')
# Pattern to extract flags from both formats: word/FLAG or word/~~FLAGS
FLAG_EXTRACTION_PATTERN = re.compile(r'^/[^/]+/(.+)$')


def get_language_paths(language):
    """Get paths for dictionary and annotations files for a given language."""
    base_path = Path(f"harper-core/src/language/{language}")
    dict_path = base_path / "dictionary.dict"
    annotations_path = base_path / "annotations.json"
    return dict_path, annotations_path


def validate_file_existence(language):
    """Check if required files exist."""
    dict_path, annotations_path = get_language_paths(language)
    
    errors = []
    
    if not dict_path.exists():
        errors.append(f"❌ Dictionary file not found: {dict_path}")
    elif not dict_path.is_file():
        errors.append(f"❌ Dictionary path is not a file: {dict_path}")
    elif not os.access(dict_path, os.R_OK):
        errors.append(f"❌ Dictionary file is not readable: {dict_path}")
    else:
        print(f"✅ Dictionary file found: {dict_path}")
    
    if not annotations_path.exists():
        errors.append(f"❌ Annotations file not found: {annotations_path}")
    elif not annotations_path.is_file():
        errors.append(f"❌ Annotations path is not a file: {annotations_path}")
    elif not os.access(annotations_path, os.R_OK):
        errors.append(f"❌ Annotations file is not readable: {annotations_path}")
    else:
        print(f"✅ Annotations file found: {annotations_path}")
    
    return errors


def validate_annotations_format(annotations_path):
    """Validate the format of annotations.json file."""
    errors = []
    
    try:
        with open(annotations_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Check if file is empty
        if not content.strip():
            errors.append(f"❌ Annotations file is empty: {annotations_path}")
            return errors
        
        # Try to parse as JSON
        try:
            annotations = json.loads(content)
        except json.JSONDecodeError as e:
            errors.append(f"❌ Invalid JSON in annotations file: {e}")
            return errors
        
        # Check structure
        if not isinstance(annotations, dict):
            errors.append(f"❌ Annotations should be a JSON object, got {type(annotations).__name__}")
            return errors
        
        # Check for required sections
        if 'properties' not in annotations:
            errors.append("❌ Annotations missing 'properties' section")
        elif not isinstance(annotations['properties'], dict):
            errors.append("'properties' should be a JSON object")
        
        if 'affixes' in annotations:
            if not isinstance(annotations['affixes'], dict):
                errors.append("'affixes' should be a JSON object")
            else:
                if 'prefixes' in annotations['affixes'] and not isinstance(annotations['affixes']['prefixes'], list):
                    errors.append("'affixes.prefixes' should be a JSON array")
                if 'suffixes' in annotations['affixes'] and not isinstance(annotations['affixes']['suffixes'], list):
                    errors.append("'affixes.suffixes' should be a JSON array")
        
        if not errors:
            print(f"✅ Annotations format is valid")
        
        return errors
        
    except Exception as e:
        errors.append(f"❌ Error reading annotations file: {e}")
        return errors


def validate_dictionary_format(dict_path):
    """Validate the format of dictionary.dict file."""
    errors = []
    warnings = []
    
    try:
        with open(dict_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        if not lines:
            errors.append(f"❌ Dictionary file is empty: {dict_path}")
            return errors, warnings
        
        line_count = len(lines)
        valid_lines = 0
        comment_lines = 0
        empty_lines = 0
        invalid_lines = 0
        
        for line_num, line in enumerate(lines, 1):
            line = line.rstrip('\n\r')
            stripped_line = line.strip()
            
            # Skip empty lines
            if EMPTY_LINE_PATTERN.match(stripped_line):
                empty_lines += 1
                continue
            
            # Skip comments
            if COMMENT_PATTERN.match(stripped_line):
                comment_lines += 1
                continue
            
            # Check for word/flags pattern (both simple and Harper internal formats)
            # Remove trailing comments for validation
            line_without_comment = stripped_line.split('#')[0].strip() if '#' in stripped_line else stripped_line
            
            # Should be a word/flags line or metadata line (like line count)
            if SIMPLE_WORD_FLAG_PATTERN.match(stripped_line):
                valid_lines += 1
            elif HARPER_INTERNAL_PATTERN.match(stripped_line):
                valid_lines += 1
            elif WORD_FLAG_PATTERN.match(line_without_comment):
                valid_lines += 1
            elif stripped_line.isdigit():  # Allow numeric metadata lines
                valid_lines += 1
            else:
                errors.append(f"❌ Invalid format at line {line_num}: '{stripped_line}'")
                invalid_lines += 1
        
        print(f"✅ Dictionary format check: {valid_lines} valid entries, {comment_lines} comments, {empty_lines} empty lines")
        
        if invalid_lines > 0:
            errors.append(f"❌ Found {invalid_lines} invalid lines in dictionary")
        
        return errors, warnings
        
    except Exception as e:
        errors.append(f"❌ Error reading dictionary file: {e}")
        return errors, warnings


def validate_no_duplicates(dict_path):
    """Check for duplicate word+flags entries in dictionary."""
    errors = []
    warnings = []
    
    try:
        with open(dict_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        entries_seen = defaultdict(list)  # word/flags -> list of line numbers
        
        for line_num, line in enumerate(lines, 1):
            line = line.rstrip('\n\r').strip()
            
            # Skip comments and empty lines
            if COMMENT_PATTERN.match(line) or EMPTY_LINE_PATTERN.match(line):
                continue
            
            # For duplicates, we consider the full entry (word + flags) excluding trailing comments
            if '/' in line:
                entry_part = line.split('#')[0].strip() if '#' in line else line
                if entry_part and not entry_part.split('/')[0].strip().isdigit():
                    entries_seen[entry_part].append(line_num)
        
        duplicates = {entry: lines for entry, lines in entries_seen.items() if len(lines) > 1}
        
        if duplicates:
            for entry, line_nums in duplicates.items():
                errors.append(f"❌ Duplicate entry '{entry}' found at lines: {', '.join(map(str, line_nums))}")
        else:
            print(f"✅ No duplicate entries found")
        
        return errors, warnings
        
    except Exception as e:
        errors.append(f"❌ Error checking for duplicates: {e}")
        return errors, warnings


def validate_flag_consistency(dict_path, annotations_path):
    """Check that all flags used in dictionary exist in annotations.properties."""
    errors = []
    warnings = []
    
    # Load annotations
    try:
        with open(annotations_path, 'r', encoding='utf-8') as f:
            annotations = json.load(f)
    except Exception as e:
        errors.append(f"❌ Could not load annotations for flag validation: {e}")
        return errors, warnings
    
    properties = annotations.get('properties', {})
    valid_flags = set(properties.keys())
    
    if not valid_flags:
        warnings.append("⚠️  No properties found in annotations - cannot validate flags")
        return errors, warnings
    
    # Extract all flags from dictionary
    dict_flags = set()
    flag_usage = defaultdict(list)  # flag -> list of words using it
    
    try:
        with open(dict_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for line_num, line in enumerate(lines, 1):
            line = line.rstrip('\n\r').strip()
            
            # Skip comments and empty lines
            if COMMENT_PATTERN.match(line) or EMPTY_LINE_PATTERN.match(line):
                continue
            
            # Extract flags from word/flags format
            if '/' in line:
                parts = line.split('/')
                word = parts[0].strip()
                # Handle comments on same line
                word = word.split('#')[0].strip() if '#' in word else word
                if word and not word.isdigit():  # Skip numeric metadata lines
                    flags_part = '/'.join(parts[1:])  # Handle multiple / separators
                    # Remove any trailing comments from flags part
                    flags_part = flags_part.split('#')[0].strip() if '#' in flags_part else flags_part
                    flags = flags_part.split('/')
                    
                    for flag in flags:
                        flag = flag.strip()
                        if flag:
                            # Remove ~~ prefix for Harper internal flags
                            clean_flag = flag[2:] if flag.startswith('~~') else flag
                            dict_flags.add(clean_flag)
                            flag_usage[clean_flag].append(f"{word} (line {line_num})")
        
        # Check for flags not in annotations
        unknown_flags = dict_flags - valid_flags
        if unknown_flags:
            # Separate simple flags (single character) from compound flags
            simple_flags = {f for f in unknown_flags if len(f) == 1}
            compound_flags = {f for f in unknown_flags if len(f) > 1}
            
            if simple_flags:
                for flag in sorted(simple_flags):
                    usage_examples = flag_usage[flag][:3]  # Show first 3 examples
                    examples_str = ", ".join(usage_examples)
                    errors.append(f"❌ Flag '{flag}' used but not defined in annotations.properties. Examples: {examples_str}")
            
            if compound_flags:
                warnings.append(f"⚠️  Compound flags found (may be valid combinations): {', '.join(sorted(compound_flags)[:10])}" + 
                               ("..." if len(compound_flags) > 10 else ""))
            
            if not simple_flags and compound_flags:
                print(f"✅ All simple flags are defined in annotations (compound flags found)")
        else:
            print(f"✅ All flags are defined in annotations")
        
        # Check for unused flags in annotations
        unused_flags = valid_flags - dict_flags
        if unused_flags:
            warnings.append(f"⚠️  Flags in annotations but not used in dictionary: {', '.join(sorted(unused_flags)[:10])}" +
                           ("..." if len(unused_flags) > 10 else ""))
        
        return errors, warnings
        
    except Exception as e:
        errors.append(f"❌ Error validating flag consistency: {e}")
        return errors, warnings


def validate_unicode_chars(dict_path, language):
    """Basic Unicode character validation for the language."""
    errors = []
    warnings = []
    
    # Define expected character ranges for different languages
    language_chars = {
        'german': {'ä', 'ö', 'ü', 'ß', 'Ä', 'Ö', 'Ü'},
        'portuguese': {'á', 'é', 'í', 'ó', 'ú', 'ã', 'õ', 'ç', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ã', 'Õ', 'Ç'},
        'slovak': {'ä', 'č', 'ď', 'é', 'í', 'ľ', 'ň', 'ó', 'ô', 'ŕ', 'š', 'ť', 'ú', 'ý', 'ž', 
                  'Ä', 'Č', 'Ď', 'É', 'Í', 'Ľ', 'Ň', 'Ó', 'Ô', 'Ŕ', 'Š', 'Ť', 'Ú', 'Ý', 'Ž'},
        'polish': {'ą', 'ć', 'ę', 'ł', 'ń', 'ó', 'ś', 'ź', 'ż', 'Ą', 'Ć', 'Ę', 'Ł', 'Ń', 'Ó', 'Ś', 'Ź', 'Ż'},
    }
    
    expected_chars = language_chars.get(language.lower(), set())
    
    try:
        with open(dict_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Check for invalid Unicode sequences
        try:
            content.encode('utf-8').decode('utf-8')
        except UnicodeError as e:
            errors.append(f"❌ Invalid UTF-8 encoding in dictionary: {e}")
            return errors, warnings
        
        # Check for expected characters
        if expected_chars:
            found_expected = any(char in content for char in expected_chars)
            if not found_expected:
                warnings.append(f"⚠️  No expected language-specific characters found for {language}")
        
        print(f"✅ Unicode validation passed")
        return errors, warnings
        
    except Exception as e:
        errors.append(f"❌ Error validating Unicode: {e}")
        return errors, warnings


def validate_language_dict(language):
    """Main validation function for a language dictionary."""
    print(f"\n🔍 Validating dictionary for language: {language}")
    print("=" * 60)
    
    dict_path, annotations_path = get_language_paths(language)
    
    all_errors = []
    all_warnings = []
    
    # Step 1: Check file existence
    errors = validate_file_existence(language)
    all_errors.extend(errors)
    if errors:
        print("\n❌ Validation failed - file existence issues:")
        for error in errors:
            print(f"   {error}")
        return False, all_errors, all_warnings
    
    # Step 2: Validate annotations format
    errors = validate_annotations_format(annotations_path)
    all_errors.extend(errors)
    if errors:
        print("\n❌ Validation failed - annotations format issues:")
        for error in errors:
            print(f"   {error}")
        return False, all_errors, all_warnings
    
    # Step 3: Validate dictionary format
    errors, warnings = validate_dictionary_format(dict_path)
    all_errors.extend(errors)
    all_warnings.extend(warnings)
    
    # Step 4: Check for duplicates
    errors, warnings = validate_no_duplicates(dict_path)
    all_errors.extend(errors)
    all_warnings.extend(warnings)
    
    # Step 5: Validate flag consistency
    errors, warnings = validate_flag_consistency(dict_path, annotations_path)
    all_errors.extend(errors)
    all_warnings.extend(warnings)
    
    # Step 6: Unicode validation
    errors, warnings = validate_unicode_chars(dict_path, language)
    all_errors.extend(errors)
    all_warnings.extend(warnings)
    
    # Summary
    print("\n" + "=" * 60)
    if all_errors:
        print(f"❌ Validation FAILED for {language}: {len(all_errors)} errors, {len(all_warnings)} warnings")
        for error in all_errors:
            print(f"   {error}")
        for warning in all_warnings:
            print(f"   {warning}")
        return False, all_errors, all_warnings
    else:
        print(f"✅ Validation PASSED for {language}")
        if all_warnings:
            print(f"   {len(all_warnings)} warnings:")
            for warning in all_warnings:
                print(f"      {warning}")
        return True, all_errors, all_warnings


def validate_all_languages():
    """Validate all languages in the repository."""
    print("🌍 Validating all language dictionaries")
    print("=" * 60)
    
    language_dir = Path("harper-core/src/language")
    all_results = {}
    
    if not language_dir.exists():
        print("❌ Language directory not found")
        return False
    
    for lang_dir in sorted(language_dir.iterdir()):
        if lang_dir.is_dir() and not lang_dir.name.startswith('_'):
            config_file = lang_dir / "config.toml"
            if config_file.exists():
                language = lang_dir.name
                print(f"\n📚 Checking {language}...")
                success, errors, warnings = validate_language_dict(language)
                all_results[language] = {'success': success, 'errors': errors, 'warnings': warnings}
    
    # Summary
    print("\n" + "=" * 60)
    print("📊 VALIDATION SUMMARY")
    print("=" * 60)
    
    passed = sum(1 for result in all_results.values() if result['success'])
    failed = len(all_results) - passed
    
    print(f"Total languages checked: {len(all_results)}")
    print(f"✅ Passed: {passed}")
    print(f"❌ Failed: {failed}")
    
    if failed > 0:
        print("\n💥 FAILED LANGUAGES:")
        for lang, result in all_results.items():
            if not result['success']:
                print(f"\n  {lang}:")
                for error in result['errors']:
                    print(f"    {error}")
    
    return failed == 0


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        # Validate specific language
        language = sys.argv[1].lower()
        success, errors, warnings = validate_language_dict(language)
        sys.exit(0 if success else 1)
    else:
        # Validate all languages
        success = validate_all_languages()
        sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
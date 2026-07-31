#!/usr/bin/env python3
"""
Check language feature consistency across Cargo.toml files.

This script verifies that language features are consistently defined
across all Cargo.toml files in the Harper project.
"""

import os
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib


def find_cargo_toml_files(root_dir):
    """Find all Cargo.toml files in the project."""
    cargo_files = []
    for root, dirs, files in os.walk(root_dir):
        # Skip target directories
        if 'target' in root:
            continue
        for file in files:
            if file == 'Cargo.toml':
                cargo_files.append(Path(root) / file)
    return cargo_files


def extract_features(cargo_path):
    """Extract feature definitions from a Cargo.toml file."""
    features = {}
    try:
        with open(cargo_path, 'rb') as f:
            content = f.read()
        data = tomllib.loads(content.decode('utf-8'))
        if 'features' in data:
            # Handle both dict and list formats
            features_data = data['features']
            if isinstance(features_data, dict):
                for feature_name, deps in features_data.items():
                    # deps can be a list or None
                    if deps is None:
                        features[feature_name] = []
                    elif isinstance(deps, list):
                        features[feature_name] = deps
                    else:
                        features[feature_name] = []
    except Exception as e:
        print(f"Warning: Could not parse {cargo_path}: {e}")
    
    return features


def check_language_features(root_dir):
    """Check that language features are consistently defined."""
    cargo_files = find_cargo_toml_files(root_dir)
    
    if not cargo_files:
        print("No Cargo.toml files found!")
        return False
    
    print(f"Found {len(cargo_files)} Cargo.toml files")
    print()
    
    # Collect all features from all files
    all_features = {}
    for cargo_path in cargo_files:
        features = extract_features(cargo_path)
        if features:
            rel_path = str(cargo_path.relative_to(root_dir))
            all_features[rel_path] = features
            print(f"{rel_path}:")
            for feat, deps in features.items():
                print(f"  {feat} = {deps}")
            print()
    
    # Check for language features
    language_features = ['de', 'pt', 'sk', 'pl', 'multilingual', 'all-languages']
    
    errors = []
    
    # Check harper-core features
    core_features = all_features.get('harper-core/Cargo.toml', {})
    for lang in ['de', 'pt', 'sk', 'pl']:
        if lang not in core_features:
            errors.append(f"Language feature '{lang}' not found in harper-core/Cargo.toml")
    
    if 'multilingual' not in core_features:
        errors.append("Feature 'multilingual' not found in harper-core/Cargo.toml")
    
    if 'all-languages' not in core_features:
        errors.append("Feature 'all-languages' not found in harper-core/Cargo.toml")
    
    # Check that multilingual includes all language features
    if 'multilingual' in core_features:
        multilingual_deps = core_features['multilingual']
        for lang in ['de', 'pt', 'sk', 'pl']:
            if lang not in multilingual_deps:
                errors.append(f"Language '{lang}' not included in multilingual feature")
    
    # Check that all-languages includes multilingual
    if 'all-languages' in core_features:
        all_langs_deps = core_features['all-languages']
        if 'multilingual' not in all_langs_deps:
            errors.append("'multilingual' not included in all-languages feature")
    
    if errors:
        print("ERRORS:")
        for error in errors:
            print(f"  - {error}")
        return False
    else:
        print("✓ All language features are consistent!")
        return True


def main():
    root_dir = Path(__file__).parent.parent
    
    if not check_language_features(root_dir):
        sys.exit(1)


if __name__ == '__main__':
    main()

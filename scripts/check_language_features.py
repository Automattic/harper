#!/usr/bin/env python3
"""
Validate that language features are consistently defined across all relevant Cargo.toml files.

This script ensures that:
1. All languages with a feature flag in harper-core/src/language/*/config.toml
   have corresponding features in harper-core/Cargo.toml
2. All language features in harper-core/Cargo.toml have corresponding features in harper-wasm/Cargo.toml
3. The all-languages feature includes all individual language features
"""

import re
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent


def load_toml(path):
    """Load a TOML file and return its contents as a dictionary."""
    with open(path, "rb") as f:
        return tomllib.load(f)


def get_language_features_from_configs():
    """Extract all language feature names from config.toml files."""
    language_dir = REPO_ROOT / "harper-core" / "src" / "language"
    features = set()
    
    if not language_dir.exists():
        print(f"Warning: Language directory not found: {language_dir}")
        return features
    
    for lang_dir in sorted(language_dir.iterdir()):
        if not lang_dir.is_dir():
            continue
        
        config_file = lang_dir / "config.toml"
        if not config_file.exists():
            continue
        
        try:
            config = load_toml(config_file)
            language_section = config.get("language", {})
            feature = language_section.get("feature", "")
            if feature and feature not in ("null", ""):
                features.add(feature)
        except Exception as e:
            print(f"Warning: Could not parse {config_file}: {e}")
    
    return features


def get_features_from_cargo_toml(path):
    """Extract feature names from a Cargo.toml file."""
    cargo_toml = load_toml(path)
    features = cargo_toml.get("features", {})
    if not isinstance(features, dict):
        return set()
    return set(features.keys()) - {"default", "concurrent", "thesaurus", "typst"}


def get_all_languages_deps(path):
    """Extract the list of features in all-languages from a Cargo.toml."""
    cargo_toml = load_toml(path)
    features = cargo_toml.get("features", {})
    if not isinstance(features, dict):
        return set()
    all_lang = features.get("all-languages", [])
    if isinstance(all_lang, list):
        return set(all_lang)
    return set()


def check_harper_core_features():
    """Check that all language configs have corresponding features in harper-core/Cargo.toml."""
    core_cargo = REPO_ROOT / "harper-core" / "Cargo.toml"
    if not core_cargo.exists():
        print(f"Error: harper-core/Cargo.toml not found")
        return False
    
    config_features = get_language_features_from_configs()
    cargo_features = get_features_from_cargo_toml(core_cargo)
    
    missing = config_features - cargo_features
    if missing:
        print(f"Error: Language features defined in config.toml but missing from harper-core/Cargo.toml:")
        for feature in sorted(missing):
            print(f"  - {feature}")
        return False
    
    return True


def check_harper_wasm_features():
    """Check that all harper-core language features are in harper-wasm/Cargo.toml."""
    wasm_cargo = REPO_ROOT / "harper-wasm" / "Cargo.toml"
    if not wasm_cargo.exists():
        print(f"Warning: harper-wasm/Cargo.toml not found (skipping Wasm feature check)")
        return True
    
    core_cargo = REPO_ROOT / "harper-core" / "Cargo.toml"
    if not core_cargo.exists():
        print(f"Error: harper-core/Cargo.toml not found")
        return False
    
    core_features = get_features_from_cargo_toml(core_cargo)
    wasm_features = get_features_from_cargo_toml(wasm_cargo)
    
    # Get language features (those that are in core and are language codes)
    # Language features are typically 2-3 letter codes like de, pt, sk
    language_pattern = re.compile(r"^[a-z]{2,3}$")
    core_lang_features = {f for f in core_features if language_pattern.match(f)}
    wasm_lang_features = {f for f in wasm_features if language_pattern.match(f)}
    
    missing = core_lang_features - wasm_lang_features
    if missing:
        print(f"Error: Language features in harper-core but missing from harper-wasm/Cargo.toml:")
        for feature in sorted(missing):
            print(f"  - {feature}")
        return False
    
    return True


def check_all_languages_feature(cargo_path, expected_features):
    """Check that all-languages feature includes all individual language features."""
    all_lang = get_all_languages_deps(cargo_path)
    
    if not all_lang:
        print(f"Warning: No all-languages feature in {cargo_path}")
        return True
    
    missing = expected_features - all_lang
    if missing:
        print(f"Error: all-languages feature in {cargo_path} is missing:")
        for feature in sorted(missing):
            print(f"  - {feature}")
        return False
    
    return True


def main():
    """Run all validation checks."""
    print("Checking language feature consistency...\n")
    
    all_ok = True
    
    # Check harper-core features
    if not check_harper_core_features():
        all_ok = False
    
    # Check harper-wasm features
    if not check_harper_wasm_features():
        all_ok = False
    
    # Check all-languages features
    core_cargo = REPO_ROOT / "harper-core" / "Cargo.toml"
    wasm_cargo = REPO_ROOT / "harper-wasm" / "Cargo.toml"
    
    language_pattern = re.compile(r"^[a-z]{2,3}$")
    
    if core_cargo.exists():
        core_features = get_features_from_cargo_toml(core_cargo)
        core_lang_features = {f for f in core_features if language_pattern.match(f)}
        if not check_all_languages_feature(core_cargo, core_lang_features):
            all_ok = False
    
    if wasm_cargo.exists():
        wasm_features = get_features_from_cargo_toml(wasm_cargo)
        wasm_lang_features = {f for f in wasm_features if language_pattern.match(f)}
        if not check_all_languages_feature(wasm_cargo, wasm_lang_features):
            all_ok = False
    
    print()
    if all_ok:
        print("All language feature checks passed!")
        return 0
    else:
        print("Language feature checks failed!")
        return 1


if __name__ == "__main__":
    sys.exit(main())

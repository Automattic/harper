#!/usr/bin/env python3
"""
All-Languages Test Runner

Runs integration tests for all enabled Harper language modules.
"""

import json
import subprocess
import sys
from pathlib import Path


def get_all_languages():
    """Get list of all languages from harper-core Cargo.toml."""
    cargo_toml_path = Path("harper-core/Cargo.toml")
    
    if not cargo_toml_path.exists():
        print("❌ harper-core/Cargo.toml not found")
        return []
    
    with open(cargo_toml_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Find all language features (2-3 letter codes)
    languages = []
    lines = content.split('\n')
    
    for line in lines:
        line = line.strip()
        # Look for feature definitions like "de = []", "pt = []", etc.
        if line and line.endswith('= []') and len(line.split('=')[0].strip()) in [2, 3]:
            lang = line.split('=')[0].strip()
            # Skip non-language features
            if lang not in ['de', 'pt', 'sk', 'pl']:
                continue
            languages.append(lang)
    
    return sorted(languages)


def get_languages_with_integration_tests():
    """Get list of languages that have integration test files."""
    test_dir = Path("harper-core/tests")
    languages = []
    
    if test_dir.exists():
        for test_file in test_dir.glob("*_integration_test.rs"):
            # Extract language from filename like "polish_integration_test.rs"
            filename = test_file.name
            if filename.endswith("_integration_test.rs"):
                lang = filename[:-20]  # Remove "_integration_test.rs"
                if lang not in ['languages', 'language']:  # Skip generic files
                    languages.append(lang)
    
    return sorted(languages)


def run_cargo_test(lang, features=None):
    """Run cargo test for a specific language feature."""
    if features is None:
        features = []
    
    cmd = ["cargo", "test", "--lib", "--", "--nocapture"]
    
    # Add language feature
    features_with_lang = features + [lang]
    if features_with_lang:
        cmd.extend(["--features", ",".join(features_with_lang)])
    
    print(f"🧪 Running tests for {lang}...")
    
    try:
        result = subprocess.run(
            cmd,
            cwd="harper-core",
            capture_output=True,
            text=True,
            timeout=120
        )
        
        if result.returncode == 0:
            print(f"   ✅ {lang} tests passed")
            return True
        else:
            print(f"   ❌ {lang} tests failed")
            if result.stdout:
                print(f"   stdout: {result.stdout[:200]}...")
            if result.stderr:
                print(f"   stderr: {result.stderr[:200]}...")
            return False
            
    except subprocess.TimeoutExpired:
        print(f"   ⏰ {lang} tests timed out")
        return False
    except Exception as e:
        print(f"   ❌ {lang} tests error: {e}")
        return False


def run_all_language_tests(specific_languages=None):
    """Run integration tests for all languages."""
    print("🌍 Running integration tests for all languages")
    print("=" * 60)
    
    # Get languages to test
    if specific_languages:
        languages = specific_languages
        print(f"📚 Testing specific languages: {', '.join(languages)}")
    else:
        # Auto-discover languages with integration tests
        languages = get_languages_with_integration_tests()
        if not languages:
            languages = get_all_languages()
        print(f"📚 Discovered languages: {', '.join(languages) if languages else 'None'}")
    
    if not languages:
        print("❌ No languages found to test")
        return False
    
    results = {}
    for lang in languages:
        results[lang] = run_cargo_test(lang)
    
    # Summary
    print("\n" + "=" * 60)
    print("📊 TEST SUMMARY")
    print("=" * 60)
    
    passed = sum(1 for result in results.values() if result)
    failed = len(results) - passed
    
    print(f"Total languages tested: {len(results)}")
    print(f"✅ Passed: {passed}")
    print(f"❌ Failed: {failed}")
    
    if failed > 0:
        print("\n💥 FAILED LANGUAGES:")
        for lang, success in results.items():
            if not success:
                print(f"   - {lang}")
    
    return failed == 0


def main():
    """Main entry point."""
    # Parse arguments
    specific_languages = []
    
    for arg in sys.argv[1:]:
        if arg.startswith('--'):
            continue
        specific_languages.append(arg)
    
    if specific_languages:
        # Test specific languages
        success = run_all_language_tests(specific_languages)
    else:
        # Test all languages
        success = run_all_language_tests()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
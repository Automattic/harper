#!/usr/bin/env python3
"""
All-Languages Test Runner

Runs the test suite for each Harper language module.

Languages and their Cargo features are discovered from
`harper-core/src/language/<lang>/config.toml`, the same source the build script
uses, so a newly added language is picked up automatically.
"""

import re
import subprocess
import sys
from pathlib import Path

LANGUAGE_DIR = Path("harper-core/src/language")
TEST_DIR = Path("harper-core/tests")


def discover_languages():
    """Map language directory name -> Cargo feature, from each config.toml.

    English has no `feature` key (it is always compiled in) and is skipped.
    """
    languages = {}

    if not LANGUAGE_DIR.is_dir():
        print(f"❌ {LANGUAGE_DIR} not found (run from the repository root)")
        return languages

    for config in sorted(LANGUAGE_DIR.glob("*/config.toml")):
        text = config.read_text(encoding="utf-8")
        match = re.search(r'^\s*feature\s*=\s*["\']([^"\']+)["\']', text, re.MULTILINE)
        if match:
            languages[config.parent.name] = match.group(1)

    return languages


def integration_tests_for(language):
    """Integration test targets in harper-core/tests/ that belong to a language.

    Matches both `<lang>_integration_test.rs` and the `<lang>_*_test.rs` files
    the German module uses.
    """
    if not TEST_DIR.is_dir():
        return []

    return sorted(
        path.stem
        for path in TEST_DIR.glob(f"{language}_*.rs")
    )


def run(cmd):
    """Run a command, returning (ok, combined output)."""
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
    return result.returncode == 0, (result.stdout or "") + (result.stderr or "")


def run_language_tests(language, feature):
    """Run the unit and integration tests for one language."""
    print(f"🧪 Running tests for {language} (feature '{feature}')...")

    # Unit tests inside the language module.
    cmd = ["cargo", "test", "-q", "-p", "harper-core", "--features", feature, "--lib", language]

    # Plus any integration tests named after the language.
    for target in integration_tests_for(language):
        cmd += ["--test", target]

    ok, output = run(cmd)

    if ok:
        print(f"   ✅ {language} tests passed")
        return True

    print(f"   ❌ {language} tests failed")
    for line in output.splitlines():
        if line.startswith("error") or "panicked" in line or "FAILED" in line:
            print(f"      {line}")
    return False


def main():
    requested = [a for a in sys.argv[1:] if not a.startswith("--")]

    # `just language-all-test de,pt` passes one comma-separated argument.
    requested = [part for arg in requested for part in arg.split(",") if part]

    print("🌍 Running tests for all language modules")
    print("=" * 60)

    languages = discover_languages()
    if not languages:
        print("❌ No languages discovered from config.toml files")
        sys.exit(1)

    # Accept either a directory name ("german") or a feature ("de").
    by_feature = {feature: name for name, feature in languages.items()}
    if requested:
        selected = {}
        for item in requested:
            if item in languages:
                selected[item] = languages[item]
            elif item in by_feature:
                selected[by_feature[item]] = item
            else:
                known = ", ".join(sorted(languages) + sorted(by_feature))
                print(f"❌ Unknown language '{item}'. Known: {known}")
                sys.exit(1)
        languages = selected

    print(f"📚 Testing: {', '.join(f'{n} ({f})' for n, f in sorted(languages.items()))}")
    print()

    results = {name: run_language_tests(name, feature) for name, feature in sorted(languages.items())}

    print()
    print("=" * 60)
    print("📊 TEST SUMMARY")
    print("=" * 60)

    failed = [name for name, ok in results.items() if not ok]
    print(f"Total languages tested: {len(results)}")
    print(f"✅ Passed: {len(results) - len(failed)}")
    print(f"❌ Failed: {len(failed)}")

    if failed:
        print("\n💥 FAILED LANGUAGES:")
        for name in failed:
            print(f"   - {name}")

    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()

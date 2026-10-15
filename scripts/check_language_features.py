#!/usr/bin/env python3
"""Check that every language in harper-core is wired into every Cargo manifest.

The list of languages is *discovered*, not hard-coded: any directory under
`harper-core/src/language/` that holds a `config.toml` with a `feature` key is a
language, and every manifest below has to carry that feature. Adding a language
therefore turns into a failing check that names the files still to edit, rather
than into a silently half-wired build.

English has no feature of its own — it is always compiled in — so it is skipped.
"""

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / "harper-core/Cargo.toml"
LANGUAGE_DIR = ROOT / "harper-core/src/language"

# Crates that expose Harper to a user and therefore choose a language set.
CONSUMERS = [
    "harper-ls/Cargo.toml",
    "harper-cli/Cargo.toml",
    "harper-wasm/Cargo.toml",
    "harper-desktop/src-tauri/Cargo.toml",
]


def discover_features():
    """Cargo feature name -> language directory name, for every language."""
    found = {}
    for config in sorted(LANGUAGE_DIR.glob("*/config.toml")):
        data = tomllib.loads(config.read_text(encoding="utf-8"))
        feature = data.get("language", {}).get("feature")
        if feature:
            found[feature] = config.parent.name
    return found


def features_of(manifest):
    path = ROOT / manifest
    if not path.exists():
        return None
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    return {
        name: deps if isinstance(deps, list) else []
        for name, deps in data.get("features", {}).items()
    }


def check():
    languages = discover_features()
    if not languages:
        return [f"No language config.toml found under {LANGUAGE_DIR}"]

    errors = []
    core = features_of("harper-core/Cargo.toml")

    if "language-module" not in core:
        errors.append("harper-core: missing the 'language-module' feature")

    for feature, directory in sorted(languages.items()):
        if feature not in core:
            errors.append(
                f"harper-core: {directory} declares feature '{feature}', "
                f"but Cargo.toml has no such feature"
            )
        elif "language-module" not in core[feature]:
            errors.append(
                f"harper-core: feature '{feature}' must include 'language-module'"
            )

    for umbrella in ("multilingual", "all-languages"):
        if umbrella not in core:
            errors.append(f"harper-core: missing the '{umbrella}' feature")

    if "multilingual" in core:
        for feature in sorted(languages):
            if feature not in core["multilingual"]:
                errors.append(
                    f"harper-core: '{feature}' is not part of the "
                    f"'multilingual' feature"
                )
    if "all-languages" in core and "multilingual" not in core["all-languages"]:
        errors.append("harper-core: 'all-languages' must include 'multilingual'")

    for consumer in CONSUMERS:
        features = features_of(consumer)
        if features is None:
            errors.append(f"{consumer}: not found")
            continue
        for feature in sorted(languages):
            if feature not in features:
                errors.append(f"{consumer}: missing feature '{feature}'")
            elif f"harper-core/{feature}" not in features[feature]:
                errors.append(
                    f"{consumer}: feature '{feature}' must forward to "
                    f"'harper-core/{feature}'"
                )
        if "multilingual" not in features:
            errors.append(f"{consumer}: missing feature 'multilingual'")

    return errors


def main():
    languages = discover_features()
    errors = check()

    if errors:
        print("Language feature wiring is incomplete:")
        for error in errors:
            print(f"  - {error}")
        sys.exit(1)

    names = ", ".join(f"{d} ({f})" for f, d in sorted(languages.items()))
    print(f"Language features consistent across {len(CONSUMERS) + 1} manifests.")
    print(f"Optional languages: {names}")


if __name__ == "__main__":
    main()

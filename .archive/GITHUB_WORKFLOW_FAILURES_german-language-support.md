# GitHub Workflow Failures: feature/german-language-support Branch

## Overview

This document records common causes of GitHub workflow failures on the `feature/german-language-support` branch and their solutions.

## Common Causes of Workflow Failures

### 1. Import Ordering Issues (rustfmt)

**Symptom**: `cargo fmt -- --check` fails with formatting diffs.

**Cause**: Import statements not in alphabetical order or not following Rust style guidelines.

**Fix**: Ensure all import statements in modified files follow rustfmt expectations. Use `cargo fmt` to automatically format code.

**Example Commit**: 54855010f - Fixed import ordering in harper-desktop and harper-ls.

### 2. Clippy Warnings Treated as Errors

**Symptom**: `cargo clippy -- -Dwarnings` fails with clippy errors.

**Common Clippy Errors**:
- `field_reassign_with_default`: Reassigning a field after using `Default::default()`
- `unused_variable`: Variables declared but not used
- Other clippy lints configured with `-Dwarnings`

**Fix**: 
- For `field_reassign_with_default`, use struct update syntax instead:
  ```rust
  // Instead of:
  let mut x = T::default();
  x.field = value;
  
  // Use:
  let x = T {
      field: value,
      ..Default::default()
  };
  ```
- Remove unused variables or prefix with `_`

**Example Commit**: 54855010f - Fixed `field_reassign_with_default` in German compound spell module.

### 3. Missing Feature Flags in Cargo.toml

**Symptom**: Compilation errors when building packages that depend on multilingual features.

**Cause**: Packages depending on `harper-core` with multilingual support were not specifying the `multilingual` feature flag.

**Fix**: Add `features = ["multilingual"]` to harper-core dependencies in:
- `harper-desktop/src-tauri/Cargo.toml`
- `harper-cli/Cargo.toml`
- `harper-core/src/language/testing_framework/Cargo.toml`

**Important**: Use `multilingual` instead of individual language features like `de`. The `multilingual` feature includes `de`, `pt`, `sk`, and `pl`.

**Example Commit**: 54855010f - Added multilingual feature flag to desktop and CLI.

### 4. Individual Language Features vs Multilingual

**Symptom**: Tests or code using German-specific functionality fail when compiled without the `de` feature.

**Cause**: Code assuming German (`de`) feature is available, but the dependency only specifies `multilingual`.

**Fix**: Use `multilingual` feature consistently. Do not use individual language features (`de`, `pt`, `sk`, `pl`) directly in dependencies.

**Example Commit**: 54855010f - Updated testing_framework from `features = ["de"]` to `features = ["multilingual"]`.

### 5. Slow Test Execution (Timeout)

**Symptom**: `cargo test` times out during `test-rust` job. Individual German tests take over 60 seconds each.

**Cause**: 
- Large German dictionary (significantly expanded in commits 648476f72 and ebead6df9)
- Dictionary loading and compound word generation is computationally expensive
- Workspace-level `cargo test` enables all features needed by any workspace member, including `multilingual` which includes German

**Impacted Commits**:
- 648476f72: "feat: Improve German language compound word generation" - Modified ~7700 lines in German dictionary
- ebead6df9: "Improve German language support with additional words and testing enhancements" - Added 40+ German words and coverage analysis module

**Temporary Workaround**: 
```bash
# Run tests with a longer timeout
RUST_TEST_THREADS=1 cargo test --lib 2>&1 | head
```

**Proposed Solutions** (not yet implemented):
1. **Feature-gate tests explicitly**: Add `#[cfg(feature = "de")]` to German test modules to ensure they only run when the feature is enabled
2. **Separate test recipes**: Modify justfile to run Rust tests per-package with default features instead of workspace-wide
3. **Test optimization**: Reduce German dictionary size for tests or use mock dictionaries
4. **Increase timeout**: Configure cargo test timeout (requires nightly Rust or custom test runner)

### 6. Merge Conflicts from master

**Symptom**: Merge conflicts when merging master into feature branch.

**Cause**: Parallel development on master and feature branch.

**Fix**: 
- Regularly merge master into feature branch
- Resolve conflicts promptly
- Keep feature branch up to date with master

**Example Commits**: 1734bd4f8, e06930d21, ce30ac34e

### 7. Feature Unification in Workspace Tests

**Symptom**: `cargo test --workspace` enables all features from all workspace members, including `multilingual` which triggers expensive German dictionary loading and causes test timeouts.

**Cause**: When running `cargo test --workspace --all-features`, Cargo unifies features across the entire workspace. If any workspace member requires `multilingual`, all tests run with that feature enabled, even for packages that don't need it.

**Fix**: Split workspace testing into sequential steps:
1. Test `harper-core` with default features first (no multilingual)
2. Then test remaining workspace members separately

**Example**: 
```bash
# In justfile test-rust recipe:
cargo test -q -p harper-core
cargo test -q --workspace --exclude harper-core
```

**Example Commit**: 75826c4b1 - Wrapped language-specific tests with feature flags and updated test-rust

### 8. pnpm-workspace.yaml Path Issues

**Symptom**: Build failures due to incorrect workspace path references.

**Cause**: The `pnpm-workspace.yaml` file contained incorrect paths to workspace packages, particularly `harper-wasm`.

**Fix**: Ensure all workspace paths in `pnpm-workspace.yaml` are correct and relative to the repository root.

**Example Commit**: 1175a043f - Fixed harper-wasm workspace path

### 10. Incorrect harper-wasm Import Paths in TypeScript

**Symptom**: Build Web workflow fails with TypeScript error: `Cannot find module 'harper-wasm/harper_wasm.js'`

**Cause**: Import statements in BinaryModule.ts and related files used paths like `'harper-wasm/harper_wasm.js'`, but the actual files are generated by wasm-pack in the `pkg/` subdirectory (`harper-wasm/pkg/harper_wasm.js`). This mismatch occurred after changing pnpm-workspace.yaml from `harper-wasm/pkg` to `harper-wasm` (commit 1175a043f).

**Incorrect Fix Attempted First**: Updated all import paths to include the `pkg/` directory (e.g., `harper-wasm/pkg/harper_wasm.js`). However, this broke TypeScript type resolution because there were no type declarations for the deep import paths.

**Correct Fix**: Keep the original import paths (e.g., `harper-wasm/harper_wasm.js`) and add an `exports` field to `harper-wasm/package.json` that maps these paths to the actual files in `pkg/`. This allows Node.js/pnpm to resolve the imports correctly while maintaining TypeScript type resolution.

```json
{
  "exports": {
    "": "./pkg/harper_wasm.js",
    "./harper_wasm.js": "./pkg/harper_wasm.js",
    "./harper_wasm.d.ts": "./pkg/harper_wasm.d.ts",
    "./harper_wasm_slim.js": "./pkg/harper_wasm_slim.js",
    "./harper_wasm_slim.d.ts": "./pkg/harper_wasm_slim.d.ts"
  }
}
```

**Files Modified**:
- `harper-wasm/package.json` (added exports field)
- `packages/harper.js/src/BinaryModule.ts`
- `packages/harper.js/src/binaries/binary.ts`
- `packages/harper.js/src/binaries/binaryInlined.ts`
- `packages/harper.js/src/binaries/slimBinary.ts`
- `packages/harper.js/src/binaries/slimBinaryInlined.ts`
- `packages/harper.js/vite.config.ts`

**Example Commits**: 
- 8e598b19b - Incorrect attempt: Updated import paths to include pkg/
- d08f34b8b - Correct fix: Reverted import paths and added exports to package.json

### 9. Timeouts in Browser Extension Tests

**Symptom**: Playwright tests timeout in Chrome/Firefox extension tests.

**Cause**: Tests waiting for events with insufficient timeouts, especially in CI environments without display servers.

**Fix**: Use appropriate timeout values and handle headless environments:
- Set `timeout` in test configurations
- Use `xvfb-run` for Linux CI environments
- Consider reducing timeouts from 90s to 5s where appropriate

**Example Commits**: 818e3a9b1, 9298ffc4b, 50c4e750a, e32d28be5, 82ac7311c

## Workflow Status History

| Date | Commit | Just Checks | Build Web | Issue | Resolution |
|------|--------|-------------|-----------|-------|------------|
| 2026-07-31 | 54855010f | Success | Success | Clippy errors, missing features | Added features, fixed imports |
| 2026-07-31 | 31611ee10 | Success | - | Polish clippy warning | Fixed unused variable |
| 2026-08-01 | 1734bd4f8 | Pending | Failure | Merge from master | - |
| 2026-08-01 | ebead6df9 | Failure | - | German dict expansion | Tests timeout |

## Prevention Checklist

Before pushing to `feature/german-language-support`:

1. [ ] Run `cargo fmt -- --check` to verify formatting
2. [ ] Run `cargo clippy -- -Dwarnings` to check for clippy errors
3. [ ] Ensure all harper-core dependencies use `features = ["multilingual"]` not individual language features
4. [ ] Verify import ordering follows alphabetical convention
5. [ ] Test with `just check-rust` locally
6. [ ] Consider performance impact of dictionary changes
7. [ ] Feature-gate language-specific tests with `#[cfg(feature = "de")]` or `#[cfg(feature = "multilingual")]`
8. [ ] Use sequential workspace testing to avoid feature unification timeouts
9. [ ] Verify pnpm-workspace.yaml paths are correct
10. [ ] Ensure browser extension tests have appropriate timeouts for CI environments
11. [ ] Verify wasm-pack import paths are correct when pnpm-workspace.yaml points to package root (use exports field in package.json, not deep imports)

## See Also

- [GERMAN_ENHANCEMENT_PROGRESS.md](./GERMAN_ENHANCEMENT_PROGRESS.md)
- [GERMAN_PERFORMANCE_OPTIMIZATIONS.md](./GERMAN_PERFORMANCE_OPTIMIZATIONS.md)
- [TIMING_ANALYSIS.md](./TIMING_ANALYSIS.md)

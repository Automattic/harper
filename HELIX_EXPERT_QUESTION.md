# Question for Helix Experts: Multi-Language Harper Configuration

## Context

I'm working on extending the [Harper grammar checker](https://github.com/Automattic/harper) to support German language in addition to English. Harper has a language server (`harper-ls`) that supports different dialects/languages via configuration.

## Current Situation

### Harper-ls Configuration
Harper-ls supports a `dialect` configuration option that can be passed via LSP initialization options:

```toml
[language-server.harper-ls]
command = "harper-ls"
config = { dialect = "American" }  # or "German" when implemented
```

### The Problem
I need to configure Helix to use Harper with different dialects based on the file being edited:
- English documents → `dialect = "American"` 
- German documents → `dialect = "German"`

Currently, Harper-ls is configured globally in `languages.toml`, but I need per-language or per-file-type configuration.

## What I've Tried

I've looked at the Harper-ls source code and confirmed it supports:
1. `dialect` field in initialization options
2. Dynamic configuration via `did_change_configuration`
3. Per-language server configuration in Helix

## The Question

**How can I configure Helix to pass different `dialect` values to Harper-ls based on the file language or file type, without manually changing the configuration when switching between English and German documents?**

### Specific Scenarios

1. **Scenario 1**: I want to edit `README.md` (English) and `DEREADME.md` (German) in the same session
   - English files should use `dialect = "American"`
   - German files should use `dialect = "German"`

2. **Scenario 2**: I have markdown files with mixed content
   - Some files are English, some are German
   - File naming conventions: `*-en.md` vs `*-de.md`

### Possible Solutions I'm Considering

1. **Multiple language server instances** with different configs?
2. **File-type based configuration** in `languages.toml`?
3. **Workspace-specific configuration**?
4. **Dynamic configuration switching** via Helix hooks?

## Technical Details

From the Harper-ls source code, the configuration structure is:
```rust
pub struct Config {
    // ... other fields
    pub dialect: Dialect,  // Supports American, British, German, etc.
    // ... more fields
}
```

The dialect is passed via LSP `initialization_options` in the format:
```json
{
  "harper-ls": {
    "dialect": "American"
  }
}
```

## What I Need From Helix Experts

1. **Is this currently possible** in Helix's LSP configuration system?
2. **What's the recommended approach** for per-language/per-filetype LSP configuration?
3. **Are there examples** of other language servers that handle this scenario?
4. **Would I need to modify Helix** to support this, or is there an existing mechanism?

## Additional Context

- I'm using chezmoi for configuration management
- Helix version: latest from git
- Harper-ls: latest version with German dialect support
- The goal is to have seamless switching between English and German grammar checking

Thank you for any guidance on this!
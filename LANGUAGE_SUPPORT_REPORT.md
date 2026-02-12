# Harper Language Support Report
## Based on GitHub Issue #79: "feat: more languages supported"

**Report generated:** 2026-02-12  
**Source:** https://github.com/Automattic/harper/issues/79

---

## Executive Summary

This report tracks programming and markup language support requests for Harper (a grammar-checking Language Server). It consolidates requests from GitHub issue #79 and all related discussions, cross-referencing them against the currently supported languages.

**Total Languages Requested:** 40+  
**Currently Supported:** 39  
**Implemented Since Issue Creation:** 8  
**Declined/Stalled:** 2+  
**Not Yet Implemented:** 15+

---

## Original Issue Request List (Apr 27, 2024)

The initial request in issue #79 contained these languages:

| Language | Status | Notes |
|----------|--------|-------|
| ✅ Bash | **IMPLEMENTED** | Added as Shell/Bash Script (`shellscript`) |
| ✅ YAML | **IMPLEMENTED** | Supported (language ID: `yaml`) |
| ✅ HTML | **IMPLEMENTED** | Full support (not comments-only) |
| ✅ Java | **IMPLEMENTED** | Comments-only support |
| ❓ Groovy | **NOT VERIFIED** | Status unclear; not on official supported list |
| ✅ Git Commit | **IMPLEMENTED** | As `git-commit` / `gitcommit` |
| ✅ PowerShell | **IMPLEMENTED** | Supported (language ID: `powershell`) |

---

## Languages Implemented AFTER Issue #79 Creation

These were added as per the Oct 2024 update comment:

| Language | PR/Issue | Status | Type |
|----------|----------|--------|------|
| ✅ Haskell | #332 | **CLOSED/MERGED** | Comments-only |
| ✅ Vimscript | #354 | **CLOSED/MERGED** | Comments-only |
| ✅ CMake | #353 | **CLOSED/MERGED** | Comments-only |
| ✅ PHP | Mentioned | **IMPLEMENTED** | Comments-only |
| ✅ Typst | #302 | **CLOSED/MERGED** | Full support (markup language) |
| ✅ Jujutsu Commits | #2082 | **CLOSED/MERGED** | Commit buffers (`jj-commit` / `jjdescription`) |
| ✅ Quarto | #749 | **CLOSED/MERGED** | Markup (extends Markdown) |
| ✅ Solidity | #970 | **CLOSED/MERGED** | Comments-only |

---

## Community-Requested Languages (From Issue #79 Comments)

### Languages IMPLEMENTED ✅

| Language | Comments | Requestor | Notes |
|----------|----------|-----------|-------|
| ✅ PHP | "If you could add `php` to the list" | Comment 2 | Now supports comments |
| ✅ Dart | "Adding `dart` would be great" | Comment 3 | Now supports comments |
| ✅ Markdown | "markdown support seems pretty advanced" | Comment 4 | Already supported (noted as advanced) |
| ✅ Haskell | Mentioned as in progress | Comment 5 | PR #332 merged |
| ✅ CMake | Mentioned as in progress | Comment 5 | PR #353 merged |
| ✅ Vimscript | Mentioned as in progress | Comment 5 | PR #354 merged |
| ✅ PowerShell | "I plan on adding PHP and potentially PowerShell later today" | Comment 5 | Now supported |
| ✅ Kotlin | Mentioned as "probably requires least effort" | Comment 8 | Now supports comments |
| ✅ Scala | "I would love to have support for Scala" | Comment 16 | Now supports comments |

---

### Languages PARTIALLY ADDRESSED / IN PROGRESS ⚠️

| Language | Status | Details |
|----------|--------|---------|
| ⚠️ **LaTeX** | ATTEMPTED, DECLINED | Attempted by @grantlemons but "never made it in"; complexity of LaTeX parsing and turing-complete syntax makes consistent masking impossible. See ltex-ls for reference. |
| ⚠️ **Djot** | INQUIRED | User asked about Djot support "with same level of functionality as Markdown"; no response but not rejected. Tree-sitter parser available, might be feasible. |

---

### Languages NOT YET IMPLEMENTED ❌

#### Requested in Comments:

| Language | Requestor | Details | Priority |
|----------|-----------|---------|----------|
| ❌ Emacs Org-mode | Comment 6, 19 | "Org-mode support would be rad" / "If you are serious about Emacs support, then Org-mode is a must" – **Now partially implemented** (on supported list as `org`) | MEDIUM |
| ❌ Assembly | Comment 8 | Uses `;` for comments "at least the common ones" | LOW |
| ❌ Elixir | Comment 8 | No specific details provided | MEDIUM |
| ❌ Odin | Comment 8 | "C-like comments" per requester | MEDIUM |
| ❌ Zig | Comment 8 | Reference to issue #218; "C-like comments" | MEDIUM |
| ❌ Mail Format | Comment 9 | Tree-sitter parser in progress (`tree-sitter-mail`); currently uses PlainEnglish parser which applies rules to headers | HIGH |
| ❌ RMarkdown | Comment 12 | "solid tree-sitter parser available"; treat as Markdown for now; false positives on syntax elements | MEDIUM |
| ❌ Quatro | Comment 12 | Tree-sitter parser available; extends Markdown | HIGH (work in progress per comment 5) |
| ❌ LaTeX | Comment 13 | Attempted but failed; very complex (turing-complete syntax) | LOW |
| ❌ AsciiDoc | Comment 14 | **Now on supported list** (`asciidoc` - full support) | COMPLETED |
| ❌ YAML | Comment 15 | **Now on supported list** | COMPLETED |
| ❌ MDX | Comment 17 | "Please add MDX support: https://mdxjs.com/docs/what-is-mdx/" | MEDIUM |
| ❌ ReStructuredText (.rst) | Comment 18 | Use case: Sphinx documentation rendering | MEDIUM |
| ❌ Verilog / SystemVerilog | Comment 21 | "Can it support Verilog and SystemVerilog?" - Depends on Tree-sitter grammar availability | LOW |
| ❌ Ren'Py (.rpy) | Comment 24 | User requested but noted not in supported languages; architectural limitation noted | LOW |

---

### Other Languages on Current Supported List (Not Explicitly Requested in #79)

These are now supported but were not mentioned in issue #79 comments:

| Language | Support Type | Notes |
|-----------|--------------|-------|
| ✅ C | Comments-only | Standard language |
| ✅ C++ | Comments-only | Standard language |
| ✅ C# | Comments-only | Standard language |
| ✅ Clojure | Comments-only | Functional language |
| ✅ DAML | Comments-only | Data modeling language |
| ✅ Go | Comments-only | Modern systems language |
| ✅ Ink | Full support | Narrative scripting language |
| ✅ JavaScript/React | Comments-only | Web development |
| ✅ Literate Haskell | Full support | Variant of Haskell |
| ✅ Lua | Comments-only | Scripting language |
| ✅ Markdown | Full support | Core supported markup |
| ✅ Nix | Comments-only | Package manager language |
| ✅ **Org Mode** | Full support | ✅ NOW SUPPORTED (not on original list) |
| ✅ Plain Text | Full support | Fallback support |
| ✅ Python | Comments-only | Popular language |
| ✅ Ruby | Comments-only | Popular language |
| ✅ Rust | Comments-only | Modern systems language |
| ✅ Swift | Comments-only | Apple platform language |
| ✅ TOML | Comments-only | Config file format |
| ✅ TypeScript/React | Comments-only | Web development |

---

## Architectural Constraints & Design Decisions

### Key Insight (From maintainer):
> "The reason we need to add support for languages one by one is because before harper-ls can lint anything, it first needs to know how to parse the file it's given. Without knowing how to do that, harper-ls won't know what parts it need to lint vs what parts it needs to ignore (e.g., comments vs code)."

This explains why a generic "editable list of file extensions" approach isn't feasible – Tree-sitter grammar support is required for each language.

### Language Addition Process:
1. Add Tree-sitter grammar support
2. Create pull request to `nvim-lspconfig` (Neovim)
3. Create pull request to `mason` (Language server installer)
4. Update `harper-ls` to announce capability
5. Add tests for the language

---

## Declined/Unsuitable Requests

### LaTeX ❌
- **Status:** ATTEMPTED, DECLINED
- **Reason:** Latex's turing-complete syntax makes consistent Tree-sitter parsing essentially impossible
- **Complexity:** Determining what constitutes "text" vs "function input" is fundamentally intractable
- **Reference:** ltex-ls has some solutions but would require major effort
- **Verdict:** Not feasible without major architectural changes

### Djot (Markup) ❓
- **Status:** INQUIRED, NOT EXPLICITLY DECLINED
- **Request:** Support for alternative markdown-like markup with "same level of functionality"
- **Response:** "We've got an open PR for Typst support... I do not personally have time... entirely open to PRs"
- **Verdict:** Community PRs welcome; maintainer capacity limited

---

## Summary Table: Request Status Overview

```
ORIGINALLY REQUESTED (Apr 2024):
  Bash              ✅ DONE
  YAML              ✅ DONE
  HTML              ✅ DONE
  Java              ✅ DONE
  Groovy            ❓ UNCLEAR
  Git commit        ✅ DONE
  PowerShell        ✅ DONE

IMPLEMENTED AFTER ISSUE (Oct 2024 onwards):
  Haskell           ✅ PR #332
  Vimscript         ✅ PR #354
  CMake             ✅ PR #353
  PHP               ✅ DONE
  Typst             ✅ PR #302
  Jujutsu           ✅ PR #2082
  Quarto            ✅ PR #749
  Solidity          ✅ PR #970

NOT YET IMPLEMENTED:
  Assembly          ❌
  Elixir            ❌
  Odin              ❌
  Zig               ❌
  Mail Format       ❌ (IN PROGRESS)
  RMarkdown         ❌
  MDX               ❌
  ReStructuredText  ❌
  Verilog/SystemVerilog ❌
  Ren'Py            ❌
  Djot              ❌

DECLINED/NOT FEASIBLE:
  LaTeX             ❌ (Too complex, turing-complete)

BONUS (NOT IN ISSUE #79 BUT NOW SUPPORTED):
  Org Mode          ✅ IMPLEMENTED
  AsciiDoc          ✅ IMPLEMENTED
  + 16 other languages
```

---

## Key Takeaways

1. **High Success Rate:** 8 of the original 7 requested languages are implemented, plus many additional community requests
2. **Active Development:** Maintainer (elijah-potter) has been actively adding language support through 2024
3. **Tree-Sitter Dependency:** All new languages require a Tree-sitter grammar; this is the main blocker
4. **Community Contributions:** Project welcomes PRs for new languages, but maintainer capacity is limited
5. **Notable Addition:** Org-mode was successfully implemented (not originally requested but heavily asked for)
6. **Architectural Limits:** Some languages (LaTeX) are fundamentally difficult due to their syntax complexity

---

## Recommendations for Future Requests

1. **Check for Tree-sitter support first** – Language must have an actively maintained Tree-sitter grammar
2. **Provide PR if possible** – Community contributions are welcome and help move issues forward
3. **Avoid turing-complete syntaxes** – Languages like LaTeX are not good candidates
4. **Comment masking complexity varies** – JavaDoc-style comments are more complex but manageable
5. **Reference working implementations** – Point to similar languages already supported for guidance

---

## Sources
- GitHub Issue: https://github.com/Automattic/harper/issues/79
- Official Supported Languages: https://writewithharper.com/docs/integrations/language-server
- Related PRs: #332, #353, #354, #302, #2082, #749, #970
- Date Accessed: 2026-02-12

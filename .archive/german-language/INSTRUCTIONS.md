# German Language Support - Improvement Instructions

This file provides guidance on improving German language support in Harper.

## Quick Start

**Key Principle**: Dictionary flags are SINGLE CHARACTERS. `~~JOQRSTUW` means individual flags: J, O, Q, R, S, T, U, W.

**Current Status**:
- Efficiency: 2.79 (target: >2.5 for high compounding productivity)
- Words with compound flags: 7,846+ (was 3,471)
- Dictionary size: 222,621 base entries → 623,211 expanded words
- Coverage: 79.4% on external word list
- Compound adjective recognition: FIXED via enhanced compound checker

## Improvement Strategy

### 1. Add Compound Flags

**Adjectives**: Add `q` flag to enable adjective compound generation
```bash
python3 scripts/add_q_to_all_adjectives.py --apply
```

**Nouns**: Add `h` flag (no interfix) to enable noun+noun compounds
```bash
# For pure nouns (only gender flags)
python3 scripts/add_h_to_pure_nouns.py --apply --limit 1000

# For targeted nouns with FST flags
python3 scripts/add_h_flag_to_nouns_batch.py --has-affix-flags --apply --limit 1000
```

### 2. Remove Redundant Words

After adding flags, identify and remove compounds Harper can now generate:
```bash
# Dry run first
python3 scripts/remove_reducible_compounds.py --min-length 6 --dry-run

# Apply removal
python3 scripts/remove_reducible_compounds.py --min-length 6
```

### 3. Measure Impact

```bash
python3 scripts/measure_efficiency.py
```

## Available Affix Rules

- **Noun pluralization**: X (-e), Y (-n/-en), a (-er+umlaut), b (-s)
- **Adjective declension**: J (base), O (-e), Q (-em), R (-en), S (-er), T (-es), U (comparative -er), W (superlative -sten)
- **Verb conjugation**: c (-enden), d (-te), e (-ten), f (-e), h (-st), i (-t), j (-en)
- **Compound formation**: h (no interfix), i (-s), k (-n), l (-en), m (-er), o (-es), q (adjective)

## Testing Commands

```bash
# Build testing framework (once)
just language-build

# Test specific text
just language-test german "die mondlandung ist wichtig"

# Show word metadata
just language-meta german "Mondlandung"

# Measure efficiency
python3 scripts/measure_efficiency.py
```

## Key Scripts

| Script | Purpose |
|--------|---------|
| `identify_all_reducible_v2.py` | Find all reducible words (compounds, FST, prefix/suffix) |
| `identify_reducible_compounds.py` | Find reducible compound words |
| `remove_reducible_compounds.py` | Remove reducible compound words (FIXED: matches Harper logic) |
| `verify_reducible.py` | Verify words can still be recognized after removal |
| `add_q_to_all_adjectives.py` | Add q flags to adjectives |
| `add_h_to_pure_nouns.py` | Add h flags to pure nouns |
| `measure_efficiency.py` | Measure efficiency metrics |

## Compound Generation Logic (CRITICAL)

From `compound.rs`:
- **Adjective compounds**: If EITHER word has `q` flag → generate compound (no interfix)
- **Noun compounds**: If NEITHER word has `q` flag → use first word's interfix flag (h,i,k,l,m,o)
- **Both words MUST have compound flags** for any compound to be generated

This means:
- `laut` (has q) + `entwicklung` (has h) → can generate `lautentwicklung` (adjective compound)
- `Bund` (has o) + `Staat` (has h) → can generate `Bundesstaat` (noun compound with -es interfix)

## Data Quality Notes

- Many words are incorrectly flagged (e.g., verb forms marked as nouns)
- Uppercase flags (N, F, M, Z, etc.) are POS **properties**, not compound flags
- Only lowercase flags (h, i, k, l, m, o, q) are compound formation flags
- Always verify changes with `harper-lang-test` before committing large batches

## Noun Capitalization Rule and Compound Words

**Problem**: The German noun capitalization rule was incorrectly flagging compound adjectives (e.g., "mehrspuriges", "motorgetriebenes") as nouns that need capitalization.

**Root Cause**: The `get_compound_metadata()` function in `compound_checker.rs` was defaulting to noun metadata for all compound words, assuming "most German compounds are nouns".

**Solution**: Enhanced `get_compound_metadata()` to check if a compound word starts with an adjective (has `q` flag) and return adjective metadata for such compounds.

**Dictionary Requirements**:
- Adjective roots must have the `q` flag to participate in adjective compound formation
- Words like `mehrspurig`, `motorgetrieben` should have `q` flag: `mehrspurig/~~qh`
- Words like `laut`, `hoch`, `schnell` should have `q` flag for adjective compounds

**Testing**: After fixing dictionary flags, verify with:
```bash
just language-test german "das mehrspurige fahrzeug"
# Should NOT flag "mehrspuriges" as a noun
```

## Common Flag Corrections Needed

### Adjectives Incorrectly Marked as Nouns
- `mehrspurig/~~Nh` → `mehrspurig/~~qh` (adjective compound)
- `motorgetrieben/~~Nh` → `motorgetrieben/~~qh` (adjective compound)
- `bestimmt/~~NXh` → `bestimmt/~~Jh` (adjective)
- `mehr/~/~Nh` → `mehr/~/~Rh` (adverb)
- `aller/~~Mh` → `aller/~~Ih` (pronoun)
- `allem/~~Nh` → `allem/~~Ih` (pronoun)
- `alles/~~Nh` → `alles/~~Ih` (pronoun)

### Flag Meanings Reference
- `N` = noun property
- `J` = adjective property  
- `A` = adjective property (alias for J)
- `R` = adverb property
- `r` = adverb property (lowercase variant)
- `I` = pronoun property
- `V` = verb property
- `q` = adjective compound flag (lowercase)
- `h, i, k, l, m, o` = noun compound interfix flags (lowercase)

## Version Control

The dictionary is under Git. Use Git features instead of backup files:
```bash
# View changes
git diff harper-core/src/language/german/dictionary.dict

# Commit changes
git add harper-core/src/language/german/dictionary.dict
git commit -m "Your message"

# Revert changes
git checkout harper-core/src/language/german/dictionary.dict
```

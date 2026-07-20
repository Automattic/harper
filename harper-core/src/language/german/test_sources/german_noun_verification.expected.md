# Expected Results for German Noun Capitalization Verification

## Test Results Format

For each test sentence, this file documents:
1. Words that should be recognized with their correct POS
2. Words that should trigger lint warnings (if lowercase)
3. Words that should NOT trigger lint warnings

## Expected Results by Sentence

### Sentence 1: Die Mondlandung ist wieder fehlgeschlagen

**Word-by-word analysis:**
- Die: DETERMINER (article) - correct, no lint
- Mondlandung: NOUN (feminine) - correct, no lint (capitalized)
- ist: VERB (3rd person singular) - correct, no lint
- wieder: ADVERB - correct, no lint
- fehlgeschlagen: VERB (past participle) - correct, no lint

**If all lowercase: "die mondlandung ist wieder fehlgeschlagen"**
- "die": no lint (function word)
- "mondlandung": SHOULD LINT - noun not capitalized
- "ist": no lint (verb, not noun)
- "wieder": no lint (adverb, not noun)
- "fehlgeschlagen": no lint (past participle, not noun)

**Expected lint count: 1** (only mondlandung)

### Sentence 2: Ich schreibe einen Brief und du liest ein Buch

**If all lowercase: "ich schreibe einen brief und du liest ein buch"**
- "ich": no lint (pronoun)
- "schreibe": no lint (verb, not noun)
- "einen": no lint (article)
- "brief": SHOULD LINT - noun not capitalized
- "und": no lint (conjunction)
- "du": no lint (pronoun)
- "liest": no lint (verb, not noun)
- "ein": no lint (article)
- "buch": SHOULD LINT - noun not capitalized

**Expected lint count: 2** (brief, buch)

### Sentence 3: Die Freiheit ist wichtig für die Menschheit

**If all lowercase: "die freiheit ist wichtig für die menschheit"**
- "die": no lint (article)
- "freiheit": SHOULD LINT - noun not capitalized
- "ist": no lint (verb, not noun)
- "wichtig": no lint (adjective, not noun)
- "für": no lint (preposition)
- "die": no lint (article)
- "menschheit": SHOULD LINT - noun not capitalized

**Expected lint count: 2** (freiheit, menschheit)

## Known Issues to Fix

### Current Dictionary Problems
1. "Mondlandung" is missing from dictionary - needs to be added as `Mondlandung/~~NF`
2. "ist" is marked as `~~N` (noun) but should be `~~V` (verb) - this causes false positives
3. "wieder" is marked as `~~J` (adjective) but should be `~~R` (adverb) - this is a metadata issue

### Metadata Verification
Use the testing framework to verify fixes:
```bash
just language-test german --word "Mondlandung"  # Should show: Noun, gender: FEMININE
just language-test german --word "ist"         # Should show: Verb
just language-test german --word "wieder"     # Should show: Adverb
just language-test german --word "schreibe"   # Should show: Verb
just language-test german --word "fehlgeschlagen" # Should show: Verb (PAST_PARTICIPLE)
```

## How to Use These Expected Results

1. **For manual testing:** Use `just language-test german "sentence here"`
2. **For automated testing:** Create Rust tests in `tests/` directory
3. **For continuous verification:** Run `just language-rust-test german`

## Improvement Workflow

1. **Identify problems:** Run test sentences and note false positives/negatives
2. **Fix dictionary.dict:** Correct POS flags for problematic words
3. **Verify metadata:** Use `--metadata` flag to check word metadata
4. **Re-test:** Run tests again to confirm fixes
5. **Commit:** Save changes with clear commit messages

## Priority Fixes

### High Priority (Causing false positives in noun capitalization)
- [ ] Add "Mondlandung/~~NF" to dictionary.dict
- [ ] Change "ist/~~N" to "ist/~~V" in dictionary.dict
- [ ] Change "wieder/~~J" to "wieder/~~R" in dictionary.dict

### Medium Priority (Missing words)
- [ ] Add "liest/~~Vt" (3rd person singular of lesen)
- [ ] Add "wurde/~~Vt" (past tense of werden)
- [ ] Verify "gescheitert/~~g" and "abgebrochen/~~g" are present

### Low Priority (Enhancement)
- [ ] Add more compound noun examples
- [ ] Add more verb conjugation examples
- [ ] Add more adverb examples
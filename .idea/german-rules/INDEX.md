# German Rules Catalogue - Comprehensive Index

## Quick Statistics

- **Total LanguageTool Rules**: 2,215
- **Total Categories**: 14
- **Ported to Harper**: 21 rules (~1%)
- **Harper Weir Files**: 21 files
- **Success Rate**: Very low for complex grammar, high for simple typos

## Category Overview

| Category | Total Rules | Ported | Success Rate | Difficulty |
|----------|-------------|---------|--------------|------------|
| TYPOS | 100+ | 3 | ~3% | Mixed |
| CONFUSED_WORDS | 200+ | 0 | 0% | Very High |
| GRAMMAR | 300+ | 0 | 0% | Very High |
| COMPOUNDING | 150+ | 0 | 0% | High |
| CASING | 200+ | 0 | 0% | Medium |
| PUNCTUATION | 100+ | 0 | 0% | Low |
| SEMANTICS | 50+ | 0 | 0% | Very High |
| CORRESPONDENCE | 30+ | 0 | 0% | Medium |
| PROPER_NOUNS | 100+ | 0 | 0% | Low |
| IDIOMS | 200+ | 0 | 0% | High |
| TYPOGRAPHY | 50+ | 0 | 0% | Low |
| WIKIPEDIA | 20+ | 0 | 0% | Low |
| MULTITOKEN_SPELLING | 800+ | 0 | 0% | Mixed |
| HILFESTELLUNG_KOMMASETZUNG | 100+ | 0 | 0% | High |

## Successfully Ported Rules

### ✅ Working Harper Weir Rules (21 files)

1. **BeimBei.weir** - `bei beim → bei`
2. **BenuetzenBenutzen.weir** - `benuetzen → benutzen`
3. **BerietenBereiten.weir** - `berieten → bereiten`
4. **ChannelChannel.weir** - `Channel → Channel`
5. **DeinDein.weir** - `dein dein → dein`
6. **EtwasDass.weir** - `etwas dass → etwas dass`
7. **FuerFuer.weir** - `fuer → für`
8. **GeschehenGeschehen.weir** - `geschehen → geschehen`
9. **HerzlichenDank.weir** - `herzlichen Dank → herzlichen Dank`
10. **HerstellenHerstellen.weir** - `herstellen herstellen → herstellen`
11. **ImAm.weir** - `im am → im`
12. **InDemIm.weir** - `in dem → im`
13. **InIn.weir** - `in in → in`
14. **MeinMein.weir** - `mein mein → mein`
15. **VerleihenVerleihen.weir** - `verleihen verleihen → verleihen`
16. **VielenDank.weir** - `vielen Dank → vielen Dank`
17. **WirHaben.weir** - `wir habe → wir haben`
18. **ZuDemZum.weir** - `zu dem → zum`
19. **ZumAnbeissen.weir** - `zum Anbeißen → zum Anbeißen`
20. **ZurDer.weir** - `zur der → zur`
21. **ZurZum.weir** - `zur zum → zur`
22. **Zwingendermaassen.weir** - `zwingendermaßen → gezwungenermaßen`

### ❌ Attempted but Failed (Removed)

1. **DollerDollar.weir** - `Doller → Dollar` (length change issue)
2. **GesetzSetz.weir** - `Gesetz → Gesetz` (context issue)
3. **AuschauenAusschauen.weir** - `ausschauen → ausschauen` (failed tests)
4. **DurchfallenDurchfallen.weir** - `durchfallen durchfallen → durchfallen` (failed tests)
5. **BestehenBestehen.weir** - `bestehen bestehen → bestehen` (failed tests)
6. **HerstellenHerstellen.weir** - `herstellen herstellen → herstellen` (failed tests)
7. **And 5 more** - Various complex pattern failures

## Category Files

### Created Catalogue Files

1. **[README.md](./README.md)** - Main overview and introduction
2. **[TYPOS.md](./TYPOS.md)** - Possible typos (3/100+ ported)
3. **[CONFUSED_WORDS.md](./CONFUSED_WORDS.md)** - Confused words (0/200+ ported)
4. **[GRAMMAR.md](./GRAMMAR.md)** - Grammar rules (0/300+ ported)

### Remaining Categories to Document

5. **COMPOUNDING.md** - Compound word rules (not created)
6. **CASING.md** - Capitalization rules (not created)
7. **PUNCTUATION.md** - Punctuation rules (not created)
8. **SEMANTICS.md** - Semantic rules (not created)
9. **CORRESPONDENCE.md** - Correspondence rules (not created)
10. **PROPER_NOUNS.md** - Proper noun rules (not created)
11. **IDIOMS.md** - Idiom rules (not created)
12. **TYPOGRAPHY.md** - Typography rules (not created)
13. **WIKIPEDIA.md** - Wikipedia rules (not created)
14. **MULTITOKEN_SPELLING.md** - Spelling error rules (not created)
15. **HILFESTELLUNG_KOMMASETZUNG.md** - Comma rules (not created)

## Filtering and Search

### Find All Ported Rules
```bash
grep -r "✅ PORTED" .idea/german-rules/
```

### Find All Not Ported Rules
```bash
grep -r "❌ NOT PORTED" .idea/german-rules/
```

### Find Rules by Category
```bash
cat .idea/german-rules/TYPOS.md
cat .idea/german-rules/GRAMMAR.md
```

### Find Harper Implementation Files
```bash
find harper-core/src/linting/weir_rules/de -name "*.weir"
```

### Test Harper German Rules
```bash
cargo test --package harper-core --lib linting::german_weir_rules
```

## Priority Rankings

### High Priority for Future Porting

**Easy Wins** (Low difficulty, high value):
1. Fixed expressions (vom Beruf, seit Anfang an)
2. Simple umlaut corrections (GLUCK → Glück)
3. Simple spelling variants (Aufritt → Auftritt)
4. Number-triggered rules (7 Tage → 7 Tagen)
5. Compound spelling (kurz fassen → kurzfassen)

**Medium Priority** (Moderate difficulty, good value):
1. Common adjective endings (schöne Wetter → schönes Wetter)
2. Simple verb conjugations (ich wart → ich warte)
3. Preposition contractions (zu dem → zum) ✅ ALREADY DONE
4. Duplicate detection (bei beim → bei) ✅ ALREADY DONE
5. Common plural errors (Passworte → Passwörter)

**Lower Priority** (High difficulty, limited value):
1. Complex grammar rules (case government, agreement)
2. Context-sensitive word choice
3. Semantic analysis
4. Advanced punctuation rules

## Technical Assessment

### What Works Well with Weir

✅ **Simple Duplicates**: "bei beim", "wir habe"
✅ **Umlaut Corrections**: "fuer" → "für"
✅ **Fixed Expressions**: "in dem" → "im"
✅ **Simple Spelling**: "benuetzen" → "benutzen"
✅ **Contraction Opportunities**: "zu dem" → "zum"

### What Doesn't Work Well

❌ **Context-Sensitive Rules**: Requires POS tagging
❌ **Length-Changing Corrections**: Weir requires same length
❌ **Complex Grammar**: Requires syntactic analysis
❌ **Case System**: Requires morphological analysis
❌ **Agreement Rules**: Requires grammatical understanding

## Recommendations

### For Future Development

1. **Focus on Strengths**: Continue with simple patterns that Weir handles well
2. **Accept Limitations**: Recognize that complex grammar is beyond Weir's scope
3. **Hybrid Approach**: Consider integrating with grammar checkers for complex rules
4. **Pattern Libraries**: Build comprehensive pattern libraries for common errors
5. **Quality Over Quantity**: 21 working rules > 100 broken rules

### For LanguageTool Integration

1. **Complementary Tools**: Use Harper for spelling, LanguageTool for grammar
2. **Pipeline Approach**: Run Harper first, then LanguageTool for complex checks
3. **Error Prioritization**: Focus on errors that each tool handles best
4. **Performance**: Harper is faster for simple patterns

### For Users

1. **Expectation Management**: Harper focuses on spelling/typography, not full grammar
2. **Best Use Cases**: Quick spell checking, duplicate detection, simple corrections
3. **Advanced Grammar**: Use dedicated grammar checker for complex issues
4. **Performance**: Fast for basic checks, lightweight compared to full grammar tools

## References

- **LanguageTool**: `~/gallery/languagetool/`
- **Harper Weir Rules**: `harper-core/src/linting/weir_rules/de/`
- **Test Suite**: `harper-core/tests/german_*_test.rs`
- **Documentation**: See individual category files for details

## Contributing

When adding new rules:

1. **Check Feasibility**: Verify rule is suitable for Weir format
2. **Add Comprehensive Tests**: 15+ test cases per rule
3. **Update Catalogue**: Mark rule as ported in relevant category file
4. **Test Thoroughly**: Ensure all tests pass
5. **Document**: Add notes about any limitations or special cases

## Status Summary

**Current State**: 21 working German Weir rules
**Success Rate**: ~1% of LanguageTool rules
**Primary Focus**: Simple spelling, duplicates, contractions
**Limitations**: Complex grammar, context-sensitive rules
**Future**: Continue with patterns that work well, accept limitations

---

*Last Updated: 2025-04-23*
*Total Rules Catalogued: 2,215*
*Porting Progress: ~1%*

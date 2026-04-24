# German Grammar Rules Catalogue

This catalogue tracks the porting status of German grammar rules from LanguageTool to Harper.

## Overview

- **Total LanguageTool Rules**: 2,215
- **Ported to Harper**: 21
- **Not Ported**: 2,194
- **Porting Progress**: ~1%

## Categories

1. [TYPOS - Mögliche Tippfehler](./TYPOS.md) - Possible typos
2. [CONFUSED_WORDS - Leicht zu verwechselnde Wörter](./CONFUSED_WORDS.md) - Easily confused words
3. [PROPER_NOUNS - Prominente/geographische Eigennamen](./PROPER_NOUNS.md) - Prominent/geographic proper nouns
4. [COMPOUNDING - Getrennt- und Zusammenschreibung](./COMPOUNDING.md) - Separate and compound writing
5. [SEMANTICS - Semantische Unstimmigkeiten](./SEMANTICS.md) - Semantic inconsistencies
6. [CORRESPONDENCE - Briefe und E-Mails](./CORRESPONDENCE.md) - Letters and emails
7. [CASING - Groß-/Kleinschreibung](./CASING.md) - Capitalization
8. [GRAMMAR - Grammatik](./GRAMMAR.md) - Grammar
9. [PUNCTUATION - Zeichensetzung](./PUNCTUATION.md) - Punctuation
10. [TYPOGRAPHY - Typografie](./TYPOGRAPHY.md) - Typography
11. [WIKIPEDIA - Wikipedia](./WIKIPEDIA.md) - Wikipedia specific
12. [HILFESTELLUNG_KOMMASETZUNG - Kommasetzung](./HILFESTELLUNG_KOMMASETZUNG.md) - Commasetzungs
13. [IDIOMS - Redewendungen](./IDIOMS.md) - Idioms
14. [MULTITOKEN_SPELLING - Rechtschreibfehler](./MULTITOKEN_SPELLING.md) - Spelling errors

## Ported Rules

### Currently Ported to Harper (21 rules)

The following rules have been successfully ported to Harper's Weir format:

1. **BeimBei** - bei beim → bei
2. **BenuetzenBenutzen** - benuetzen → benutzen
3. **BerietenBereiten** - berieten → bereiten
4. **ChannelChannel** - Channel → Channel (YouTube, TV, etc.)
5. **DeinDein** - dein dein → dein
6. **EtwasDass** - etwas dass → etwas dass
7. **FuerFuer** - fuer → für
8. **GeschehenGeschehen** - geschehen → geschehen
9. **HerzlichenDank** - herzlichen Dank → herzlichen Dank
10. **HerstellenHerstellen** - herstellen herstellen → herstellen
11. **ImAm** - im am → im
12. **InDemIm** - in dem → im
13. **InIn** - in in → in
14. **MeinMein** - mein mein → mein
15. **VerleihenVerleihen** - verleihen verleihen → verleihen
16. **VielenDank** - vielen Dank → vielen Dank
17. **WirHaben** - wir habe → wir haben
18. **ZuDemZum** - zu dem → zum
19. **ZumAnbeissen** - zum Anbeißen → zum Anbeißen
20. **ZurDer** - zur der → zur
21. **ZurZum** - zur zum → zur
22. **Zwingendermaassen** - zwingendermaßen → gezwungenermaßen

### Harper Weir Rule Locations

All ported rules are located in: `harper-core/src/linting/weir_rules/de/`

## Not Ported Rules

The majority of LanguageTool rules (2,194) have not been ported yet. See individual category files for details.

## Porting Notes

### Challenges in Porting

Many LanguageTool rules cannot be easily ported to Harper's Weir format due to:

1. **Complex Context**: LanguageTool uses sophisticated `<antipattern>` sections with POS tagging
2. **Length-Changing Corrections**: Weir requires same-length replacements
3. **Complex Grammar**: Rules requiring sentence structure analysis
4. **Semantic Analysis**: Rules requiring understanding of meaning/context

### Successfully Ported Rule Types

The following types of rules have been successfully ported:

- ✅ **Simple duplicate detection**: "bei beim" → "bei"
- ✅ **Umlaut corrections**: "fuer" → "für"
- ✅ **Contraction opportunities**: "in dem" → "im"
- ✅ **Same-length spelling corrections**: "benuetzen" → "benutzen"

### Difficult to Port

The following rule types are difficult or impossible to port with current Weir limitations:

- ❌ **Context-sensitive word choices**: Requires POS tagging
- ❌ **Complex grammar patterns**: Requires sentence structure analysis
- ❌ **Length-changing corrections**: "Channel" → "Channel" (different lengths)
- ❌ **Semantic rules**: Requires understanding meaning

## References

- **LanguageTool German Rules**: `~/gallery/languagetool/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml`
- **Harper Weir Rules**: `harper-core/src/linting/weir_rules/de/`
- **Weir Format Documentation**: See Harper documentation for Weir rule syntax

## Filtering

To filter the catalogue:

```bash
# Find all ported rules
grep -r "✅ PORTED" .idea/german-rules/

# Find all not ported rules
grep -r "❌ NOT PORTED" .idea/german-rules/

# Find rules by category
cat .idea/german-rules/TYPOS.md
```

## Contributing

When porting new rules:

1. Create the Weir rule file in `harper-core/src/linting/weir_rules/de/`
2. Add comprehensive tests (15+ test cases)
3. Update this catalogue with PORTED status
4. Run tests: `cargo test --package harper-core --lib linting::german_weir_rules`

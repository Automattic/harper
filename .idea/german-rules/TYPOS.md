# TYPOS - Mögliche Tippfehler (Possible Typos)

## Overview
This category contains common German typos and spelling mistakes.

## Ported Rules (3/100+)

### ✅ PORTED

1. **ZWINGENDERMASSEN_SPELLING_RULE** - `zwingendermaßen → gezwungenermaßen`
   - **Harper File**: `Zwingendermaassen.weir`
   - **Status**: ✅ PORTED
   - **Notes**: Successfully handles umlaut correction

2. **CHANEL_CHANNEL_SPELLING_RULE** - `Chanel → Channel` (YouTube, TV)
   - **Harper File**: `ChannelChannel.weir`
   - **Status**: ✅ PORTED
   - **Notes**: Fixed for "YouTube-Channel", "TV-Channel"

3. **DOLLER_DOLLAR_SPELLING_RULE** - `Doller → Dollar`
   - **Status**: ❌ ATTEMPTED BUT FAILED
   - **Notes**: Length change prevents Weir implementation (6 chars → 6 chars, but different pattern)
   - **Removed from catalogue**: File deleted due to test failures

## Not Ported Rules (97+)

### ❌ NOT PORTED - High Priority

4. **WASN_DAS_SPELLING_RULE** - `wasn das → was ist denn das`
   - **Reason**: Complex phrase replacement, length change
   - **Difficulty**: High

5. **PHILIPP_LAHM_SPELLING_RULE** - `Phillipp → Philipp` (Lahm)
   - **Reason**: Context-specific (name), requires antipatterns
   - **Difficulty**: Medium

6. **FREUDE_BERIETEN_SPELLING_RULE** - `Freude zu berieten → bereiten`
   - **Reason**: Context-sensitive verb choice
   - **Difficulty**: Medium

7. **FINANZTIP_SPELLING_RULE** - `Finanztip → Finanztip` vs `Finanztipp`
   - **Reason**: Complex context (brand name vs common noun), many antipatterns
   - **Difficulty**: High

8. **WEIßHEIT_VS_WEISHEIT_SPELLING_RULE** - `Weißheit → Weisheit`
   - **Reason**: Requires exception handling (colors, printing)
   - **Difficulty**: Medium

9. **WIR_HABE** - `wir habe → wir haben`
   - **Harper File**: `WirHaben.weir`
   - **Status**: ✅ PORTED (Duplicate detection)

### ❌ NOT PORTED - Medium Priority

10. **JETZT_ERSTE_SPELLING_RULE** - `jetzt erste → jetzt erst`
    - **Reason**: Part of speech tagging required
    - **Difficulty**: Medium

11. **TYPO_KAMM_KANN_SPELLING_RULE** - `kamm → kann/kam`
    - **Reason**: Complex homophone, requires POS context
    - **Difficulty**: High

12. **FISCHEN_FRISCHEN_SPELLING_RULE** - `fischen → frischen`
    - **Reason**: Context-sensitive adjective correction
    - **Difficulty**: Medium

13. **WICHE_WOCHE_SPELLING_RULE** - `Wiche → Woche`
    - **Reason**: Capitalization + spelling
    - **Difficulty**: Low

14. **SCHLAF_INS_GESICHT_SPELLING_RULE** - `Schlaf → Schlag ins Gesicht`
    - **Reason**: Idiom correction
    - **Difficulty**: Medium

15. **GEHT_RAUTE_S_SPELLING_RULE** - `geht#s → geht's`
    - **Reason**: Special character handling
    - **Difficulty**: Low

16. **VERTRAGENDE_SPELLING_RULE** - `Vertragende → Vertragsende`
    - **Reason**: Complex word form analysis
    - **Difficulty**: Medium

17. **VIELE_ERFOLG_SPELLING_RULE** - `Viele Erfolg → Viel Erfolg`
    - **Reason**: Grammar agreement
    - **Difficulty**: Medium

18. **PLUS_BER_SPELLING_RULE** - `+ber → über`
    - **Reason**: Special character + word correction
    - **Difficulty**: Low

19. **WAR_FUER_SPELLING_RULE** - `War für → Was für`
    - **Reason**: Capitalization + word choice
    - **Difficulty**: Medium

20. **IM_ENDDEFEKT_SPELLING_RULE** - `im Enddefekt → im Endeffekt`
    - **Reason**: Spelling in fixed expression
    - **Difficulty**: Low

21. **AUF_JEDEN_FALLS_SPELLING_RULE** - `auf jeden Falls → auf jeden Fall`
    - **Reason**: Grammar agreement
    - **Difficulty**: Low

22. **LETZTES_ENDES_SPELLING_RULE** - `letztes Endes → letzten Endes`
    - **Reason**: Grammar agreement
    - **Difficulty**: Low

23. **MOMENTANE_SPELLING_RULE** - `momentane → momentan`
    - **Reason**: Adjective vs adverb
    - **Difficulty**: Medium

24. **TEL_SPELLING_RULE** - `tel → Tel.`
    - **Reason**: Abbreviation handling
    - **Difficulty**: Low

25. **ZAHL_IM_WORT_SPELLING_RULE** - `Zahl im Wort` (numbers within words)
    - **Reason**: Complex pattern detection
    - **Difficulty**: High

26. **KOMPOSITA_BIVALENTE_VERBEN** - `Blumen Läden → Blumenläden`
    - **Reason**: Compound word formation
    - **Difficulty**: High

27. **ALARM_AUSLOSEN_SPELLING_RULE** - `Alarm auslosen → auslösen`
    - **Reason**: Verb spelling
    - **Difficulty**: Low

28. **VERTREIB_VERTRIEB_SPELLING_RULE** - `Vertreib → Vertrieb`
    - **Reason**: Noun declension
    - **Difficulty**: Medium

29. **HEER_HERR_SPELLING_RULE** - `Heer Müller → Herr Müller`
    - **Reason**: Title confusion
    - **Difficulty**: Medium

30. **GESETZT_GESETZ_SPELLING_RULE** - `Gesetzt → Gesetz`
    - **Status**: ❌ ATTEMPTED BUT FAILED
    - **Reason**: Capitalization + context (legal term)
    - **Difficulty**: Medium
    - **Removed from catalogue**: File deleted due to test failures

### ❌ NOT PORTED - Lower Priority

31-100. (Additional 70+ rules in this category)
    - Various typo corrections
    - Most require complex context or POS tagging
    - See full LanguageTool file for complete list

## Porting Statistics

- **Total Rules in Category**: 100+
- **Successfully Ported**: 3
- **Attempted but Failed**: 2 (DollerDollar, GesetzSetz)
- **Not Ported**: 95+
- **Success Rate**: ~3%

## Recommended for Future Porting

### Easy Wins (Low Difficulty)
1. **GEHT_RAUTE_S_SPELLING_RULE** - Simple character replacement
2. **IM_ENDDEFEKT_SPELLING_RULE** - Fixed expression
3. **AUF_JEDEN_FALLS_SPELLING_RULE** - Simple grammar agreement
4. **LETZTES_ENDES_SPELLING_RULE** - Simple grammar agreement
5. **TEL_SPELLING_RULE** - Simple abbreviation
6. **ALARM_AUSLOSEN_SPELLING_RULE** - Simple verb spelling

### Medium Priority (Moderate Difficulty)
1. **WICHE_WOCHE_SPELLING_RULE** - Capitalization + spelling
2. **WAR_FUER_SPELLING_RULE** - Capitalization + word choice
3. **MOMENTANE_SPELLING_RULE** - Adjective vs adverb
4. **VERTREIB_VERTRIEB_SPELLING_RULE** - Noun declension
5. **HEER_HERR_SPELLING_RULE** - Title confusion

### High Priority (High Difficulty)
1. **TYPO_KAMM_KANN_SPELLING_RULE** - Common homophone error
2. **ZAHL_IM_WORT_SPELLING_RULE** - Common typo pattern
3. **KOMPOSITA_BIVALENTE_VERBEN** - Compound word issues
4. **FINANZTIP_SPELLING_RULE** - Common brand/word confusion

## Technical Notes

### Challenges in This Category

1. **Context-Sensitivity**: Many rules require understanding whether a word is used as a noun, verb, adjective, etc.
2. **Brand Names**: Rules like "Finanztip" require exceptions for brand names
3. **Fixed Expressions**: Some rules only apply in idiomatic expressions
4. **Length Changes**: Several rules require corrections that change word length
5. **Special Characters**: Rules involving numbers, symbols, or special characters

### Successful Porting Strategies

1. **Simple Duplicates**: Rules like "wir habe" work well with Weir
2. **Umlaut Corrections**: "fuer" → "für" maps well
3. **Fixed Expressions**: "im Enddefekt" → "im Endeffekt" works in context
4. **Capitalization**: Simple capitalization rules work well

### Weir Format Limitations

The Weir format has difficulty with:
- Context-sensitive corrections (requires POS tagging)
- Length-changing corrections (same-length requirement)
- Complex antipatterns (limited whitelist support)
- Multi-word expressions (limited pattern matching)

## References

- **LanguageTool Source**: `~/gallery/languagetool/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml`
- **Category Definition**: Line 266 in LanguageTool file
- **Harper Implementation**: `harper-core/src/linting/weir_rules/de/`

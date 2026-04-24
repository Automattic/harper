# CONFUSED_WORDS - Leicht zu verwechselnde Wörter (Easily Confused Words)

## Overview
This category contains rules for commonly confused German words and homophones.

## Ported Rules (0/200+)

### ❌ NO RULES PORTED YET

This category has not been targeted for porting yet. Many rules here require sophisticated context understanding that is difficult to achieve with the Weir format.

## Not Ported Rules (200+)

### ❌ NOT PORTED - High Priority (Common Errors)

1. **BIST_BIS** - `bist vs. bis`
   - **Reason**: Homophone confusion, requires POS context
   - **Example**: "Ich bist du" → "Ich bin du" / "bis morgen"
   - **Difficulty**: High

2. **ET_ER** - `et vs. er/es`
   - **Reason**: Context-sensitive pronoun/noun confusion
   - **Difficulty**: High

3. **MICH_MIR** - `mich vs. mir` (accusative vs dative)
   - **Reason**: Case system understanding required
   - **Example**: "Ich sehe mich" vs. "Er hilft mir"
   - **Difficulty**: Very High

4. **DICH_DIR** - `dich vs. dir` (accusative vs dative)
   - **Reason**: Case system understanding required
   - **Difficulty**: Very High

5. **SEHR_SEHT** - `sehr vs. seht`
   - **Reason**: Adverb vs verb form confusion
   - **Example**: "sehr gut" vs. "ihr seht"
   - **Difficulty**: High

6. **WEISST_WEIST** - `weißt vs. weist`
   - **Reason**: Verb conjugation confusion
   - **Example**: "du weißt" vs. "er weist"
   - **Difficulty**: High

7. **BISSEN_BISSCHEN** - `bissen vs. bisschen`
   - **Reason**: Noun vs adverb confusion
   - **Example**: "ein Bissen Brot" vs. "ein bisschen"
   - **Difficulty**: Medium

8. **GRUNDE** - `Grunde vs. Gründe`
   - **Reason**: Plural formation
   - **Example**: "aus diesem Grunde" vs. "vieler Gründe"
   - **Difficulty**: Medium

### ❌ NOT PORTED - Medium Priority

9. **AUFRITT_AUFTRITT** - `Aufritt vs. Auftritt`
   - **Reason**: Spelling variant
   - **Difficulty**: Low

10. **FIRME_FIRMA** - `Firme vs. Firma`
    - **Reason**: Regional variant vs standard
    - **Difficulty**: Low

11. **BESCHWERE_BESCHWERDE** - `Beschwere vs. Beschwerde`
    - **Reason**: Verb vs noun confusion
    - **Difficulty**: Medium

12. **ICH_MICH** - `ich vs. mich`
    - **Reason**: Subject vs object pronouns
    - **Difficulty**: Very High

13. **INTER_UNTER** - `inter vs. unter`
    - **Reason**: Prefix confusion
    - **Difficulty**: Low

14. **MINDESTEN_MINDESTENS** - `mindesten vs. mindestens`
    - **Reason**: Adjective vs adverb
    - **Difficulty**: Medium

15. **WAL_WAHL** - `Wal vs. Wahl`
    - **Reason**: Common typo
    - **Difficulty**: Low

16. **PARTIE_VS_PARTEI** - `Partie vs. Partei`
    - **Reason**: False friends
    - **Difficulty**: Medium

17. **BLEIB_BLIEB** - `bleib vs. blieb`
    - **Reason**: Verb tense confusion
    - **Difficulty**: High

18. **METERN_METREN** - `Metern vs. Metren`
    - **Reason**: Genitive vs plural
    - **Difficulty**: Medium

19. **HAKEN_HACKEN** - `Hacken vs. Haken`
    - **Reason**: False friends
    - **Difficulty**: Medium

20. **LAUT_LAUF** - `laut vs. lauf`
    - **Reason**: Preposition vs noun/verb
    - **Difficulty**: Medium

21. **LIEBE_LIEBER** - `liebe vs. lieber`
    - **Reason**: Adjective vs adverb
    - **Difficulty**: Medium

22. **GLUCK** - `Gluck vs. Glück`
    - **Reason**: Umlaut correction
    - **Difficulty**: Low

23. **HOHL_HOL** - `hohl vs. hol`
    - **Reason**: Adjective vs verb form
    - **Difficulty**: Medium

24. **UND_UNS** - `und vs. uns`
    - **Reason**: Conjunction vs pronoun
    - **Difficulty**: High

25. **ALL_ALLE** - `all vs. alle`
    - **Reason**: Indefinite pronoun forms
    - **Difficulty**: High

26. **WIRD_WIR** - `Wird vs. Wir sind`
    - **Reason**: Word order confusion
    - **Difficulty**: High

27. **LEIDE_LEIDER** - `leide vs. leider`
    - **Reason**: Verb vs adverb
    - **Difficulty**: Medium

28. **BISS_BIS** - `biss vs. bis`
    - **Reason**: Noun vs preposition
    - **Difficulty**: Medium

29. **FASST_FAST** - `fasst vs. fast`
    - **Reason**: Verb vs adverb
    - **Difficulty**: Medium

30. **SCHON_SCHÖN** - `schonen vs. schönen`
    - **Reason**: Verb vs adjective
    - **Difficulty**: Medium

### ❌ NOT PORTED - Lower Priority

31-200. (Additional 170+ rules)
    - Various word confusion rules
    - Most require case system understanding
    - Many require verb conjugation knowledge
    - See full LanguageTool file for complete list

## Porting Statistics

- **Total Rules in Category**: 200+
- **Successfully Ported**: 0
- **Not Ported**: 200+
- **Success Rate**: 0%

## Challenges in Porting This Category

### Major Obstacles

1. **Case System**: Most rules require understanding nominative, accusative, dative, genitive
2. **Verb Conjugation**: Many rules distinguish between verb forms (ich/du/er/sie/es)
3. **Part of Speech**: Requires distinguishing nouns, verbs, adjectives, adverbs, pronouns
4. **Context Sensitivity**: Same word can be correct in different contexts
5. **Sentence Position**: Word order often determines correctness

### Why Weir Format Struggles

- **No POS Tagging**: Weir cannot identify parts of speech
- **No Morphology**: Cannot understand verb conjugation or noun declension
- **Limited Context**: Cannot analyze sentence structure
- **No Case System**: Cannot handle German case distinctions
- **Simple Patterns**: Works best with fixed word patterns, not grammar rules

## Recommended for Future Porting

### Low-Hanging Fruit (Simple Spelling)

1. **GLUCK** - `Gluck → Glück` (simple umlaut)
2. **AUFRITT_AUFTRITT** - `Aufritt → Auftritt` (simple spelling)
3. **WAL_WAHL** - `Wal → Wahl` (simple spelling)
4. **GRUNDE** - `Grunde → Gründe` (simple plural)

### Medium Difficulty (Context-Required)

1. **MINDESTEN_MINDESTENS** - Can use context clues
2. **LEIDE_LEIDER** - Some contexts work
3. **LIEBE_LIEBER** - Limited context handling
4. **BISSEN_BISSCHEN** - Some patterns detectable

### High Value (Common Errors)

1. **MICH_MIR** - Very common error, but very difficult
2. **DICH_DIR** - Very common error, but very difficult
3. **SEHR_SEHT** - Common error, difficult
4. **WEISST_WEIST** - Common error, difficult
5. **BIST_BIS** - Common error, very difficult

## Implementation Strategies

### Strategy 1: Pattern-Based (Limited Success)

Focus on fixed expressions where context is predictable:
- "aus diesem Grunde" (always singular)
- "ein bisschen" (always adverb)
- "und schon" (fixed expression)

### Strategy 2: Position-Based

Use sentence position to guess correctness:
- First word: likely subject
- After preposition: likely object
- After verb: likely object

### Strategy 3: Frequency-Based

Prioritize most common error patterns and accept false positives

## Realistic Assessment

**Honest Assessment**: This category is poorly suited for Weir format. Most rules require:

1. Full grammatical analysis
2. POS tagging
3. Case system understanding
4. Verb conjugation tables
5. Semantic analysis

**Recommendation**: Consider implementing a separate grammar checker module for these rules, or accept that Harper will focus on spelling/typo rules rather than grammatical confusion rules.

## Alternative Approaches

1. **Statistical Models**: Use n-gram frequency to detect unlikely word combinations
2. **Pattern Libraries**: Build libraries of common incorrect patterns
3. **Hybrid Approach**: Use Weir for simple cases, integrate with external grammar checker for complex ones
4. **Accept Limitations**: Focus on what Weir does well (spelling, duplicates) and accept grammatical confusion is out of scope

## References

- **LanguageTool Source**: `~/gallery/languagetool/languagetool-language-modules/de/src/main/resources/org/languagetool/rules/de/grammar.xml`
- **Category Definition**: `<category id="CONFUSED_WORDS">` in LanguageTool
- **Linguistic Background**: German case system, verb conjugation, POS tagging

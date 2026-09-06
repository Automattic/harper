# Adjective Declension Test

This test focuses on adjective declension forms, particularly testing words like "unbekannt" and "bekannt" that should generate inflected forms.

## Test Sentence

"An den Wänden hingen Gemälde von bekannten und unbekannten Künstlern"

## Expected Results

- All words should be recognized by Harper
- "unbekannten" should be recognized as a valid inflected form of "unbekannt"
- "bekannten" should be recognized as a valid inflected form of "bekannt"
- No unknown words should be reported

## Words to Test Individually

- unbekannt (base adjective)
- unbekannten (dative plural/masculine genitive singular form)
- bekannt (base adjective)
- bekannten (dative plural/masculine genitive singular form)

## Verification Commands

```bash
# Test the full sentence
just language-test german "An den Wänden hingen Gemälde von bekannten und unbekannten Künstlern"

# Test individual words
just language-meta german "unbekannt"
just language-meta german "unbekannten"
just language-meta german "bekannt"
just language-meta german "bekannten"
```

## Expected Metadata

- unbekannt: should show 🎨 Adjective
- unbekannten: should be found in dictionary (generated form)
- bekannt: should show 🎨 Adjective  
- bekannten: should be found in dictionary (generated form)
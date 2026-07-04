# German Noun Capitalization Verification Tests

## Test Cases for Noun vs Verb/Adjective/Adverb Recognition

This file contains test sentences to verify that Harper correctly identifies:
- Nouns (should be capitalized)
- Verbs (should NOT be capitalized in middle of sentence)
- Adjectives (should NOT be capitalized in middle of sentence)
- Adverbs (should NOT be capitalized in middle of sentence)
- Past participles (should NOT be capitalized in middle of sentence)

## Critical Test Sentences

### Sentence 1: The User's Original Example
Die Mondlandung ist wieder fehlgeschlagen

**Expected behavior:**
- "Mondlandung" should be recognized as a noun (FEMININE)
- "ist" should be recognized as a verb (3rd person singular of "sein")
- "wieder" should be recognized as an adverb
- "fehlgeschlagen" should be recognized as a past participle (verb form)

### Sentence 2: Verb Conjugation
Ich schreibe einen Brief und du liest ein Buch

**Expected behavior:**
- "schreibe" should be recognized as a verb (1st person singular)
- "liest" should be recognized as a verb (3rd person singular)
- No false positives for noun capitalization

### Sentence 3: Mixed POS
Die Freiheit ist wichtig für die Menschheit

**Expected behavior:**
- "Freiheit" should be recognized as a noun (FEMININE)
- "ist" should be recognized as a verb
- "wichtig" should be recognized as an adjective
- "Menschheit" should be recognized as a noun (FEMININE)

### Sentence 4: Adverb Test
Er läuft schnell und spricht langsam

**Expected behavior:**
- "läuft" should be recognized as a verb
- "schnell" should be recognized as an adverb
- "spricht" should be recognized as a verb
- "langsam" should be recognized as an adverb

### Sentence 5: Past Participle
Das Projekt ist gescheitert und wurde abgebrochen

**Expected behavior:**
- "Projekt" should be recognized as a noun (NEUTER)
- "ist" should be recognized as a verb
- "gescheitert" should be recognized as a past participle
- "wurde" should be recognized as a verb
- "abgebrochen" should be recognized as a past participle

### Sentence 6: Verb Forms
Die Forschung hat Ergebnisse erzielt

**Expected behavior:**
- "Forschung" should be recognized as a noun (FEMININE)
- "hat" should be recognized as a verb (past participle of haben)
- "Ergebnisse" should be recognized as a noun (PLURAL)
- "erzielt" should be recognized as a verb (past participle of erzielen)

## Common Problem Areas

### False Positives to Avoid
- "schreibe" (verb) should NOT be flagged as a noun
- "fehlgeschlagen" (past participle) should NOT be flagged as a noun
- "ist" (verb) should NOT be flagged as a noun
- "wieder" (adverb) should NOT be flagged as an adjective

### True Positives to Catch
- "Mondlandung" (noun) SHOULD be flagged if lowercase
- "Freiheit" (noun) SHOULD be flagged if lowercase
- "Menschheit" (noun) SHOULD be flagged if lowercase
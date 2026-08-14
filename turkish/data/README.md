# turkish/data

## Sözlük (faz 4)

`wordlist-tr.txt` — GhostEdit `zemberek-bridge/data/extra-dictionary-tr.txt`
kopyası (~64k satır). Derlemede
`harper-core/src/spell/turkish_dictionary.rs` `include_str!` ile gömer.

Kaynak notu: Zemberek ek sözlüğü; affix genişletmesi yok. `kelime` var,
`kelme` yok. Boş `DialectFlags` = tüm İngilizce diyalekt bayraklarında geçerli
(SpellCheck’in diyalekt filtresi için).

## POS (faz 5 — repo dışında eğitim)

`Document::new_lexicon` / `parse_lexicon_only` İngilizce Brill/Burn’ü atlar.
UD Türkçe CoNLL-U + `harper-pos-utils --features training` ile model üretmek
ayrı bir iştir; çıktı dosyaları buraya konur, `harper-brill` İngilizce
singleton’ı kırılmadan ikinci model eklenemez.

Şimdilik: `new_basic_tokenize` ve lexicon parse. Özne-yüklem kuralları yok.

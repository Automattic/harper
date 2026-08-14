# Geliştirme planı — Harper Türkçe

Tarih: 2026-08-14  
Kaynak: [`GELISTIRME-ANALIZI.md`](./GELISTIRME-ANALIZI.md)  
Repo: yalnız `harper-fork`. GhostEdit = okuma kaynağı.

Harper ajan politikası: her faz **küçük, tek konulu** değişiklik. `Dialect::Turkish` yok.

---

## Hedef (ürün)

Türkçe düz metinde, bulutsuz:

1. Kalıp hataları (yazım birleşikliği, gereksiz sözcük, bağlaç/soru eki)
2. İngilizce SpellCheck’in metni bozmaması
3. İleride genel yazım (sözlük) ve dilbilgisi (POS)

Şimdilik UI yok; `harper-cli` + birim test + isteğe bağlı WASM script.

---

## Yapılmayacaklar (tüm fazlar)

- GhostEdit / Electron / Java Zemberek / ONNX BERT / LLM taşımak
- `Dialect` veya WASM `Language`’e Türkçe eklemek
- İngilizce `new_curated` varsayılanında TR kurallarını tekrar açmak
- Homograph’ları (`kar`, `hala`, `bende`) bağlamsız genişletmek
- POS eğitimi bitmeden özne-yüklem kuralı

---

## Faz 0 — Profil iskeleti

**Durum: yapıldı (commit edilmedi)**

- `LintGroup::new_turkish_profile`
- EN `default_config.json`: `TurkishUsage` / `TurkishRedundancy` = `false`

**Bitti sayılır:** TR metinde redundancy çalışır; `extention` SpellCheck üretmez.

---

## Faz 1 — Kuralları sağlamlaştır (şimdi)

Amaç: mevcut iki linteri üretim barına çek; yeni kural yok.

| Dilim | İş | Bitti |
|---|---|---|
| 1a | Her linterde ≥15 test: TP, FP, TN, İ/ı, noktalama | **yapıldı**. `asıl esas` düzeltildi (düz `any_of` + kelime anahtarı). `gittimi` QuestionParticle’da TP. |
| 1b | Homograph FP: `kar yağdı`, `evde kaldım`, `hala geldi` (bağlama göre) — ya kural daralt veya “bilinen FP” test + yorum | **yapıldı:** `kar`/`hala`/`hakim`/`adet`/`alem`/`asık`/`bende`/`sende` listeden çıktı |
| 1c | `copy_casing` / `replace_with_match_case` İ/ı: `İLK ÖNCE` → `ÖNCE` (İ→i birleşik nokta yok) | **yapıldı:** `turkish_match_case` (çekirdek `copy_casing` değişmedi). `Yaşlı ihtiyar`→`İhtiyar`, `BİRŞEY`→`BİR ŞEY` |
| 1d | `birçoğu` no-op GhostEdit’ten **eklenmez** | **yapıldı** (eklenmedi) |

**Harvest:** yok (fork zaten kaynak).  
**Dosyalar:** `turkish_usage.rs`, `turkish_redundancy.rs` (gerekirse küçük `turkish_case.rs`).

---

## Faz 2 — GhostEdit’ten güvenli kalıplar

Amaç: modelsiz, dar, otomatik düzeltilebilir kurallar.

| Dilim | İş | Bitti |
|---|---|---|
| 2a | `hiç bir` → `hiçbir` (ve gerekirse `hiçbirşey` zaten Usage’da) | **yapıldı:** `REDUNDANT_PHRASES` + testler |
| 2b | Kesmeli bağlaç: `X'de`/`X'da` → `X de`/`X da` yalnızca kesme + de/da | **yapıldı:** `TurkishDeDaApostrophe`. `evde`/`park'ta`/`'den` TN. `Ankara'da` locative risk belgelendi |
| 2c | Soru eki: `geldimi` listesi yerine fiil token + `mi/mı/mu/mü` bitişik | **yapıldı:** `TurkishQuestionParticle`. `gittimi` TP; `yirmi`/`ismi`/`adamı` TN |
| 2d | `ki` bağlacı: yalnızca güvenli kapalı liste (aşırı genel `ki` yok) | **yapıldı:** Usage çiftleri (`demekki`…). TN: `benimki`/`halbuki`/`evdeki`/`belki` |

Kayıt: `lint_group` + `default_config.json` (**EN’de kapalı**, profilde açık).  
`new_turkish_profile` yeni kural adlarını enable eder.

**Harvest:** `turkish-grammar-tagger-checker.ts` (id 5, 22), mappings.

---

## Faz 3 — Profilin dışarıdan kullanımı

Amaç: CLI/WASM TR profili tek çağrı.

| Dilim | İş | Bitti |
|---|---|---|
| 3a | `harper-cli lint --profile turkish` | **yapıldı** |
| 3b | WASM `Linter::new_turkish`; `fill_with_curated` TR’de yok | **yapıldı** |
| 3c | `turkish/README.md` nasıl lint edilir | **yapıldı** |

**Harvest:** yok.

---

## Faz 4 — Sözlük (yazım)

Amaç: genel “bu kelime var mı”; Zemberek süreci yok.

| Dilim | İş | Bitti |
|---|---|---|
| 4a | `extra-dictionary-tr.txt` → `turkish/data/` | **yapıldı** (`wordlist-tr.txt`) |
| 4b | `MutableDictionary` yükleme | **yapıldı** (`turkish_dictionary()`) |
| 4c | TR profilde SpellCheck TR dict ile | **yapıldı** (`ve`/`kelime` TN, `kelme` TP) |
| 4d | EN curated dict TR profilde yok | **yapıldı** (CLI/WASM TR dict) |

**Harvest:** kelime listesi. Affix/Hunspell yığması yok; liste ham.

---

## Faz 5 — POS (büyük, ayrı)

Amaç: özne-yüklem vb. Ancak `Document::parse` dil almadan başlanmaz.

| Dilim | İş |
|---|---|
| 5a | `parse`’a tagger/chunker enjekte veya `skip_pos` | **kısmi:** `Document::new_lexicon` / `new_from_chars_lexicon` |
| 5b | UD CoNLL-U + `harper-pos-utils --features training` | repo dışı |
| 5c | `harper-brill` TR model dosyaları | yapılmadı |
| 5d | POS’lu kurallar | yapılmadı |

**Harvest:** BERT kategori listesi checklist; model değil.

---

## Faz 6 — Masaüstü (isteğe bağlı, core’dan sonra)

`harper-desktop` highlighter + Windows broker. GhostEdit UIA **kopyalanmaz**.
Not: `turkish/DESKTOP.md`. Windows hâlâ `NoopBroker`.

---

## Önerilen sıra (önümüzdeki oturumlar)

```
1a → 1b → 1c → 2a → 2b → 2c → 3a → 3b → 4…
```

Bir oturum = bir dilim. Faz 5’e Faz 4 oturmadan girilmez.

---

## Her dilim sonunda

- `PROJECT-MAP.md` güncelle
- İlgili `cargo test -p harper-core --lib …`
- `just format` veya `rustfmt` (toolchain’de varsa)
- Commit ancak kullanıcı isterse

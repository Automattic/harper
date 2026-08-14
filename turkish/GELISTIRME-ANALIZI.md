# Geliştirme analizi — Harper Türkçe

Tarih: 2026-08-14  
Kapsam: `D:\Projeler\harper-fork` + kaynak arşiv `D:\Projeler\Harper türkçe projesi`  
Kural: kod yalnız fork’ta yazılır; GhostEdit’ten veri/fikir çekilir, klasörler birleşmez.

Görsel özet (Cursor canvas): sohbet yanındaki `harper-gelistirme-analiz.canvas.tsx`.

---

## 1. Harper fork

Türkçe **dil tipi yok**. `Dialect` yalnızca İngilizce bölgeler (`American` … `Indian`, `en-` BCP-47). WASM `Language` işaretleme: `Plain` | `Markdown` | `Typst`. **`Dialect::Turkish` eklenmemeli.**

### Kilit noktalar

| Parça | Durum | TR etkisi |
|---|---|---|
| Tokenizer (`PlainEnglish`, Latin script) | Hazır | ç ğ ı ö ş ü İ, kesme ekleri |
| `Document::parse` | `brill_tagger()` + `burn_chunker()` sabit EN | Her TR belgeye İngilizce POS |
| Sözlük | `harper-core/dictionary.dict` gömülü | TR yükleme yolu yok |
| `SpellCheck` | EN dict + `Dialect` | TR kelime = yazım hatası seli |
| `TurkishUsage` / `TurkishRedundancy` | `LintGroup` kayıtlı | Homograph FP (`kar`→`kâr`) |
| `is_doc_likely_english` | EN sözlük oranı ≥ 0.7 | `isolateEnglish: true` TR’yi silebilir |
| WASM `fill_with_curated` | Unset → curated true | Profil verilmezse SpellCheck açılır |

Önemli dosyalar: `harper-core/src/linting/lint_group/mod.rs`, `document.rs`, `spell/mutable_dictionary.rs`, `language_detection.rs`, `linting/spell_check.rs`, `harper-wasm/src/lib.rs`.

### Mevcut TR kuralları

- `turkish_usage.rs` — ~32–35 tek-kelime çifti (bitişik yazım, de/da, mi, â/î/û). Test: 5 (Harper barı ≥15).
- `turkish_redundancy.rs` — 15 öbek. `turkish_lower` / `turkish_word` (ASCII case-fold aşılıyor). Test: 6.
- Risk: `bende`/`sende`, `kar`, `hala` bağlamsız; öneri uygulamasında `copy_casing` İ/ı zayıf.

### Bu turda yazılan kod

`LintGroup::new_turkish_profile(dictionary, dialect)`:

- `new_curated` + `config.clear()`
- Yalnız `TurkishUsage` + `TurkishRedundancy` açık
- `SpellCheck` kapalı

İngilizce `default_config.json` içinde bu iki kural artık varsayılan **kapalı**.

Testler (geçti): `turkish_profile_flags_redundancy_without_spellcheck`, `turkish_profile_ignores_english_misspelling`.

---

## 2. GhostEdit arşivi (çekilecek kaynak)

`language === 'tr'` iken Harper.js / nspell **atlanır**.

Katman sırası (`dictionary-checker.ts`):

1. `getTurkishUsageIssues` (sync) — `TURKISH_USAGE_MAPPINGS`; **phrase map TS’de kullanılmıyor**
2. Zemberek (async) — yazım önerisi
3. GECTurk BERT ONNX (async) — 25 kategori; otomatik düzeltme yalnızca 2 dar kural
4. Çakışma: usage > Zemberek > BERT (`mergeIssues`)

### Çek

| Öğe | Nasıl |
|---|---|
| Usage + phrase tabloları | Fork zaten kaynak gerçeği; TS phrase ölü kod |
| `X'de` / `X'da` bağlaç (BERT id 5, modelsiz) | ExprLinter: `Ayşe'de` → `Ayşe de`; locative `evde` kalmalı |
| `hiç bir` → `hiçbir` | Tek kalıp (BERT id 22’nin tek güvenli çifti) |
| `extra-dictionary-tr.txt` (~64k satır) | `turkish/data/` adayı; Hunspell `.dict` değil |
| 25 BERT kategori başlığı | Yıllık kural checklist; model gömülmez |
| Overlay / caret | `harper-desktop`, core değil |

### Çekme

Java Zemberek, ONNX BERT (~440MB), Electron, clipboard-paste, Bonsai/Ollama, unigram sıralayıcı (Shire sapması).

---

## 3. Uygulama sırası

1. **(yapıldı)** `new_turkish_profile` + EN curated’da TR kurallarını kapat
2. Usage/Redundancy testlerini ≥15; homograph FP testleri
3. Kesmeli de/da + `hiç bir`→`hiçbir`
4. `geldimi` yerine genel soru eki deseni
5. TR sözlük + SpellCheck (sonra)
6. `Document::parse` dil parametresi + UD POS (sonra)

---

## 4. Git durumu (analiz anı)

`HEAD` = `origin/master` (`52c6aac`). Türkçe iş **commit edilmemiş**: 3 izlenen dosya + untracked kural/modül/`turkish/`.

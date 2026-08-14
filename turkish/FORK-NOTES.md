# Harper — Türkçe Fork Notları

[Harper](https://github.com/automattic/harper)'ın (Automattic, Apache-2.0) Türkçe
dilbilgisi/yazım denetimi desteği eklenmiş fork'u. Motor kodunun klasör yapısı
orijinal Harper ile aynı bırakıldı; `origin` remote'u orijinal Harper reposuna
işaret eder. Bu dosya fork’a özel eklemeleri belgeler.

Türkçe çalışma alanı: `turkish/` (bu dosyanın bulunduğu ağaç). Kurallar
`harper-core/src/linting/` altında kalır.

## Neden bu proje var

Türkçe için Grammarly benzeri, tamamen yerel (bulut gerektirmeyen) bir yazım/
dilbilgisi denetleyicisi kurmak amacıyla başladı. Önce ayrı bir Electron
uygulaması (Zemberek + BERT tabanlı GECTurk etiketleyici + elle yazılmış
kurallar) üzerinden geliştirildi, ardından Harper'ın mimarisinin Türkçe için
de genişletilebilir olduğu keşfedildi ve odak buraya kaydı — **artık asıl
motor bu proje.**

## Kurulum (Windows, MSVC gerekmez)

- Rust: `rustup-init.exe --default-host x86_64-pc-windows-gnu --profile minimal`
- MinGW-w64: `winget install BrechtSanders.WinLibs.POSIX.UCRT` (dlltool/gcc/ld sağlar)
- WASM derlemesi için: [wasm-pack](https://github.com/wasm-bindgen/wasm-pack) (portable binary yeterli)

```bash
# Test
cargo test -p harper-core --lib turkish

# WASM (Node.js hedefi)
cd harper-wasm
wasm-pack build --target nodejs --out-dir pkg-node

# WASM uçtan uca (paket derlendikten sonra)
node turkish/scripts/test_turkish_full.mjs
```

## Türkçe için yapılan değişiklikler

### 1. Tokenizer
`is_english_lingual()` Latin script ile Türkçe harfleri kapsar. `lex_plural_digit`
yalnızca `0s` / `1's` gibi **rakam** kalıplarına bakmalı; aksi halde `asıl`
`as`+`ıl` diye bölünüyordu (Issue #774 geniş `alphanumeric` kontrolü).

### 2. Büyük/küçük harf eşleştirme — ASCII-özel sorunu bulundu ve aşıldı
Harper'ın yerleşik `SequenceExpr::fixed_phrase`/`any_capitalization_of`
karşılaştırması (`char_string.rs`, `eq_ignore_ascii_case` ailesi) SADECE
ASCII A-Z/a-z çiftlerini eşleştiriyor — Türkçe'nin İ/ı, Ö/ö, Ü/ü, Ş/ş, Ğ/ğ,
Ç/ç büyük/küçük harf çiftlerini (özellikle "noktalı-noktasız I" sorununu)
tanımıyor. Çekirdek koda dokunmadan, `turkish_redundancy.rs`'deki
`turkish_lower()` + `turkish_word()` closure tabanlı eşleştiriciyle
(Harper'ın `SingleTokenPattern` blanket-impl'i sayesinde) bu sorun izole
şekilde çözüldü.

### 3. Yeni linter modülleri
- **`harper-core/src/linting/turkish_usage.rs`**: bitişik/ayrı yazım ve
  açık de/da birleşikleri (`onunda`→`onun da`), kapalı `mi` listesi.
  Homograph circumflex (`kar`/`hala`/`hakim`) ve locative `bende`/`sende`
  bağlamsız düzeltilmez. `LintKind::Usage`.
- **`harper-core/src/linting/turkish_redundancy.rs`**: 15 çok-kelimelik
  "gereksiz sözcük kullanımı" kalıbı (kısa özet→özet, geri iade→iade, hür
  özgür→özgür, vb.). `LintKind::Redundancy`.
- İkisi de `ExprLinter` trait'i ile yazıldı (POS etiketleyici gerektirmiyor),
  `LintGroup::new_curated()`'a `insert_expr_rule!` ile kayıtlı, ve
  `default_config.json`'da İngilizce curated profilde varsayılan **kapalı**;
  `LintGroup::new_turkish_profile` bunları açar.

- 47+ Türkçe kalıp; `cargo test -p harper-core --lib turkish`
- Harper config self-check geçiyor.
- İngilizce curated profilde TR kuralları kapalı.
- CLI: `harper-cli lint --profile turkish`
- WASM: `Linter.new_turkish()` (İngilizce SpellCheck yok; TR sözlük var)

## Bilinmeyen/yapılmamış

- **POS etiketleyici** (`harper-brill`): UD eğitimi repo dışı. Şimdilik
  `Document::new_lexicon` İngilizce Brill/Burn’ü atlar.
- **Sözlük:** `turkish/data/wordlist-tr.txt` gömülü; affix yok, ham liste.
- **Masaüstü:** `turkish/DESKTOP.md`; Windows `NoopBroker`.

## İlgili geçmiş

Önceki Electron denemesi (Zemberek + BERT + GhostEdit forku)
`D:\Projeler\Harper türkçe projesi` — arşiv; yeni Türkçe kural buraya
yazılmaz (`TURKCE-UYARLAMA-PLANI.md`, `ARSIV.md`).

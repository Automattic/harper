# Türkçe dil desteği — çalışma alanı

Bu klasör **Harper fork’una Türkçe eklerken** kullanılan notlar, elle
çalıştırılan doğrulama scriptleri ve ileride sözlük/POS veri dosyaları içindir.

**Motor kodu burada durmaz.** Kurallar ve çekirdek hâlâ Harper’ın kendi
ağacında kalır; aksi halde upstream ile birleştirmek zorlaşır.

| Ne | Nerede |
|---|---|
| Kullanım / gereksizlik kuralları | `harper-core/src/linting/turkish_usage.rs`, `turkish_redundancy.rs` |
| Kural kaydı | `harper-core/src/linting/lint_group/mod.rs`, `default_config.json` |
| Birim testleri | aynı `.rs` dosyalarının `#[cfg(test)]` bölümleri |
| Cargo örnekleri (keşif) | `harper-core/examples/turkish_*.rs` |
| Mimari harita | kök `PROJECT-MAP.md` |
| Fork notları | `turkish/FORK-NOTES.md` |
| Geliştirme analizi (2026-08-14) | `turkish/GELISTIRME-ANALIZI.md` |
| Geliştirme planı | `turkish/GELISTIRME-PLANI.md` |
| **Tüm kuralların referans listesi** | `turkish/KURALLAR.md` |
| WASM uçtan uca scriptleri | `turkish/scripts/` |
| Sözlük | `turkish/data/wordlist-tr.txt` (`turkish_dictionary()`) |
| Masaüstü notu | `turkish/DESKTOP.md` |

## Nasıl lint edilir

İngilizce diyalekt **değildir**. Profil ayrıdır:

```text
cargo run -p harper-cli --release -- lint --profile turkish "Bana birşey söyle."
```

WASM (paket derlendikten sonra):

```js
const linter = Linter.new_turkish();
linter.lint(text, Language.Plain, false, undefined, true, false);
```

`harper.js`: `new LocalLinter({ binary, turkish: true })`.

`fill_with_curated` Türkçe linter’da çağrılmaz (İngilizce SpellCheck geri açılmaz).


## Kaynak arşiv (GhostEdit)

`D:\Projeler\Harper türkçe projesi` — buraya yazılmaz. İşe yarar şeyler
Harper’ın modeline uyarlanarak çekilir (kopyala-yapıştır ürün birleştirme yok).

Çekmeye değer:

- Kalıp listeleri (`turkish-usage-mappings.ts`) — Usage/Redundancy’ye zaten kısmen taşındı
- Ek kelime listesi (`zemberek-bridge/data/extra-dictionary-tr.txt`) — ileride `turkish/data/`
- GECTurk kural kategorileri — işaret/düzeltme politikası referansı (BERT modeli Harper’a gömülmez)
- UI Automation / overlay fikirleri — `harper-desktop` highlighter ile karşılaştırılır, Electron taşınmaz

Taşınmaz: Java Zemberek süreci, ONNX BERT (~440MB), Electron UI, Bonsai/Ollama katmanı.

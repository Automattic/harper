# Geliştirme planı — Harper Türkçe

Son güncelleme: 2026-08-14  
Kaynak: [`GELISTIRME-ANALIZI.md`](./GELISTIRME-ANALIZI.md)  
Repo: yalnız `harper-fork`. GhostEdit = okuma kaynağı.

Harper ajan politikası: her faz **küçük, tek konulu** değişiklik. `Dialect::Turkish` yok.

---

## Yerel Yollar (Windows)

| Klasör | Amaç |
|--------|------|
| `D:\Projeler\harper-fork` | Harper fork'u — kod burada yazılır |
| `D:\Projeler\Harper türkçe projesi` | GhostEdit arşivi — sadece okuma/referans |

**Senkronizasyon:**
```powershell
cd "D:\Projeler\harper-fork"
git pull origin turkish-support
```

---

## Hedef (ürün)

Türkçe düz metinde, bulutsuz:

1. ✅ Kalıp hataları (yazım birleşikliği, gereksiz sözcük, bağlaç/soru eki)
2. ✅ İngilizce SpellCheck'in metni bozmaması
3. ✅ Türkçe yazım denetimi (sözlük)
4. 🔄 İleride dilbilgisi (POS)

Şimdilik UI yok; `harper-cli` + birim test + isteğe bağlı WASM script.

---

## Yapılmayacaklar (tüm fazlar)

- GhostEdit / Electron / Java Zemberek / ONNX BERT / LLM taşımak
- `Dialect` veya WASM `Language`'e Türkçe eklemek
- İngilizce `new_curated` varsayılanında TR kurallarını tekrar açmak
- Homograph'ları (`kar`, `hala`, `bende`) bağlamsız genişletmek
- POS eğitimi bitmeden özne-yüklem kuralı

---

## Faz 0 — Profil iskeleti ✅

**Durum: tamamlandı**

- `LintGroup::new_turkish_profile`
- EN `default_config.json`: `TurkishUsage` / `TurkishRedundancy` = `false`

**Bitti:** TR metinde redundancy çalışır; `extention` SpellCheck üretmez.

---

## Faz 1 — Kuralları sağlamlaştır ✅

Amaç: mevcut iki linteri üretim barına çek; yeni kural yok.

| Dilim | İş | Bitti |
|---|---|---|
| 1a | Her linterde ≥15 test: TP, FP, TN, İ/ı, noktalama | ✅ `asıl esas` düzeltildi. `gittimi` QuestionParticle'da TP. |
| 1b | Homograph FP: `kar yağdı`, `evde kaldım`, `hala geldi` | ✅ `kar`/`hala`/`hakim`/`adet`/`alem`/`asık`/`bende`/`sende` listeden çıktı |
| 1c | `copy_casing` İ/ı: `İLK ÖNCE` → `ÖNCE` | ✅ `turkish_match_case` eklendi |
| 1d | `birçoğu` no-op GhostEdit'ten **eklenmez** | ✅ (eklenmedi) |

**Dosyalar:** `turkish_usage.rs`, `turkish_redundancy.rs`

---

## Faz 2 — GhostEdit'ten güvenli kalıplar ✅

Amaç: modelsiz, dar, otomatik düzeltilebilir kurallar.

| Dilim | İş | Bitti |
|---|---|---|
| 2a | `hiç bir` → `hiçbir` | ✅ `REDUNDANT_PHRASES` + testler |
| 2b | Kesmeli bağlaç: `X'de`/`X'da` → `X de`/`X da` | ✅ `TurkishDeDaApostrophe` |
| 2c | Soru eki: fiil token + `mi/mı/mu/mü` bitişik | ✅ `TurkishQuestionParticle` |
| 2d | `ki` bağlacı: güvenli kapalı liste | ✅ Usage çiftleri (`demekki`…) |

**Harvest:** `turkish-grammar-tagger-checker.ts` (id 5, 22), mappings.

---

## Faz 3 — Profilin dışarıdan kullanımı ✅

Amaç: CLI/WASM TR profili tek çağrı.

| Dilim | İş | Bitti |
|---|---|---|
| 3a | `harper-cli lint --profile turkish` | ✅ |
| 3b | WASM `Linter::new_turkish` | ✅ |
| 3c | `turkish/README.md` nasıl lint edilir | ✅ |

---

## Faz 4 — Sözlük (yazım) ✅

Amaç: genel "bu kelime var mı"; Zemberek süreci yok.

| Dilim | İş | Bitti |
|---|---|---|
| 4a | `extra-dictionary-tr.txt` → `turkish/data/` | ✅ (`wordlist-tr.txt`, 64k+ kelime) |
| 4b | `MutableDictionary` yükleme | ✅ (`turkish_dictionary()`) |
| 4c | TR profilde SpellCheck TR dict ile | ✅ (`ve`/`kelime`/`da` TN, `kelme` TP) |
| 4d | EN curated dict TR profilde yok | ✅ (CLI/WASM TR dict) |
| 4e | Eksik temel kelimeler | ✅ `da`, `mu`, `mı`, `mü`, `ise` eklendi |

**Harvest:** kelime listesi. Affix/Hunspell yığması yok; liste ham.

---

## Faz 4b — GitHub'dan ek güvenli kalıplar ✅

Amaç: GhostEdit dışında, GitHub'daki açık kaynak Türkçe dilbilgisi
projelerini tara; TDK'yle çelişmeyen, homograph riski taşımayan kalıpları al.

| Dilim | İş | Bitti |
|---|---|---|
| 4b-1 | `Denomas/Turkce-yazim-denetimi` (MIT) taraması | ✅ bkz. `turkish/KURALLAR.md` §2.4 |
| 4b-2 | `TurkishUsage`'a ek bitişik/yazım hatası çiftleri | ✅ 22 yeni çift (`bişey`, `süpriz`, `deyil`, `gelicem`…) |
| 4b-3 | Yeni kural: `TurkishMergeWords` (ayrı→bitişik) | ✅ `bir kaç`→`birkaç`, `vaz geçmek`→`vazgeçmek`, vb. |
| 4b-4 | Yeni kural: `TurkishProperNouns` (özel isim büyük harf) | ✅ `türkiye`→`Türkiye`, `istanbul`→`İstanbul`, vb. — gün/ay adları bilerek **alınmadı** (bağlama bağlı) |
| 4b-5 | Sözlüğe eksik kelimeler | ✅ `birçok`/`vazgeçti`/`teşekkürler`/`gideceğim` vb. |
| 4b-6 | Önceden var olan gizli test hatası düzeltildi | ✅ `lint_descriptions_are_clean` — Türkçe açıklamalardaki örnekler artık backtick içinde |

**Harvest:** `Denomas/Turkce-yazim-denetimi` MIT lisanslı, doğrudan alınabilir.
Reddedilen/ertelenen kısımlar (argo kısaltmalar, deyimler, plaza Türkçesi,
akademik dil sadeleştirme, gün/ay büyük harf) `KURALLAR.md` §2.4 ve §3.2'de
gerekçesiyle listelendi.

---

## Faz 5 — POS (büyük, ayrı) 🔄

Amaç: özne-yüklem vb. Ancak `Document::parse` dil almadan başlanmaz.

| Dilim | İş | Durum |
|---|---|---|
| 5a | `parse`'a tagger/chunker enjekte veya `skip_pos` | **kısmi:** `Document::new_lexicon` |
| 5b | UD CoNLL-U + `harper-pos-utils --features training` | repo dışı |
| 5c | `harper-brill` TR model dosyaları | yapılmadı |
| 5d | POS'lu kurallar | yapılmadı |

**Harvest:** BERT kategori listesi checklist; model değil.

---

## Faz 6 — Masaüstü (isteğe bağlı) 📋

`harper-desktop` highlighter + Windows broker. GhostEdit UIA **kopyalanmaz**.
Not: `turkish/DESKTOP.md`. Windows hâlâ `NoopBroker`.

---

## Mevcut Durum Özeti

```
Faz 0 ✅ → Faz 1 ✅ → Faz 2 ✅ → Faz 3 ✅ → Faz 4 ✅ → Faz 4b ✅ → Faz 5 🔄 → Faz 6 📋
```

**Tamamlanan özellikler:**
- 163 birim test (tümü geçiyor), toplam `harper-core` paketi 6286 test
- 6 Türkçe lint kuralı (Usage, MergeWords, Redundancy, DeDaApostrophe,
  QuestionParticle, ProperNouns)
- 64.000+ kelimelik Türkçe sözlük
- CLI `--profile turkish` desteği
- WASM `Linter::new_turkish()` desteği
- GhostEdit + GitHub (`Denomas/Turkce-yazim-denetimi`, MIT) çapraz kaynaklı
  kural referansı: `turkish/KURALLAR.md`

**Sonraki adımlar:**
- Faz 5: POS eğitimi (opsiyonel, büyük iş)
- Faz 6: Masaüstü Windows desteği (opsiyonel)

---

## Her dilim sonunda

- `PROJECT-MAP.md` güncelle
- İlgili `cargo test -p harper-core --lib …`
- `cargo fmt` (veya `just format`)
- Commit ancak kullanıcı isterse

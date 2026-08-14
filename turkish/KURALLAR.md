# Harper Türkçe — Kural Referansı

Son güncelleme: 2026-08-14  
Kapsam: `harper-fork` içindeki **uygulanmış** kurallar + `GELISTIRME-ANALIZI.md`'de
belgelenmiş GhostEdit arşiv taraması (`D:\Projeler\Harper türkçe projesi`)
sonuçları tek yerde.

> Not: Bu ajan (cloud) yalnızca GitHub'daki `turkish-support` branch'ine erişir;
> yerel `D:\Projeler\...` klasörlerini doğrudan tarayamaz. Buradaki GhostEdit
> bölümü, daha önce yerelde yapılmış ve `GELISTIRME-ANALIZI.md`'ye yazılmış
> analizin özetidir.

---

## 1. Uygulanmış Kurallar (harper-fork)

### 1.1 `TurkishUsage` — bitişik/ayrı yazım + bağlaç hataları

Dosya: `harper-core/src/linting/turkish_usage.rs` · `LintKind::Usage` · varsayılan İngilizce profilde **kapalı**

**a) Bitişik yazılan, ayrı yazılması gereken kalıplar**

| Hatalı | Doğru |
|---|---|
| birşey | bir şey |
| birşeyler | bir şeyler |
| herşey | her şey |
| hiçbirşey | hiçbir şey |
| herkez | herkes |
| yalnış | yanlış |
| yanlız | yalnız |
| malesef | maalesef |
| yada | ya da |
| arasıra | ara sıra |
| bazan | bazen |
| herzaman | her zaman |
| hergün | her gün |
| heryer | her yer |
| heryerde | her yerde |
| birkez | bir kez |
| şuan | şu an |
| şuanda | şu anda |

**b) Circumflex/şapka eksikliği**

| Hatalı | Doğru |
|---|---|
| eger | eğer |
| gercek | gerçek |
| gercekten | gerçekten |

**c) "de/da" bağlacı — açık bitişik biçimler**

| Hatalı | Doğru |
|---|---|
| benimde | benim de |
| seninde | senin de |
| onunda | onun da |
| bizimde | bizim de |
| sizinde | sizin de |
| onuda | onu da |
| bunuda | bunu da |
| şunuda | şunu da |
| kendide | kendi de |
| kendiside | kendisi de |

**d) "ki" bağlacı — yalnızca güvenli kapalı liste**

| Hatalı | Doğru |
|---|---|
| demekki | demek ki |
| öyleki | öyle ki |
| taaki | ta ki |
| yeterki | yeter ki |
| gördümki | gördüm ki |
| dedimki | dedim ki |
| eminki | emin ki |
| açıkki | açık ki |
| yazıkki | yazık ki |
| belliki | belli ki |

**Bilinçli olarak dokunulmayanlar (homograph riski):**
`kar`/`kâr`, `hala`/`hâlâ`, `hakim`/`hâkim`, `adet`/`âdet`, `alem`/`âlem`,
`asık`/`âşık`, `bende`, `sende` (locative olarak geçerli) — bağlam (POS) olmadan
güvenli değiştirilemez.

---

### 1.2 `TurkishRedundancy` — gereksiz sözcük tekrarı

Dosya: `harper-core/src/linting/turkish_redundancy.rs` · `LintKind::Redundancy`

| Hatalı öbek | Öneri |
|---|---|
| kısa özet | özet |
| geri iade | iade |
| ilk önce | önce |
| yeniden tekrar | tekrar |
| asıl esas | asıl |
| eski antika | antika |
| yalnız sadece | sadece |
| yaklaşık tahminen | yaklaşık |
| gizli sır | sır |
| yeni buluş | buluş |
| hür özgür | özgür |
| yaşlı ihtiyar | ihtiyar |
| güç kuvvet | güç |
| karşılıklı diyalog | diyalog |
| ani sürpriz | sürpriz |
| hiç bir | hiçbir |

---

### 1.3 `TurkishDeDaApostrophe` — kesmeli bağlaç

Dosya: `harper-core/src/linting/turkish_de_da_apostrophe.rs` · `LintKind::Usage`

Kalıp: `Özel_isim'de` / `Özel_isim'da` → `Özel_isim de` / `Özel_isim da`
(yalnızca kesme + tam olarak `de`/`da` klitiği).

Örnek: `Ayşe'de geldi.` → `Ayşe de geldi.`

**Dokunulmayan (true negative):**
- Lokatif: `evde`, `odada`, `Park'ta` (kesme + `ta`/`ta` değil, farklı ek)
- Ablatif: `İstanbul'dan`
- İyelik: `Ayşe'nin`
- İngilizce kesme: `We'd rather wait.`
- `-deyim/-deymiş` gibi ek+ek biçimler: `Ayşe'deymiş`

**Bilinen risk:** Özel isim + lokatif de aynı yüzey biçimi kullanır
(`Ankara'da yaşıyorum` → yanlışlıkla `Ankara da yaşıyorum`'a çevrilebilir).
POS etiketleyici olmadan ayrıştırılamaz; şimdilik kabul edilmiş bir risk.

---

### 1.4 `TurkishQuestionParticle` — soru eki

Dosya: `harper-core/src/linting/turkish_question_particle.rs` · `LintKind::Usage`

Kalıp: fiil kökü (biten ekler: `-ecek/-acak/-miş/-mış/-muş/-müş/-yor/-di/-dı/-du/-dü/-ti/-tı/-tu/-tü/-ir/-ır/-ur/-ür/-er/-ar`, veya `var`/`yok`)
+ bitişik `mi/mı/mu/mü` → ayrı yazım.

Örnekler: `gittimi` → `gitti mi`, `varmı` → `var mı`, `gelecekmi` → `gelecek mi`

**Dokunulmayan (true negative — isim gövdeleri):**
`yirmi`, `ismi`, `resmi`, `kamu`, `adamı` — fiil eki örüntüsüne uymadığı için ayrılmaz.

---

### 1.5 Sözlük — `turkish_dictionary()`

Dosya: `harper-core/src/spell/turkish_dictionary.rs` · Kaynak: `turkish/data/wordlist-tr.txt`

- ~64.600 kelime (GhostEdit `extra-dictionary-tr.txt` + eklenen fiil çekimleri)
- İ/ı katlama (`turkish_fold_chars`): `İstanbul` → `istanbul` eşleşmesi için
- Affix/morfoloji **yok** — ham liste. Bu yüzden çekim ekleri elle eklenmek zorunda
  (bkz. Bölüm 3, "Bilinen Kısıtlar").

---

## 2. GhostEdit Arşiv Analizi (yerel tarama özeti)

Kaynak: `turkish/GELISTIRME-ANALIZI.md` (2026-08-14 tarihli, yerel `D:\Projeler\harper-fork`
ortamında yapılmış analiz — bu ajan tarafından tekrar taranamaz, sadece aktarılır).

`D:\Projeler\Harper türkçe projesi` = önceki Electron + Zemberek + BERT denemesi.
**Kod olarak taşınmaz**, yalnızca fikir/veri kaynağı.

### 2.1 Katman sırası (`dictionary-checker.ts`)

1. `getTurkishUsageIssues` (sync) — `TURKISH_USAGE_MAPPINGS` → **zaten Harper'a taşındı** (Bölüm 1.1)
2. Zemberek (async) — yazım önerisi → taşınmadı (Java süreci, taşınmayacak)
3. GECTurk BERT ONNX (async) — 25 kategori, yalnızca 2 dar kural otomatik düzeltiliyordu
4. Çakışma çözümü: usage > Zemberek > BERT

### 2.2 Çekilmesi değerlendirilen / çekilmiş öğeler

| Öğe | Durum |
|---|---|
| Usage + phrase tabloları | ✅ Taşındı (`turkish_usage.rs`) |
| `X'de`/`X'da` bağlaç (BERT id 5) | ✅ Taşındı (`TurkishDeDaApostrophe`) |
| `hiç bir` → `hiçbir` (BERT id 22) | ✅ Taşındı (`TurkishRedundancy`) |
| `extra-dictionary-tr.txt` (~64k satır) | ✅ Taşındı (`turkish/data/wordlist-tr.txt`) |
| Soru eki genel deseni | ✅ Taşındı (`TurkishQuestionParticle`, liste yerine kural) |
| 25 BERT kategori başlığı | 📋 Checklist olarak referans; model gömülmeyecek |
| Overlay/caret (UI Automation) | 📋 `harper-desktop` highlighter ile ayrı değerlendirilecek |

### 2.3 Kesinlikle taşınmayacaklar

- Java Zemberek süreci
- ONNX BERT modeli (~440 MB)
- Electron UI
- Bonsai/Ollama LLM katmanı
- Clipboard-paste mekanizması
- Unigram sıralayıcı ("Shire sapması" olarak not edilmiş)

---

## 3. Bilinen Kısıtlar ve Sonraki Adaylar

### 3.1 Sözlük — eksik çekim kalıpları

Türkçe eklemeli bir dil olduğu için tek bir fiil kökünden yüzlerce çekim türetilebilir.
Şu ana kadar elle eklenenler: bilmek/gelmek/gitmek/yapmak/çalışmak/istemek/söylemek
gibi ~30 yaygın fiilin şimdiki zaman + geçmiş zaman + koşul çekimleri (~300 kelime).

**Eklenmeyi bekleyen fiil kategorileri:**
- Gelecek zaman olumsuz (`gelmeyeceğim`, `yapmayacaksın`)
- Emir kipi (`gel`, `gelsin`, `geliniz`)
- Dilek-şart kipi (`gelsem`, `gelseydik`)
- Yeterlik kipi (`gelebilirim`, `yapamam`)
- İsimden türeyen sıfat/zarf ekleri (`-lik`, `-siz`, `-ce`)

Kalıcı çözüm: Zemberek benzeri bir **morfolojik analizör** (affix-stripping) —
bu, Faz 4'ün ötesinde ayrı bir iş olarak `GELISTIRME-PLANI.md`'ye eklenebilir.

### 3.2 Henüz kural yazılmamış yaygın hatalar (bu oturumda test edilerek bulundu)

| Aday kalıp | Not |
|---|---|
| `herhangibi` → `herhangi bir` | Tespit edildi, henüz eklenmedi |
| Emir kipi + soru eki (`gelirmisin`) | `TurkishQuestionParticle` şu an yalnız 3. tekil/çoğul kapsıyor |
| `-daki/-deki` bitişik hataları (`evdeki` DIŞINDA farklı kalıplar) | Araştırılmadı |
| Büyük ünlü uyumu ihlalleri (`geliyoruz` yerine `geliyoruz` gibi yanlış ek) | POS/morfoloji gerektirir, Faz 5 |

### 3.3 Faz 5 (POS) beklemede

`Document::new_lexicon` İngilizce Brill/Burn etiketleyiciyi atlıyor; Türkçe POS modeli
yok. Özne-yüklem uyumu, ünlü uyumu gibi kurallar bu olmadan yazılamaz.

---

## 4. Dosya Haritası

| Ne | Nerede |
|---|---|
| Kullanım kuralları | `harper-core/src/linting/turkish_usage.rs` |
| Gereksizlik kuralları | `harper-core/src/linting/turkish_redundancy.rs` |
| Kesmeli de/da | `harper-core/src/linting/turkish_de_da_apostrophe.rs` |
| Soru eki | `harper-core/src/linting/turkish_question_particle.rs` |
| Sözlük yükleyici | `harper-core/src/spell/turkish_dictionary.rs` |
| Sözlük verisi | `turkish/data/wordlist-tr.txt` |
| Kural kaydı | `harper-core/src/linting/lint_group/mod.rs` (`new_turkish_profile`) |
| Varsayılan config | `harper-core/default_config.json` (TR kuralları EN'de kapalı) |
| CLI | `harper-cli lint --profile turkish` |
| WASM | `Linter.new_turkish()` (`harper-wasm/src/lib.rs`) |
| Geliştirme planı | `turkish/GELISTIRME-PLANI.md` |
| Geliştirme analizi (GhostEdit) | `turkish/GELISTIRME-ANALIZI.md` |
| Fork notları | `turkish/FORK-NOTES.md` |

---

## 5. Test Durumu

```bash
cargo test -p harper-core --lib turkish
```

115 test, tamamı geçiyor (2026-08-14 itibarıyla). Dağılım:

| Modül | Test sayısı |
|---|---|
| `turkish_usage` | 50 |
| `turkish_redundancy` | 24 |
| `turkish_de_da_apostrophe` | 17 |
| `turkish_question_particle` | 17 |
| `turkish_dictionary` | 3 |
| `lint_group` (Türkçe profil) | 2 |
| `lexing` (tokenizer) | 1 |
| diğer (`document`, örnekler) | 1 |

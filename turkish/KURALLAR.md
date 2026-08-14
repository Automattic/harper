# Harper Türkçe — Kural Referansı

Son güncelleme: 2026-08-14  
Kapsam: `harper-fork` içindeki **uygulanmış** kurallar + `GELISTIRME-ANALIZI.md`'de
belgelenmiş GhostEdit arşiv taraması (`D:\Projeler\Harper türkçe projesi`)
+ GitHub'da bulunan açık kaynak Türkçe dilbilgisi projelerinden alınan
güvenli kalıplar (Bölüm 2.4) sonuçları tek yerde.

**Kaynaklar özeti (kullanıcı sorusu "Daha fazla kural olmalıydı, bunları
bulmuştuk GitHub'da" için):**

| Kaynak | Ne alındı | Bölüm |
|---|---|---|
| GhostEdit yerel arşivi (`Harper türkçe projesi`) | Usage/redundancy tabloları, sözlük, soru eki deseni | 2 |
| [`Denomas/Turkce-yazim-denetimi`](https://github.com/Denomas/Turkce-yazim-denetimi) (MIT, GitHub) | `BitisikYazim`, `YanlisTurkce` içinden güvenli alt kümeler + yeni `TurkishMergeWords`/`TurkishProperNouns` kuralları | 1.1b, 1.4b, 2.4 |

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
| birgün | bir gün |
| heryer | her yer |
| heryerde | her yerde |
| birkez | bir kez |
| şuan | şu an |
| şuanda | şu anda |
| hiçbirzaman | hiçbir zaman |
| okadar | o kadar |
| bukadar | bu kadar |
| şukadar | şu kadar |
| nekadar | ne kadar |
| herhangibir | herhangi bir |
| bişey | bir şey |
| bişi | bir şey |

**b) Circumflex/şapka eksikliği**

| Hatalı | Doğru |
|---|---|
| eger | eğer |
| gercek | gerçek |
| gercekten | gerçekten |

**b2) Yaygın yazım hataları / konuşma dili kısaltmaları**

Kaynak: [`Denomas/Turkce-yazim-denetimi`](https://github.com/Denomas/Turkce-yazim-denetimi)
(MIT lisans) — `styles/Turkish/YanlisTurkce.yml` dosyasından, homograph riski
taşımayan alt küme.

| Hatalı | Doğru |
|---|---|
| süpriz | sürpriz |
| şarz | şarj |
| espiri | espri |
| insiyatif | inisiyatif |
| teşekürler | teşekkürler |
| teşekür | teşekkür |
| diğil | değil |
| deyil | değil |
| yokki | yok ki |
| varki | var ki |
| gelicem | geleceğim |
| gidicem | gideceğim |
| yapıcam | yapacağım |
| edicek | edecek |

Aynı kaynaktan **bilinçli olarak alınmayanlar**: `naber`/`nbr`/`slm`/`mrb`/
`eyw`/`tşk`/`sağol` — bunlar sohbet dilinde kısaltma/argo, "yazım hatası"
değil; biçimsellik tercihi. Harper'ın `Usage` kategorisiyle karıştırılmaması
için eklenmedi.

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

### 1.1b `TurkishMergeWords` — ayrı yazılan ama bitişik olması gereken kalıplar

Dosya: `harper-core/src/linting/turkish_merge_words.rs` · `LintKind::Usage` ·
`TurkishUsage`'ın tam tersi yönü (ayrı → bitişik).

Kaynak: [`Denomas/Turkce-yazim-denetimi`](https://github.com/Denomas/Turkce-yazim-denetimi)
(MIT) — `styles/Turkish/BitisikYazim.yml`'nin "Bitişik yazılması gerekenler"
bölümü, TDK kurallarıyla çapraz kontrol edilip tek-anlamlı olanları alındı.

| Hatalı (ayrı) | Doğru (bitişik) |
|---|---|
| bir kaç | birkaç |
| bir çok | birçok |
| her hangi | herhangi |
| vaz geçmek | vazgeçmek |
| vaz geçti | vazgeçti |
| vaz geçtim | vazgeçtim |

**Bilinçli olarak eklenmeyenler (kaynaktaki hatalı/riskli girdiler):**
- `bir takım` → `birtakım`: yalnızca "birkaç/bazı" anlamında bitişik yazılır;
  "tek bir takım/set" anlamında ayrı kalması doğrudur. POS olmadan
  ayrıştırılamaz, homograph riski nedeniyle eklenmedi.
- `nasıl ki`/`öyle ki`/`şöyle ki` → `nasılki`/`öyleki`/`şöyleki`: kaynak
  projenin `BitisikYazim.yml` dosyası bunları **bitişik** öneriyor, ancak bu
  hem TDK kuralına hem de aynı projenin kendi `KiEki.yml` dosyasına
  (bkz. aşağı) **ters** düşüyor. TDK'ye göre bağlaç "ki" ayrı yazılır
  (yalnızca `sanki`, `halbuki`, `mademki`, `meğerki`, `oysaki`, `belki`
  kalıplaşmış istisnalardır). Harper zaten `öyleki` → `öyle ki` yönünde
  doğru kuralı uyguluyor (bkz. 1.1d); bu çelişkili girdi bilerek atlandı.

### 1.1c Değerlendirilip **atlanan** büyük harf kuralı: gün/ay adları

Kaynak projenin `Buyukharf.yml` dosyası `pazartesi`, `ocak` gibi gün/ay
adlarının her zaman büyük harfle başlamasını istiyor. Bu **yanlıştır**: TDK'ye
göre gün/ay adları yalnızca belirli bir tarih ifadesinde büyük yazılır
("29 Ekim 1923"), genel kullanımda küçük kalır ("pazartesi günleri", "her
ay"). Bu kural bilerek **alınmadı** (yanlış-pozitif riski yüksek). Bkz. 1.4
için yalnızca bağlamdan bağımsız güvenli özel isimler.

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

### 1.4b `TurkishProperNouns` — özel isim büyük harf kuralı

Dosya: `harper-core/src/linting/turkish_proper_nouns.rs` · `LintKind::Capitalization`

Kaynak: [`Denomas/Turkce-yazim-denetimi`](https://github.com/Denomas/Turkce-yazim-denetimi)
(MIT) — `styles/Turkish/Buyukharf.yml`'den yalnızca **bağlamdan bağımsız her
zaman büyük** olması gereken alt küme (ülke/şehir/kişi/dil adları). Gün/ay
adları TDK'ye göre bağlama bağlı olduğu için bilerek dışarıda bırakıldı
(bkz. 1.1c).

| Küçük yazılmış | Doğru |
|---|---|
| türkiye | Türkiye |
| istanbul | İstanbul |
| ankara | Ankara |
| izmir | İzmir |
| bursa | Bursa |
| antalya | Antalya |
| adana | Adana |
| konya | Konya |
| atatürk | Atatürk |
| türkçe | Türkçe |
| ingilizce | İngilizce |

Yalnızca **tamamen küçük harfle** yazılan tam kelime eşleşmesinde tetiklenir
(`türkiye`, `istanbul'a` gibi kesmeli ek durumları da desteklenir); zaten
büyük veya TAMAMEN BÜYÜK (başlık amaçlı) yazımlar dokunulmaz. Ek almış ama
kesme işaretsiz biçimler (`türkiyedeki`) kapsam dışıdır — bunlar hem
büyütme hem kesme eklemeyi gerektirir, ayrı bir kural adayıdır (bkz. 3.2).

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

### 2.4 GitHub'dan bulunan ek açık kaynak kurallar (2026-08-14 taraması)

GhostEdit dışında, GitHub'da halka açık Türkçe dilbilgisi/yazım denetimi
projeleri arandı. En doğrudan uygulanabilir olanı:

**[`Denomas/Turkce-yazim-denetimi`](https://github.com/Denomas/Turkce-yazim-denetimi)**
(MIT lisans, [Vale](https://vale.sh) prose-linter için 16 Türkçe kural
dosyası + Hunspell sözlüğü):

| Kaynak dosyası | İçerik | Harper'a durumu |
|---|---|---|
| `BitisikYazim.yml` | Ayrı↔bitişik kalıplar | Kısmen alındı: `TurkishUsage` (1.1a) + yeni `TurkishMergeWords` (1.1b). Çelişkili "ki" girdileri atlandı. |
| `YanlisTurkce.yml` | Yaygın yazım hataları + argo kısaltmalar | Yazım hataları alındı (1.1b2); argo kısaltmalar (`naber`, `slm`...) bilerek atlandı. |
| `DeDABaglaci.yml` | `bende`/`sende`/... regex'i ile de/da uyarısı | **Alınmadı** — bu yaklaşım "Bende kalem var" gibi geçerli lokatif kullanımını da yanlış pozitif olarak işaretler; Harper'ın kendi `TurkishUsage` "Bilinçli olarak dokunulmayanlar" listesi (1.1 sonu) zaten bu riski bilerek dışarıda tutuyor. |
| `KiEki.yml` | "ki" bağlacı ayrı yazım kalıpları | Zaten `TurkishUsage` (1.1d) içinde eşdeğeri var, tutarlı. |
| `Buyukharf.yml` | Özel isim + gün/ay adı büyük harf | Yalnızca özel isim (ülke/şehir/kişi/dil) alt kümesi → yeni `TurkishProperNouns` (1.4b). Gün/ay adları bağlama bağlı olduğu için **alınmadı** (1.1c). |
| `Tekrar.yml` | Kelime tekrarı | Harper'da zaten dil-bağımsız `RepeatedWords` kuralı var, ek iş gerekmedi. |
| `Deyimler.yml`, `Akademik.yml`, `Plaza.yml`, `Fabrika.yml`, `Teknik.yml`, `Sadelik.yml`, `UzunCumle.yml`, `Noktalama.yml`, `CumleBasi.yml` | Deyim düzeltme, iş jargonu sadeleştirme, akademik dil, uzun cümle, noktalama | **İncelenmedi/alınmadı** — kapsam dışı (stil/jargon kategorisi, "yazım hatası" değil) veya bu oturumda zaman kısıtı nedeniyle ertelendi. Sonraki adaylar (Bölüm 3.2). |

Diğer taranan ama uygun bulunmayan kaynaklar:
- **LanguageTool**: Türkçe için resmi dil desteği/kural dosyası **yok**
  (bkz. proje forumunun kendi cevabı — henüz kimse portlamamış).
- **Zemberek-NLP** (`ahmetaa/zemberek-nlp`): morfolojik analiz + Java
  bağımlılığı; GhostEdit analizi zaten bunu "taşınmayacak" olarak
  işaretlemişti (2.3), yeniden doğrulandı.
- **StarlangSoftware/TurkishSpellChecker**: FSM tabanlı morfolojik yazım
  denetleyici, ayrı bir dil (Java/Python/C++) altyapısı gerektiriyor; Harper
  Rust ile bütünleşmiyor, kod olarak taşınmadı.

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

### 3.2 Henüz kural yazılmamış yaygın hatalar

| Aday kalıp | Not |
|---|---|
| Emir kipi + soru eki (`gelirmisin`) | `TurkishQuestionParticle` şu an yalnız 3. tekil/çoğul kapsıyor |
| `-daki/-deki` bitişik hataları (`evdeki` DIŞINDA farklı kalıplar) | Araştırılmadı |
| Büyük ünlü uyumu ihlalleri (`geliyoruz` yerine `geliyoruz` gibi yanlış ek) | POS/morfoloji gerektirir, Faz 5 |
| Özel isim + ek ama kesme işaretsiz (`türkiyedeki`, `istanbulda`) | `TurkishProperNouns` (1.4b) bilerek kapsam dışı bıraktı; hem büyütme hem kesme eklemeyi gerektirir |
| Deyim/galat-ı meşhur düzeltmeleri (`Denomas` `Deyimler.yml`) | Tartışmalı/düşük öncelikli, incelenmedi (2.4) |
| Plaza Türkçesi / iş jargonu sadeleştirme (`deadline` → `son teslim tarihi`) | Farklı `LintKind` (Regionalism/Style) gerektirir, incelenmedi (2.4) |
| Akademik dil sadeleştirme (`mamafih` → `bununla birlikte`) | Stil kategorisi, incelenmedi (2.4) |
| Noktalama ve cümle başı büyük harf kontrolü | Harper'ın genel (dil-bağımsız) noktalama kuralları zaten kısmen kapsıyor olabilir, doğrulanmadı |

### 3.3 Faz 5 (POS) beklemede

`Document::new_lexicon` İngilizce Brill/Burn etiketleyiciyi atlıyor; Türkçe POS modeli
yok. Özne-yüklem uyumu, ünlü uyumu gibi kurallar bu olmadan yazılamaz.

---

## 4. Dosya Haritası

| Ne | Nerede |
|---|---|
| Kullanım kuralları (ayrı yazılması gereken bitişikler) | `harper-core/src/linting/turkish_usage.rs` |
| Birleştirme kuralları (bitişik yazılması gereken ayrılar) | `harper-core/src/linting/turkish_merge_words.rs` |
| Gereksizlik kuralları | `harper-core/src/linting/turkish_redundancy.rs` |
| Kesmeli de/da | `harper-core/src/linting/turkish_de_da_apostrophe.rs` |
| Soru eki | `harper-core/src/linting/turkish_question_particle.rs` |
| Özel isim büyük harf | `harper-core/src/linting/turkish_proper_nouns.rs` |
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

163 test, tamamı geçiyor (2026-08-14 itibarıyla). Dağılım:

| Modül | Test sayısı |
|---|---|
| `turkish_usage` | 72 |
| `turkish_redundancy` | 24 |
| `turkish_merge_words` | 11 |
| `turkish_de_da_apostrophe` | 17 |
| `turkish_question_particle` | 17 |
| `turkish_proper_nouns` | 14 |
| `turkish_dictionary` | 3 |
| `lint_group` (Türkçe profil) | 2 |
| `lexing` (tokenizer) | 1 |
| diğer (`document`, örnekler) | 1 |

Ayrıca tüm `harper-core` paketi (6286 test) ve `harper-cli`/`harper-ls`
derlemeleri bu değişikliklerle birlikte kontrol edildi, hepsi geçiyor.

**Bu oturumda düzeltilen önceden var olan (bu değişikliklerden bağımsız)
gizli hata:** `lint_group::tests::lint_descriptions_are_clean` testi,
Türkçe kural açıklamalarındaki tırnaklı örnek kelimelerin (`gitti`, `birkaç`,
`şey`, `da`) İngilizce `SpellCheck`/`Capitalization` kurallarını tetiklemesi
yüzünden sessizce kırıktı (yalnızca ilk hatayı gösterip orada duruyordu).
Tüm Türkçe `description()` metinlerinde örnek kelimeler artık backtick
(`` ` ``) içine alınıyor — Harper'ın markdown ayrıştırıcısı kod aralıklarını
yazım denetiminden hariç tutuyor, bu da diğer tüm İngilizce linter açıklama
metinlerinin zaten kullandığı kural.

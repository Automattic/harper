# Harper Türkçe Fork'u — Proje Mimarisi Haritası

> Bu dosya `CLAUDE.md`'deki kurala göre her önemli değişiklikte güncellenmelidir.
> Son güncelleme: 2026-08-15 — 4 paralel keşif ajanının ürettiği, ~1.780 gerçek
> kaynak dosyasının (target/, .git/, node_modules/ hariç) tek tek okunmasına
> dayanan detaylı dosya envanteri Bölüm 12-15'e eklendi (bkz. altındaki özet
> bölümler 1-11, mimari anlayış zaten oradaydı — bu ekleme sadece "hangi
> dosyada ne var" sorusuna dosya-bazlı, kesin cevap sağlıyor).

Kaynak: `D:\Projeler\harper-fork` (automattic/harper'ın forku, `harper-core` v2.8.0). Workspace ~22 crate + `packages/` altında JS paketleri içeriyor.

---

## 1. Genel workspace yapısı

`Cargo.toml` bir workspace tanımlıyor; her crate `harper-core`'a `path` bağımlılığıyla bağlı. Çoğu format-özel crate'in `description` alanı jenerik ("The language checker for developers.") — gerçek amaçları README yerine kaynak kod yapısından anlaşılıyor. `harper-desktop` tek workspace dışı bir Tauri/SvelteKit projesi (kendi `package.json`'ı var, Cargo workspace'ine dahil değil, `src-tauri/` altında ayrı bir Rust crate'i barındırıyor).

Fork’a özel Türkçe çalışma alanı `turkish/`:

- `turkish/README.md` — ne nerede (motor vs. not/script/veri)
- `turkish/FORK-NOTES.md` — 47 kalıp (32 usage + 15 redundancy), dictionary/POS henüz yok
- `turkish/GELISTIRME-ANALIZI.md` — 2026-08-14 derin tarama + çek listesi + `new_turkish_profile`
- `turkish/GELISTIRME-PLANI.md` — faz 0–6, dilimler, yapılmayacaklar
- `turkish/scripts/` — WASM Node doğrulama (`test_turkish*.mjs`); harper-wasm kökünde tutulmaz
- `turkish/data/wordlist-tr.txt` — GhostEdit ek sözlük kopyası; `turkish_dictionary()` gömer
- `turkish/DESKTOP.md` — highlighter / Windows broker notu

Motor: `harper-core/src/linting/turkish_*.rs`, `spell/turkish_dictionary.rs`.
`Document::new_lexicon` İngilizce Brill/Burn’ü atlar.

---

## 2. harper-core — çekirdek motor

### 2.1 Üst düzey modüller (`harper-core/src/lib.rs`)
`document.rs`, `token.rs`, `token_kind.rs` çekirdek veri modelini; `expr/`, `patterns/` desen eşleştirme motorunu; `parsers/` format ayrıştırıcılarını; `linting/` (326 dosya) kuralları; `spell/` sözlük sistemlerini; ek olarak `weir/` ve `weirpack/` adında **DSL tabanlı, harici olarak yazılabilen kural sistemi** barındırıyor.

### 2.2 `Document` (`harper-core/src/document.rs`)
`Document::new()`/`new_from_chars()` bir `Parser` + `Dictionary` alıp `tokens: Vec<Token>` üretiyor, sonra `parse()` çağrılıyor (satır ~175-220):

```rust
fn parse(&mut self, dictionary: &impl Dictionary) {
    self.apply_fixups();
    let chunker = burn_chunker();   // harper_brill — İngilizce'ye özel, hardcoded
    let tagger = brill_tagger();    // harper_brill — İngilizce'ye özel, hardcoded
    for sent in self.tokens.iter_sentences_mut() {
        let token_tags = tagger.tag_sentence(&token_strings);
        let np_flags = chunker.chunk_sentence(&token_strings, &token_tags);
        // ... DictWordMetadata.pos_tag ve np_member atanıyor
    }
}
```

**KRİTİK BULGU (Türkçe için çok önemli):** `Document::parse()` her zaman global `brill_tagger()`/`burn_chunker()` fonksiyonlarını çağırıyor. Türkçe yol: **`Document::new_lexicon` / `new_from_chars_lexicon`** — yalnızca sözlük metadata’sı, İngilizce POS yok. UD modeli hâlâ yok.

`apply_fixups()` (satır 156-170) dil-agnostik metin normalizasyonları yapıyor; bazıları (`condense_dotted_initialisms`, `condense_common_top_level_domains`) İngilizce'ye özgü sezgiler içerebilir, ileride incelenmeli.

### 2.3 `Token` / `TokenKind` (`token.rs`, `token_kind.rs`)
`Token { span: Span<char>, kind: TokenKind }`. `TokenKind::Word(Option<DictWordMetadata>)` — `None` sözlükte yoksa. `delegate_to_metadata!` makrosuyla ~60 POS sorgulama metodu üretiliyor, hepsi `DictWordMetadata`'ya delege ediyor. Ayrıca `EmailAddress`, `Url`, `Hostname`, `Decade`, `Regexish`, `HeadingStart` gibi dil-agnostik özel tokenlar var.

### 2.4 `expr/` — desen eşleştirme motoru
Merkezi `Expr` trait'i (`expr/mod.rs`):
```rust
pub trait Expr: LSend {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>>;
}
```
`Step` trait'i (`expr/step.rs`) — `patterns::Pattern` implement eden her tip otomatik `Step`/`Expr` oluyor (blanket impl). `SequenceExpr` (en çok kullanılan yapı taşı) `then()`, `then_whitespace()`, `fixed_phrase()`, `any_capitalization_of()` gibi builder metodları sunuyor. **ASCII-only case-folding sorunu tam olarak burada**: `fixed_phrase`/`any_capitalization_of`, `char_string.rs`'deki `eq_ignore_ascii_case` ailesini kullanıyor — çekirdeğe dokunmadan `turkish_redundancy.rs`'deki closure tabanlı `turkish_word()` matcher'ıyla (blanket `Step` impl'i sayesinde) atlatıldı. Kural dosyaları `Step`/`Expr` implement eden herhangi bir closure/struct kullanabiliyor, bu genişletilebilirliğin kanıtı.

Diğer `Expr` implementasyonları (`FirstMatchOf`, `LongestMatchOf`, `Optional`, `Repeating`, `AnchorStart`/`AnchorEnd`, `PronounBe`, `SpelledNumberExpr` vb.) çoğunlukla İngilizce dilbilgisine özel, şablon olarak yol gösterici.

### 2.5 `patterns/`
`Pattern` trait'i (`Step`'in temeli). `upos_set.rs` **doğrudan POS tagger çıktısına bağımlı** (Türkçe POS etiketleyici olmadan kullanılamaz). `indefinite_article.rs`, `modal_verb.rs`, `relative_pronoun.rs` — İngilizce'ye özel kavramlar.

### 2.6 `parsers/` — format ayrıştırıcıları
Ortak `Parser` trait'i:
```rust
pub trait Parser: LSend { fn parse(&self, source: &[char]) -> Vec<Token>; }
```
- `PlainEnglish` — `is_english_lingual()` Türkçe harfleri kapsar. **`lex_plural_digit` yalnızca rakam+`s`/`'s` (0s, 1's)**; harf+`s`+non-ASCII (`asıl`) artık bölünmez.
- `Markdown` (pulldown-cmark), `OrgMode`, `Mask`, `IsolateEnglish`, `OopsAllHeadings`, `CollapseIdentifiers`.

Format-özel crate'ler (harper-html, harper-typst vb.) bu `Parser` trait'ini kendi üst-seviye tipleri için implement ediyor.

### 2.7 `weir/` ve `weirpack/` — harici DSL kural sistemi
Küçük bir dil (basit desen/kural sözdizimi) parse edip `Expr`'e derleyen sistem. `linting/weir_rules/` bu sistemle tanımlı kuralları barındırıyor. **Rust kodu yazmadan yeni kural eklemek için ikinci bir yol olabilir** — Türkçe kuralları genişletirken (özellikle basit kelime/kalıp değişimleri) bu DSL değerlendirilebilir. Case-folding/Unicode davranışının Türkçe'yle uyumlu olup olmadığı kontrol edilmeli (muhtemelen aynı ASCII-only sorunu burada da var).

### 2.8 `language_detection.rs`
```rust
pub fn is_doc_likely_english(doc: &Document, dict: &impl Dictionary) -> bool
```
Sözlükte bulunan kelime oranına (`< 0.7` ise İngilizce değil) bakan istatistiksel tespit. **KRİTİK:** `harper-wasm/src/lib.rs` ve `parsers/isolate_english.rs`'de kullanılıyor — Türkçe metin İngilizce sözlüğe göre "İngilizce değil" damgalanıp bazı adımlardan hariç tutulabilir. `turkish/FORK-NOTES.md`'deki "WASM'da sadece Türkçe kuralları açık bırakma" çözümü muhtemelen bu gate'in etkisini bertaraf ediyor, ama `IsolateEnglish` parser'ını kullanan başka bir akışta (örn. harper-comments üzerinden Türkçe yorumlar) bu fonksiyon devreye girebilir. Tam entegrasyon için dil-parametrik hale getirilmeli veya Türkçe için eşdeğeri eklenmeli.

### 2.9 `spell/` — sözlük sistemleri
`Dictionary` trait'i (`spell/dictionary.rs`) — dil-agnostik arayüz (`contains_word`, `fuzzy_match`, `get_word_metadata` vb.). **Türkçe sözlük entegrasyonu için doğru soyutlama noktası.**

**`MutableDictionary`** — kritik kısım:
```rust
fn uncached_inner_new() -> Arc<MutableDictionary> {
    MutableDictionary::from_rune_files(
        include_str!("../../dictionary.dict"),
        include_str!("../../annotations.json"),
    )...
}
```
İngilizce "kürasyonlu" sözlük derleme zamanında `harper-core/dictionary.dict` (54.735 kök + Hunspell tarzı ek bayrak kodları) + `harper-core/annotations.json`'dan gömülüyor. Türkçe için paralel mekanizma mümkün ama şu an yollar hardcoded — çoklu dil desteği için "hangi dil" parametresi/feature flag gerekir.

**`rune/` modülü** — Hunspell tarzı ek (affix) sistemi (`word_list.rs`, `attribute_list.rs`, `affix_replacement.rs`, `expansion.rs`, `matcher.rs`). **Dil-agnostik ama Hunspell affix kuralları genelde tek seviyeli** (bir kökten bir adım ek) — Türkçe'nin çok basamaklı ek yığılması (ev-ler-imiz-den) için muhtemelen yetersiz kalır. **Öneri: Zemberek kaynaklı kelime listesini önceden çekimlenmiş geniş bir liste olarak (affix sistemi pas geçilerek) `.dict` formatında yüklemek** daha güvenli bir başlangıç.

**`FstDictionary`** — `fst` crate ile hızlı fuzzy-match, `MutableDictionary`'yi sarmalıyor; dil-agnostik mekanizma, Türkçe sözlük eklenince otomatik faydalanır.

**`MergedDictionary`** — birden fazla `Dictionary`'yi birleştiriyor; `harper-wasm` kullanıcı+curated+weirpack sözlüklerini birleştirmek için kullanıyor. Türkçe+İngilizce aynı anda desteklenecekse doğal birleştirme noktası.

**`spell/mod.rs`'deki yanlış-yazım sezgileri** (`is_ou_misspelling`, `is_cksz_misspelling`, `is_er_misspelling`, `is_ay_ey_misspelling`, `is_ei_ie_misspelling`, `is_th_h_missing`) — öneri sıralamasında (`score_suggestion`) kullanılan **tamamen İngilizceye özgü** sezgiler. Türkçe öneri sıralaması için Türkçe'ye özgü eşdeğerleri (ünlü uyumu, yumuşama/sertleşme, ğ-düşmesi) gerekecek.

### 2.10 `linting/` — 326+ dosya, kural motoru
`ExprLinter` trait'i — Türkçe kurallarının kullandığı arayüz, **POS tagger gerektirmiyor**. `LintGroup::new_curated(dictionary, dialect)` (satır 548, `lint_group/mod.rs`) tüm kuralları kaydeden merkezi fonksiyon:
- `insert_struct_rule!` — `Default` implement eden `Linter` struct'ı.
- `insert_expr_rule!` — `ExprLinter` implement eden ifade-tabanlı kural (Türkçe kuralları burada).
- `_with_dict`/`_with_dialect` varyantları.

`Dialect` (American/British/Australian/Canadian/Indian) — Türkçe için karşılığı yok; muhtemelen paralel bir "dil" seçici gerekecek çünkü şu an İngilizce ve Türkçe kuralları aynı fonksiyonda, aynı listede.

**Config sistemi**: `flat_config.rs` + `structured_config/`, `default_config.json` üzerinden. `curated_default_config_lists_every_registered_rule` self-check testi her kayıtlı kuralın config'de olmasını zorunlu kılıyor — Türkçe kuralları bu testi geçiyor.

`LintKind` — `Usage`, `Redundancy` dahil; Türkçe kuralları mevcut değerleri kullanıyor.

---

## 3. harper-brill — POS etiketleyici + chunker

İki gömülü model dosyası (`trained_tagger_model.json`, `trained_chunker_model.json`, `finished_chunker/model.mpk` — burn nöral ağ) `include_str!`/`include_bytes!` ile paketlenip `brill_tagger()`, `brill_chunker()`, `burn_chunker()` fonksiyonlarıyla erişiliyor. `Document::parse()` `burn_chunker()` (nöral) ve `brill_tagger()` (kural-tabanlı) kullanıyor.

`harper-pos-utils/src/` — **`training` adlı bir Cargo feature'ı var** (eğitim kodu mevcut, harici araç yazmaya gerek olmayabilir). `conllu_utils.rs` CoNLL-U formatı desteği — **Türkçe UD Treebank'leri (IMST, BOUN, Kenet) doğrudan bu formatta mevcut**, veri kaynağı sorunu büyük ölçüde çözülü. `upos.rs` — evrensel POS etiket seti (UPOS), Türkçe için de uygulanabilir.

**Türkçe POS için somut yol haritası:** (1) Türkçe UD treebank'i CoNLL-U formatında temin et, (2) `harper-pos-utils --features training` ile model eğit, (3) `harper-brill`'e Türkçe modelleri ikinci bir `include_str!` seti olarak ekle, paralel `brill_tagger_tr()`/`burn_chunker_tr()` fonksiyonları oluştur, (4) `Document::parse()`'ı dil parametresi alacak şekilde refactor et.

---

## 4. harper-ls — Language Server Protocol

`tower_lsp_server` + `tokio`, stdio veya TCP (`127.0.0.1:4000`). `backend.rs`, `config.rs`, `diagnostics.rs`, `document_state.rs`, `pos_conv.rs`, `ignored_lints_io.rs`. Türkçe kuralları `LintGroup`'a kayıtlı olduğundan harper-ls üzerinden otomatik expose ediliyor olmalı — ek değişiklik gerekmez.

---

## 5. harper-wasm — WASM binding'leri

Ana tip `Linter` — `lint_group`, `user_dictionary`, `dictionary: Arc<MergedDictionary>`, `weirpack_dictionaries`, `ignored_lints`, `dialect`. `Language` enum'u (`Plain`, `Markdown`, `Typst`) — **Türkçe için yeni varyant gerekmiyor**, mevcutlar zaten Türkçe metni doğru tokenize ediyor.

`is_doc_likely_english` burada da import ediliyor — İngilizce olmayan metni filtrelemek için kullanılıyor olabilir, WASM entegrasyonunda önemli bir kontrol noktası.

`packages/harper.js/src/LocalLinter.ts` — JS sarmalayıcı, benzer mantık içerebilir, ayrıca incelenmeli.

---

## 6. Format-özel parser crate'leri

Hepsi `harper_core::parsers::Parser` implement ediyor; çoğu `harper-tree-sitter`'ın `TreeSitterMasker`'ını temel alıyor.

- **harper-tree-sitter**: `TreeSitterMasker { language, node_condition }`. `create_ident_dict()` kod tanımlayıcılarını geçici sözlüğe ekliyor (yanlış yazım işaretlenmesin diye). `extract_comments()` yorum span'lerini çıkarıyor.
- **harper-comments**: Çok sayıda programlama dili için tree-sitter grammar bağımlılığı, yorum sözdizimini ayrıştırıp doğal dil metnini besliyor.
- **harper-html**, **harper-jjdescription**, **harper-git-commit**, **harper-ink**, **harper-python**, **harper-asciidoc** — benzer `TreeSitterMasker` deseni.
- **harper-typst**: Kendi `typst-syntax` tabanlı çevirici.
- **harper-tex**: Regex/elle LaTeX komut/ortam maskeleme.
- **harper-literate-haskell**: `.lhs` formatı, harper-comments'e bağımlı.

Hiçbiri Türkçe için değişiklik gerektirmiyor — dil-agnostik. Türkçe kod yorumu denetimi otomatik çalışmalı (doğrulanmadı).

---

## 7. harper-desktop — Tauri masaüstü uygulaması

SvelteKit frontend + Tauri backend (`commands.rs`, `highlighter/`, `communication/`, `mac_broker/`, `os_broker.rs`, `windows.rs`, `tray.rs`). Workspace `Cargo.toml`'a dahil değil. **Görece olgun/tam bir uygulama** (tray, highlighter servisi, platform-özel broker'lar) — ama Türkçe motoruyla henüz bağlanmamış. **Potansiyel hızlı-kazanım**: sıfırdan UI yazmak yerine bu uygulamaya Türkçe motorunu bağlamak.

---

## 8. harper-thesaurus — eş anlamlı kelime sözlüğü

`zstd` ile sıkıştırılmış İngilizce (WordNet benzeri) veri, `build.rs` seviyesinde derleniyor. Türkçe için paralel bir kaynak (TDK Eş Anlamlılar Sözlüğü) ve benzer şema kurulabilir — düşük öncelikli, izole.

---

## 9. harper-stats — istatistik toplama

`record.rs`, `summary.rs` — lint tetiklenme sıklığı gibi olaylar. Dil-agnostik, Türkçe kuralları otomatik dahil olur.

---

## 10. harper-cli, harper-pos-utils, harper-dictionary-wordlist, fuzz

- **harper-cli**: `lint --profile turkish` TR sözlük + `LintGroup::new_turkish_profile` + lexicon parse. Diyalekt hâlâ İngilizce bölge (`--dialect us`).
- **harper-dictionary-wordlist**: `MutableDictionary`'den kelime listesi dışa aktarma/işleme aracı — Türkçe sözlük dosyası hazırlarken referans alınabilir.
- **fuzz**: `cargo-fuzz` tabanlı parser fuzz testleri.

---

## 11. Türkçe desteği için eksik parçalar (güncel özet)

1. **POS etiketleyici** — lexicon parse atlıyor; UD eğitimi repo dışı (`turkish/data/README.md`).

2. **Türkçe sözlük** — `turkish_dictionary()` + SpellCheck TR profilde açık. Affix/Hunspell yok; ham liste.

3. **`is_doc_likely_english()`** — WASM TR linter `isolate_english` uygulamaz.

4. **Yazım önerisi sezgileri** hâlâ İngilizce skor fonksiyonları; TR dict ile yanlış öneri kalitesi sınırlı.

5. **`LintProfile::{Curated,Turkish}`** — `new_for_profile`. EN `default_config.json`’da TR kuralları kapalı. `Dialect::Turkish` yok.

6. **weir** — TR Unicode için henüz doğrulanmadı.

7. **harper-desktop** — `turkish/DESKTOP.md`; Windows `NoopBroker`.

---

## Önceki oturumlarda kanıtlanan çalışan parçalar (özet)

- Tokenizer: Latin script + `lex_plural_digit` rakam sınırı (2026-08-14).
- ASCII case-fold sorunu: `turkish_word()` closure matcher ile çekirdeğe dokunmadan aşıldı.
- `turkish_redundancy.rs` (15 kalıp) + `turkish_usage.rs` (31 kalıp) — 11/11 test geçiyor, `default_config.json`'a kayıtlı.
- WASM derlemesi (`wasm-pack build --target nodejs`) ve `harper-cli lint --only TurkishRedundancy,TurkishUsage` üzerinden canlı doğrulandı.

---

## 12. Detaylı dosya envanteri — harper-core (linting/ hariç, ~186 dosya)

Aşağıdaki bölüm 4 paralel keşif ajanının 2026-08-15'te ürettiği, tüm gerçek
kaynak dosyalarının (binary/build-artifact hariç) tek tek okunmasına dayanan
envanterdir. Amaç: yeni bir düzenlemeye başlarken "bu dosyada ne olduğunu
keşfetmek" yerine burada arayıp bulmak.

**Kök (`harper-core/src/`)**: `lib.rs` (modül birleştirici + public re-export
yüzeyi), `document.rs` (bkz. §2.2), `token.rs`/`token_kind.rs` (bkz. §2.3),
`span.rs` (generic `Span<T>` — char/token aralığı), `char_ext.rs`
(`is_english_lingual` — Latin script kontrolü, Türkçe'yi kapsıyor),
`char_string.rs` (ASCII-only case-fold fonksiyonları — Türkçe sorununun
kaynağı, bkz. §2.4), `punctuation.rs`, `word_metadata.rs` (`DictWordMetadata`
— POS/morfoloji bayrakları struct'ı), `vec_ext.rs`, `lsp_lint.rs`,
`title_case.rs` (İngilizce'ye özel başlık büyütme kuralları), `case.rs`
(`WordCase` enum — Lower/Upper/Title/Mixed tespiti, dil-agnostik),
`fix_case.rs`, `language.rs`, `dialect.rs` (`Dialect` enum — American/
British/Australian/Canadian/Indian, Türkçe karşılığı yok), `attribute.rs`,
`fun_facts.rs` (İngilizce dilbilgisi "fun fact" mesajları), `paragraph.rs`,
`sentence.rs`.

**`expr/`** (bkz. §2.4): `mod.rs`, `expr.rs` (`Expr` trait), `step.rs`
(`Step` trait — blanket impl kaynağı), `sequence_expr.rs` (ana builder),
`first_match_of.rs`, `longest_match_of.rs`, `optional.rs`, `repeating.rs`,
`anchor_start.rs`/`anchor_end.rs`, `pronoun_be.rs`, `spelled_number_expr.rs`,
`all.rs`, `any.rs` — çoğu İngilizce dilbilgisine özel ama mekanizma
dil-agnostik.

**`patterns/`** (bkz. §2.5): `mod.rs` (`Pattern` trait), `word_pattern.rs`,
`upos_set.rs` (POS'a bağımlı — TR POS olmadan kullanılamaz),
`indefinite_article.rs`, `modal_verb.rs`, `relative_pronoun.rs`,
`whitespace_pattern.rs`, `edit_distance.rs`.

**`parsers/`** (bkz. §2.6): `mod.rs` (`Parser` trait), `plain_english.rs`
(TR metni değişiklik gerekmeden tokenize eden ana parser), `markdown.rs`
(pulldown-cmark tabanlı), `org_mode.rs`, `mask.rs` (`Masker` altyapısı —
format-özel crate'lerin temeli), `isolate_english.rs` (`is_doc_likely_english`
kullanıyor — TR için risk noktası, bkz. §2.8), `oops_all_headings.rs`,
`collapse_identifiers.rs`.

**`spell/`** (bkz. §2.9): `mod.rs` (yanlış-yazım sezgileri — İngilizce'ye
özel), `dictionary.rs` (`Dictionary` trait), `mutable_dictionary.rs`
(`MutableDictionary`, `from_rune_files`), `fst_dictionary.rs`
(`FstDictionary::curated()`), `merged_dictionary.rs`, `word_list.rs`,
`turkish_dictionary.rs` (**fork'a özel** — `turkish/data/wordlist-tr.txt`'i
gömen TR sözlük fonksiyonu). Alt modül **`rune/`**: `word_list.rs`,
`attribute_list.rs`, `affix_replacement.rs`, `expansion.rs`, `matcher.rs` —
Hunspell tarzı tek-seviyeli ek sistemi.

**`weir/` ve `weirpack/`** (bkz. §2.7): DSL parser/derleyici modülleri —
`.weir` dosyalarını `Expr`'e çeviren lexer/parser/compiler zinciri.

**`examples/`**: `turkish_tokenize_test.rs`, `turkish_redundancy_test.rs`,
`turkish_redundancy_expr_test.rs`, `turkish_redundancy_casefix_test.rs`
(fork'a özel, geliştirme sırasında kanıt-of-concept olarak yazıldı, repoda
tutuluyor).

**`benches/`**: Criterion tabanlı performans ölçümleri (dil-agnostik,
İngilizce test verisiyle).

**`tests/`**: Harper'ın kendi self-check testleri (`curated_default_config_
lists_every_registered_rule` dahil) + entegrasyon testleri.

---

## 13. Detaylı dosya envanteri — packages/ (JS/TS ekosistemi, ~489 dosya)

9 alt paket, `pnpm-workspace.yaml` ile tek workspace'te:

- **`packages/chrome-plugin/`** — **En umut verici Türkçe entegrasyon
  hedefi.** `LocalLinter` ile `harper-wasm` WASM binary'sini doğrudan
  kullanıyor. Kural listesi (`LintConfig`) **dinamik tipli** — yani
  `default_config.json`'a kayıtlı her yeni kural (Türkçe kuralları dahil)
  WASM yeniden derlenip pakete konduğunda **otomatik olarak** ayarlar
  arayüzünde belirip çalışıyor, ekstra JS/TS kodu yazmaya gerek yok. Firefox
  sürümü de aynı kod tabanından üretiliyor (`chrome_plugin.yml` CI'sinde
  görüldüğü gibi). Tek eksik: dil tespiti (`detectDialect.ts` benzeri bir
  dosya) Türkçe'yi sessizce bir İngilizce lehçesine düşürebilir — bağlama
  bakılmalı.
- **`packages/vscode-plugin/`** — `harper-ls` (WASM değil, native LSP
  sunucusu) kullanıyor; ayrı bir mimari, ayrı build/publish pipeline'ı
  (OpenVSX + VS Marketplace).
- **`packages/obsidian-plugin/`** — WASM tabanlı, chrome-plugin'e benzer
  entegrasyon deseni.
- **`packages/wordpress-plugin/`** — WordPress editörüne (Gutenberg) entegre
  WASM tabanlı eklenti.
- **`packages/harper.js/`** — WASM'ın (`harper-wasm`) etrafındaki resmi JS
  sarmalayıcı; `LocalLinter`/`WorkerLinter` sınıfları burada tanımlı, diğer
  tüm paketler bunu tüketiyor.
- **`packages/harper-editor/`** — Yeniden kullanılabilir Svelte editör
  bileşeni (`Editor`); `harper-desktop/src/lib/EditorView.svelte` bunu
  kullanıyor.
- **`packages/lint-framework/`** — Lint sonuçlarını UI'ye bağlayan ortak
  çerçeve (renk/ikon eşlemeleri, `LintKind`'a göre stil — `harper-desktop`'taki
  `lint_kind_color.rs`'nin JS eşdeğeri, senkron tutulması gerekiyor).
- **`packages/components/`** — Paylaşılan Svelte UI bileşen kütüphanesi.
- **`packages/web/`** — writewithharper.com sitesi (demo + dokümantasyon);
  `AGENTS.md`'nin referans verdiği asıl dokümantasyon dosyaları burada.

**Sonuç:** Türkçe motorunu tarayıcıya taşımanın en düşük efor gerektiren yolu
`chrome-plugin` — WASM'ı Türkçe kurallarla yeniden derleyip paketlemek
yeterli olabilir (dil tespiti sorunu hariç).

---

## 14. Detaylı dosya envanteri — linting/ kural kataloğu (326+ dosya)

`harper-core/src/linting/` altındaki **tüm** İngilizce kurallarının (Türkçe
kuralları `turkish_redundancy.rs`/`turkish_usage.rs` hariç) tek tek
`description()` metodundan okunmuş kısa özeti. Amacı: yeni bir Türkçe kural
yazarken "böyle bir kural zaten İngilizce tarafta nasıl yazılmış, hangi
altyapı kullanılmış" sorusuna hızlı referans sağlamak.

### 14.1 Kategori dağılımı (büyükten küçüğe)

1. **Deyim/kalıp ifade düzeltmeleri (idiom & eggcorn)** — en büyük kategori:
   `weir_rules/` (318 bağımsız `.weir` dosyası + 11 çok-dosyalı grup) ve
   `phrase_set_corrections/mod.rs` (~90 kural, `add_1_to_1_mappings!` /
   `add_many_to_many_mappings!` makrolarıyla toplu tanımlı). Örnek: `hone in
   on → home in on`, `wreck havoc → wreak havoc`, `damp squid → damp squib`,
   `moot point`, `peek behind the curtain`.
2. **Edat (preposition) hataları** — `accuse_of.rs`, `aspire_to.rs`,
   `fascinated_by.rs`, `interested_in.rs`, `jealous_of.rs`, `crave_for.rs`,
   `obsess_preposition.rs` gibi ~20 tekil dosya.
3. **Sık karışan kelime/homofon çiftleri** — `to_two_too/` (9 alt dosya),
   `then_than.rs`, `theyre_confusions/`, `its_contraction/`,
   `effect_affect/` (noun_verb_confusion altında), `cliché`, `confident.rs`.
4. **Kesme işareti/iyelik** — `its_possessive.rs`, `possessive_noun.rs`,
   `wrong_apostrophe.rs`, `plural_decades/`, `pronoun_contraction/`.
5. **Tire/birleşik yazım** — `closed_compounds.rs` (~40 alt kural),
   `compound_nouns/` (3 dosya), `disjoint_prefixes.rs`, `merge_words.rs`,
   `split_words.rs`, `open_compounds.rs`.
6. **Özne-fiil/zamir uyumu** — `pronoun_verb_agreement.rs`,
   `pronoun_inflection_be.rs`, `there_is_agreement.rs`, `i_am_agreement.rs`
   — **bunlar POS gerektirir**, Türkçe eşdeğeri için TR POS tagger şart.
7. **Kısaltma genişletme** — `expand_memory_shorthands.rs`,
   `expand_time_shorthands.rs`, `initialisms.rs`, `dot_initialisms.rs`.
8. **Noktalama/boşluk** — `comma_fixes.rs`, `missing_space.rs`, `spaces.rs`,
   `dashes.rs`, `quote_spacing.rs`, `unclosed_quotes.rs` — çoğu dil-agnostik,
   doğrudan Türkçe'ye uygulanabilir olabilir (henüz test edilmedi).
9. **Büyük harf kullanımı** — `sentence_capitalization.rs`,
   `capitalize_personal_pronouns.rs`, `months.rs` — kısmen dil-agnostik.
10. **Bölgesel/lehçe ve üslup** — `regionalisms.rs` (AmE/BrE/AuE/CaE/InE
    terim farkları), `avoid_contractions.rs`, `avoid_curses.rs`,
    `boring_words.rs`, `hedging.rs`, `long_sentences.rs`.

### 14.2 Altyapı dosyaları (kural değil, mekanizma)

`mod.rs`, `lint.rs`, `lint_kind.rs`, `suggestion.rs`, `expr_linter.rs`
(Türkçe kurallarının kullandığı trait), `merge_linters.rs`,
`initialism_linter.rs`, `map_phrase_linter.rs` (tek-ifade eşleştirme
altyapısı), `map_phrase_set_linter.rs` (çoklu-ifade eşleştirme altyapısı),
`lint_group/mod.rs` (kayıt orkestrasyonu, bkz. §2.10), `pooled_linter/`
(thread havuzlama — kural içermiyor).

### 14.3 Boş/iskelet dosyalar (henüz yazılmamış İngilizce kurallar)

`arrive_to.rs`, `handful_of_more.rs`, `little_known.rs`,
`pale_by_comparison.rs` — katkıda bulunanlar için kopyalanacak boş şablonlar.
Türkçe kural eklerken bu dosyaların yapısı örnek alınabilir.

### 14.4 Not

Kural sayısının büyük kısmı (`weir_rules/`'daki 318 dosya) tek tek bu
haritaya yazılmadı — hepsi deyim/eggcorn türünde, kategori 14.1'de temsili
örneklerle özetlendi. Tam liste gerekirse `harper-core/src/linting/
weir_rules/*.weir` dizini `let description "..."` alanına göre taranabilir.

---

## 15. Detaylı dosya envanteri — harper-desktop + küçük crate'ler + kök dosyalar

### 15.1 harper-desktop/src-tauri/src/ (Rust backend)

- `main.rs`/`lib.rs` — giriş noktası; `run_tauri()` (normal uygulama) veya
  `run_highlighter()` (bağımsız overlay süreci) seçimi. **`Config::
  create_linter()` (`config/mod.rs`) `LintGroup::new_curated(dict, dialect)`
  çağırıyor — tamamen İngilizce'ye sabit, dil parametresi yok.** Türkçe
  motoruna hiç referans yok.
- `commands.rs` — Tauri komutları (JS↔Rust köprüsü); `Dialect` tipi
  (`harper_core::Dialect`) Türkçe karşılığı olmadan JS tarafına (`client.ts`)
  aktarılıyor.
- `config/` — `mod.rs` (Config struct + `create_linter`/
  `dictionary_from_user_dictionary` — **İngilizce curated dictionary'e
  sabit**), `error.rs`, `integration.rs` (`curated_integrations()` sadece
  macOS bundle ID'leri — TextEdit, Mail, Slack vb., platform-özel).
- `debounce.rs`, `color.rs`, `lint_kind_color.rs` (LintKind→RGB, JS
  tarafıyla senkron tutulmalı), `rect.rs`.
- `os_broker.rs` — `OsBroker` trait'i; **`NoopBroker`** macOS-dışı
  platformlar için no-op — **Windows/Linux'ta highlighter tamamen
  işlevsiz**, sadece macOS gerçek implementasyona (`mac_broker/`) sahip.
- `tray.rs`, `windows.rs` — sistem tepsisi ve pencere yönetimi.
- `highlighter/` — `mod.rs`, `window.rs` (winit/egui/wgpu şeffaf overlay),
  `window_manager.rs` (16.67ms'de bir dikdörtgen güncelleme), `render_state.rs`
  (vurgu/popup çizimi, `LintKind`'a göre renk teması — TR `Usage`/
  `Redundancy` zaten var olan renklere düşüyor, ek değişiklik gerekmiyor),
  `error.rs`.
- `highlighter_service/` — alt-süreç yönetimi (`mod.rs`,
  `highlighter_process.rs`, `highlighter_worker.rs`, ayrı tokio runtime).
- `communication/` — newline-delimited JSON IPC protokolü (`client.rs`,
  `server.rs`, `message.rs`, `framing.rs`, `error.rs`) — yüksek test
  kapsamı, round-trip testleri.
- `mac_broker/` — **`OsBroker`'ın tek gerçek implementasyonu**, sadece
  macOS: `mod.rs`, `accessibility_activation.rs`, `accessibility_text.rs`,
  `app_catalog.rs`, `app_icons.rs`, `app_search_index.rs`,
  `core_foundation_utilities.rs`, `focused_window_pid.rs`,
  `window_stability.rs` — AX (Accessibility) API'siyle ekran-üstü vurgulama.
- `build.rs`, `Cargo.toml`, `tauri.conf.json`, `capabilities/default.json`.

### 15.2 harper-desktop/src/ (SvelteKit frontend)

- `routes/+page.svelte`, `+layout.ts` — SPA giriş noktası.
- `lib/EditorView.svelte` — `harper.js` `WorkerLinter` + `harper-editor`
  `Editor` bileşeni.
- `lib/client.ts` — Tauri `invoke()` sarmalayıcıları; `RustDialect ↔
  Dialect` dönüşümü (Türkçe için genişletilmesi gereken nokta).
- `lib/DesktopUpdater.ts` — otomatik güncelleme mantığı.
- `lib/settings/` — `SettingsApp.svelte`, `SettingsSidebar.svelte`,
  `settings-data.ts` (**mock veri** — gerçek kural kataloğuyla bağlantısız
  görsel prototip), `settings.css`, `components/` (AppIcon, AppPickerModal).
  **Sayfalar**: `GettingStartedPage` (macOS'a özel onboarding),
  `GeneralPage` (**`DIALECT_OPTIONS`** — American/British/Canadian/
  Australian/Indian, **Türkçe seçeneği yok**), `DictionaryPage` (kullanıcı
  sözlüğü — gerçek, çalışan), `RulesPage` (**gerçek entegre** —
  `getStructuredLintConfig()`'ten kural kataloğunu çekiyor; **Türkçe
  kuralları `default_config.json`'da kayıtlıysa burada otomatik listelenir**,
  ekstra kod gerekmez), `ShortcutsPage`/`WeirpacksPage` (tamamen mock/statik,
  "Not wired yet"), `WritingPage` (boş placeholder), `IntegrationsPage`,
  `AboutPage`.

**Türkçe motoruna bağlamak için somut değişiklik noktaları (öncelik
sırasıyla):** (1) `config/mod.rs`'te `create_linter()`'ı dil parametresi
alacak şekilde genişlet, (2) `Dialect`'ten ayrı bir "dil" kavramı ekle
(Rust + `client.ts` + `GeneralPage.svelte`'te `DIALECT_OPTIONS` yanına),
(3) Windows'ta highlighter'ın çalışması isteniyorsa `OsBroker`'ın Windows
implementasyonunu yaz (ayrı, büyük bir iş — UI Automation API'si
kullanılabilir, bkz. sohbetin başındaki GhostEdit araştırması).

### 15.3 Küçük crate'ler (özet — tam liste ajan raporunda)

- **harper-comments**: ~25 tree-sitter dil grameri, `CommentParser`,
  dile özel yorum ayrıştırıcıları (Go/JavaDoc/JsDoc/Lua/Solidity/Unit).
- **harper-pos-utils**: `Chunker`/`Tagger` trait'leri, `BrillChunker`/
  `BurnChunker` (nöral, `burn` framework), `training` feature'ı altında tam
  eğitim döngüsü, CoNLL-U desteği (`conllu_utils.rs`) — **Türkçe UD
  treebank'i doğrudan besleyebilir**, veri formatı sorunu yok.
- **harper-brill**: `brill_tagger()`/`brill_chunker()`/`burn_chunker()` —
  tamamen İngilizce'ye sabit gömülü modeller; `Document::parse()` bunları
  koşulsuz çağırıyor (Türkçe `Document::new_lexicon` bunu atlıyor, bkz. §2.2).
- **harper-wasm**: Ana WASM API (`Linter`, `Language` enum, `is_likely_
  english`/`isolate_english` — dil kapısı burada).
- **harper-typst**, **harper-tex**, **harper-ls**, **harper-cli**,
  **harper-thesaurus**, **harper-stats**, **harper-ink**,
  **harper-jjdescription**, **harper-html**, **harper-python**,
  **harper-asciidoc**, **harper-git-commit**, **harper-literate-haskell**,
  **harper-tree-sitter**, **harper-dictionary-wordlist**, **fuzz** — bkz.
  §4-10, dosya bazlı ayrıntılar için ajan çıktısı arşivde.

### 15.4 Kök dizin dosyaları (özet)

`Cargo.toml` (workspace, 21 crate üye), `README.md`, `ARCHITECTURE.md`/
`CONTRIBUTING.md` (placeholder), `COMPARISON.md`, `demo.md`, `AGENTS.md`
(ajanlar için dokümantasyon haritası + harper-desktop rehberi),
`AGENT_POLICY.md` (proje sahibi Elijah Potter'ın LLM/ajan PR politikası —
kısa, gerekçeli, dürüst PR'lar; düşük kaliteli ajan kodu genelde
reddediliyor), `justfile` (~50 dev komutu), `flake.nix`, `rust-toolchain.toml`
(stable + wasm32-unknown-unknown), `biome.json`, `pnpm-workspace.yaml`,
`.github/workflows/` (binaries, build_web, chrome_plugin, just_checks —ana CI
matrix'i, `check-desktop` dahil—, vscode_plugin, wp_plugin, stale, dependabot),
`.buildkite/` (macOS imzalama/notarization pipeline'ı).



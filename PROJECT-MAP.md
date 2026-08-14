# Harper Türkçe Fork'u — Proje Mimarisi Haritası

> Bu dosya `CLAUDE.md`'deki kurala göre her önemli değişiklikte güncellenmelidir.
> Son güncelleme: 2026-08-14 (`lex_plural_digit` yalnızca rakam+s; `asıl esas` çalışır).

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

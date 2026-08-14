# Proje Kuralları — Harper Türkçe Fork'u

## Proje Haritasını Güncel Tut (ZORUNLU KURAL)

`PROJECT-MAP.md` (bu klasörün kökünde), projenin tüm crate'lerini ve
modüllerini kapsayan derinlemesine bir mimari haritadır.

**Bu projede yeni bir dosya eklendiğinde, bir modül değiştirildiğinde veya
mimaride bir değişiklik yapıldığında `PROJECT-MAP.md` MUTLAKA güncellenmelidir.**
Özellikle:

- Yeni bir Türkçe kural modülü eklendiğinde (`harper-core/src/linting/`)
- `default_config.json` veya `lint_group/mod.rs`'e yeni bir kayıt eklendiğinde
- Sözlük sistemi (`spell/`) değiştirildiğinde veya Türkçe sözlük entegrasyonu
  ilerledi kçe
- POS etiketleyici (`harper-brill`) için Türkçe çalışması başladığında
- Yeni bir crate/paket eklendiğinde veya mevcut birinin amacı değiştiğinde

Harita eskimeye bırakılmamalı — her önemli değişiklikten sonra ilgili bölüm
elden geçirilmeli. Amaç: projeye her dönüşte (yeni oturum, yeni katkı) mevcut
durumun tek bakışta anlaşılabilmesi.

## Türkçe çalışma alanı

Asıl iş bu repoda Türkçe dil desteği üretmektir. Kod **yalnızca burada** yazılır.

- Motor kodu Harper ağacında kalır (`harper-core/src/linting/turkish_*.rs` vb.).
- Fork notları, WASM doğrulama scriptleri ve ileride sözlük/POS verisi:
  `turkish/` — bkz. `turkish/README.md` ve `turkish/FORK-NOTES.md`.
- `D:\Projeler\Harper türkçe projesi` (GhostEdit) **kaynak arşivdir**: işe
  yarar özellikler buradan okunup Harper’a uyarlanır. Oraya yeni kural,
  sözlük veya özellik eklenmez; dosya kopyalanmaz, fikir/veri/kalıp çekilir.

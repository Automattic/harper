# Masaüstü (faz 6)

GhostEdit’teki Electron + PowerShell UI Automation köprüsü **kopyalanmaz**.
Windows üzerinde metin okuma `harper-desktop` highlighter + OS broker
üzerinden ilerlemelidir.

Durum (2026-08-14):

- `harper-desktop` highlighter macOS’ta Accessibility ile çalışır.
- Windows/Linux tarafı `NoopBroker` — ekran metni okunmaz.
- Overlay, öneri penceresi ve IPC zaten Harper’da var; TR profili
  `Config::create_linter()` / `create_dictionary()` içine `LintProfile::Turkish`
  ve `turkish_dictionary()` bağlanınca masaüstüne düşer.

Yapılacak (ayrı iş): Windows UI Automation veya UI Automation köprüsünü
`os_broker` trait’ine uyarlamak. GhostEdit `ui-automation-bridge.ps1` yalnızca
referans davranıştır.

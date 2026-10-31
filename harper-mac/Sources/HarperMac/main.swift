// main.swift
//
// Entry point. `.accessory` means the app has no Dock icon and no main window; it lives only in
// the menu bar.

import AppKit

let app = NSApplication.shared
let delegate = MainActor.assumeIsolated { AppDelegate() }
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()

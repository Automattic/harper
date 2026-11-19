// Settings.swift
//
// The few things the user can change from the menu bar: the master on/off switch, which
// spelling dialect to use, and which apps to check in. They are saved with UserDefaults,
// so they are remembered between launches.

import Foundation

/// User preferences, persisted in UserDefaults.
final class Settings {
    static let shared = Settings()
    private let defaults = UserDefaults.standard

    static let dialects = ["American", "British", "Australian", "Canadian", "Indian"]

    /// Apps where checking is off until the user turns it on. Terminals expose their whole
    /// scrollback as one text area, which would be checked on every keystroke.
    private static let defaultDisabled: [String] = [
        "com.apple.Terminal", "com.googlecode.iterm2", "dev.warp.Warp-Stable",
        "com.mitchellh.ghostty", "net.kovidgoyal.kitty", "org.alacritty",
        "com.apple.dt.Xcode", "com.microsoft.VSCode", "com.todesktop.230313mzl4w4u92",
    ]

    var enabled: Bool {
        get { defaults.object(forKey: "enabled") as? Bool ?? true }
        set { defaults.set(newValue, forKey: "enabled") }
    }

    var dialect: String {
        get { defaults.string(forKey: "dialect") ?? "British" }
        set { defaults.set(newValue, forKey: "dialect") }
    }

    /// Bundle IDs the user has explicitly turned on / off, overriding the defaults.
    private var overrides: [String: Bool] {
        get { defaults.dictionary(forKey: "appOverrides") as? [String: Bool] ?? [:] }
        set { defaults.set(newValue, forKey: "appOverrides") }
    }

    func isEnabled(bundleID: String?) -> Bool {
        guard enabled else { return false }
        guard let id = bundleID else { return true }
        if let o = overrides[id] { return o }
        return !Self.defaultDisabled.contains(id)
    }

    func setEnabled(_ on: Bool, bundleID: String) {
        var o = overrides
        o[bundleID] = on
        overrides = o
    }
}

// AppDelegate.swift
//
// App start-up and the menu-bar icon. On launch it starts Harper, the browser bridge and the
// checker, asks for Accessibility permission, and builds the menu (on/off, per-app switch,
// spelling dialect, open at login, quit).
//
// Command-line options, handy for testing: --port=<n> (bridge port), --allow-origin=<url>
// (let a test web page talk to the bridge) and --no-prompt (don't show the permission dialog).

import AppKit
import ServiceManagement

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private var statusItem: NSStatusItem!
    private var server: HarperServer!
    private var checker: Checker!
    private var bridge: Bridge!
    private var targetApp: NSRunningApplication?

    /// `--name=value` from the command line (used for testing the bridge).
    private static func argument(_ name: String) -> String? {
        CommandLine.arguments.first { $0.hasPrefix("--\(name)=") }.map { String($0.dropFirst(name.count + 3)) }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        server = HarperServer(dialect: { Settings.shared.dialect })
        let port = Self.argument("port").flatMap(UInt16.init) ?? Bridge.defaultPort
        let origins = CommandLine.arguments.filter { $0.hasPrefix("--allow-origin=") }.map { String($0.dropFirst(15)) }
        bridge = Bridge(server: server, port: port, extraOrigins: origins)
        checker = Checker(server: server, browserHandles: { [bridge] id in bridge?.isHandling(bundleID: id) ?? false })

        if HarperServer.binaryURL() == nil {
            let alert = NSAlert()
            alert.messageText = "harper-ls not found"
            alert.informativeText = "Run tools/fetch_harper.sh, then build again."
            alert.runModal()
            NSApp.terminate(nil)
            return
        }

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        statusItem.button?.image = NSImage(systemSymbolName: "text.badge.checkmark",
                                           accessibilityDescription: "Harper")
        let menu = NSMenu()
        menu.delegate = self
        statusItem.menu = menu

        NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification, object: nil, queue: .main
        ) { [weak self] note in
            let app = note.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication
            MainActor.assumeIsolated {
                if let app, app.processIdentifier != ProcessInfo.processInfo.processIdentifier {
                    self?.targetApp = app
                }
            }
        }

        if !AX.isTrusted(prompt: !CommandLine.arguments.contains("--no-prompt")) { NSLog("Waiting for Accessibility permission") }
        server.start()
        bridge.start()
        checker.start()
    }

    func applicationWillTerminate(_ notification: Notification) {}

    // MARK: - Menu

    func menuNeedsUpdate(_ menu: NSMenu) {
        menu.removeAllItems()

        if !AX.isTrusted(prompt: false) {
            let item = NSMenuItem(title: "Grant Accessibility access…", action: #selector(openAccessibility), keyEquivalent: "")
            item.target = self
            menu.addItem(item)
            menu.addItem(.separator())
        }

        let enabled = NSMenuItem(title: "Check my writing", action: #selector(toggleEnabled), keyEquivalent: "")
        enabled.target = self
        enabled.state = Settings.shared.enabled ? .on : .off
        menu.addItem(enabled)

        if let app = targetApp, let id = app.bundleIdentifier, let name = app.localizedName {
            let item = NSMenuItem(title: "Check in \(name)", action: #selector(toggleApp(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = id
            item.state = Settings.shared.isEnabled(bundleID: id) ? .on : .off
            menu.addItem(item)
        }

        let dialect = NSMenuItem(title: "Spelling", action: nil, keyEquivalent: "")
        let sub = NSMenu()
        for d in Settings.dialects {
            let item = NSMenuItem(title: d, action: #selector(setDialect(_:)), keyEquivalent: "")
            item.target = self
            item.state = d == Settings.shared.dialect ? .on : .off
            sub.addItem(item)
        }
        dialect.submenu = sub
        menu.addItem(dialect)

        menu.addItem(.separator())
        let login = NSMenuItem(title: "Open at login", action: #selector(toggleLogin), keyEquivalent: "")
        login.target = self
        login.state = SMAppService.mainApp.status == .enabled ? .on : .off
        menu.addItem(login)
        menu.addItem(NSMenuItem(title: "Quit Harper", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))
    }

    @objc private func openAccessibility() {
        NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!)
    }

    @objc private func toggleEnabled() {
        Settings.shared.enabled.toggle()
    }

    @objc private func toggleApp(_ sender: NSMenuItem) {
        guard let id = sender.representedObject as? String else { return }
        Settings.shared.setEnabled(!Settings.shared.isEnabled(bundleID: id), bundleID: id)
    }

    @objc private func setDialect(_ sender: NSMenuItem) {
        Settings.shared.dialect = sender.title
        server.restart()
        checker.recheck()
    }

    @objc private func toggleLogin() {
        do {
            if SMAppService.mainApp.status == .enabled { try SMAppService.mainApp.unregister() }
            else { try SMAppService.mainApp.register() }
        } catch {
            NSLog("login item change failed: \(error)")
        }
    }
}

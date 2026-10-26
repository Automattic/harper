// Checker.swift
//
// The brain of the app. Five times a second it looks at whatever text field you are typing in
// and, when the text has changed, waits for you to pause and then asks Harper to check it. The
// results become underlines (Overlay) and, when you click one, a popup (SuggestionPanel).
//
// Browsers are special: if a browser extension is already handling the field (see Bridge.swift),
// this class stays out of the way so nothing is underlined twice.

import AppKit

/// Ties everything together: polls the focused field, asks Harper, draws the result,
/// and handles clicks on underlined words.
@MainActor
final class Checker {
    let server: HarperServer
    private let overlay = Overlay()
    private let popup = SuggestionPanel()

    private var timer: Timer?
    private var clickMonitor: Any?

    private var field: FocusedField?
    private var lints: [Lint] = []
    private var markers: [Marker] = []
    private var checkedText: String?
    private var generation = 0
    private var checkPending = false
    private var selected: Marker?

    /// Asks whether a browser extension is already handling the focused field in that app.
    private let browserHandles: (String?) -> Bool

    init(server: HarperServer, browserHandles: @escaping (String?) -> Bool) {
        self.server = server
        self.browserHandles = browserHandles
        popup.onChoose = { [weak self] action in self?.perform(action) }
    }

    func start() {
        timer = Timer.scheduledTimer(withTimeInterval: 0.2, repeats: true) { [weak self] _ in
            MainActor.assumeIsolated { self?.tick() }
        }
        clickMonitor = NSEvent.addGlobalMonitorForEvents(matching: .leftMouseUp) { [weak self] _ in
            MainActor.assumeIsolated { self?.handleClick(at: NSEvent.mouseLocation) }
        }
    }

    /// Forgets the current result and checks the field again, e.g. after a dialect change.
    func recheck() {
        checkedText = nil
        lints = []
        markers = []
        overlay.clear()
    }

    // MARK: - Polling

    private func tick() {
        guard Settings.shared.enabled, AX.isTrusted(prompt: false),
              let current = AX.focusedField(),
              Settings.shared.isEnabled(bundleID: current.bundleID),
              !browserHandles(current.bundleID) else {
            reset()
            return
        }

        let sameField = field.map { CFEqual($0.element, current.element) } ?? false
        if !sameField {
            lints = []
            checkedText = nil
            popup.hide()
        }
        field = current

        if current.text != checkedText {
            if let old = checkedText, sameField {
                lints = Self.carryOver(lints, from: old, to: current.text)
            }
            if !sameField || checkedText != nil { popup.hide() }
            checkedText = current.text
            scheduleCheck(current.text)
        }

        refreshMarkers(for: current)
    }

    private func reset() {
        guard field != nil || !markers.isEmpty else { return }
        field = nil
        lints = []
        markers = []
        checkedText = nil
        generation += 1
        popup.hide()
        overlay.clear()
    }

    /// Debounced: waits for a pause in typing before bothering the server.
    private func scheduleCheck(_ text: String) {
        generation += 1
        let mine = generation
        checkPending = true
        Task { [weak self] in
            try? await Task.sleep(nanoseconds: 300_000_000)
            guard let self, self.generation == mine else { return }
            let result = await self.server.check(text)
            guard self.generation == mine else { return }
            self.checkPending = false
            if let result { self.lints = Array(result.prefix(60)) }
        }
    }

    private func refreshMarkers(for current: FocusedField) {
        let text = current.text as NSString
        var out: [Marker] = []
        for lint in lints where NSMaxRange(lint.range) <= text.length {
            let rects = AX.lineRects(current.element, range: lint.range, text: text)
                .filter { $0.intersects(current.frame) }
            if !rects.isEmpty { out.append(Marker(lint: lint, rects: rects)) }
        }
        markers = out
        overlay.show(out, clippedTo: current.frame)
    }

    /// Keeps lints that the latest keystrokes did not touch, shifted to their new positions,
    /// so underlines don't flicker away while typing.
    private static func carryOver(_ lints: [Lint], from old: String, to new: String) -> [Lint] {
        let o = Array(old.utf16), n = Array(new.utf16)
        var prefix = 0
        while prefix < o.count, prefix < n.count, o[prefix] == n[prefix] { prefix += 1 }
        var suffix = 0
        while suffix < o.count - prefix, suffix < n.count - prefix,
              o[o.count - 1 - suffix] == n[n.count - 1 - suffix] { suffix += 1 }
        let oldEnd = o.count - suffix
        let delta = n.count - o.count
        return lints.compactMap { lint in
            if NSMaxRange(lint.range) < prefix { return lint }
            if lint.range.location > oldEnd {
                return Lint(range: NSRange(location: lint.range.location + delta, length: lint.range.length),
                            original: lint.original, message: lint.message, code: lint.code,
                            diagnostic: lint.diagnostic)
            }
            return nil
        }
    }

    // MARK: - Clicks

    private func handleClick(at point: CGPoint) {
        guard !checkPending, field != nil else { return }
        if popup.isVisible, popup.frame.contains(point) { return }
        let hit = markers.first { marker in
            marker.rects.contains { Geometry.cocoa($0).insetBy(dx: -2, dy: -3).contains(point) }
        }
        guard let hit else { return }
        selected = hit
        Task { [weak self] in
            guard let self else { return }
            let actions = await self.server.actions(for: hit.lint)
            guard !actions.isEmpty, !self.checkPending, self.selected?.lint.range == hit.lint.range else { return }
            let rect = hit.rects.first { Geometry.cocoa($0).insetBy(dx: -2, dy: -3).contains(point) } ?? hit.rects[0]
            self.popup.show(lint: hit.lint, actions: actions, below: rect)
        }
    }

    private func perform(_ action: FixAction) {
        guard let marker = selected, let element = field?.element else { return }
        switch action.kind {
        case .replace(let text):
            Task {
                let ok = await AX.replace(in: element, range: marker.lint.range,
                                          expecting: marker.lint.original, with: text)
                if !ok { NSSound.beep() }
            }
        case .command(let name, let args):
            server.run(command: name, arguments: args)
            // Dictionary and ignore changes only show up once Harper re-reads the text.
            let text = checkedText
            Task { [weak self] in
                try? await Task.sleep(nanoseconds: 250_000_000)
                guard let self, let text, self.checkedText == text else { return }
                self.scheduleCheck(text)
            }
        }
    }
}

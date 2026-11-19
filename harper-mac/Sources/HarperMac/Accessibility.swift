// Accessibility.swift
//
// How the app "sees" and edits text in other apps. macOS has an Accessibility API (the same one
// screen readers use) that lets a trusted app ask another app things like "what text is in the
// focused field?" and "where on screen is characters 10 to 15?". The user has to approve this in
// System Settings > Privacy & Security > Accessibility.
//
// Coordinates: the Accessibility API measures from the TOP-left of the main screen, while
// Cocoa windows measure from the BOTTOM-left. See `Geometry` in Overlay.swift for the conversion.

import AppKit
import ApplicationServices

/// The text field the user is currently typing in, read through the Accessibility API.
struct FocusedField {
    let element: AXUIElement
    let pid: pid_t
    let bundleID: String?
    let text: String
    let frame: CGRect   // AX coordinates: global, origin top-left of the primary screen
}

enum AX {
    static func isTrusted(prompt: Bool) -> Bool {
        AXIsProcessTrustedWithOptions(["AXTrustedCheckOptionPrompt" as CFString: prompt] as CFDictionary)
    }

    private static var enhancedPIDs = Set<pid_t>()

    /// Reads the focused text field of the frontmost app, or nil if there isn't a checkable one.
    static func focusedField() -> FocusedField? {
        guard let app = NSWorkspace.shared.frontmostApplication,
              app.processIdentifier != ProcessInfo.processInfo.processIdentifier else { return nil }
        let pid = app.processIdentifier
        let appElement = AXUIElementCreateApplication(pid)
        AXUIElementSetMessagingTimeout(appElement, 0.25)

        // Chromium and Electron only build their accessibility tree when asked.
        if !enhancedPIDs.contains(pid) {
            enhancedPIDs.insert(pid)
            AXUIElementSetAttributeValue(appElement, "AXManualAccessibility" as CFString, kCFBooleanTrue)
        }

        guard let element: AXUIElement = attribute(appElement, kAXFocusedUIElementAttribute) else { return nil }
        AXUIElementSetMessagingTimeout(element, 0.25)

        let role: String = attribute(element, kAXRoleAttribute) ?? ""
        let subrole: String = attribute(element, kAXSubroleAttribute) ?? ""
        if subrole == kAXSecureTextFieldSubrole as String { return nil }

        let textRoles: Set<String> = [kAXTextAreaRole as String, kAXTextFieldRole as String, kAXComboBoxRole as String]
        let hasSelection: CFTypeRef? = attribute(element, kAXSelectedTextRangeAttribute)
        guard textRoles.contains(role) || hasSelection != nil else { return nil }

        guard let text: String = attribute(element, kAXValueAttribute),
              !text.isEmpty, text.utf16.count <= 30_000 else { return nil }

        // URL bars and the like: one run of characters with no spaces and URL punctuation.
        if !text.contains(where: \.isWhitespace), text.contains(where: { "/.@:".contains($0) }) { return nil }

        guard let frame = frame(of: element) else { return nil }
        return FocusedField(element: element, pid: pid, bundleID: app.bundleIdentifier, text: text, frame: frame)
    }

    static func attribute<T>(_ element: AXUIElement, _ name: String) -> T? {
        var out: CFTypeRef?
        guard AXUIElementCopyAttributeValue(element, name as CFString, &out) == .success else { return nil }
        return out as? T
    }

    static func frame(of element: AXUIElement) -> CGRect? {
        var pos = CGPoint.zero, size = CGSize.zero
        guard let p: AXValue = attribute(element, kAXPositionAttribute),
              let s: AXValue = attribute(element, kAXSizeAttribute),
              AXValueGetValue(p, .cgPoint, &pos), AXValueGetValue(s, .cgSize, &size) else { return nil }
        return CGRect(origin: pos, size: size)
    }

    // MARK: - Geometry

    private static func bounds(_ element: AXUIElement, _ range: NSRange) -> CGRect? {
        var cf = CFRange(location: range.location, length: range.length)
        guard let value = AXValueCreate(.cfRange, &cf) else { return nil }
        var out: CFTypeRef?
        guard AXUIElementCopyParameterizedAttributeValue(
            element, kAXBoundsForRangeParameterizedAttribute as CFString, value, &out) == .success,
              let v = out, CFGetTypeID(v) == AXValueGetTypeID() else { return nil }
        var rect = CGRect.zero
        guard AXValueGetValue(v as! AXValue, .cgRect, &rect), rect.width >= 1, rect.height >= 1 else { return nil }
        return rect
    }

    /// One rect per visual line the range occupies (a word can wrap in a narrow field).
    static func lineRects(_ element: AXUIElement, range: NSRange, text: NSString) -> [CGRect] {
        guard let whole = bounds(element, range) else { return [] }
        guard range.length > 1,
              let first = bounds(element, NSRange(location: range.location, length: 1)),
              whole.height > first.height * 1.5 else { return [whole] }

        var lines: [CGRect] = []
        for i in range.location..<NSMaxRange(range) {
            let ch = text.substring(with: NSRange(location: i, length: 1))
            guard ch.rangeOfCharacter(from: .whitespacesAndNewlines) == nil,
                  let r = bounds(element, NSRange(location: i, length: 1)) else { continue }
            if let last = lines.last, abs(last.minY - r.minY) < first.height * 0.5 {
                lines[lines.count - 1] = last.union(r)
            } else {
                lines.append(r)
            }
        }
        return lines
    }

    // MARK: - Editing

    /// Replaces `range` of the field's text. Tries the AX route first, then paste.
    static func replace(in element: AXUIElement, range: NSRange, expecting original: String,
                        with replacement: String) async -> Bool {
        guard let before: String = attribute(element, kAXValueAttribute),
              (before as NSString).length >= NSMaxRange(range),
              (before as NSString).substring(with: range) == original else { return false }
        let expected = (before as NSString).replacingCharacters(in: range, with: replacement)

        var cf = CFRange(location: range.location, length: range.length)
        guard let sel = AXValueCreate(.cfRange, &cf),
              AXUIElementSetAttributeValue(element, kAXSelectedTextRangeAttribute as CFString, sel) == .success
        else { return false }

        if AXUIElementSetAttributeValue(element, kAXSelectedTextAttribute as CFString, replacement as CFString) == .success {
            try? await Task.sleep(nanoseconds: 80_000_000)
            if let after: String = attribute(element, kAXValueAttribute), after == expected { return true }
        }

        // Fallback for apps that accept the call but ignore it (browsers, some Electron apps).
        return await paste(replacement)
    }

    private static func paste(_ string: String) async -> Bool {
        let pb = NSPasteboard.general
        let saved: [[(NSPasteboard.PasteboardType, Data)]] = (pb.pasteboardItems ?? []).map { item in
            item.types.compactMap { t in item.data(forType: t).map { (t, $0) } }
        }
        pb.clearContents()
        pb.setString(string, forType: .string)

        let src = CGEventSource(stateID: .combinedSessionState)
        let down = CGEvent(keyboardEventSource: src, virtualKey: 9, keyDown: true)   // "v"
        let up = CGEvent(keyboardEventSource: src, virtualKey: 9, keyDown: false)
        down?.flags = .maskCommand
        up?.flags = .maskCommand
        down?.post(tap: .cghidEventTap)
        up?.post(tap: .cghidEventTap)

        try? await Task.sleep(nanoseconds: 200_000_000)
        pb.clearContents()
        for item in saved {
            let restored = NSPasteboardItem()
            for (type, data) in item { restored.setData(data, forType: type) }
            pb.writeObjects([restored])
        }
        return true
    }
}

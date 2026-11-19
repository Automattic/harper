// SuggestionPanel.swift
//
// The small popup that appears when you click an underlined word: the explanation, a few
// suggested replacements, and "add to dictionary" / "ignore". It is built in code (no storyboard)
// and is "non-activating", which means clicking it does not pull keyboard focus away from the
// app you are typing in.

import AppKit

/// The popup shown when you click an underlined word. Non-activating, so the text field
/// you are editing keeps keyboard focus the whole time.
@MainActor
final class SuggestionPanel {
    private let panel: NSPanel
    private var monitor: Any?
    var onChoose: ((FixAction) -> Void)?

    var isVisible: Bool { panel.isVisible }
    var frame: CGRect { panel.frame }

    init() {
        panel = NSPanel(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel],
                        backing: .buffered, defer: false)
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.level = .popUpMenu
        panel.hidesOnDeactivate = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
    }

    func show(lint: Lint, actions: [FixAction], below rect: CGRect) {
        var rows: [(String, FixAction)] = []
        for a in actions {
            switch a.kind {
            case .replace(let text):
                if rows.filter({ if case .replace = $0.1.kind { return true } else { return false } }).count < 5 {
                    rows.append((text.isEmpty ? a.title : text, a))
                }
            case .command:
                let t = a.title.lowercased()
                if t.contains("user dictionary") || t.hasPrefix("ignore") { rows.append((a.title, a)) }
            }
        }

        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 6
        stack.edgeInsets = NSEdgeInsets(top: 10, left: 12, bottom: 10, right: 12)

        let message = NSTextField(wrappingLabelWithString: lint.message)
        message.font = .systemFont(ofSize: 12)
        message.textColor = .secondaryLabelColor
        message.preferredMaxLayoutWidth = 260
        stack.addArrangedSubview(message)

        for (index, (label, action)) in rows.enumerated() {
            let button = ActionButton(title: label, action: action) { [weak self] a in
                self?.hide()
                self?.onChoose?(a)
            }
            if case .replace = action.kind {
                button.font = .systemFont(ofSize: 13, weight: .semibold)
            } else {
                button.font = .systemFont(ofSize: 11)
            }
            if index > 0, case .command = action.kind, case .replace = rows[index - 1].1.kind {
                stack.setCustomSpacing(10, after: stack.arrangedSubviews.last!)
            }
            stack.addArrangedSubview(button)
        }

        let background = NSVisualEffectView()
        background.material = .popover
        background.state = .active
        background.wantsLayer = true
        background.layer?.cornerRadius = 10
        background.layer?.masksToBounds = true
        background.addSubview(stack)
        stack.translatesAutoresizingMaskIntoConstraints = false
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: background.leadingAnchor),
            stack.trailingAnchor.constraint(equalTo: background.trailingAnchor),
            stack.topAnchor.constraint(equalTo: background.topAnchor),
            stack.bottomAnchor.constraint(equalTo: background.bottomAnchor),
        ])
        panel.contentView = background
        let size = stack.fittingSize

        // Place under the word, clamped onto the screen that holds it.
        let cocoaRect = Geometry.cocoa(rect)
        let screen = NSScreen.screens.first { $0.frame.intersects(cocoaRect) } ?? NSScreen.main
        let visible = screen?.visibleFrame ?? .zero
        var origin = CGPoint(x: cocoaRect.minX, y: cocoaRect.minY - size.height - 4)
        if origin.y < visible.minY { origin.y = cocoaRect.maxY + 4 }
        origin.x = max(visible.minX + 4, min(origin.x, visible.maxX - size.width - 4))
        panel.setFrame(CGRect(origin: origin, size: size), display: true)
        panel.orderFrontRegardless()

        // Any click outside dismisses it.
        removeMonitor()
        monitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] _ in
            MainActor.assumeIsolated {
                guard let self, !self.panel.frame.contains(NSEvent.mouseLocation) else { return }
                self.hide()
            }
        }
    }

    func hide() {
        panel.orderOut(nil)
        removeMonitor()
    }

    private func removeMonitor() {
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
    }
}

private final class ActionButton: NSButton {
    private let fix: FixAction
    private let handler: (FixAction) -> Void

    init(title: String, action: FixAction, handler: @escaping (FixAction) -> Void) {
        self.fix = action
        self.handler = handler
        super.init(frame: .zero)
        self.title = title
        isBordered = false
        alignment = .left
        contentTintColor = .labelColor
        target = self
        self.action = #selector(clicked)
    }

    required init?(coder: NSCoder) { fatalError() }

    @objc private func clicked() { handler(fix) }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
}

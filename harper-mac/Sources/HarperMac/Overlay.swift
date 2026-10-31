// Overlay.swift
//
// Draws the wavy underlines on top of other apps' windows. We can't draw inside another app, so
// instead we place a transparent, click-through window over each screen and paint the squiggles
// on that. Because the window ignores the mouse, you can still click and type in the app below.

import AppKit

/// A lint plus where it currently sits on screen (AX coordinates, one rect per visual line).
struct Marker {
    let lint: Lint
    let rects: [CGRect]
}

/// Converts between AX coordinates (global, top-left origin) and Cocoa (bottom-left origin).
enum Geometry {
    static var primaryHeight: CGFloat { NSScreen.screens.first?.frame.height ?? 0 }

    static func cocoa(_ r: CGRect) -> CGRect {
        CGRect(x: r.minX, y: primaryHeight - r.maxY, width: r.width, height: r.height)
    }
}

/// Transparent, click-through windows (one per screen) that draw squiggles over other apps' text.
@MainActor
final class Overlay {
    private var windows: [(NSPanel, OverlayView)] = []

    init() {
        rebuild()
        NotificationCenter.default.addObserver(
            forName: NSApplication.didChangeScreenParametersNotification, object: nil, queue: .main
        ) { [weak self] _ in MainActor.assumeIsolated { self?.rebuild() } }
    }

    private func rebuild() {
        windows.forEach { $0.0.orderOut(nil) }
        windows = NSScreen.screens.map { screen in
            let panel = NSPanel(contentRect: screen.frame, styleMask: [.borderless, .nonactivatingPanel],
                                backing: .buffered, defer: false)
            panel.isOpaque = false
            panel.backgroundColor = .clear
            panel.hasShadow = false
            panel.ignoresMouseEvents = true
            panel.level = .statusBar
            panel.hidesOnDeactivate = false
            panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary, .ignoresCycle]
            let view = OverlayView(frame: CGRect(origin: .zero, size: screen.frame.size))
            view.screenFrame = screen.frame
            panel.contentView = view
            panel.orderFrontRegardless()
            return (panel, view)
        }
    }

    func show(_ markers: [Marker], clippedTo field: CGRect?) {
        for (_, view) in windows {
            view.markers = markers
            view.clip = field
            view.needsDisplay = true
        }
    }

    func clear() { show([], clippedTo: nil) }
}

private final class OverlayView: NSView {
    var markers: [Marker] = []
    var clip: CGRect?
    var screenFrame: CGRect = .zero

    override var isFlipped: Bool { true }

    override func draw(_ dirtyRect: NSRect) {
        guard !markers.isEmpty else { return }
        // View space = AX space shifted so this screen's top-left is (0, 0).
        let originX = screenFrame.minX
        let originY = Geometry.primaryHeight - screenFrame.maxY

        NSGraphicsContext.current?.saveGraphicsState()
        if let clip {
            NSBezierPath(rect: clip.offsetBy(dx: -originX, dy: -originY)).addClip()
        }
        for marker in markers {
            let color: NSColor = marker.lint.code == "SpellCheck" ? .systemRed : .systemBlue
            color.setStroke()
            for rect in marker.rects {
                squiggle(in: rect.offsetBy(dx: -originX, dy: -originY))
            }
        }
        NSGraphicsContext.current?.restoreGraphicsState()
    }

    private func squiggle(in r: CGRect) {
        let path = NSBezierPath()
        path.lineWidth = 1.4
        path.lineCapStyle = .round
        path.lineJoinStyle = .round
        let baseline = r.maxY - 1
        var x = r.minX
        path.move(to: CGPoint(x: x, y: baseline))
        var up = true
        while x < r.maxX {
            x = min(x + 2.5, r.maxX)
            path.line(to: CGPoint(x: x, y: baseline + (up ? -2 : 0)))
            up.toggle()
        }
        path.stroke()
    }
}

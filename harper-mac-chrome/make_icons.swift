// Draws the extension icons. Run: swift make_icons.swift
import AppKit

let out = "icons"
try? FileManager.default.createDirectory(atPath: out, withIntermediateDirectories: true)

for size in [16, 32, 48, 128] {
    let s = CGFloat(size)
    let image = NSImage(size: NSSize(width: s, height: s))
    image.lockFocus()
    let rect = NSRect(x: 0, y: 0, width: s, height: s)
    NSColor(calibratedRed: 0.11, green: 0.11, blue: 0.12, alpha: 1).setFill()
    NSBezierPath(roundedRect: rect, xRadius: s * 0.22, yRadius: s * 0.22).fill()

    let attrs: [NSAttributedString.Key: Any] = [
        .font: NSFont.systemFont(ofSize: s * 0.62, weight: .bold),
        .foregroundColor: NSColor.white,
    ]
    let str = NSAttributedString(string: "H", attributes: attrs)
    let sz = str.size()
    str.draw(at: NSPoint(x: (s - sz.width) / 2, y: (s - sz.height) / 2 + s * 0.03))

    // A small wavy underline, the extension's whole job.
    NSColor(calibratedRed: 0.90, green: 0.28, blue: 0.30, alpha: 1).setStroke()
    let wave = NSBezierPath()
    wave.lineWidth = max(1, s * 0.06)
    let y = s * 0.17, step = s * 0.08
    wave.move(to: NSPoint(x: s * 0.2, y: y))
    var x = s * 0.2, up = true
    while x < s * 0.8 { x += step; wave.line(to: NSPoint(x: x, y: y + (up ? s * 0.05 : 0))); up.toggle() }
    wave.stroke()
    image.unlockFocus()

    let rep = NSBitmapImageRep(data: image.tiffRepresentation!)!
    try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: "\(out)/icon-\(size).png"))
}
print("icons written to \(out)")

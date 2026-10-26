// Bridge.swift
//
// A tiny web server, reachable only from this Mac (127.0.0.1), that the Chrome and Safari
// extensions use to borrow the app's grammar checker. It understands three requests:
//
//   POST /ping     "I'm alive" (and "I'm handling the focused field", so the app steps aside)
//   POST /check    send text, get back the problems and their fixes
//   POST /command  add a word to the dictionary, or ignore a suggestion
//
// Safety: requests must come from a browser extension (not an ordinary web page), must carry a
// custom header, and must be addressed to 127.0.0.1. See `route` for the checks.

import Foundation
import Network

/// A loopback-only HTTP server the browser extensions talk to. The extensions find and
/// underline text inside web pages (far more reliably than Accessibility can); this app
/// stays the single source of truth for checking, dialect and the personal dictionary.
final class Bridge: @unchecked Sendable {
    static let defaultPort: UInt16 = 47321

    private let server: HarperServer
    private let port: UInt16
    private let extraOrigins: [String]
    private let queue = DispatchQueue(label: "harper.bridge")
    private var listener: NWListener?
    private let gate = Gate()

    private let lock = NSLock()
    private var handlingSince: [String: Date] = [:]

    init(server: HarperServer, port: UInt16, extraOrigins: [String]) {
        self.server = server
        self.port = port
        self.extraOrigins = extraOrigins
    }

    // MARK: - Browser presence

    private static let chromiumBundles: Set<String> = [
        "com.google.Chrome", "com.google.Chrome.canary", "com.brave.Browser", "com.microsoft.edgemac",
        "company.thebrowser.Browser", "com.vivaldi.Vivaldi", "org.chromium.Chromium", "com.operasoftware.Opera",
    ]
    private static let safariBundles: Set<String> = ["com.apple.Safari", "com.apple.SafariTechnologyPreview"]

    /// True when an extension in this browser reported, within the last few seconds, that it is
    /// handling the focused field. The Accessibility checker steps aside so nothing is drawn twice.
    func isHandling(bundleID: String?) -> Bool {
        guard let id = bundleID else { return false }
        let family = Self.chromiumBundles.contains(id) ? "chromium" : Self.safariBundles.contains(id) ? "safari" : nil
        guard let family else { return false }
        lock.lock(); defer { lock.unlock() }
        guard let t = handlingSince[family] else { return false }
        return Date().timeIntervalSince(t) < 3.5
    }

    private func markHandling(_ family: String) {
        lock.lock(); handlingSince[family] = Date(); lock.unlock()
    }

    // MARK: - Listener

    func start() {
        let params = NWParameters.tcp
        params.requiredInterfaceType = .loopback
        guard let nwPort = NWEndpoint.Port(rawValue: port),
              let listener = try? NWListener(using: params, on: nwPort) else {
            NSLog("bridge: could not listen on port \(port)")
            return
        }
        listener.newConnectionHandler = { [weak self] conn in self?.accept(conn) }
        listener.stateUpdateHandler = { state in
            if case .failed(let e) = state { NSLog("bridge failed: \(e)") }
        }
        listener.start(queue: queue)
        self.listener = listener
    }

    private func accept(_ conn: NWConnection) {
        conn.start(queue: queue)
        receive(conn, buffer: Data())
    }

    private func receive(_ conn: NWConnection, buffer: Data) {
        conn.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isDone, error in
            guard let self else { conn.cancel(); return }
            var buf = buffer
            if let data { buf.append(data) }
            if let request = Request(parsing: buf) {
                Task {
                    let response = await self.route(request)
                    conn.send(content: response, completion: .contentProcessed { _ in conn.cancel() })
                }
            } else if error != nil || isDone || buf.count > 2_000_000 {
                conn.cancel()
            } else {
                self.receive(conn, buffer: buf)
            }
        }
    }

    // MARK: - Routing

    private struct Request {
        let method: String
        let path: String
        let headers: [String: String]
        let body: Data

        init?(parsing data: Data) {
            guard let split = data.range(of: Data("\r\n\r\n".utf8)) else { return nil }
            let head = String(decoding: data[data.startIndex..<split.lowerBound], as: UTF8.self)
            var lines = head.components(separatedBy: "\r\n")
            let first = lines.removeFirst().split(separator: " ")
            guard first.count >= 2 else { return nil }
            var headers: [String: String] = [:]
            for line in lines {
                guard let colon = line.firstIndex(of: ":") else { continue }
                headers[line[..<colon].lowercased()] = line[line.index(after: colon)...].trimmingCharacters(in: .whitespaces)
            }
            let length = Int(headers["content-length"] ?? "0") ?? 0
            let rest = data[split.upperBound...]
            guard rest.count >= length else { return nil }
            self.method = String(first[0])
            self.path = String(first[1])
            self.headers = headers
            self.body = Data(rest.prefix(length))
        }
    }

    private func originAllowed(_ origin: String) -> Bool {
        origin.hasPrefix("chrome-extension://") || origin.hasPrefix("safari-web-extension://")
            || origin.hasPrefix("moz-extension://") || extraOrigins.contains(origin)
    }

    private func route(_ req: Request) async -> Data {
        let origin = req.headers["origin"]
        let host = req.headers["host"] ?? ""
        // Stops DNS-rebinding: a page on another name can't reach us even if it resolves here.
        guard host == "127.0.0.1:\(port)" || host == "localhost:\(port)" else {
            return Self.response(403, ["error": "bad host"], origin: nil)
        }
        if let origin, !originAllowed(origin) { return Self.response(403, ["error": "origin not allowed"], origin: nil) }

        if req.method == "OPTIONS" { return Self.response(204, nil, origin: origin) }
        // Every non-preflight request must carry our header, which forces a CORS preflight
        // and means a web page cannot send it blind.
        guard req.headers["x-harpermac"] == "1" else { return Self.response(403, ["error": "missing header"], origin: origin) }

        let json = (try? JSONSerialization.jsonObject(with: req.body)) as? [String: Any] ?? [:]
        switch (req.method, req.path) {
        case ("POST", "/ping"):
            if json["handling"] as? Bool == true, let family = json["family"] as? String { markHandling(family) }
            return Self.response(200, ["ok": true, "enabled": Settings.shared.enabled,
                                       "dialect": Settings.shared.dialect], origin: origin)
        case ("POST", "/check"):
            guard let text = json["text"] as? String else { return Self.response(400, ["error": "no text"], origin: origin) }
            guard Settings.shared.enabled else { return Self.response(200, ["enabled": false, "lints": []], origin: origin) }
            guard let lints = await check(text) else { return Self.response(503, ["error": "unavailable"], origin: origin) }
            return Self.response(200, ["enabled": true, "lints": lints], origin: origin)
        case ("POST", "/command"):
            guard let name = json["command"] as? String,
                  ["HarperAddToUserDict", "HarperIgnoreLint"].contains(name) else {
                return Self.response(400, ["error": "unsupported command"], origin: origin)
            }
            server.run(command: name, arguments: json["arguments"] as? [Any] ?? [])
            return Self.response(200, ["ok": true], origin: origin)
        default:
            return Self.response(404, ["error": "not found"], origin: origin)
        }
    }

    /// One check at a time on the shared "web" document, with the fixes for each lint resolved
    /// inside the same turn so they always match the text that was checked.
    private func check(_ text: String) async -> [[String: Any]]? {
        await gate.run { [server] in
            guard let lints = await server.check(text, doc: "web") else { return nil }
            var out: [[String: Any]] = []
            for lint in lints.prefix(60) {
                let actions = await server.actions(for: lint, doc: "web")
                var fixes: [[String: String]] = []
                var commands: [[String: Any]] = []
                for a in actions {
                    switch a.kind {
                    case .replace(let t):
                        if fixes.count < 5, !fixes.contains(where: { $0["text"] == t }) {
                            fixes.append(["label": a.title, "text": t])
                        }
                    case .command(let name, let args):
                        let title = a.title.lowercased()
                        if title.contains("user dictionary") {
                            commands.append(["label": a.title, "command": name, "arguments": args, "kind": "dictionary"])
                        } else if title.hasPrefix("ignore") {
                            commands.append(["label": a.title, "command": name, "arguments": args, "kind": "ignore"])
                        }
                    }
                }
                out.append([
                    "start": lint.range.location, "end": NSMaxRange(lint.range),
                    "message": lint.message, "kind": lint.code,
                    "fixes": fixes, "commands": commands,
                ])
            }
            return out
        }
    }

    private static func response(_ status: Int, _ body: Any?, origin: String?) -> Data {
        let reason = [200: "OK", 204: "No Content", 400: "Bad Request", 403: "Forbidden",
                      404: "Not Found", 503: "Service Unavailable"][status] ?? "OK"
        let payload = body.flatMap { try? JSONSerialization.data(withJSONObject: $0) } ?? Data()
        var head = "HTTP/1.1 \(status) \(reason)\r\nContent-Length: \(payload.count)\r\nConnection: close\r\n"
        if body != nil { head += "Content-Type: application/json\r\n" }
        if let origin {
            head += "Access-Control-Allow-Origin: \(origin)\r\nVary: Origin\r\n"
            head += "Access-Control-Allow-Methods: POST, OPTIONS\r\n"
            head += "Access-Control-Allow-Headers: Content-Type, X-HarperMac\r\n"
            head += "Access-Control-Allow-Private-Network: true\r\n"
        }
        return Data((head + "\r\n").utf8) + payload
    }
}

/// Runs async operations strictly one after another.
private final class Gate: @unchecked Sendable {
    private let lock = NSLock()
    private var tail: Task<Void, Never> = Task {}

    func run<T: Sendable>(_ op: @escaping @Sendable () async -> T) async -> T {
        await enqueue(op).value
    }

    private func enqueue<T: Sendable>(_ op: @escaping @Sendable () async -> T) -> Task<T, Never> {
        lock.lock(); defer { lock.unlock() }
        let previous = tail
        let task = Task { await previous.value; return await op() }
        tail = Task { _ = await task.value }
        return task
    }
}

// HarperServer.swift
//
// Harper's grammar engine ships as `harper-ls`, a "language server": a small program you talk
// to by sending JSON messages down a pipe (the Language Server Protocol, or LSP). This file
// starts that program and wraps the conversation in two easy calls:
//
//   check(text)      -> the list of problems in the text
//   actions(for:)    -> the possible fixes for one problem
//
// All the JSON plumbing (framing, request IDs, replies) lives at the bottom of the file.

import Foundation

/// One grammar or spelling issue, positioned in UTF-16 units (what NSString and AX use).
struct Lint {
    let range: NSRange
    let original: String
    let message: String
    let code: String
    let diagnostic: [String: Any]
}

/// Something the user can do about a lint: replace the text, or run a server command.
struct FixAction {
    enum Kind { case replace(String), command(String, [Any]) }
    let title: String
    let kind: Kind
}

/// Talks to a bundled `harper-ls --stdio` over JSON-RPC. One virtual document is
/// kept open per client and re-synced (full text) whenever its text changes.
final class HarperServer: @unchecked Sendable {
    private let queue = DispatchQueue(label: "harper.server")
    private var process: Process?
    private var input: FileHandle?
    private var buffer = Data()
    private var nextID = 100
    private var replies: [Int: ([String: Any]) -> Void] = [:]
    private struct Doc {
        var version = 0
        var open = false
        var waiter: (token: Int, done: ([[String: Any]]?) -> Void)?
    }
    /// One virtual document per client ("ax" for the Accessibility checker, "web" for browsers).
    private var docs: [String: Doc] = [:]
    private var waiterToken = 0

    private static func uri(_ doc: String) -> String { "file:///harper-mac/\(doc).txt" }

    private let dialect: () -> String
    init(dialect: @escaping () -> String) { self.dialect = dialect }

    // MARK: - Lifecycle

    static func binaryURL() -> URL? {
        if let bundled = Bundle.main.url(forResource: "harper-ls", withExtension: nil) { return bundled }
        let dev = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
            .appendingPathComponent("vendor/harper-ls/harper-ls")
        return FileManager.default.isExecutableFile(atPath: dev.path) ? dev : nil
    }

    func start() {
        queue.async { self.launch() }
    }

    func restart() {
        queue.async {
            self.process?.terminationHandler = nil
            self.process?.terminate()
            self.process = nil
            self.launch()
        }
    }

    private func launch() {
        guard process == nil, let url = Self.binaryURL() else {
            if process == nil { NSLog("harper-ls binary not found") }
            return
        }
        let p = Process()
        p.executableURL = url
        p.arguments = ["--stdio"]
        let stdin = Pipe(), stdout = Pipe()
        p.standardInput = stdin
        p.standardOutput = stdout
        p.standardError = FileHandle.nullDevice
        stdout.fileHandleForReading.readabilityHandler = { [weak self] h in
            let data = h.availableData
            guard !data.isEmpty else { h.readabilityHandler = nil; return }
            self?.queue.async { self?.consume(data) }
        }
        p.terminationHandler = { [weak self] _ in
            self?.queue.asyncAfter(deadline: .now() + 1) {
                self?.process = nil
                self?.launch()
            }
        }
        do { try p.run() } catch {
            NSLog("could not start harper-ls: \(error)")
            return
        }
        process = p
        input = stdin.fileHandleForWriting
        buffer = Data()
        for doc in docs.values { doc.waiter?.done(nil) }
        docs = [:]
        request("initialize", [
            "processId": NSNull(), "rootUri": NSNull(),
            "capabilities": ["workspace": ["configuration": true]],
        ]) { [weak self] _ in self?.notify("initialized", [:]) }
    }

    // MARK: - Public API

    /// Returns nil if the check was superseded, timed out, or the server is down.
    /// Callers must not overlap checks on the same `doc`; a newer check cancels the older one.
    func check(_ text: String, doc name: String = "ax") async -> [Lint]? {
        await withCheckedContinuation { (cont: CheckedContinuation<[Lint]?, Never>) in
            queue.async {
                guard self.process != nil else { cont.resume(returning: nil); return }
                var doc = self.docs[name] ?? Doc()
                doc.waiter?.done(nil)
                self.waiterToken += 1
                let token = self.waiterToken
                doc.waiter = (token, { diags in
                    cont.resume(returning: diags.map { $0.compactMap { Self.parse($0, in: text) } })
                })
                doc.version += 1
                let uri = Self.uri(name)
                if doc.open {
                    self.notify("textDocument/didChange", [
                        "textDocument": ["uri": uri, "version": doc.version],
                        "contentChanges": [["text": text]],
                    ])
                } else {
                    doc.open = true
                    self.notify("textDocument/didOpen", ["textDocument": [
                        "uri": uri, "languageId": "plaintext",
                        "version": doc.version, "text": text,
                    ]])
                }
                self.docs[name] = doc
                self.queue.asyncAfter(deadline: .now() + 4) {
                    if let w = self.docs[name]?.waiter, w.token == token {
                        self.docs[name]?.waiter = nil
                        w.done(nil)
                    }
                }
            }
        }
    }

    func actions(for lint: Lint, doc: String = "ax") async -> [FixAction] {
        await withCheckedContinuation { (cont: CheckedContinuation<[FixAction], Never>) in
            queue.async {
                guard self.process != nil else { cont.resume(returning: []); return }
                self.request("textDocument/codeAction", [
                    "textDocument": ["uri": Self.uri(doc)],
                    "range": lint.diagnostic["range"] ?? [:],
                    "context": ["diagnostics": [lint.diagnostic]],
                ]) { reply in
                    let items = reply["result"] as? [[String: Any]] ?? []
                    cont.resume(returning: items.compactMap { self.fixAction(from: $0) })
                }
            }
        }
    }

    func run(command: String, arguments: [Any]) {
        queue.async {
            self.request("workspace/executeCommand", ["command": command, "arguments": arguments]) { _ in }
        }
    }

    // MARK: - Parsing

    private func fixAction(from item: [String: Any]) -> FixAction? {
        let title = item["title"] as? String ?? ""
        if let edit = item["edit"] as? [String: Any],
           let changes = edit["changes"] as? [String: [[String: Any]]],
           let text = changes.values.first?.first?["newText"] as? String {
            return FixAction(title: title, kind: .replace(text))
        }
        // Either a CodeAction with a `command` object, or a bare Command.
        if let cmd = item["command"] as? [String: Any], let name = cmd["command"] as? String {
            return FixAction(title: cmd["title"] as? String ?? title,
                             kind: .command(name, cmd["arguments"] as? [Any] ?? []))
        }
        if let name = item["command"] as? String {
            return FixAction(title: title, kind: .command(name, item["arguments"] as? [Any] ?? []))
        }
        return nil
    }

    private static func parse(_ d: [String: Any], in text: String) -> Lint? {
        guard let r = d["range"] as? [String: Any],
              let s = r["start"] as? [String: Any], let e = r["end"] as? [String: Any],
              let sl = s["line"] as? Int, let sc = s["character"] as? Int,
              let el = e["line"] as? Int, let ec = e["character"] as? Int else { return nil }
        let starts = lineStarts(text)
        guard sl < starts.count, el < starts.count else { return nil }
        let lo = starts[sl] + sc, hi = starts[el] + ec
        let ns = text as NSString
        guard lo >= 0, hi > lo, hi <= ns.length else { return nil }
        let range = NSRange(location: lo, length: hi - lo)
        return Lint(range: range, original: ns.substring(with: range),
                    message: d["message"] as? String ?? "", code: d["code"] as? String ?? "",
                    diagnostic: d)
    }

    private static func lineStarts(_ text: String) -> [Int] {
        var starts = [0]
        let u = Array(text.utf16)
        var i = 0
        while i < u.count {
            if u[i] == 10 { starts.append(i + 1) }
            else if u[i] == 13 {
                if i + 1 < u.count, u[i + 1] == 10 { i += 1 }
                starts.append(i + 1)
            }
            i += 1
        }
        return starts
    }

    // MARK: - JSON-RPC plumbing (queue only)

    private func request(_ method: String, _ params: [String: Any], reply: @escaping ([String: Any]) -> Void) {
        nextID += 1
        replies[nextID] = reply
        write(["jsonrpc": "2.0", "id": nextID, "method": method, "params": params])
    }

    private func notify(_ method: String, _ params: [String: Any]) {
        write(["jsonrpc": "2.0", "method": method, "params": params])
    }

    private func write(_ message: [String: Any]) {
        guard let body = try? JSONSerialization.data(withJSONObject: message),
              let header = "Content-Length: \(body.count)\r\n\r\n".data(using: .utf8) else { return }
        try? input?.write(contentsOf: header + body)
    }

    private func consume(_ data: Data) {
        buffer.append(data)
        let sep = Data("\r\n\r\n".utf8)
        while let split = buffer.range(of: sep) {
            let header = String(decoding: buffer[buffer.startIndex..<split.lowerBound], as: UTF8.self)
            guard let line = header.split(separator: "\r\n").first(where: { $0.lowercased().hasPrefix("content-length") }),
                  let len = Int(line.split(separator: ":")[1].trimmingCharacters(in: .whitespaces)) else {
                buffer.removeAll(); return
            }
            let bodyStart = split.upperBound
            guard buffer.distance(from: bodyStart, to: buffer.endIndex) >= len else { return }
            let body = buffer[bodyStart..<buffer.index(bodyStart, offsetBy: len)]
            buffer = Data(buffer[buffer.index(bodyStart, offsetBy: len)...])
            if let msg = try? JSONSerialization.jsonObject(with: body) as? [String: Any] { handle(msg) }
        }
    }

    private func handle(_ msg: [String: Any]) {
        let method = msg["method"] as? String
        if let method, let id = msg["id"] {
            // Server-to-client request.
            var result: Any = NSNull()
            if method == "workspace/configuration" {
                let items = (msg["params"] as? [String: Any])?["items"] as? [Any] ?? []
                result = items.map { _ in ["harper-ls": ["dialect": dialect()]] }
            }
            write(["jsonrpc": "2.0", "id": id, "result": result])
        } else if method == "textDocument/publishDiagnostics" {
            let params = msg["params"] as? [String: Any]
            let diags = params?["diagnostics"] as? [[String: Any]] ?? []
            if let uri = params?["uri"] as? String,
               let name = docs.first(where: { Self.uri($0.key) == uri })?.key,
               let w = docs[name]?.waiter {
                docs[name]?.waiter = nil
                w.done(diags)
            }
        } else if let id = msg["id"] as? Int, let cb = replies.removeValue(forKey: id) {
            cb(msg)
        }
    }
}

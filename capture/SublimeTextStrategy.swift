import AppKit
import ApplicationServices
import Foundation

struct SublimeTextStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = [
        "com.sublimetext.4",
        "com.sublimetext.3",
        "com.sublimetext.2"
    ]

    private static let pluginDest: URL = {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        return support.appendingPathComponent("Sublime Text/Packages/User/KosmosCapture.py")
    }()

    private static let tempFile = URL(fileURLWithPath: "/tmp/kosmos-sublime.json")

    func setup(_ axApp: AXUIElement) {
        AXUIElementSetMessagingTimeout(axApp, axTimeout)
        installPlugin()
    }

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        guard let data = try? Data(contentsOf: Self.tempFile),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: String],
              let content = json["content"],
              !content.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        else { return nil }
        let filePath = json["file"] ?? ""
        log.debug("Sublime: \(content.count) chars from \(filePath.isEmpty ? "untitled" : filePath)")
        return (content, filePath.isEmpty ? nil : "file://\(filePath)")
    }

    private func installPlugin() {
        let dest = Self.pluginDest
        let fm = FileManager.default
        if fm.fileExists(atPath: dest.path) {
            log.debug("Sublime: plugin already installed at \(dest.path)")
            return
        }
        log.info("Sublime: installing plugin at \(dest.path)")
        do {
            try fm.createDirectory(at: dest.deletingLastPathComponent(), withIntermediateDirectories: true)
            try sublimePluginSource.write(to: dest, atomically: true, encoding: .utf8)
            log.info("Sublime: plugin installed — Sublime will hot-reload it automatically")
        } catch {
            log.error("Sublime: plugin install failed: \(error)")
        }
    }
}

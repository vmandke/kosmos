import AppKit
import ApplicationServices

struct SublimeTextStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = [
        "com.sublimetext.4",
        "com.sublimetext.3",
        "com.sublimetext.2"
    ]

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        let texts = extractAllText(window)
            .map    { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
        let content = texts.joined(separator: "\n")
        guard !content.isEmpty else { return nil }
        log.debug("Sublime: \(content.count) chars")
        return (content, nil)
    }
}

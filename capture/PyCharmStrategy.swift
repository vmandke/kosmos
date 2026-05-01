import AppKit
import ApplicationServices

struct PyCharmStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = [
        "com.jetbrains.pycharm",
        "com.jetbrains.pycharm-CE",
        "com.jetbrains.intellij"
    ]

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        var texts: [String] = []
        collectEditorText(window, into: &texts, depth: 0)
        let content = texts.joined(separator: "\n")
        guard !content.isEmpty else { return nil }
        log.debug("PyCharm: \(content.count) chars")
        return (content, nil)
    }

    private func collectEditorText(_ el: AXUIElement, into results: inout [String], depth: Int) {
        guard depth < 40 else { return }
        if axStr(el, kAXRoleAttribute) == "AXTextArea" {
            if let t = axStr(el, kAXValueAttribute), t.count > 10,
               !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                results.append(t)
            }
            return
        }
        axChildren(el).forEach { collectEditorText($0, into: &results, depth: depth + 1) }
    }
}

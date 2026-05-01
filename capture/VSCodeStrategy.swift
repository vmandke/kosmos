import AppKit
import ApplicationServices

struct VSCodeStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = [
        "com.microsoft.VSCode",
        "com.microsoft.VSCodeInsiders"
    ]
    let pollInterval: TimeInterval? = 20
    let setupDelay:   TimeInterval  = 0.6

    func setup(_ axApp: AXUIElement) {
        AXUIElementSetMessagingTimeout(axApp, axTimeout)
        // Required for Electron — exposes the editor's DOM to the AX tree.
        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, true as CFTypeRef)
    }

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        var texts: [String] = []
        collectEditorText(window, into: &texts, depth: 0)
        let content = texts.joined(separator: "\n")
        guard !content.isEmpty else { return nil }
        log.debug("VSCode: \(content.count) chars")
        return (content, nil)
    }

    private func collectEditorText(_ el: AXUIElement, into results: inout [String], depth: Int) {
        guard depth < 40 else { return }
        switch axStr(el, kAXRoleAttribute) ?? "" {
        case "AXTextArea":
            // Grab the full value; don't recurse — children are individual text runs.
            if let t = axStr(el, kAXValueAttribute), t.count > 10,
               !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                results.append(t)
            }
        case "AXWebArea":
            results += extractLeafText(el)
        default:
            axChildren(el).forEach { collectEditorText($0, into: &results, depth: depth + 1) }
        }
    }
}

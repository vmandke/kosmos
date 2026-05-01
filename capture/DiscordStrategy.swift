import AppKit
import ApplicationServices

struct DiscordStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = [
        "com.hnc.Discord",
        "com.hnc.Discord-ptb",
        "com.hnc.Discord-canary"
    ]
    let pollInterval: TimeInterval? = 20

    func setup(_ axApp: AXUIElement) {
        AXUIElementSetMessagingTimeout(axApp, axTimeout)
        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, true as CFTypeRef)
    }

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        var msgs: [String] = []
        collectMessages(window, into: &msgs, depth: 0)
        let content = msgs.joined(separator: "\n")
        guard !content.isEmpty else { return nil }
        log.debug("Discord: \(content.count) chars")
        return (content, nil)
    }

    private func collectMessages(_ el: AXUIElement, into results: inout [String], depth: Int) {
        guard depth < 50 else { return }
        switch axStr(el, kAXRoleAttribute) ?? "" {
        case "AXWebArea":
            // Discord renders the message list inside a web area.
            results += extractLeafText(el)
        case "AXStaticText":
            if let t = axStr(el, kAXValueAttribute),
               !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                results.append(t)
            }
        default:
            axChildren(el).forEach { collectMessages($0, into: &results, depth: depth + 1) }
        }
    }
}

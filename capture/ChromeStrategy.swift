import AppKit
import ApplicationServices

struct ChromeStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = [
        "com.google.Chrome",
        "com.google.Chrome.canary"
    ]
    let pollInterval: TimeInterval? = 20

    func setup(_ axApp: AXUIElement) {
        AXUIElementSetMessagingTimeout(axApp, axTimeout)
        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, true as CFTypeRef)
    }

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        let url  = findAddressBar(in: window, depth: 0)
        let text = collectWebContent(window, depth: 0).joined(separator: "\n")
        guard !text.isEmpty else { return nil }
        log.debug("Chrome: \(text.count) chars, url=\(url ?? "(none)")")
        return (text, url)
    }

    // Chrome's address bar is an AXTextField whose AXDescription contains "address".
    private func findAddressBar(in el: AXUIElement, depth: Int) -> String? {
        guard depth < 12 else { return nil }
        if axStr(el, kAXRoleAttribute) == "AXTextField" {
            let desc = (axStr(el, kAXDescriptionAttribute) ?? "").lowercased()
            if desc.contains("address") || desc.contains("search") {
                return axStr(el, kAXValueAttribute)
            }
        }
        for child in axChildren(el) {
            if let u = findAddressBar(in: child, depth: depth + 1) { return u }
        }
        return nil
    }

    private func collectWebContent(_ el: AXUIElement, depth: Int) -> [String] {
        guard depth < 50 else { return [] }
        if axStr(el, kAXRoleAttribute) == "AXWebArea" { return extractLeafText(el) }
        return axChildren(el).flatMap { collectWebContent($0, depth: depth + 1) }
    }
}

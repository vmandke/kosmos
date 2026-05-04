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
        AXUIElementSetAttributeValue(axApp, "AXEnhancedUserInterface" as CFString, true as CFTypeRef)
    }

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        nil  // extractAll() handles VSCode
    }

    func extractAll(from window: AXUIElement, app: NSRunningApplication) -> [(content: String, url: String?)] {
        guard let webArea = findRootWebArea(window, depth: 0) else {
            log.debug("VSCode: no web area found")
            return []
        }

        var panels: [(label: String, text: String)] = []
        findLandmarks(webArea, into: &panels, depth: 0)

        let filtered = panels.filter { !isExcluded($0.label) }
        log.debug("VSCode: \(filtered.count) panel(s) after filter")

        if filtered.isEmpty {
            let text = extractLeafText(webArea)
                .map    { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { $0.count > 2 }
                .joined(separator: "\n")
            return text.isEmpty ? [] : [(content: text, url: nil)]
        }

        return filtered.map { panel in
            log.debug("VSCode panel: \"\(panel.label)\" — \(panel.text.count) chars")
            return (content: "## \(panel.label)\n\(panel.text)", url: nil)
        }
    }

    // MARK: - Helpers

    private static let excludedPanels: [String] = ["Explorer", "Terminal"]

    private func isExcluded(_ label: String) -> Bool {
        Self.excludedPanels.contains { label.localizedCaseInsensitiveContains($0) }
    }

    private func findRootWebArea(_ el: AXUIElement, depth: Int) -> AXUIElement? {
        guard depth < 10 else { return nil }
        if axStr(el, kAXRoleAttribute) == "AXWebArea" { return el }
        for child in axChildren(el) {
            if let found = findRootWebArea(child, depth: depth + 1) { return found }
        }
        return nil
    }

    private func findLandmarks(
        _ el: AXUIElement,
        into panels: inout [(label: String, text: String)],
        depth: Int
    ) {
        guard depth < 12 else { return }
        let subrole = axStr(el, kAXSubroleAttribute) ?? ""
        if subrole.hasPrefix("AXLandmark") {
            let label = axStr(el, kAXDescriptionAttribute)
                     ?? axStr(el, kAXTitleAttribute)
                     ?? subrole
            let text = extractLeafText(el)
                .map    { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { $0.count > 2 }
                .joined(separator: "\n")
            if text.count > 20 { panels.append((label: label, text: text)) }
            return
        }
        axChildren(el).forEach { findLandmarks($0, into: &panels, depth: depth + 1) }
    }
}

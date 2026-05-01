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
        guard let webArea = findRootWebArea(window, depth: 0) else {
            log.debug("VSCode: no web area found")
            axDump(window, maxDepth: 5)
            return nil
        }

        var panels: [(label: String, text: String)] = []
        findLandmarks(webArea, into: &panels, depth: 0)
        log.debug("VSCode: \(panels.count) landmark(s) found")

        if panels.isEmpty {
            // No landmarks detected — dump the tree so we can see the actual structure.
            print("[kosmos] VSCode: no landmarks found, dumping web area:")
            axDump(webArea, maxDepth: 6)
            // Fall back to the whole blob.
            let text = extractLeafText(webArea)
                .map    { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { $0.count > 2 }
                .joined(separator: "\n")
            return text.isEmpty ? nil : (text, nil)
        }

        let content = panels
            .map { "## \($0.label)\n\($0.text)" }
            .joined(separator: "\n\n")

        log.debug("VSCode: \(content.count) chars across \(panels.count) panel(s)")
        return (content, nil)
    }

    // MARK: - Root web area

    private func findRootWebArea(_ el: AXUIElement, depth: Int) -> AXUIElement? {
        guard depth < 10 else { return nil }
        if axStr(el, kAXRoleAttribute) == "AXWebArea" { return el }
        for child in axChildren(el) {
            if let found = findRootWebArea(child, depth: depth + 1) { return found }
        }
        return nil
    }

    // MARK: - Dynamic landmark detection

    /// Walks the AX subtree and collects every element whose subrole starts with
    /// "AXLandmark". Labels come from whatever descriptive attribute the element
    /// carries — no panel names are hardcoded.
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
                     ?? subrole   // fall back to the raw subrole name

            let text = extractLeafText(el)
                .map    { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { $0.count > 2 }
                .joined(separator: "\n")

            if text.count > 20 {
                panels.append((label: label, text: text))
                log.debug("VSCode panel: \"\(label)\" (\(subrole)) — \(text.count) chars")
            }
            // Don't recurse further — we want top-level landmarks only.
            return
        }

        axChildren(el).forEach { findLandmarks($0, into: &panels, depth: depth + 1) }
    }
}

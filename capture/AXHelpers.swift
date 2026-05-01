import ApplicationServices

/// Seconds before an AX call gives up — prevents the monitor from hanging on
/// unresponsive apps (known issue with some Electron builds).
let axTimeout: Float = 2.0

// MARK: - Attribute accessors

func axStr(_ el: AXUIElement, _ attr: String) -> String? {
    var v: AnyObject?
    guard AXUIElementCopyAttributeValue(el, attr as CFString, &v) == .success else { return nil }
    return v as? String
}

func axChildren(_ el: AXUIElement) -> [AXUIElement] {
    var v: AnyObject?
    guard AXUIElementCopyAttributeValue(el, kAXChildrenAttribute as CFString, &v) == .success
    else { return [] }
    return v as? [AXUIElement] ?? []
}

func axFocusedWindow(_ axApp: AXUIElement) -> AXUIElement? {
    var v: AnyObject?
    guard AXUIElementCopyAttributeValue(axApp, kAXFocusedWindowAttribute as CFString, &v) == .success,
          let raw = v else { return nil }
    return (raw as! AXUIElement)
}

// MARK: - Text extractors

/// Collects only `AXStaticText` leaf values — suited to web content where
/// intermediate container nodes would otherwise produce duplicate strings.
func extractLeafText(_ el: AXUIElement, depth: Int = 0) -> [String] {
    guard depth < 50 else { return [] }
    if axStr(el, kAXRoleAttribute) == "AXStaticText" {
        if let t = axStr(el, kAXValueAttribute),
           !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return [t] }
        return []
    }
    return axChildren(el).flatMap { extractLeafText($0, depth: depth + 1) }
}

/// Full UI traversal with role-aware shortcuts.
/// - `AXWebArea` → leaf-only (avoids container duplication)
/// - `AXTextArea` / `AXTextField` → grab value and stop (children are text runs)
/// - Everything else → collect value + title then recurse
func extractAllText(_ el: AXUIElement, depth: Int = 0) -> [String] {
    guard depth < 40 else { return [] }
    let role = axStr(el, kAXRoleAttribute) ?? ""
    switch role {
    case "AXWebArea":
        return extractLeafText(el)
    case "AXTextArea", "AXTextField":
        if let t = axStr(el, kAXValueAttribute),
           !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return [t] }
        return []
    default:
        var out: [String] = []
        for attr in [kAXValueAttribute, kAXTitleAttribute] {
            if let t = axStr(el, attr), t.count > 2,
               !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { out.append(t) }
        }
        out += axChildren(el).flatMap { extractAllText($0, depth: depth + 1) }
        return out
    }
}

import ApplicationServices

/// Seconds before an AX call gives up — prevents the monitor from hanging on
/// unresponsive apps (known issue with some Electron builds).
let axTimeout: Float = 2.0

// MARK: - Attribute accessors

func axStr(_ el: AXUIElement, _ attr: String) -> String? {
    var v: AnyObject?
    guard AXUIElementCopyAttributeValue(el, attr as CFString, &v) == .success else { return nil }
    if let s = v as? String { return s }
    if let a = v as? NSAttributedString { return a.string }  // Chrome returns attributed strings
    return nil
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

/// Returns the focused UI element within an application (e.g. the active editor field).
func axFocusedElement(_ axApp: AXUIElement) -> AXUIElement? {
    var v: AnyObject?
    guard AXUIElementCopyAttributeValue(axApp, kAXFocusedUIElementAttribute as CFString, &v) == .success,
          let raw = v else { return nil }
    return (raw as! AXUIElement)
}

/// Returns the parent element, used to walk up the AX tree.
func axParent(_ el: AXUIElement) -> AXUIElement? {
    var v: AnyObject?
    guard AXUIElementCopyAttributeValue(el, kAXParentAttribute as CFString, &v) == .success,
          let raw = v else { return nil }
    return (raw as! AXUIElement)
}

/// Prints the AX subtree to stdout for diagnostics (visible in the `make capture` terminal).
/// os.log redacts dynamic values as <private>, so print is used here intentionally.
func axDump(_ el: AXUIElement, maxDepth: Int = 5, depth: Int = 0) {
    guard depth < maxDepth else { return }
    let role  = axStr(el, kAXRoleAttribute)        ?? "?"
    let sub   = axStr(el, kAXSubroleAttribute).map { " [\($0)]" }              ?? ""
    let title = axStr(el, kAXTitleAttribute).map   { " title=\"\($0.prefix(60))\"" } ?? ""
    let desc  = axStr(el, kAXDescriptionAttribute).map { " desc=\"\($0.prefix(60))\"" } ?? ""
    let chars = axStr(el, kAXValueAttribute).map   { " [\($0.count)c]" }       ?? ""
    let pad   = String(repeating: "  ", count: depth)
    print("AX: \(pad)\(role)\(sub)\(title)\(desc)\(chars)")
    axChildren(el).forEach { axDump($0, maxDepth: maxDepth, depth: depth + 1) }
}

// MARK: - Text extractors

/// Collects text from a web content subtree.
/// - Explicit `AXStaticText` nodes are returned as-is.
/// - Any other leaf node (no children) that carries a value is also returned —
///   Chrome puts text directly on AXParagraph / AXHeading without a nested static text child.
func extractLeafText(_ el: AXUIElement, depth: Int = 0) -> [String] {
    guard depth < 60 else { return [] }
    let role     = axStr(el, kAXRoleAttribute) ?? ""
    let children = axChildren(el)

    if role == "AXStaticText" || children.isEmpty {
        if let t = axStr(el, kAXValueAttribute),
           !t.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return [t] }
        return []
    }
    return children.flatMap { extractLeafText($0, depth: depth + 1) }
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

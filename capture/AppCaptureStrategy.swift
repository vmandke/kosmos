import AppKit
import ApplicationServices

// MARK: - Protocol

protocol AppCaptureStrategy {
    /// Bundle identifiers handled by this strategy.
    var bundleIDs: Set<String> { get }
    /// AX notifications the observer subscribes to on the app element.
    var axNotifications: [String] { get }
    /// Non-nil enables periodic polling (seconds). Needed for Electron apps where
    /// AX change events are unreliable.
    var pollInterval: TimeInterval? { get }
    /// Seconds to wait after setup() before the first capture fires.
    /// Electron apps need this — AXEnhancedUserInterface triggers an async AX tree
    /// rebuild in the target process and capture() will see an empty tree if it
    /// runs before that rebuild completes.
    var setupDelay: TimeInterval { get }
    /// Called once on app activation — configure AXEnhancedUserInterface, timeouts, etc.
    func setup(_ axApp: AXUIElement)
    /// Extract text (and optional URL) from the focused window.
    /// Return nil to suppress emit (e.g. empty or unsupported view).
    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)?
}

extension AppCaptureStrategy {
    var axNotifications: [String] {
        [kAXFocusedWindowChangedNotification, kAXTitleChangedNotification]
    }
    var pollInterval: TimeInterval? { nil }
    var setupDelay: TimeInterval { 0 }
    func setup(_ axApp: AXUIElement) {
        AXUIElementSetMessagingTimeout(axApp, axTimeout)
    }
}

// MARK: - Registry

final class StrategyRegistry {
    private var table: [String: AppCaptureStrategy] = [:]
    let fallback: AppCaptureStrategy = GenericStrategy()

    init(strategies: [AppCaptureStrategy]) {
        for s in strategies {
            for id in s.bundleIDs { table[id] = s }
        }
    }

    var targetBundleIDs: Set<String> { Set(table.keys) }

    func strategy(for bundleID: String) -> AppCaptureStrategy {
        table[bundleID] ?? fallback
    }
}

// MARK: - Generic fallback

struct GenericStrategy: AppCaptureStrategy {
    let bundleIDs: Set<String> = []

    func extract(from window: AXUIElement, app: NSRunningApplication) -> (content: String, url: String?)? {
        let texts = extractAllText(window)
            .map    { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
            .filter { !$0.isEmpty }
        let content = texts.joined(separator: "\n")
        guard !content.isEmpty else { return nil }
        return (content, nil)
    }
}

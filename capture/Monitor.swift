import Cocoa
import ApplicationServices

final class KosmosMonitor {
    private let socket = SocketWriter(path: "/tmp/kosmos.sock")
    private let dedup  = Deduplicator()
    private let registry = StrategyRegistry(strategies: [
        ChromeStrategy(),
        VSCodeStrategy(),
        SublimeTextStrategy(),
        DiscordStrategy(),
        PyCharmStrategy()
    ])

    private var axObserver:  AXObserver?
    private var pollTimer:   Timer?
    private var tccTimer:    Timer?  // periodic TCC-revocation guard

    // MARK: - Lifecycle

    func start() {
        checkTCCOrExit()

        log.info("Kosmos capture started — \(self.registry.targetBundleIDs.count) target apps")
        print("=== Kosmos Capture ===")
        print("Watching: Chrome · VS Code · Sublime Text · Discord · PyCharm")
        print("Mode:     AXObserver events + Electron \(Int(Config.electronPollInterval))s poll")
        print("Socket:   /tmp/kosmos.sock")
        print("Logs:     log stream --predicate 'subsystem == \"dev.kosmos.capture\"'\n")

        NSWorkspace.shared.notificationCenter.addObserver(
            self, selector: #selector(appActivated(_:)),
            name: NSWorkspace.didActivateApplicationNotification, object: nil
        )

        // Detect silent TCC revocation — macOS can remove our AX grant without notice.
        tccTimer = Timer.scheduledTimer(withTimeInterval: 60, repeats: true) { [weak self] _ in
            self?.checkTCCOrExit()
        }

        if let front = NSWorkspace.shared.frontmostApplication { handleActivation(front) }
        RunLoop.main.run()
    }

    // MARK: - App activation

    @objc private func appActivated(_ note: Notification) {
        guard let app = note.userInfo?[NSWorkspace.applicationUserInfoKey]
                as? NSRunningApplication else { return }
        handleActivation(app)
    }

    private func handleActivation(_ app: NSRunningApplication) {
        guard let bundle = app.bundleIdentifier,
              registry.targetBundleIDs.contains(bundle) else {
            teardownObserver(); stopPoll(); return
        }
        let name = app.localizedName ?? bundle
        log.info("Activated: \(name) (\(bundle))")
        print("[→] \(name)")

        let axApp    = AXUIElementCreateApplication(app.processIdentifier)
        let strategy = registry.strategy(for: bundle)
        // setup() sets AXEnhancedUserInterface and messaging timeout — expensive for Electron,
        // so it must only run here (once per activation), not inside capture().
        strategy.setup(axApp)

        setupObserver(for: app, strategy: strategy)
        if strategy.setupDelay > 0 {
            // Wait for the target process to rebuild its AX tree after setup().
            DispatchQueue.main.asyncAfter(deadline: .now() + strategy.setupDelay) { [weak self] in
                guard NSWorkspace.shared.frontmostApplication?.bundleIdentifier == bundle else { return }
                self?.capture(app)
            }
        } else {
            capture(app)
        }
        if let iv = strategy.pollInterval { startPoll(for: app, interval: iv) } else { stopPoll() }
    }

    // MARK: - Capture

    func capture(_ app: NSRunningApplication) {
        guard let bundle = app.bundleIdentifier else { return }
        let strategy = registry.strategy(for: bundle)

        // Create the element handle and apply only the messaging timeout (cheap, per-handle).
        let axApp = AXUIElementCreateApplication(app.processIdentifier)
        AXUIElementSetMessagingTimeout(axApp, axTimeout)

        guard let window = axFocusedWindow(axApp) else {
            log.debug("No focused window for \(bundle)")
            return
        }
        let title = axStr(window, kAXTitleAttribute) ?? ""

        guard let result = strategy.extract(from: window, app: app) else { return }

        let content = result.content
            .components(separatedBy: .newlines)
            .map    { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
            .joined(separator: "\n")

        guard !content.isEmpty else { return }
        guard !dedup.isDuplicate(bundleID: bundle, content: content) else {
            log.debug("Duplicate skipped: \(bundle)")
            return
        }

        emit(app: app, title: title, content: content, url: result.url)
    }

    private func emit(app: NSRunningApplication, title: String, content: String, url: String?) {
        let payload = CapturePayload(
            timestamp: Date(),
            appName:   app.localizedName ?? app.bundleIdentifier ?? "?",
            bundleID:  app.bundleIdentifier ?? "",
            title:     title,
            content:   content,
            url:       url
        )
        let ts = ISO8601DateFormatter().string(from: payload.timestamp)
        log.info("Emit: \(payload.appName) — \"\(title)\" — \(content.count) chars")
        print("\n[\(ts)] \(payload.appName) — \"\(title)\" (\(content.count) chars)")
        if let u = url { print("  URL: \(u)") }
        print(content)
        print(String(repeating: "─", count: 44))

        socket.send(payload)
    }

    // MARK: - TCC guard

    private func checkTCCOrExit() {
        guard AXIsProcessTrusted() else {
            log.error("Accessibility permission revoked — exiting")
            fputs("ERROR: Accessibility permission revoked. Re-enable in System Settings → Privacy & Security → Accessibility.\n", stderr)
            exit(1)
        }
    }

    // MARK: - AXObserver

    private func setupObserver(for app: NSRunningApplication, strategy: AppCaptureStrategy) {
        teardownObserver()
        let ctx = Unmanaged.passUnretained(self).toOpaque()
        var obs: AXObserver?
        guard AXObserverCreate(app.processIdentifier, { _, _, _, refcon in
            guard let refcon else { return }
            let m = Unmanaged<KosmosMonitor>.fromOpaque(refcon).takeUnretainedValue()
            if let front = NSWorkspace.shared.frontmostApplication { m.capture(front) }
        }, &obs) == .success, let obs else { return }

        let axApp = AXUIElementCreateApplication(app.processIdentifier)
        for n in strategy.axNotifications {
            AXObserverAddNotification(obs, axApp, n as CFString, ctx)
        }
        CFRunLoopAddSource(RunLoop.main.getCFRunLoop(), AXObserverGetRunLoopSource(obs), .defaultMode)
        axObserver = obs
        log.debug("AXObserver: \(app.bundleIdentifier ?? "?") — \(strategy.axNotifications)")
    }

    private func teardownObserver() {
        guard let obs = axObserver else { return }
        CFRunLoopRemoveSource(RunLoop.main.getCFRunLoop(), AXObserverGetRunLoopSource(obs), .defaultMode)
        axObserver = nil
    }

    // MARK: - Poll Timer

    private func startPoll(for app: NSRunningApplication, interval: TimeInterval) {
        stopPoll()
        log.debug("Poll: \(app.bundleIdentifier ?? "?") every \(interval)s")
        pollTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { [weak self] _ in
            guard let front = NSWorkspace.shared.frontmostApplication,
                  front.bundleIdentifier == app.bundleIdentifier else { return }
            self?.capture(front)
        }
    }

    private func stopPoll() { pollTimer?.invalidate(); pollTimer = nil }
}

// MARK: - Config

private enum Config {
    static let electronPollInterval: TimeInterval = 5
}

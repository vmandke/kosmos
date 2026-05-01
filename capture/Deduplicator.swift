/// Keeps a small sliding window of recent FNV-1a hashes per app.
/// Catches immediate re-sends without accumulating unbounded state.
final class Deduplicator {
    private let windowSize = 5
    private var recent: [String: [UInt64]] = [:]

    func isDuplicate(bundleID: String, content: String) -> Bool {
        let h = fnv1a(content)
        var window = recent[bundleID] ?? []
        if window.contains(h) { return true }
        window.append(h)
        if window.count > windowSize { window.removeFirst() }
        recent[bundleID] = window
        return false
    }

    private func fnv1a(_ s: String) -> UInt64 {
        var hash: UInt64 = 14695981039346656037
        for byte in s.utf8 { hash ^= UInt64(byte); hash &*= 1099511628211 }
        return hash
    }
}

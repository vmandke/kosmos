import Foundation

struct CapturePayload {
    let timestamp: Date
    let appName:   String
    let bundleID:  String
    let title:     String
    let content:   String
    let url:       String?

    func toDict() -> [String: Any] {
        var d: [String: Any] = [
            "ts":      ISO8601DateFormatter().string(from: timestamp),
            "app":     appName,
            "title":   title,
            "content": content,
            "chars":   content.count
        ]
        if let url { d["url"] = url }
        return d
    }
}

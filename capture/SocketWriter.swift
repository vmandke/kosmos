import Darwin
import Foundation

final class SocketWriter {
    private let path: String
    private var fd: Int32 = -1

    init(path: String) { self.path = path; tryConnect() }

    private func tryConnect() {
        let sock = socket(AF_UNIX, SOCK_STREAM, 0)
        guard sock >= 0 else { return }
        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
            path.withCString { src in
                UnsafeMutableRawPointer(ptr)
                    .copyMemory(from: src, byteCount: min(path.utf8.count + 1, 104))
            }
        }
        let connected = withUnsafePointer(to: &addr) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                Darwin.connect(sock, $0, socklen_t(MemoryLayout<sockaddr_un>.size)) == 0
            }
        }
        if connected {
            fd = sock
            log.info("Socket connected: \(self.path)")
        } else {
            Darwin.close(sock)
            log.debug("Socket not yet available: \(self.path)")
        }
    }

    func send(_ payload: CapturePayload) {
        if fd < 0 { tryConnect() }
        guard fd >= 0,
              let data = try? JSONSerialization.data(withJSONObject: payload.toDict()),
              let json = String(data: data, encoding: .utf8) else { return }
        let line = json + "\n"
        let n = line.withCString { Darwin.write(fd, $0, strlen($0)) }
        if n < 0 {
            log.error("Socket write failed — will reconnect on next send")
            Darwin.close(fd); fd = -1
        }
    }

    deinit { if fd >= 0 { Darwin.close(fd) } }
}

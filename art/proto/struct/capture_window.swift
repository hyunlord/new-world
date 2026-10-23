import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 3,
      let processID = Int(CommandLine.arguments[2]) else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
guard let windows = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]],
      let window = windows.first(where: {
          ($0[kCGWindowOwnerName as String] as? String)?.contains("Godot") == true &&
          ($0[kCGWindowOwnerPID as String] as? Int) == processID
      }),
      let identifier = window[kCGWindowNumber as String] as? Int else {
    fputs("No on-screen STRUCT Godot window found\n", stderr)
    exit(1)
}

do {
    try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
    let metadata = try JSONSerialization.data(withJSONObject: window, options: [.prettyPrinted, .sortedKeys])
    try metadata.write(to: output.appendingPathComponent("os_window.json"))
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    process.arguments = ["-x", "-o", "-l", String(identifier), output.appendingPathComponent("os_window.png").path]
    try process.run()
    process.waitUntilExit()
    exit(process.terminationStatus)
} catch {
    fputs("Window capture failed: \(error)\n", stderr)
    exit(1)
}

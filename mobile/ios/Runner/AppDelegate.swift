import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let controller = window?.rootViewController as! FlutterViewController
    let channel = FlutterMethodChannel(name: "liberty/security", binaryMessenger: controller.binaryMessenger)
    channel.setMethodCallHandler { call, result in
      switch call.method {
      case "setSecureFlag":
        // iOS screenshot protection is limited; FLAG_SECURE is Android-only.
        result(nil)
      case "keystoreEnsure":
        result(LibertyKeychain.ensureKey())
      case "keystoreWrap":
        guard let args = call.arguments as? [String: Any],
              let data = args["data"] as? FlutterStandardTypedData else {
          result(FlutterError(code: "ARG", message: "data required", details: nil))
          return
        }
        result(LibertyKeychain.wrap(data.data))
      case "keystoreUnwrap":
        guard let args = call.arguments as? [String: Any],
              let wrapped = args["wrapped"] as? String else {
          result(FlutterError(code: "ARG", message: "wrapped required", details: nil))
          return
        }
        if let plain = LibertyKeychain.unwrap(wrapped) {
          result(FlutterStandardTypedData(bytes: plain))
        } else {
          result(nil)
        }
      case "keystoreDelete":
        LibertyKeychain.deleteKey()
        result(nil)
      case "keystoreIsHardware":
        result(true)
      default:
        result(FlutterMethodNotImplemented)
      }
    }
    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}

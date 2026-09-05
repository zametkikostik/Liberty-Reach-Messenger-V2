import Foundation
import CryptoKit
import Security

/// iOS Keychain + CryptoKit AES-GCM wrapper for vault key material
enum LibertyKeychain {
    private static let service = "com.liberty.messenger.vault"
    private static let account = "wrap-key"

    static func ensureKey() -> Bool {
        if loadKey() != nil { return true }
        let key = SymmetricKey(size: .bits256)
        return saveKey(key)
    }

    private static func saveKey(_ key: SymmetricKey) -> Bool {
        let data = key.withUnsafeBytes { Data($0) }
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
        ]
        SecItemDelete(q as CFDictionary)
        return SecItemAdd(q as CFDictionary, nil) == errSecSuccess
    }

    private static func loadKey() -> SymmetricKey? {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
        ]
        var item: CFTypeRef?
        let status = SecItemCopyMatching(q as CFDictionary, &item)
        guard status == errSecSuccess, let data = item as? Data else { return nil }
        return SymmetricKey(data: data)
    }

    static func wrap(_ data: Data) -> String? {
        guard ensureKey(), let key = loadKey() else { return nil }
        guard let sealed = try? AES.GCM.seal(data, using: key) else { return nil }
        guard let combined = sealed.combined else { return nil }
        return combined.base64EncodedString()
    }

    static func unwrap(_ b64: String) -> Data? {
        guard let key = loadKey(), let data = Data(base64Encoded: b64) else { return nil }
        guard let box = try? AES.GCM.SealedBox(combined: data) else { return nil }
        return try? AES.GCM.open(box, using: key)
    }

    static func deleteKey() {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        SecItemDelete(q as CFDictionary)
    }
}

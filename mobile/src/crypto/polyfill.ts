/**
 * Install native WebCrypto on React Native via react-native-quick-crypto.
 * Safe no-op on web / when the module is missing.
 */
export function installNativeCrypto(): boolean {
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    require("react-native-get-random-values");
  } catch {
    /* optional */
  }
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const QuickCrypto = require("react-native-quick-crypto");
    if (QuickCrypto?.install) {
      QuickCrypto.install();
    } else if (QuickCrypto?.default?.install) {
      QuickCrypto.default.install();
    }
    if (!globalThis.crypto && QuickCrypto) {
      (globalThis as unknown as { crypto: unknown }).crypto =
        QuickCrypto.crypto || QuickCrypto.default?.crypto || QuickCrypto;
    }
    return !!(globalThis.crypto && globalThis.crypto.subtle);
  } catch {
    return !!(globalThis.crypto && globalThis.crypto.subtle);
  }
}

/**
 * Secure key storage: Keychain (iOS) / Keystore (Android) via expo-secure-store.
 * Memory fallback for Expo web / environments without native module.
 */

const mem = new Map<string, string>();

let SecureStore: {
  getItemAsync: (k: string) => Promise<string | null>;
  setItemAsync: (k: string, v: string) => Promise<void>;
  deleteItemAsync: (k: string) => Promise<void>;
} | null = null;

try {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  SecureStore = require("expo-secure-store");
} catch {
  SecureStore = null;
}

export async function secureGet(key: string): Promise<string | null> {
  if (SecureStore) {
    try {
      return await SecureStore.getItemAsync(key);
    } catch {
      return mem.get(key) ?? null;
    }
  }
  return mem.get(key) ?? null;
}

export async function secureSet(key: string, value: string): Promise<void> {
  mem.set(key, value);
  if (SecureStore) {
    try {
      await SecureStore.setItemAsync(key, value);
    } catch {
      /* memory already set */
    }
  }
}

export async function secureDelete(key: string): Promise<void> {
  mem.delete(key);
  if (SecureStore) {
    try {
      await SecureStore.deleteItemAsync(key);
    } catch {
      /* ignore */
    }
  }
}

export function usingNativeSecureStore(): boolean {
  return SecureStore != null;
}

/**
 * Mobile E2EE — same wire format as web:
 *   "lr1:" + base64(nonce[12] || ciphertext)
 * AES-256-GCM. Keys live in SecureStore (Keychain/Keystore).
 */

import { secureGet, secureSet } from "../storage/secure";

const enc = new TextEncoder();
const dec = new TextDecoder();

function keyId(chatId: string): string {
  return `lr_e2ee_key_v1_${chatId}`;
}

function b64encode(bytes: Uint8Array): string {
  let s = "";
  bytes.forEach((b) => (s += String.fromCharCode(b)));
  return typeof btoa !== "undefined"
    ? btoa(s)
    : Buffer.from(bytes).toString("base64");
}

function b64decode(b64: string): Uint8Array {
  const bin =
    typeof atob !== "undefined"
      ? atob(b64)
      : Buffer.from(b64, "base64").toString("binary");
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

function subtle(): SubtleCrypto {
  const c = globalThis.crypto;
  if (!c?.subtle) {
    throw new Error(
      "WebCrypto SubtleCrypto missing — use Expo web, or add react-native-quick-crypto for native AES-GCM"
    );
  }
  return c.subtle;
}

function randomBytes(n: number): Uint8Array {
  const c = globalThis.crypto;
  if (c?.getRandomValues) {
    return c.getRandomValues(new Uint8Array(n));
  }
  const a = new Uint8Array(n);
  for (let i = 0; i < n; i++) a[i] = Math.floor(Math.random() * 256);
  return a;
}

async function importRawKey(raw: Uint8Array): Promise<CryptoKey> {
  return subtle().importKey("raw", raw, { name: "AES-GCM" }, false, [
    "encrypt",
    "decrypt",
  ]);
}

export async function getOrCreateChatKey(chatId: string): Promise<CryptoKey> {
  const id = keyId(chatId);
  const existing = await secureGet(id);
  if (existing) {
    return importRawKey(b64decode(existing));
  }
  const raw = randomBytes(32);
  await secureSet(id, b64encode(raw));
  return importRawKey(raw);
}

export async function exportChatKey(chatId: string): Promise<string | null> {
  return secureGet(keyId(chatId));
}

export async function importChatKey(chatId: string, b64Key: string): Promise<void> {
  const raw = b64decode(b64Key.trim());
  if (raw.length !== 32) {
    throw new Error("chat key must be 32 bytes (AES-256)");
  }
  await secureSet(keyId(chatId), b64encode(raw));
}

export async function encryptBody(chatId: string, plaintext: string): Promise<string> {
  const key = await getOrCreateChatKey(chatId);
  const nonce = randomBytes(12);
  const ct = await subtle().encrypt(
    { name: "AES-GCM", iv: nonce },
    key,
    enc.encode(plaintext)
  );
  const packed = new Uint8Array(12 + ct.byteLength);
  packed.set(nonce, 0);
  packed.set(new Uint8Array(ct), 12);
  return "lr1:" + b64encode(packed);
}

export async function decryptBody(chatId: string, body: string): Promise<string> {
  if (!body.startsWith("lr1:")) return body;
  try {
    const key = await getOrCreateChatKey(chatId);
    const packed = b64decode(body.slice(4));
    if (packed.length < 13) return "[decrypt error]";
    const nonce = packed.slice(0, 12);
    const ct = packed.slice(12);
    const pt = await subtle().decrypt(
      { name: "AES-GCM", iv: nonce },
      key,
      ct
    );
    return dec.decode(pt);
  } catch {
    return "[encrypted — нет ключа или повреждено]";
  }
}

export function isEncrypted(body: string): boolean {
  return body.startsWith("lr1:");
}

export function hasWebCrypto(): boolean {
  return !!(globalThis.crypto && globalThis.crypto.subtle);
}

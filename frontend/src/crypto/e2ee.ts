/**
 * Client-side E2EE (Web Crypto API).
 * Server only ever sees base64 ciphertext — never plaintext body.
 *
 * Per-chat AES-256-GCM key is kept in localStorage (device-local).
 * For multi-device / multi-user key exchange use exportChatKey / importChatKey
 * (out-of-band or future X25519+Kyber handshake).
 */

const enc = new TextEncoder();
const dec = new TextDecoder();

function keyStorageId(chatId: string): string {
  return `lr_e2ee_key_v1_${chatId}`;
}

function b64encode(buf: ArrayBuffer | Uint8Array): string {
  const bytes = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
  let s = "";
  for (let i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
  return btoa(s);
}

function b64decode(s: string): Uint8Array {
  const bin = atob(s);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

async function importRawKey(raw: Uint8Array): Promise<CryptoKey> {
  return crypto.subtle.importKey("raw", raw, { name: "AES-GCM" }, false, [
    "encrypt",
    "decrypt",
  ]);
}

/** Create or load AES-256 key for a chat */
export async function getOrCreateChatKey(chatId: string): Promise<CryptoKey> {
  const id = keyStorageId(chatId);
  const existing = localStorage.getItem(id);
  if (existing) {
    return importRawKey(b64decode(existing));
  }
  const raw = crypto.getRandomValues(new Uint8Array(32));
  localStorage.setItem(id, b64encode(raw));
  return importRawKey(raw);
}

/** Export key as base64 (share out-of-band with peer) */
export function exportChatKey(chatId: string): string | null {
  return localStorage.getItem(keyStorageId(chatId));
}

/** Import peer key for this chat */
export function importChatKey(chatId: string, b64Key: string): void {
  const raw = b64decode(b64Key.trim());
  if (raw.length !== 32) {
    throw new Error("chat key must be 32 bytes (AES-256)");
  }
  localStorage.setItem(keyStorageId(chatId), b64encode(raw));
}

/**
 * Encrypt plaintext → "lr1:" + base64(nonce[12] || ciphertext)
 */
export async function encryptBody(chatId: string, plaintext: string): Promise<string> {
  const key = await getOrCreateChatKey(chatId);
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const ct = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: nonce },
    key,
    enc.encode(plaintext)
  );
  const packed = new Uint8Array(12 + ct.byteLength);
  packed.set(nonce, 0);
  packed.set(new Uint8Array(ct), 12);
  return "lr1:" + b64encode(packed);
}

/** Decrypt body; if not lr1: prefix, return as-is (legacy) */
export async function decryptBody(chatId: string, body: string): Promise<string> {
  if (!body.startsWith("lr1:")) return body;
  try {
    const key = await getOrCreateChatKey(chatId);
    const packed = b64decode(body.slice(4));
    if (packed.length < 13) return "[decrypt error]";
    const nonce = packed.slice(0, 12);
    const ct = packed.slice(12);
    const pt = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: nonce },
      key,
      ct
    );
    return dec.decode(pt);
  } catch {
    return "[encrypted — no key or corrupted]";
  }
}

export function isEncrypted(body: string): boolean {
  return body.startsWith("lr1:");
}

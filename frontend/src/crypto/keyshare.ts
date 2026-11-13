/**
 * Out-of-band chat key exchange (QR / paste / deep link).
 * Server never sees these keys.
 *
 * liberty://key?c=<chatId>&k=<base64-32-byte-aes-key>&v=1
 */

export type KeySharePayload = {
  v: 1;
  chat_id: string;
  key: string;
};

export function buildKeyShareUri(chatId: string, keyB64: string): string {
  const q = new URLSearchParams({ c: chatId, k: keyB64, v: "1" });
  return `liberty://key?${q.toString()}`;
}

export function buildKeyShareJson(chatId: string, keyB64: string): string {
  const p: KeySharePayload = { v: 1, chat_id: chatId, key: keyB64 };
  return JSON.stringify(p);
}

export function parseKeyShare(input: string): KeySharePayload {
  const s = input.trim();
  if (s.startsWith("{")) {
    const p = JSON.parse(s) as KeySharePayload;
    if (p.v !== 1 || !p.chat_id || !p.key) throw new Error("invalid key JSON");
    return p;
  }
  let url: URL;
  try {
    url = new URL(s);
  } catch {
    url = new URL(s.includes("://") ? s : `liberty://key?${s}`);
  }
  const chat_id = url.searchParams.get("c") || url.searchParams.get("chat_id") || "";
  const key = url.searchParams.get("k") || url.searchParams.get("key") || "";
  if (!chat_id || !key) throw new Error("missing c/k in key URI");
  return { v: 1, chat_id, key };
}

/**
 * Liberty Wake Relay — Cloudflare Worker (no Firebase)
 * No message plaintext accepted or stored.
 */
const WAKE_TTL_SEC = 86400;

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;
    if (request.method === "OPTIONS") return cors(new Response(null, { status: 204 }));
    try {
      if (path === "/health") return cors(json({ ok: true, service: "liberty-wake-relay" }));
      if (path === "/privacy" || path === "/privacy/") return cors(privacyHtml());
      if (path === "/v1/register" && request.method === "POST") return cors(await register(request, env));
      if (path === "/v1/wake" && request.method === "POST") return cors(await wake(request, env));
      if (path === "/v1/poll" && request.method === "POST") return cors(await poll(request, env));
      if (path === "/v1/vapidPublicKey" && request.method === "GET")
        return cors(json({ publicKey: env.VAPID_PUBLIC_KEY || null }));
      return cors(json({ error: "not_found" }, 404));
    } catch (e) {
      return cors(json({ error: String(e.message || e) }, 500));
    }
  },
};

async function register(request, env) {
  const body = await request.json();
  const deviceId = body.device_id;
  if (!deviceId || typeof deviceId !== "string") return json({ error: "device_id required" }, 400);
  if (!(await checkHmac(request, env, body))) return json({ error: "unauthorized" }, 401);
  const record = {
    device_id: deviceId,
    push_subscription: body.push_subscription || null,
    route: body.route || deviceId,
    updated_at: Date.now(),
  };
  await env.LIBERTY_KV.put(`dev:${deviceId}`, JSON.stringify(record), { expirationTtl: 60 * 60 * 24 * 30 });
  await env.LIBERTY_KV.put(`route:${record.route}`, deviceId, { expirationTtl: 60 * 60 * 24 * 30 });
  return json({ ok: true, device_id: deviceId });
}

async function wake(request, env) {
  const body = await request.json();
  if (!(await checkHmac(request, env, body))) return json({ error: "unauthorized" }, 401);
  const target = body.target;
  const mid = body.mid;
  if (!target || !mid) return json({ error: "target and mid required" }, 400);
  if (typeof mid === "string" && (mid.length > 128 || /[\s\n\r]{3,}/.test(mid)))
    return json({ error: "invalid mid" }, 400);
  const deviceId = await env.LIBERTY_KV.get(`route:${target}`);
  if (!deviceId) return json({ ok: true, queued: false, reason: "unknown_route" });
  const key = `wake:${deviceId}`;
  const existing = JSON.parse((await env.LIBERTY_KV.get(key)) || "[]");
  existing.push({ mid, ts: Date.now() });
  const trimmed = existing.slice(-50);
  await env.LIBERTY_KV.put(key, JSON.stringify(trimmed), { expirationTtl: WAKE_TTL_SEC });
  return json({ ok: true, queued: true });
}

async function poll(request, env) {
  const body = await request.json();
  const deviceId = body.device_id;
  if (!deviceId) return json({ error: "device_id required" }, 400);
  if (!(await checkHmac(request, env, body))) return json({ error: "unauthorized" }, 401);
  const key = `wake:${deviceId}`;
  const wakes = JSON.parse((await env.LIBERTY_KV.get(key)) || "[]");
  await env.LIBERTY_KV.delete(key);
  return json({ ok: true, wakes });
}

async function checkHmac(request, env, body) {
  if (!env.WAKE_HMAC_SECRET) return true;
  const sig = request.headers.get("x-liberty-sig");
  if (!sig) return false;
  const payload = JSON.stringify(body);
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(env.WAKE_HMAC_SECRET),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const mac = await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(payload));
  const hex = [...new Uint8Array(mac)].map((b) => b.toString(16).padStart(2, "0")).join("");
  if (hex.length !== sig.length) return false;
  let x = 0;
  for (let i = 0; i < hex.length; i++) x |= hex.charCodeAt(i) ^ sig.charCodeAt(i);
  return x === 0;
}

function json(obj, status = 200) {
  return new Response(JSON.stringify(obj), {
    status,
    headers: { "content-type": "application/json; charset=utf-8" },
  });
}

function cors(res) {
  const h = new Headers(res.headers);
  h.set("access-control-allow-origin", "*");
  h.set("access-control-allow-methods", "GET,POST,OPTIONS");
  h.set("access-control-allow-headers", "content-type,x-liberty-sig");
  return new Response(res.body, { status: res.status, headers: h });
}

function privacyHtml() {
  const html = `<!DOCTYPE html><html><head><meta charset="utf-8"/><title>Liberty Privacy</title>
<style>body{font-family:system-ui;max-width:720px;margin:40px auto;padding:0 16px;color:#e6edf3;background:#0d1117}h1{color:#58a6ff}</style></head>
<body><h1>Privacy — Liberty Messenger</h1>
<p>Local-first / P2P. Optional wake relay stores device route + opaque wake ids only. <strong>No message plaintext.</strong></p>
</body></html>`;
  return new Response(html, { headers: { "content-type": "text/html; charset=utf-8" } });
}

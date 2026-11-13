/**
 * Tauri 2 bridge — safe in pure browser (no-ops when not in desktop shell).
 */

type Invoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

function getInvoke(): Invoke | null {
  const w = window as unknown as {
    __TAURI__?: { core?: { invoke?: Invoke } };
    __TAURI_INTERNALS__?: { invoke?: Invoke };
  };
  return w.__TAURI__?.core?.invoke ?? w.__TAURI_INTERNALS__?.invoke ?? null;
}

export function isTauri(): boolean {
  return getInvoke() !== null;
}

export async function tauriAppInfo(): Promise<Record<string, unknown> | null> {
  const inv = getInvoke();
  if (!inv) return null;
  try {
    return await inv("app_info");
  } catch {
    return null;
  }
}

export async function tauriBackendUrl(): Promise<string | null> {
  const inv = getInvoke();
  if (!inv) return null;
  try {
    return await inv<string>("backend_url");
  } catch {
    return null;
  }
}

export async function tauriBackendHealth(): Promise<Record<string, unknown> | null> {
  const inv = getInvoke();
  if (!inv) return null;
  try {
    return await inv("backend_health");
  } catch {
    return null;
  }
}

export async function resolveApiBase(fallback = "http://127.0.0.1:8008"): Promise<string> {
  const fromTauri = await tauriBackendUrl();
  if (fromTauri) return fromTauri;
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_API_URL || fallback;
}

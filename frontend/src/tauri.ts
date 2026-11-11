/**
 * Optional Tauri 2 bridge. Safe in pure browser (no-ops).
 * Enabled when desktop shell sets withGlobalTauri or @tauri-apps/api is present.
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

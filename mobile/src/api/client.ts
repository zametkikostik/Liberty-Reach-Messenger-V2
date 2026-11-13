const API =
  (typeof process !== "undefined" &&
    (process as unknown as { env?: Record<string, string> }).env?.EXPO_PUBLIC_API_URL) ||
  "http://127.0.0.1:8008";

export type Auth = { token: string; user_id: string; username: string };
export type Chat = { id: string; title: string };
export type Msg = {
  id: string;
  body: string;
  sender_id: string;
  chat_id?: string;
  created_at?: string;
};

async function req<T>(
  path: string,
  opts: RequestInit & { token?: string } = {}
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(opts.headers as Record<string, string>),
  };
  if (opts.token) headers.Authorization = `Bearer ${opts.token}`;
  const res = await fetch(`${API}${path}`, { ...opts, headers });
  if (!res.ok) {
    const t = await res.text().catch(() => "");
    throw new Error(`${res.status} ${t || res.statusText}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

export function apiBase(): string {
  return API;
}

export async function register(
  username: string,
  password: string
): Promise<Auth> {
  return req<Auth>("/api/auth/register", {
    method: "POST",
    body: JSON.stringify({ username, password }),
  });
}

export async function login(username: string, password: string): Promise<Auth> {
  return req<Auth>("/api/auth/login", {
    method: "POST",
    body: JSON.stringify({ username, password }),
  });
}

export async function listChats(token: string): Promise<Chat[]> {
  return req<Chat[]>("/api/chats", { token });
}

export async function createChat(token: string, title: string): Promise<Chat> {
  return req<Chat>("/api/chats", {
    method: "POST",
    token,
    body: JSON.stringify({ title }),
  });
}

export async function listMessages(
  token: string,
  chatId: string
): Promise<Msg[]> {
  return req<Msg[]>(`/api/chats/${chatId}/messages`, { token });
}

export async function sendMessage(
  token: string,
  chatId: string,
  body: string
): Promise<Msg> {
  return req<Msg>(`/api/chats/${chatId}/messages`, {
    method: "POST",
    token,
    body: JSON.stringify({ body }),
  });
}

export function wsUrl(token: string): string {
  const base = API.replace(/^http/, "ws");
  return `${base}/ws?token=${encodeURIComponent(token)}`;
}

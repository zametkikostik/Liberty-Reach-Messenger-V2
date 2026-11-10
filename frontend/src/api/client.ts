const API = import.meta.env.VITE_API_URL || "";

export type Auth = { token: string; user_id: string; username: string };

export type Chat = {
  id: string;
  chat_type: string;
  title?: string | null;
  created_at: string;
};

export type Msg = {
  id: string;
  chat_id: string;
  sender_id: string;
  body: string;
  reply_to?: string | null;
  self_destruct_at?: string | null;
  created_at: string;
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
  const r = await fetch(`${API}${path}`, { ...opts, headers });
  if (!r.ok) throw new Error(`${r.status}`);
  return r.json();
}

export const api = {
  register: (username: string, password: string) =>
    req<Auth>("/api/v1/auth/register", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    }),
  login: (username: string, password: string) =>
    req<Auth>("/api/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    }),
  chats: (token: string) => req<Chat[]>("/api/v1/chats", { token }),
  createChat: (token: string, title: string) =>
    req<Chat>("/api/v1/chats", {
      method: "POST",
      token,
      body: JSON.stringify({ chat_type: "private", title, member_ids: [] }),
    }),
  messages: (token: string, chatId: string) =>
    req<Msg[]>(`/api/v1/chats/${chatId}/messages`, { token }),
  send: (token: string, chatId: string, body: string, ttl?: number) =>
    req<Msg>(`/api/v1/chats/${chatId}/messages`, {
      method: "POST",
      token,
      body: JSON.stringify({
        body,
        self_destruct_minutes: ttl ?? null,
      }),
    }),
};

import { useCallback, useEffect, useRef, useState } from "react";
import { api, Auth, Chat, Msg } from "./api/client";
import {
  decryptBody,
  encryptBody,
  exportChatKey,
  importChatKey,
  isEncrypted,
} from "./crypto/e2ee";
import { isTauri, tauriAppInfo } from "./tauri";

const AUTH_KEY = "liberty_auth_v2";

type ViewMsg = Msg & { plain?: string };

export default function App() {
  const [auth, setAuth] = useState<Auth | null>(() => {
    try {
      const raw = localStorage.getItem(AUTH_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch {
      return null;
    }
  });
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [chats, setChats] = useState<Chat[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [messages, setMessages] = useState<ViewMsg[]>([]);
  const [text, setText] = useState("");
  const [ttl, setTtl] = useState<number | "">("");
  const [wsStatus, setWsStatus] = useState<"off" | "on" | "err">("off");
  const [e2eeOn] = useState(true);
  const [desktopLabel, setDesktopLabel] = useState("");
  const wsRef = useRef<WebSocket | null>(null);
  const activeRef = useRef<string | null>(null);

  useEffect(() => {
    activeRef.current = active;
  }, [active]);

  useEffect(() => {
    if (!isTauri()) return;
    tauriAppInfo().then((info) => {
      if (info) setDesktopLabel(`Desktop · ${String(info.version ?? "2.0")}`);
      else setDesktopLabel("Desktop");
    });
  }, []);

  const persist = (a: Auth | null) => {
    setAuth(a);
    if (a) localStorage.setItem(AUTH_KEY, JSON.stringify(a));
    else localStorage.removeItem(AUTH_KEY);
  };

  const decryptList = useCallback(async (chatId: string, list: Msg[]) => {
    const out: ViewMsg[] = [];
    for (const m of list) {
      const plain = await decryptBody(chatId, m.body);
      out.push({ ...m, plain });
    }
    return out;
  }, []);

  const loadChats = useCallback(async () => {
    if (!auth) return;
    try {
      setChats(await api.chats(auth.token));
    } catch {
      setError("Не удалось загрузить чаты");
    }
  }, [auth]);

  useEffect(() => {
    if (auth) loadChats();
  }, [auth, loadChats]);

  // WebSocket: realtime ciphertext fan-out
  useEffect(() => {
    if (!auth) {
      wsRef.current?.close();
      wsRef.current = null;
      setWsStatus("off");
      return;
    }
    const proto = location.protocol === "https:" ? "wss" : "ws";
    const host = import.meta.env.VITE_WS_URL || `${proto}://${location.host}/ws`;
    const url = `${host}?token=${encodeURIComponent(auth.token)}`;
    let stopped = false;
    let sock: WebSocket;
    try {
      sock = new WebSocket(url);
    } catch {
      setWsStatus("err");
      return;
    }
    wsRef.current = sock;
    sock.onopen = () => {
      if (!stopped) setWsStatus("on");
      if (activeRef.current) {
        sock.send(JSON.stringify({ type: "join", chat_id: activeRef.current }));
      }
    };
    sock.onclose = () => {
      if (!stopped) setWsStatus("off");
    };
    sock.onerror = () => setWsStatus("err");
    sock.onmessage = async (ev) => {
      try {
        const v = JSON.parse(String(ev.data));
        if (v.type === "message" && v.chat_id === activeRef.current) {
          const plain = await decryptBody(v.chat_id, v.body);
          setMessages((prev) => {
            if (prev.some((m) => m.id === v.id)) return prev;
            return [
              ...prev,
              {
                id: v.id,
                chat_id: v.chat_id,
                sender_id: v.sender_id,
                body: v.body,
                created_at: v.created_at,
                plain,
              },
            ];
          });
        }
      } catch {
        /* ignore */
      }
    };
    return () => {
      stopped = true;
      sock.close();
    };
  }, [auth]);

  async function doRegister() {
    setError("");
    try {
      persist(await api.register(username.trim(), password));
    } catch {
      setError("Регистрация не удалась (имя занято или слишком короткое)");
    }
  }

  async function doLogin() {
    setError("");
    try {
      persist(await api.login(username.trim(), password));
    } catch {
      setError("Неверный логин или пароль");
    }
  }

  async function newChat() {
    if (!auth) return;
    const title = prompt("Название чата", "Новый чат") || "Новый чат";
    try {
      await api.createChat(auth.token, title);
      await loadChats();
    } catch {
      setError("Не удалось создать чат");
    }
  }

  async function openChat(id: string) {
    if (!auth) return;
    setActive(id);
    try {
      const list = await api.messages(auth.token, id);
      setMessages(await decryptList(id, list));
    } catch {
      setMessages([]);
    }
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "join", chat_id: id }));
    }
  }

  async function send() {
    if (!auth || !active || !text.trim()) return;
    const plain = text.trim();
    try {
      const body = e2eeOn ? await encryptBody(active, plain) : plain;
      const sent = await api.send(
        auth.token,
        active,
        body,
        typeof ttl === "number" ? ttl : undefined
      );
      setText("");
      const plainView = await decryptBody(active, sent.body);
      setMessages((prev) => {
        if (prev.some((m) => m.id === sent.id)) return prev;
        return [...prev, { ...sent, plain: plainView }];
      });
    } catch {
      setError("Не удалось отправить");
    }
  }

  function shareKey() {
    if (!active) return;
    const k = exportChatKey(active);
    if (k) {
      navigator.clipboard?.writeText(k);
      alert("Ключ скопирован в буфер. Передайте собеседнику безопасно.");
    }
  }

  function loadPeerKey() {
    if (!active) return;
    const raw = prompt("Вставьте ключ чата (base64)");
    if (raw) {
      importChatKey(active, raw.trim());
      alert("Ключ импортирован. Переоткройте чат для расшифровки.");
    }
  }

  function logout() {
    persist(null);
    setChats([]);
    setActive(null);
    setMessages([]);
  }

  if (!auth) {
    return (
      <div className="auth">
        <div className="card">
          <h1>Liberty Messenger V2</h1>
          <p className="muted">E2EE · WebSocket · Post-quantum ready</p>
          {error && <div className="error">{error}</div>}
          <input
            placeholder="Имя пользователя"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoComplete="username"
          />
          <input
            type="password"
            placeholder="Пароль"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
            onKeyDown={(e) => e.key === "Enter" && doLogin()}
          />
          <div className="row">
            <button className="btn" onClick={doLogin}>
              Войти
            </button>
            <button className="btn ghost" onClick={doRegister}>
              Регистрация
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="side-head">
          <strong>@{auth.username}</strong>
          <span className={`ws ${wsStatus}`} title="WebSocket">
            {wsStatus === "on" ? "● live" : wsStatus === "err" ? "○ err" : "○"}
          </span>
          <button className="btn sm ghost" onClick={logout}>
            Выйти
          </button>
        </div>
        <button className="btn block" onClick={newChat}>
          + Новый чат
        </button>
        <ul className="chat-list">
          {chats.map((c) => (
            <li
              key={c.id}
              className={c.id === active ? "active" : ""}
              onClick={() => openChat(c.id)}
            >
              {c.title}
            </li>
          ))}
        </ul>
      </aside>
      <main className="main">
        {active && (
          <div
            style={{
              display: "flex",
              gap: 8,
              padding: "8px 16px",
              borderBottom: "1px solid var(--border)",
              background: "var(--panel)",
              fontSize: 13,
            }}
          >
            <span style={{ color: "var(--muted)", marginRight: "auto" }}>
              {desktopLabel ? desktopLabel + " · " : ""}Клиентское AES-256-GCM · сервер хранит только ciphertext
            </span>
            <button className="btn sm ghost" onClick={shareKey}>
              Экспорт ключа
            </button>
            <button className="btn sm ghost" onClick={loadPeerKey}>
              Импорт ключа
            </button>
          </div>
        )}
        <div className="messages">
          {!active && <p className="empty">Выберите или создайте чат</p>}
          {messages.map((m) => (
            <div
              key={m.id}
              className={`bubble ${m.sender_id === auth.user_id ? "mine" : ""}`}
            >
              {m.plain ?? m.body}
              <div style={{ fontSize: 11, opacity: 0.55, marginTop: 4 }}>
                {isEncrypted(m.body) ? "🔒 e2ee" : "plaintext"}
                {m.self_destruct_at ? " · ⏱ self-destruct" : ""}
              </div>
            </div>
          ))}
        </div>
        {active && (
          <div className="composer">
            <input
              value={text}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && send()}
              placeholder="Сообщение (шифруется до отправки)…"
            />
            <select
              value={ttl === "" ? "" : String(ttl)}
              onChange={(e) =>
                setTtl(e.target.value === "" ? "" : Number(e.target.value))
              }
              style={{
                background: "var(--bg)",
                color: "var(--text)",
                border: "1px solid var(--border)",
                borderRadius: 10,
                padding: "0 8px",
              }}
              title="Self-destruct"
            >
              <option value="">∞</option>
              <option value="1">1 мин</option>
              <option value="60">1 час</option>
              <option value="1440">1 день</option>
            </select>
            <button className="btn" onClick={send}>
              Отправить
            </button>
          </div>
        )}
      </main>
    </div>
  );
}

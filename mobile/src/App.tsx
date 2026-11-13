/**
 * Liberty Messenger V2 — React Native / Expo mobile shell.
 *
 * E2EE: AES-256-GCM (`lr1:`) before POST; keys in Keychain/Keystore via expo-secure-store.
 * Delivery: REST + optional WebSocket join (server WsHub ciphertext fan-out).
 */

import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  SafeAreaView,
  View,
  Text,
  TextInput,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  StatusBar,
  ActivityIndicator,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from "react-native";
import {
  Auth,
  Chat,
  Msg,
  apiBase,
  createChat,
  listChats,
  listMessages,
  login,
  register,
  sendMessage,
  wsUrl,
} from "./api/client";
import {
  decryptBody,
  encryptBody,
  hasWebCrypto,
  isEncrypted,
} from "./crypto/e2ee";
import { usingNativeSecureStore } from "./storage/secure";

type Row = Msg & { plain?: string };

export default function App() {
  const [auth, setAuth] = useState<Auth | null>(null);
  const [user, setUser] = useState("");
  const [pass, setPass] = useState("");
  const [busy, setBusy] = useState(false);
  const [chats, setChats] = useState<Chat[]>([]);
  const [active, setActive] = useState<Chat | null>(null);
  const [msgs, setMsgs] = useState<Row[]>([]);
  const [draft, setDraft] = useState("");
  const [e2ee, setE2ee] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);

  const refreshChats = useCallback(async (token: string) => {
    const list = await listChats(token);
    setChats(list);
  }, []);

  const loadMessages = useCallback(
    async (token: string, chat: Chat) => {
      const list = await listMessages(token, chat.id);
      const rows: Row[] = [];
      for (const m of list) {
        const plain = e2ee ? await decryptBody(chat.id, m.body) : m.body;
        rows.push({ ...m, plain });
      }
      setMsgs(rows);
    },
    [e2ee]
  );

  useEffect(() => {
    if (!auth || !active) return;
    loadMessages(auth.token, active).catch((e) =>
      Alert.alert("Messages", String(e.message || e))
    );
  }, [auth, active, loadMessages]);

  useEffect(() => {
    if (!auth || !active) return;
    try {
      const ws = new WebSocket(wsUrl(auth.token));
      wsRef.current = ws;
      ws.onopen = () => {
        ws.send(JSON.stringify({ type: "join", chat_id: active.id }));
      };
      ws.onmessage = async (ev) => {
        try {
          const data = JSON.parse(String(ev.data));
          if (data.type === "message" && data.body) {
            const plain = e2ee
              ? await decryptBody(active.id, data.body)
              : data.body;
            setMsgs((prev) => [
              ...prev,
              {
                id: data.id || String(Date.now()),
                body: data.body,
                sender_id: data.sender_id || "?",
                plain,
              },
            ]);
          }
        } catch {
          /* ignore */
        }
      };
      return () => {
        ws.close();
        wsRef.current = null;
      };
    } catch {
      return;
    }
  }, [auth, active, e2ee]);

  async function doAuth(mode: "login" | "register") {
    if (!user.trim() || !pass) {
      Alert.alert("Auth", "username and password required");
      return;
    }
    setBusy(true);
    try {
      const a =
        mode === "login"
          ? await login(user.trim(), pass)
          : await register(user.trim(), pass);
      setAuth(a);
      await refreshChats(a.token);
    } catch (e: unknown) {
      Alert.alert("Auth", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onNewChat() {
    if (!auth) return;
    setBusy(true);
    try {
      const c = await createChat(auth.token, `Chat ${chats.length + 1}`);
      setChats((prev) => [c, ...prev]);
      setActive(c);
    } catch (e: unknown) {
      Alert.alert("Chat", e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onSend() {
    if (!auth || !active || !draft.trim()) return;
    const plain = draft.trim();
    setDraft("");
    setBusy(true);
    try {
      const body = e2ee ? await encryptBody(active.id, plain) : plain;
      const m = await sendMessage(auth.token, active.id, body);
      setMsgs((prev) => [...prev, { ...m, plain }]);
    } catch (e: unknown) {
      Alert.alert("Send", e instanceof Error ? e.message : String(e));
      setDraft(plain);
    } finally {
      setBusy(false);
    }
  }

  if (!auth) {
    return (
      <SafeAreaView style={styles.root}>
        <StatusBar barStyle="light-content" />
        <View style={styles.card}>
          <Text style={styles.logo}>Liberty</Text>
          <Text style={styles.sub}>Messenger V2 · Mobile</Text>
          <Text style={styles.meta}>
            API {apiBase()}
            {"\n"}
            SecureStore{" "}
            {usingNativeSecureStore() ? "native Keychain" : "memory fallback"}
            {" · "}WebCrypto {hasWebCrypto() ? "ok" : "missing"}
          </Text>
          <TextInput
            style={styles.input}
            placeholder="username"
            placeholderTextColor="#64748b"
            autoCapitalize="none"
            value={user}
            onChangeText={setUser}
          />
          <TextInput
            style={styles.input}
            placeholder="password"
            placeholderTextColor="#64748b"
            secureTextEntry
            value={pass}
            onChangeText={setPass}
          />
          {busy ? (
            <ActivityIndicator color="#38bdf8" />
          ) : (
            <View style={styles.row}>
              <TouchableOpacity style={styles.btn} onPress={() => doAuth("login")}>
                <Text style={styles.btnText}>Login</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.btn, styles.btnAlt]}
                onPress={() => doAuth("register")}
              >
                <Text style={styles.btnText}>Register</Text>
              </TouchableOpacity>
            </View>
          )}
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.root}>
      <StatusBar barStyle="light-content" />
      <KeyboardAvoidingView
        style={{ flex: 1 }}
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <View style={styles.top}>
          <Text style={styles.topTitle}>@{auth.username}</Text>
          <TouchableOpacity onPress={() => setE2ee((v) => !v)}>
            <Text style={styles.badge}>{e2ee ? "E2EE on" : "E2EE off"}</Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => {
              setAuth(null);
              setActive(null);
              setMsgs([]);
            }}
          >
            <Text style={styles.link}>Logout</Text>
          </TouchableOpacity>
        </View>

        <View style={styles.body}>
          <View style={styles.sidebar}>
            <TouchableOpacity style={styles.newBtn} onPress={onNewChat}>
              <Text style={styles.btnText}>+ Chat</Text>
            </TouchableOpacity>
            <FlatList
              data={chats}
              keyExtractor={(c) => c.id}
              renderItem={({ item }) => (
                <TouchableOpacity
                  style={[
                    styles.chatItem,
                    active?.id === item.id && styles.chatActive,
                  ]}
                  onPress={() => setActive(item)}
                >
                  <Text style={styles.chatTitle} numberOfLines={1}>
                    {item.title || item.id.slice(0, 8)}
                  </Text>
                </TouchableOpacity>
              )}
            />
          </View>

          <View style={styles.main}>
            {active ? (
              <>
                <Text style={styles.room}>{active.title}</Text>
                <FlatList
                  data={msgs}
                  keyExtractor={(m) => m.id}
                  style={{ flex: 1 }}
                  renderItem={({ item }) => (
                    <View style={styles.msg}>
                      <Text style={styles.msgMeta}>
                        {item.sender_id.slice(0, 8)}
                        {isEncrypted(item.body) ? " · lr1" : ""}
                      </Text>
                      <Text style={styles.msgBody}>{item.plain ?? item.body}</Text>
                    </View>
                  )}
                />
                <View style={styles.composer}>
                  <TextInput
                    style={[styles.input, { flex: 1, marginBottom: 0 }]}
                    placeholder={e2ee ? "Encrypted message…" : "Message…"}
                    placeholderTextColor="#64748b"
                    value={draft}
                    onChangeText={setDraft}
                    onSubmitEditing={onSend}
                  />
                  <TouchableOpacity style={styles.btn} onPress={onSend}>
                    <Text style={styles.btnText}>Send</Text>
                  </TouchableOpacity>
                </View>
              </>
            ) : (
              <Text style={styles.placeholder}>Select or create a chat</Text>
            )}
          </View>
        </View>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0b1220" },
  card: {
    margin: 24,
    marginTop: 80,
    padding: 20,
    borderRadius: 16,
    backgroundColor: "#111827",
    gap: 10,
  },
  logo: { color: "#f8fafc", fontSize: 28, fontWeight: "700" },
  sub: { color: "#94a3b8", marginBottom: 8 },
  meta: { color: "#64748b", fontSize: 11, marginBottom: 8 },
  input: {
    backgroundColor: "#0f172a",
    borderWidth: 1,
    borderColor: "#1e293b",
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
    color: "#e2e8f0",
    marginBottom: 8,
  },
  row: { flexDirection: "row", gap: 10 },
  btn: {
    backgroundColor: "#0284c7",
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderRadius: 10,
  },
  btnAlt: { backgroundColor: "#334155" },
  btnText: { color: "#f8fafc", fontWeight: "600" },
  top: {
    flexDirection: "row",
    alignItems: "center",
    paddingHorizontal: 12,
    paddingVertical: 10,
    borderBottomWidth: 1,
    borderBottomColor: "#1e293b",
    gap: 12,
  },
  topTitle: { color: "#e2e8f0", fontWeight: "600", flex: 1 },
  badge: {
    color: "#38bdf8",
    fontSize: 12,
    borderWidth: 1,
    borderColor: "#0369a1",
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 999,
  },
  link: { color: "#94a3b8", fontSize: 13 },
  body: { flex: 1, flexDirection: "row" },
  sidebar: {
    width: 120,
    borderRightWidth: 1,
    borderRightColor: "#1e293b",
    padding: 8,
  },
  newBtn: {
    backgroundColor: "#1e293b",
    padding: 10,
    borderRadius: 8,
    marginBottom: 8,
    alignItems: "center",
  },
  chatItem: { padding: 10, borderRadius: 8, marginBottom: 4 },
  chatActive: { backgroundColor: "#1e3a5f" },
  chatTitle: { color: "#cbd5e1", fontSize: 13 },
  main: { flex: 1, padding: 10 },
  room: { color: "#94a3b8", marginBottom: 8, fontSize: 13 },
  msg: {
    backgroundColor: "#111827",
    padding: 10,
    borderRadius: 10,
    marginBottom: 6,
  },
  msgMeta: { color: "#64748b", fontSize: 11, marginBottom: 2 },
  msgBody: { color: "#e2e8f0", fontSize: 15 },
  composer: { flexDirection: "row", gap: 8, alignItems: "center", marginTop: 8 },
  placeholder: { color: "#64748b", marginTop: 40, textAlign: "center" },
});

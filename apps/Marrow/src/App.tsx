import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, PublicIdentityDto, Session } from "./api/tauri";
import { AuthScreen } from "./components/screens/AuthScreen";
import { DashboardScreen } from "./components/screens/DashboardScreen";
import { ChatScreen } from "./components/screens/ChatScreen";

export function App() {
  const identity = useSignal<PublicIdentityDto | null>(null);
  const activeSession = useSignal<Session | null>(null);
  const loading = useSignal(true);

  useEffect(() => {
    (async () => {
      try {
        await api.initStorage();
        const current = await api.getCurrentIdentity();
        if (current) {
          identity.value = current;
        }
      } catch (e) {
        console.warn("No active vault session:", e);
      } finally {
        loading.value = false;
      }
    })();
  }, []);

  if (loading.value) {
    return (
      <div style={{ display: "flex", height: "100vh", alignItems: "center", justifyContent: "center", backgroundColor: "#0f172a", color: "#fff" }}>
        Loading Vault...
      </div>
    );
  }

  if (!identity.value) {
    return <AuthScreen onAuthenticated={(id) => (identity.value = id)} />;
  }

  if (activeSession.value) {
    return (
      <ChatScreen
        session={activeSession.value}
        onBack={() => (activeSession.value = null)}
      />
    );
  }

  return (
    <DashboardScreen
      identity={identity.value}
      onSelectSession={(session) => (activeSession.value = session)}
    />
  );
}
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

        await api.logoutIdentity();
      } catch (e) {
        console.warn("Storage init warning:", e);
      } finally {
        loading.value = false;
      }
    })();
  }, []);

  const handleLogout = async () => {
    try {
      await api.logoutIdentity();
    } catch (e) {
      console.error("Failed to logout:", e);
    } finally {
      identity.value = null;
      activeSession.value = null;
    }
  };

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
      onLogout={handleLogout}
    />
  );
}

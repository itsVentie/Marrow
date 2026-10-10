import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, PublicIdentityDto, Session } from "./api/tauri";
import { AuthScreen } from "./components/screens/AuthScreen";
import { DashboardScreen } from "./components/screens/DashboardScreen";
import { ChatScreen } from "./components/screens/ChatScreen";
import { SettingsModal } from "./components/modals/SettingsModal";

export function App() {
  const identity = useSignal<PublicIdentityDto | null>(null);
  const activeSession = useSignal<Session | null>(null);
  const showSettings = useSignal(false);
  const loading = useSignal(true);

  useEffect(() => {
    (async () => {
      try {
        await api.initStorage();

        const currentId = await api.getCurrentIdentity();
        if (currentId) {
          identity.value = currentId;
        }
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
      showSettings.value = false;
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
    return <AuthScreen onUnlocked={(id: PublicIdentityDto) => (identity.value = id)} />;
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
    <>
      <DashboardScreen
        identity={identity.value}
        onSelectSession={(session: Session) => (activeSession.value = session)}
        onOpenSettings={() => (showSettings.value = true)}
      />

      {showSettings.value && (
        <SettingsModal
          onClose={() => (showSettings.value = false)}
          onLogout={handleLogout}
        />
      )}
    </>
  );
}
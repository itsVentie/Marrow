import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, Contact, Session, PublicIdentityDto } from "../../api/tauri";
import styles from "../../styles/DashboardScreen.module.css";

interface Props {
  identity: PublicIdentityDto;
  onSelectSession: (session: Session) => void;
  onLogout: () => void;
}

export function DashboardScreen({ identity, onSelectSession, onLogout }: Props) {
  const contacts = useSignal<Contact[]>([]);
  const sessions = useSignal<Session[]>([]);

  const newContactAlias = useSignal("");
  const newContactPubkey = useSignal("");
  const error = useSignal<string | null>(null);

  const loadData = async () => {
    try {
      const [cList, sList] = await Promise.all([
        api.listContacts(),
        api.listSessions(),
      ]);
      contacts.value = cList;
      sessions.value = sList;
    } catch (err: any) {
      error.value = String(err);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleAddContact = async (e: Event) => {
    e.preventDefault();
    if (!newContactPubkey.value.trim()) return;

    try {
      await api.saveContact(
        newContactPubkey.value.trim(),
        newContactAlias.value.trim() || "Peer"
      );
      newContactAlias.value = "";
      newContactPubkey.value = "";
      await loadData();
    } catch (err: any) {
      error.value = "Failed to save contact: " + String(err);
    }
  };

  const handleStartSession = async (peerPubkey: string) => {
    try {
      const session = await api.createSession(peerPubkey);
      onSelectSession(session);
    } catch (err: any) {
      error.value = "Failed to create session: " + String(err);
    }
  };

  const handleDeleteSession = async (sessionId: string, e: Event) => {
    e.stopPropagation();
    try {
      await api.deleteSession(sessionId);
      await loadData();
    } catch (err: any) {
      error.value = String(err);
    }
  };

  const handleLogoutClick = async () => {
    await api.logoutIdentity();
    onLogout();
  };

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <div>
          <h3>Marrow Node</h3>
          <div className={styles.pubkey}>ID: {identity.pubkey_hex.slice(0, 16)}...</div>
        </div>
        <button onClick={handleLogoutClick} className={styles.logoutBtn}>Logout</button>
      </header>

      {error.value && <div className={styles.error}>{error.value}</div>}

      <div className={styles.grid}>
        {/* Active Sessions Panel */}
        <section className={styles.panel}>
          <h4>Active Sessions</h4>
          <div className={styles.list}>
            {sessions.value.map((s) => (
              <div
                key={s.id}
                onClick={() => onSelectSession(s)}
                className={styles.sessionCard}
              >
                <div>
                  <div className={styles.sessionTitle}>{s.id.slice(0, 12)}...</div>
                  <div className={styles.peerId}>Peer: {s.peer_pubkey_hex.slice(0, 10)}...</div>
                </div>
                <button
                  onClick={(e) => handleDeleteSession(s.id, e)}
                  className={styles.deleteBtn}
                >
                  ✕
                </button>
              </div>
            ))}
            {sessions.value.length === 0 && <p className={styles.empty}>No active sessions</p>}
          </div>
        </section>

        {/* Contacts & Add Panel */}
        <section className={styles.panel}>
          <h4>Add Contact</h4>
          <form onSubmit={handleAddContact} className={styles.form}>
            <input
              placeholder="Alias (e.g. Alice)"
              value={newContactAlias.value}
              onInput={(e) => newContactAlias.value = (e.target as HTMLInputElement).value}
            />
            <input
              placeholder="Public Key Hex"
              value={newContactPubkey.value}
              onInput={(e) => newContactPubkey.value = (e.target as HTMLInputElement).value}
            />
            <button type="submit">Save Contact</button>
          </form>

          <h4>Contacts</h4>
          <div className={styles.list}>
            {contacts.value.map((c) => (
              <div key={c.pubkey_hex} className={styles.contactCard}>
                <div>
                  <strong>{c.alias}</strong>
                  <div className={styles.peerId}>{c.pubkey_hex.slice(0, 12)}...</div>
                </div>
                <button
                  onClick={() => handleStartSession(c.pubkey_hex)}
                  className={styles.startBtn}
                >
                  Chat
                </button>
              </div>
            ))}
            {contacts.value.length === 0 && <p className={styles.empty}>No saved contacts</p>}
          </div>
        </section>
      </div>
    </div>
  );
}

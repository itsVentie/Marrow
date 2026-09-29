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

  const showProfileModal = useSignal(false);
  const showSettingsModal = useSignal(false);
  const copiedKey = useSignal(false);

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

  const handleCopyPubkey = () => {
    navigator.clipboard.writeText(identity.pubkey_hex);
    copiedKey.value = true;
    setTimeout(() => {
      copiedKey.value = false;
    }, 2000);
  };

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <div>
          <h3>Marrow</h3>
          <div className={styles.pubkey}>ID: {identity.pubkey_hex.slice(0, 16)}...</div>
        </div>
        <div className={styles.headerActions}>
          <button onClick={() => (showProfileModal.value = true)} className={styles.secondaryBtn}>
            Profile
          </button>
          <button onClick={() => (showSettingsModal.value = true)} className={styles.secondaryBtn}>
            Settings
          </button>
          <button onClick={handleLogoutClick} className={styles.logoutBtn}>
            Logout
          </button>
        </div>
      </header>

      {error.value && <div className={styles.error}>{error.value}</div>}

      <div className={styles.grid}>
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

        <section className={styles.panel}>
          <h4>Add Contact</h4>
          <form onSubmit={handleAddContact} className={styles.form}>
            <input
              placeholder="Alias (e.g. Alice)"
              value={newContactAlias.value}
              onInput={(e) => (newContactAlias.value = (e.target as HTMLInputElement).value)}
            />
            <input
              placeholder="Public Key Hex"
              value={newContactPubkey.value}
              onInput={(e) => (newContactPubkey.value = (e.target as HTMLInputElement).value)}
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

      {showProfileModal.value && (
        <div className={styles.modalOverlay} onClick={() => (showProfileModal.value = false)}>
          <div className={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <h4>Identity</h4>
            <div className={styles.modalField}>
              <label>Full Public Key (Hex):</label>
              <textarea readOnly value={identity.pubkey_hex} rows={3} />
              <button onClick={handleCopyPubkey} className={styles.primaryModalBtn}>
                {copiedKey.value ? "Copied!" : "Copy Public Key"}
              </button>
            </div>
            <div className={styles.modalInfo}>
              <p><strong>Crypto Suite:</strong> ML-KEM-768 + X25519</p>
              <p><strong>Status:</strong> Active & Loaded</p>
            </div>
            <button onClick={() => (showProfileModal.value = false)} className={styles.closeModalBtn}>
              Close
            </button>
          </div>
        </div>
      )}

      {showSettingsModal.value && (
        <div className={styles.modalOverlay} onClick={() => (showSettingsModal.value = false)}>
          <div className={styles.modalContent} onClick={(e) => e.stopPropagation()}>
            <h4>Settings</h4>
            <div className={styles.modalField}>
              <label>Relay Server Node:</label>
              <input type="text" placeholder="127.0.0.1:9090" disabled />
            </div>
            <div className={styles.modalField}>
              <label>Local Binding Port:</label>
              <input type="text" placeholder="0 (Auto-assigned)" disabled />
            </div>
            <p className={styles.stubNotice}>Network runtime options can be modified in config.toml</p>
            <button onClick={() => (showSettingsModal.value = false)} className={styles.closeModalBtn}>
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, Contact, Session, PublicIdentityDto } from "../../api/tauri";
import styles from "../../styles/DashboardScreen.module.css";

interface Props {
  identity: PublicIdentityDto;
  onSelectSession: (session: Session) => void;
}

export function DashboardScreen({ identity, onSelectSession }: Props) {
  const contacts = useSignal<Contact[]>([]);
  const sessions = useSignal<Session[]>([]);

  const newAlias = useSignal("");
  const newPubkey = useSignal("");
  const peerSessionPubkey = useSignal("");

  const loadData = async () => {
    try {
      const [cList, sList] = await Promise.all([
        api.listContacts(),
        api.listSessions(),
      ]);
      contacts.value = cList;
      sessions.value = sList;
    } catch (err) {
      console.error("Failed to load dashboard data:", err);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleAddContact = async (e: Event) => {
    e.preventDefault();
    if (!newAlias.value || !newPubkey.value) return;
    try {
      await api.saveContact(newPubkey.value, newAlias.value);
      newAlias.value = "";
      newPubkey.value = "";
      loadData();
    } catch (err) {
      alert("Error adding contact: " + err);
    }
  };

  const handleCreateSession = async (e: Event) => {
    e.preventDefault();
    if (!peerSessionPubkey.value) return;
    try {
      const session = await api.createSession(peerSessionPubkey.value);
      peerSessionPubkey.value = "";
      onSelectSession(session);
    } catch (err) {
      alert("Error creating session: " + err);
    }
  };

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <h3>Marrow</h3>
        <div className={styles.myKey}>My Key: {identity.pubkey_hex.slice(0, 16)}...</div>
      </header>

      <div className={styles.grid}>
        <section className={styles.section}>
          <h4>Active Sessions</h4>
          <form onSubmit={handleCreateSession} className={styles.inlineForm}>
            <input
              placeholder="Public Key (Hex)"
              value={peerSessionPubkey.value}
              onInput={(e) => (peerSessionPubkey.value = (e.target as HTMLInputElement).value)}
              className={styles.input}
            />
            <button type="submit" className={styles.btn}>Start Chat</button>
          </form>

          <div className={styles.list}>
            {sessions.value.map((s) => (
              <div key={s.id} onClick={() => onSelectSession(s)} className={styles.item}>
                <div>
                  <strong>Session:</strong> {s.id.slice(0, 8)}...
                </div>
                <div className={styles.subtext}>Peer: {s.peer_pubkey_hex.slice(0, 12)}...</div>
              </div>
            ))}
          </div>
        </section>

        <section className={styles.section}>
          <h4>Contacts</h4>
          <form onSubmit={handleAddContact} className={styles.verticalForm}>
            <input
              placeholder="Alias"
              value={newAlias.value}
              onInput={(e) => (newAlias.value = (e.target as HTMLInputElement).value)}
              className={styles.input}
            />
            <input
              placeholder="Public Key (Hex)"
              value={newPubkey.value}
              onInput={(e) => (newPubkey.value = (e.target as HTMLInputElement).value)}
              className={styles.input}
            />
            <button type="submit" className={styles.btn}>Save Contact</button>
          </form>

          <div className={styles.list}>
            {contacts.value.map((c) => (
              <div key={c.pubkey_hex} className={styles.item}>
                <div><strong>{c.alias}</strong></div>
                <div className={styles.subtext}>{c.pubkey_hex.slice(0, 16)}...</div>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}
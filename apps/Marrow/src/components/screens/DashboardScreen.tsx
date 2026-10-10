import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";

import {
  api,
  Contact,
  Session,
  PublicIdentityDto,
} from "@/api/tauri";

import styles from "@/styles/Dashboard/DashboardScreen.module.css";

interface Props {
  identity: PublicIdentityDto;
  onSelectSession: (session: Session) => void;
  onOpenProfile: () => void;
  onOpenSettings: () => void;
}

interface PluginStub {
  id: string;
  name: string;
  version: string;
  description: string;
  enabled: boolean;
}

export function DashboardScreen({
  identity,
  onSelectSession,
  onOpenProfile,
  onOpenSettings,
}: Props) {
  const contacts = useSignal<Contact[]>([]);
  const sessions = useSignal<Session[]>([]);

  const newContactAlias = useSignal("");
  const newContactPubkey = useSignal("");
  const newContactMultiaddr = useSignal("");

  const error = useSignal<string | null>(null);

  const showPluginsModal = useSignal(false);

  const plugins = useSignal<PluginStub[]>([
    {
      id: "1",
      name: "PassClip",
      version: "1.0.0",
      description: "Nothing",
      enabled: true,
    },
    {
      id: "2",
      name: "JarTight",
      version: "1.0.0",
      description: "Nothing",
      enabled: false,
    },
    {
      id: "3",
      name: "NearCloud",
      version: "1.0.0",
      description: "Nothing",
      enabled: false,
    },
  ]);

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

    if (!newContactPubkey.value.trim()) {
      return;
    }

    try {
      const multiaddr =
        newContactMultiaddr.value.trim();

      await api.saveContact(
        newContactPubkey.value.trim(),
        newContactAlias.value.trim() || "Peer",
        multiaddr || undefined,
      );

      newContactAlias.value = "";
      newContactPubkey.value = "";
      newContactMultiaddr.value = "";

      await loadData();
    } catch (err: any) {
      error.value =
        "Failed to save contact: " +
        String(err);
    }
  };

  const handleStartSession = async (
    peerPubkey: string,
  ) => {
    try {
      const session =
        await api.createSession(peerPubkey);

      onSelectSession(session);
    } catch (err: any) {
      error.value =
        "Failed to create session: " +
        String(err);
    }
  };

  const handleDeleteSession = async (
    sessionId: string,
    e: Event,
  ) => {
    e.stopPropagation();

    try {
      await api.deleteSession(sessionId);
      await loadData();
    } catch (err: any) {
      error.value = String(err);
    }
  };

  const togglePlugin = (id: string) => {
    plugins.value = plugins.value.map((p) =>
      p.id === id
        ? {
            ...p,
            enabled: !p.enabled,
          }
        : p,
    );
  };

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <div className={styles.headerBrand}>
          <h3>Marrow</h3>

          <div className={styles.pubkey}>
            ID: {identity.pubkey_hex.slice(0, 16)}...
          </div>
        </div>

        <div className={styles.headerActions}>
          <button
            onClick={() =>
              (showPluginsModal.value = true)
            }
            className={styles.secondaryBtn}
          >
            Plugins
          </button>

          <button
            onClick={onOpenSettings}
            className={styles.secondaryBtn}
          >
            Settings
          </button>

          <button
            onClick={onOpenProfile}
            className={styles.avatarBtn}
            title="Profile"
          >
            <div
              className={
                styles.avatarPlaceholder
              }
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.8"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118A7.5 7.5 0 0119.499 20.118 17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z"
                />
              </svg>
            </div>

            <span
              className={styles.avatarStatus}
            />
          </button>
        </div>
      </header>

      {error.value && (
        <div className={styles.error}>
          {error.value}
        </div>
      )}

      <div className={styles.grid}>
        <section className={styles.panel}>
          <div className={styles.panelHeader}>
            <h4>Active Sessions</h4>

            <span className={styles.badge}>
              {sessions.value.length}
            </span>
          </div>

          <div className={styles.list}>
            {sessions.value.map((s) => (
              <div
                key={s.id}
                onClick={() =>
                  onSelectSession(s)
                }
                className={styles.sessionCard}
              >
                <div>
                  <div
                    className={
                      styles.sessionTitle
                    }
                  >
                    {s.id.slice(0, 12)}...
                  </div>

                  <div
                    className={
                      styles.peerId
                    }
                  >
                    Peer:{" "}
                    {s.peer_pubkey_hex.slice(
                      0,
                      10,
                    )}
                    ...
                  </div>
                </div>

                <button
                  onClick={(e) =>
                    handleDeleteSession(
                      s.id,
                      e,
                    )
                  }
                  className={
                    styles.deleteBtn
                  }
                >
                  ✕
                </button>
              </div>
            ))}

            {sessions.value.length === 0 && (
              <p className={styles.empty}>
                No active sessions
              </p>
            )}
          </div>
        </section>

        <section className={styles.panel}>
          <h4>Add Contact</h4>

          <form
            onSubmit={handleAddContact}
            className={styles.form}
          >
            <input
              placeholder="Alias (e.g. Alice)"
              value={newContactAlias.value}
              onInput={(e) =>
                (newContactAlias.value = (
                  e.target as HTMLInputElement
                ).value)
              }
            />

            <input
              placeholder="Public Key Hex"
              value={newContactPubkey.value}
              onInput={(e) =>
                (newContactPubkey.value = (
                  e.target as HTMLInputElement
                ).value)
              }
            />

            <input
              placeholder="/ip4/.../tcp/.../p2p/..."
              value={
                newContactMultiaddr.value
              }
              onInput={(e) =>
                (newContactMultiaddr.value = (
                  e.target as HTMLInputElement
                ).value)
              }
            />

            <button type="submit">
              Save Contact
            </button>
          </form>

          <div className={styles.panelHeader}>
            <h4>Contacts</h4>

            <span className={styles.badge}>
              {contacts.value.length}
            </span>
          </div>

          <div className={styles.list}>
            {contacts.value.map((c) => (
              <div
                key={c.pubkey_hex}
                className={styles.contactCard}
              >
                <div>
                  <strong>{c.alias}</strong>

                  <div
                    className={
                      styles.peerId
                    }
                  >
                    {c.pubkey_hex.slice(
                      0,
                      12,
                    )}
                    ...
                  </div>
                </div>

                <button
                  onClick={() =>
                    handleStartSession(
                      c.pubkey_hex,
                    )
                  }
                  className={
                    styles.startBtn
                  }
                >
                  Chat
                </button>
              </div>
            ))}

            {contacts.value.length === 0 && (
              <p className={styles.empty}>
                No saved contacts
              </p>
            )}
          </div>
        </section>
      </div>

      {showPluginsModal.value && (
        <div
          className={
            styles.modalOverlay
          }
          onClick={() =>
            (showPluginsModal.value = false)
          }
        >
          <div
            className={
              styles.modalContent
            }
            onClick={(e) =>
              e.stopPropagation()
            }
          >
            <div
              className={
                styles.modalHeaderWithBadge
              }
            >
              <h4>Plugin Manager</h4>
            </div>

            <p
              className={
                styles.stubNotice
              }
            >
              Extensions can be toggled below:
            </p>

            <div
              className={
                styles.pluginList
              }
            >
              {plugins.value.map(
                (plugin) => (
                  <div
                    key={plugin.id}
                    className={
                      styles.pluginItem
                    }
                  >
                    <div
                      className={
                        styles.pluginInfo
                      }
                    >
                      <div
                        className={
                          styles.pluginTitle
                        }
                      >
                        <strong>
                          {plugin.name}
                        </strong>

                        <span
                          className={
                            styles.pluginVersion
                          }
                        >
                          v{plugin.version}
                        </span>
                      </div>

                      <p
                        className={
                          styles.pluginDesc
                        }
                      >
                        {plugin.description}
                      </p>
                    </div>

                    <button
                      onClick={() =>
                        togglePlugin(
                          plugin.id,
                        )
                      }
                      className={
                        plugin.enabled
                          ? styles.pluginDisableBtn
                          : styles.pluginEnableBtn
                      }
                    >
                      {plugin.enabled
                        ? "Disable"
                        : "Enable"}
                    </button>
                  </div>
                ),
              )}
            </div>

            <button
              onClick={() =>
                (showPluginsModal.value = false)
              }
              className={
                styles.closeModalBtn
              }
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
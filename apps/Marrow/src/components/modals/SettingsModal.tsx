import { useEffect } from "preact/hooks";

import styles from "../../styles/SettingsModal.module.css";

interface Props {
  onClose: () => void;
  onLogout: () => void;
}

interface StubField {
  id: string;
  label: string;
  placeholder: string;
}

const NETWORK_FIELDS: StubField[] = [
  {
    id: "relay",
    label: "Relay Server Node",
    placeholder: "127.0.0.1:9090",
  },
  {
    id: "port",
    label: "Local Binding Port",
    placeholder: "0 (Auto-assigned)",
  },
  {
    id: "protocol",
    label: "Network Protocol",
    placeholder: "QUIC over UDP",
  },
];

export function SettingsModal({ onClose, onLogout }: Props) {
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);

    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div
        className={styles.content}
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-modal-title"
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.header}>
          <h4 id="settings-modal-title" className={styles.title}>
            Settings
          </h4>

          <span className={styles.stubTag}>STUB</span>
        </div>

        {NETWORK_FIELDS.map((field) => (
          <div key={field.id} className={styles.field}>
            <label htmlFor={`settings-${field.id}`}>
              {field.label} [STUB]
            </label>

            <input
              id={`settings-${field.id}`}
              type="text"
              placeholder={field.placeholder}
              disabled
            />
          </div>
        ))}

        <p className={styles.notice}>
          Network runtime options can be modified in config.toml
          [STUB]
        </p>

        <div className={styles.dangerZone}>
          <label className={styles.dangerLabel}>
            Session Control:
          </label>

          <button onClick={onLogout} className={styles.logoutBtn}>
            Logout
          </button>
        </div>

        <button onClick={onClose} className={styles.closeBtn}>
          Close
        </button>
      </div>
    </div>
  );
}
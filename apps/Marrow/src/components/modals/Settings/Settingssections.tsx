import { useSignal } from "@preact/signals";

import { PublicIdentityDto } from "@/api/tauri";

import styles from "@/styles/Settings/SettingsModal.module.css";

export type StubRow =
  | {
      kind: "toggle";
      id: string;
      label: string;
      description?: string;
      checked: boolean;
    }
  | {
      kind: "field";
      id: string;
      label: string;
      description?: string;
      placeholder: string;
    };

export interface StubSectionData {
  rows: StubRow[];
  note?: string;
}

export function StubSection({ rows, note }: StubSectionData) {
  return (
    <>
      <p className={styles.stubBanner}>
        Preview only. These options are not functional yet.
      </p>

      <div className={styles.rows}>
        {rows.map((row) => (
          <div key={row.id} className={styles.row}>
            <div className={styles.rowText}>
              <label
                htmlFor={`settings-${row.id}`}
                className={styles.rowLabel}
              >
                {row.label}
              </label>

              {row.description && (
                <span className={styles.rowDesc}>
                  {row.description}
                </span>
              )}
            </div>

            {row.kind === "toggle" ? (
              <span className={styles.toggle}>
                <input
                  id={`settings-${row.id}`}
                  type="checkbox"
                  checked={row.checked}
                  disabled
                />

                <span className={styles.slider} />
              </span>
            ) : (
              <input
                id={`settings-${row.id}`}
                className={styles.fieldInput}
                type="text"
                placeholder={row.placeholder}
                disabled
              />
            )}
          </div>
        ))}
      </div>

      {note && <p className={styles.notice}>{note}</p>}
    </>
  );
}

interface AccountSectionProps {
  identity: PublicIdentityDto;
}

export function AccountSection({ identity }: AccountSectionProps) {
  const copyState = useSignal<"idle" | "copied" | "failed">("idle");

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(identity.pubkey_hex);
      copyState.value = "copied";
    } catch {
      copyState.value = "failed";
    }

    setTimeout(() => {
      copyState.value = "idle";
    }, 2000);
  };

  const copyLabel =
    copyState.value === "copied"
      ? "Copied!"
      : copyState.value === "failed"
        ? "Copy failed"
        : "Copy Public Key";

  return (
    <>
      <div className={styles.profileCard}>
        <div className={styles.avatar}>
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z"
            />
          </svg>
        </div>

        <div className={styles.profileText}>
          <span className={styles.profileName}>User</span>

          <span className={styles.profileId}>
            ID: {identity.pubkey_hex.slice(0, 16)}...
          </span>
        </div>
      </div>

      <div className={styles.keyBlock}>
        <label
          htmlFor="settings-pubkey"
          className={styles.rowLabel}
        >
          Public key (hex)
        </label>

        <textarea
          id="settings-pubkey"
          className={styles.keyArea}
          readOnly
          rows={3}
          value={identity.pubkey_hex}
        />

        <span className={styles.rowDesc}>
          Share this key so others can add you as a contact.
        </span>

        <button onClick={handleCopy} className={styles.primaryBtn}>
          {copyLabel}
        </button>
      </div>
    </>
  );
}

export function AboutSection() {
  return (
    <dl className={styles.about}>
      <div className={styles.aboutRow}>
        <dt>Application</dt>
        <dd>Marrow</dd>
      </div>

      <div className={styles.aboutRow}>
        <dt>Message encryption</dt>
        <dd>Double Ratchet</dd>
      </div>

      <div className={styles.aboutRow}>
        <dt>Local storage</dt>
        <dd>XChaCha20-Poly1305</dd>
      </div>
    </dl>
  );
}
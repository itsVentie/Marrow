import { useEffect } from "preact/hooks";

import { PublicIdentityDto } from "@/api/tauri";

import { AccountSection } from "../Settings/Settingssections";

import styles from "@/styles/Profile/ProfileModal.module.css";

interface Props {
  identity: PublicIdentityDto;
  onClose: () => void;
}

export function ProfileModal({ identity, onClose }: Props) {
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
        aria-labelledby="profile-modal-title"
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.header}>
          <h4 id="profile-modal-title" className={styles.title}>
            Profile
          </h4>

          <button
            className={styles.closeBtn}
            onClick={onClose}
            aria-label="Close profile"
          >
            ✕
          </button>
        </div>

        <div className={styles.body}>
          <AccountSection identity={identity} />
        </div>
      </div>
    </div>
  );
}
import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";

import type { PublicIdentityDto } from "@/api/tauri";

import {
  AboutSection,
  AccountSection,
  StubSection,
  type StubSectionData,
} from "./Settingssections";

import { Web3Section } from "./Sections/Web3Section";
import { GeneralSection } from "./Sections/GeneralSection";
import styles from "@/styles/Settings/SettingsModal.module.css";

interface Props {
  identity: PublicIdentityDto;
  onClose: () => void;
  onLogout: () => void;
}

type SectionId =
  | "general"
  | "account"
  | "privacy"
  | "network"
  | "web3"
  | "notifications"
  | "appearance"
  | "about";

const SECTIONS: { id: SectionId; label: string }[] = [
  { id: "general", label: "General" },
  { id: "account", label: "My Account" },
  { id: "privacy", label: "Privacy & Security" },
  { id: "network", label: "Network" },
  { id: "web3", label: "Web3 & Wallets" },
  { id: "notifications", label: "Notifications" },
  { id: "appearance", label: "Appearance" },
  { id: "about", label: "About" },
];

const STUB_DATA: Partial<Record<SectionId, StubSectionData>> = {
  privacy: {
    rows: [
      {
        kind: "field",
        id: "autolock",
        label: "Auto-lock",
        description: "Lock the app after a period of inactivity",
        placeholder: "Never",
      },
      {
        kind: "toggle",
        id: "search-index",
        label: "Local message search",
        description: "Keep a local index so messages can be searched",
        checked: true,
      },
      {
        kind: "toggle",
        id: "clear-on-logout",
        label: "Clear chats on logout",
        description: "Remove local message history when you log out",
        checked: false,
      },
    ],
  },

  network: {
    rows: [
      {
        kind: "field",
        id: "relay",
        label: "Relay Server Node",
        placeholder: "127.0.0.1:9090",
      },
      {
        kind: "field",
        id: "port",
        label: "Local Binding Port",
        placeholder: "0 (Auto-assigned)",
      },
      {
        kind: "field",
        id: "protocol",
        label: "Network Protocol",
        placeholder: "QUIC over UDP",
      },
    ],
    note: "Network runtime options can be modified in config.toml [STUB]",
  },

  notifications: {
    rows: [
      {
        kind: "toggle",
        id: "notify-messages",
        label: "Message notifications",
        description: "Show a notification for new messages",
        checked: true,
      },
      {
        kind: "toggle",
        id: "notify-previews",
        label: "Show message previews",
        description: "Show message content in notifications",
        checked: true,
      },
      {
        kind: "toggle",
        id: "keep-in-tray",
        label: "Keep running in tray",
        description: "Hide the window instead of quitting when it is closed",
        checked: true,
      },
    ],
  },

  appearance: {
    rows: [
      {
        kind: "field",
        id: "theme",
        label: "Theme",
        placeholder: "Dark",
      },
      {
        kind: "field",
        id: "text-size",
        label: "Text size",
        placeholder: "Default",
      },
    ],
  },
};

export function SettingsModal({
  identity,
  onClose,
  onLogout,
}: Props) {
  const section = useSignal<SectionId>("account");

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);

    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [onClose]);

  const currentLabel =
    SECTIONS.find((item) => item.id === section.value)?.label ?? "Settings";

  const renderSection = () => {
    switch (section.value) {
      case "general":
        return <GeneralSection />;
      case "account":
        return <AccountSection identity={identity} />;

      case "web3":
        return <Web3Section />;

      case "about":
        return <AboutSection />;

      default: {
        const data = STUB_DATA[section.value];

        return data ? <StubSection {...data} /> : null;
      }
    }
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div
        className={styles.content}
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-modal-title"
        onClick={(event) => event.stopPropagation()}
      >
        <nav
          className={styles.sidebar}
          aria-label="Settings sections"
        >
          <h4
            id="settings-modal-title"
            className={styles.sidebarTitle}
          >
            Settings
          </h4>

          {SECTIONS.map((item) => (
            <button
              key={item.id}
              type="button"
              className={[
                styles.navItem,
                section.value === item.id
                  ? styles.navItemActive
                  : "",
              ]
                .filter(Boolean)
                .join(" ")}
              aria-current={
                section.value === item.id ? "page" : undefined
              }
              onClick={() => {
                section.value = item.id;
              }}
            >
              {item.label}
            </button>
          ))}

          <button
            type="button"
            className={styles.logoutBtn}
            onClick={onLogout}
          >
            Log out
          </button>
        </nav>

        <div className={styles.detail}>
          <div className={styles.detailHeader}>
            <h4 className={styles.detailTitle}>{currentLabel}</h4>

            <button
              type="button"
              className={styles.closeBtn}
              onClick={onClose}
              aria-label="Close settings"
            >
              ✕
            </button>
          </div>

          <div className={styles.detailBody}>
            {renderSection()}
          </div>
        </div>
      </div>
    </div>
  );
}
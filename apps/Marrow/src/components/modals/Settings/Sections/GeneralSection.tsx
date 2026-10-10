import { useEffect } from "preact/hooks";
import { useSignal } from "@preact/signals";

import styles from "@/styles/Settings/SettingsModal.module.css";

interface GeneralSettings {
  language: string;
  theme: string;
  fontSize: string;
  launchAtStartup: boolean;
  minimizeToTray: boolean;
  startMinimized: boolean;
  sendOnEnter: boolean;
  confirmBeforeLogout: boolean;
  animations: boolean;
}

const STORAGE_KEY = "marrow.general-settings.v1";

const DEFAULT_SETTINGS: GeneralSettings = {
  language: "en",
  theme: "dark",
  fontSize: "medium",
  launchAtStartup: false,
  minimizeToTray: true,
  startMinimized: false,
  sendOnEnter: true,
  confirmBeforeLogout: true,
  animations: true,
};

function loadSettings(): GeneralSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);

    if (!raw) return { ...DEFAULT_SETTINGS };

    const parsed: unknown = JSON.parse(raw);

    if (!parsed || typeof parsed !== "object") {
      return { ...DEFAULT_SETTINGS };
    }

    const value = parsed as Partial<GeneralSettings>;

    return {
      language:
        typeof value.language === "string"
          ? value.language
          : DEFAULT_SETTINGS.language,
      theme:
        typeof value.theme === "string"
          ? value.theme
          : DEFAULT_SETTINGS.theme,
      fontSize:
        typeof value.fontSize === "string"
          ? value.fontSize
          : DEFAULT_SETTINGS.fontSize,
      launchAtStartup:
        typeof value.launchAtStartup === "boolean"
          ? value.launchAtStartup
          : DEFAULT_SETTINGS.launchAtStartup,
      minimizeToTray:
        typeof value.minimizeToTray === "boolean"
          ? value.minimizeToTray
          : DEFAULT_SETTINGS.minimizeToTray,
      startMinimized:
        typeof value.startMinimized === "boolean"
          ? value.startMinimized
          : DEFAULT_SETTINGS.startMinimized,
      sendOnEnter:
        typeof value.sendOnEnter === "boolean"
          ? value.sendOnEnter
          : DEFAULT_SETTINGS.sendOnEnter,
      confirmBeforeLogout:
        typeof value.confirmBeforeLogout === "boolean"
          ? value.confirmBeforeLogout
          : DEFAULT_SETTINGS.confirmBeforeLogout,
      animations:
        typeof value.animations === "boolean"
          ? value.animations
          : DEFAULT_SETTINGS.animations,
    };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export function GeneralSection() {
  const settings = useSignal<GeneralSettings>({ ...DEFAULT_SETTINGS });

  useEffect(() => {
    settings.value = loadSettings();
  }, []);

  function update<K extends keyof GeneralSettings>(
    key: K,
    value: GeneralSettings[K],
  ) {
    const next = {
      ...settings.value,
      [key]: value,
    };

    settings.value = next;

    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch (error) {
      console.error("Failed to save general settings:", error);
    }
  }

  function resetSettings() {
    settings.value = { ...DEFAULT_SETTINGS };

    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify(DEFAULT_SETTINGS),
      );
    } catch (error) {
      console.error("Failed to reset general settings:", error);
    }
  }

  const current = settings.value;

  return (
    <section className={styles.generalSection}>
      <p className={styles.generalDescription}>
        Configure the everyday behavior and appearance of Marrow.
      </p>

      <div className={styles.generalGroup}>
        <h5>Language & appearance</h5>

        <label className={styles.generalRow}>
          <span>
            <strong>Interface language</strong>
            <small>Language used throughout the application</small>
          </span>

          <select
            value={current.language}
            onChange={(event) =>
              update(
                "language",
                (event.currentTarget as HTMLSelectElement).value,
              )
            }
          >
            <option value="en">English</option>
            <option value="ru">Русский</option>
            <option value="fr">Français</option>
            <option value="de">Deutsch</option>
            <option value="kk">Қазақша</option>
          </select>
        </label>

        <label className={styles.generalRow}>
          <span>
            <strong>Theme</strong>
            <small>Application color scheme</small>
          </span>

          <select
            value={current.theme}
            onChange={(event) =>
              update(
                "theme",
                (event.currentTarget as HTMLSelectElement).value,
              )
            }
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">Follow system</option>
          </select>
        </label>

        <label className={styles.generalRow}>
          <span>
            <strong>Font size</strong>
            <small>Text size in conversations and menus</small>
          </span>

          <select
            value={current.fontSize}
            onChange={(event) =>
              update(
                "fontSize",
                (event.currentTarget as HTMLSelectElement).value,
              )
            }
          >
            <option value="small">Small</option>
            <option value="medium">Medium</option>
            <option value="large">Large</option>
          </select>
        </label>

        <SettingToggle
          title="Interface animations"
          description="Enable transitions and motion effects"
          checked={current.animations}
          onChange={(value) => update("animations", value)}
        />
      </div>

      <div className={styles.generalGroup}>
        <h5>Startup & window</h5>

        <SettingToggle
          title="Launch at startup"
          description="Start Marrow when you sign in to your computer"
          checked={current.launchAtStartup}
          onChange={(value) => update("launchAtStartup", value)}
        />

        <SettingToggle
          title="Minimize to tray"
          description="Keep the application running when its window is hidden"
          checked={current.minimizeToTray}
          onChange={(value) => update("minimizeToTray", value)}
        />

        <SettingToggle
          title="Start minimized"
          description="Open Marrow in the background on launch"
          checked={current.startMinimized}
          onChange={(value) => update("startMinimized", value)}
        />
      </div>

      <div className={styles.generalGroup}>
        <h5>Messaging & interaction</h5>

        <SettingToggle
          title="Send messages with Enter"
          description="Use Shift + Enter for a new line"
          checked={current.sendOnEnter}
          onChange={(value) => update("sendOnEnter", value)}
        />

        <SettingToggle
          title="Confirm before logout"
          description="Ask for confirmation before ending your session"
          checked={current.confirmBeforeLogout}
          onChange={(value) => update("confirmBeforeLogout", value)}
        />
      </div>

      <div className={styles.generalFooter}>
        <button type="button" onClick={resetSettings}>
          Restore defaults
        </button>
      </div>
    </section>
  );
}

interface SettingToggleProps {
  title: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function SettingToggle({
  title,
  description,
  checked,
  onChange,
}: SettingToggleProps) {
  return (
    <label className={styles.generalRow}>
      <span>
        <strong>{title}</strong>
        <small>{description}</small>
      </span>

      <input
        type="checkbox"
        checked={checked}
        onChange={(event) =>
          onChange(
            (event.currentTarget as HTMLInputElement).checked,
          )
        }
      />
    </label>
  );
}
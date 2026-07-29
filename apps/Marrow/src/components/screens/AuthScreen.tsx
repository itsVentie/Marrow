import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, KeyFileInfoDto, PublicIdentityDto } from "../../api/tauri";
import styles from "../../styles/AuthScreen.module.css";

interface Props {
  onAuthenticated: (identity: PublicIdentityDto) => void;
}

export function AuthScreen({ onAuthenticated }: Props) {
  const password = useSignal("");
  const keyAlias = useSignal("");
  const isCreateMode = useSignal(false);
  const loading = useSignal(false);
  const error = useSignal<string | null>(null);

  const availableKeys = useSignal<KeyFileInfoDto[]>([]);
  const selectedKeyPath = useSignal<string>("");

  const refreshKeyList = async () => {
    try {
      const keys = await api.listIdentityFiles();
      availableKeys.value = keys;
      if (keys.length > 0 && !selectedKeyPath.value) {
        selectedKeyPath.value = keys[0].path;
      }
    } catch (err) {
      console.error("Failed to fetch keys:", err);
    }
  };

  useEffect(() => {
    const checkState = async () => {
      try {
        await api.initStorage();
        const active = await api.getCurrentIdentity();
        if (active) {
          onAuthenticated(active);
          return;
        }
        await refreshKeyList();
      } catch (err) {
        console.error("Storage init error:", err);
      }
    };
    checkState();
  }, []);

  const handleAction = async (e: Event) => {
    e.preventDefault();
    if (!password.value) {
      error.value = "Password cannot be empty";
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      if (isCreateMode.value) {
        const identity = await api.createIdentity(
          password.value,
          keyAlias.value || undefined
        );
        onAuthenticated(identity);
      } else {
        if (!selectedKeyPath.value) {
          error.value = "Select a key file first";
          loading.value = false;
          return;
        }
        const identity = await api.unlockIdentityFromFile(
          selectedKeyPath.value,
          password.value
        );
        onAuthenticated(identity);
      }
    } catch (err: any) {
      error.value = typeof err === "string" ? err : "Authentication failed";
    } finally {
      loading.value = false;
    }
  };

  const handleExternalImport = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [{ name: "Identity Key", extensions: ["key"] }],
      });

      if (selected && typeof selected === "string") {
        const imported = await api.importIdentityFile(selected);
        await refreshKeyList();
        selectedKeyPath.value = imported.path;
      }
    } catch (err: any) {
      error.value = typeof err === "string" ? err : "Failed to import key file";
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <h2 className={styles.title}>
          {isCreateMode.value ? "Create New Key" : "Unlock Identity"}
        </h2>
        {error.value && <div className={styles.error}>{error.value}</div>}

        <form onSubmit={handleAction} className={styles.form}>
          {!isCreateMode.value && (
            <div className={styles.fieldGroup}>
              <label className={styles.label}>Select Identity Key</label>
              {availableKeys.value.length > 0 ? (
                <select
                  value={selectedKeyPath.value}
                  onChange={(e) =>
                    (selectedKeyPath.value = (
                      e.target as HTMLSelectElement
                    ).value)
                  }
                  className={styles.select}
                  disabled={loading.value}
                >
                  {availableKeys.value.map((k) => (
                    <option key={k.path} value={k.path}>
                      {k.filename}
                    </option>
                  ))}
                </select>
              ) : (
                <div className={styles.emptyNotice}>
                  No saved keys found. Import one or create new.
                </div>
              )}
            </div>
          )}

          {isCreateMode.value && (
            <div className={styles.fieldGroup}>
              <label className={styles.label}>Key Name / Alias (Optional)</label>
              <input
                type="text"
                placeholder="e.g. main_account"
                value={keyAlias.value}
                onInput={(e) =>
                  (keyAlias.value = (e.target as HTMLInputElement).value)
                }
                disabled={loading.value}
                className={styles.input}
              />
            </div>
          )}

          <div className={styles.fieldGroup}>
            <label className={styles.label}>Master Password</label>
            <input
              type="password"
              placeholder={
                isCreateMode.value
                  ? "Set Master Password"
                  : "Enter Master Password"
              }
              value={password.value}
              onInput={(e) =>
                (password.value = (e.target as HTMLInputElement).value)
              }
              disabled={loading.value}
              className={styles.input}
            />
          </div>

          <button
            type="submit"
            disabled={
              loading.value ||
              (!isCreateMode.value && availableKeys.value.length === 0)
            }
            className={styles.button}
          >
            {loading.value
              ? "Processing..."
              : isCreateMode.value
              ? "Generate Key & Login"
              : "Unlock Account"}
          </button>
        </form>

        {!isCreateMode.value && (
          <button
            onClick={handleExternalImport}
            className={styles.importBtn}
            disabled={loading.value}
          >
            Import .key file...
          </button>
        )}

        <button
          onClick={() => {
            isCreateMode.value = !isCreateMode.value;
            error.value = null;
          }}
          className={styles.toggleBtn}
        >
          {isCreateMode.value
            ? "Already have a key? Unlock existing"
            : "No key? Generate new identity"}
        </button>
      </div>
    </div>
  );
}

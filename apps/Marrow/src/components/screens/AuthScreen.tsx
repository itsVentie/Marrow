import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, KeyFileInfoDto, PublicIdentityDto } from "../../api/tauri";
import styles from "../../styles/AuthScreen.module.css";

interface Props {
  onUnlocked: (identity: PublicIdentityDto) => void;
}

export function AuthScreen({ onUnlocked }: Props) {
  const isCreateMode = useSignal(false);
  const keyFiles = useSignal<KeyFileInfoDto[]>([]);
  const selectedFile = useSignal<string>("");

  const password = useSignal("");
  const alias = useSignal("");
  const error = useSignal<string | null>(null);
  const isLoading = useSignal(false);

  const loadKeys = async () => {
    try {
      await api.initStorage();
      const files = await api.listIdentityFiles();
      keyFiles.value = files;
      if (files.length > 0 && !selectedFile.value) {
        selectedFile.value = files[0].path;
      }
    } catch (err: any) {
      error.value = String(err);
    }
  };

  useEffect(() => {
    loadKeys();
  }, []);

  const handleUnlock = async (e: Event) => {
    e.preventDefault();
    if (!selectedFile.value || !password.value) return;

    isLoading.value = true;
    error.value = null;

    try {
      const identity = await api.unlockIdentityFromFile(selectedFile.value, password.value);
      onUnlocked(identity);
    } catch (err: any) {
      error.value = "Failed to unlock key: " + String(err);
    } finally {
      isLoading.value = false;
    }
  };

  const handleCreate = async (e: Event) => {
    e.preventDefault();
    if (!password.value) return;

    isLoading.value = true;
    error.value = null;

    try {
      const identity = await api.createIdentity(password.value, alias.value || undefined);
      onUnlocked(identity);
    } catch (err: any) {
      error.value = "Failed to create identity: " + String(err);
    } finally {
      isLoading.value = false;
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <h2>{isCreateMode.value ? "Create New Vault" : "Unlock Identity"}</h2>

        {error.value && <div className={styles.error}>{error.value}</div>}

        {!isCreateMode.value ? (
          <form onSubmit={handleUnlock}>
            <div className={styles.field}>
              <label>Select Key File</label>
              <select
                value={selectedFile.value}
                onChange={(e) => selectedFile.value = (e.target as HTMLSelectElement).value}
              >
                {keyFiles.value.map((f) => (
                  <option key={f.path} value={f.path}>{f.filename}</option>
                ))}
              </select>
            </div>

            <div className={styles.field}>
              <label>Password</label>
              <input
                type="password"
                value={password.value}
                onInput={(e) => password.value = (e.target as HTMLInputElement).value}
              />
            </div>

            <button type="submit" disabled={isLoading.value} className={styles.primaryBtn}>
              {isLoading.value ? "Unlocking..." : "Unlock"}
            </button>
          </form>
        ) : (
          <form onSubmit={handleCreate}>
            <div className={styles.field}>
              <label>Key Alias (Optional)</label>
              <input
                type="text"
                placeholder="my_device"
                value={alias.value}
                onInput={(e) => alias.value = (e.target as HTMLInputElement).value}
              />
            </div>

            <div className={styles.field}>
              <label>Set Master Password</label>
              <input
                type="password"
                value={password.value}
                onInput={(e) => password.value = (e.target as HTMLInputElement).value}
              />
            </div>

            <button type="submit" disabled={isLoading.value} className={styles.primaryBtn}>
              {isLoading.value ? "Generating..." : "Generate Keypair"}
            </button>
          </form>
        )}

        <button
          onClick={() => {
            isCreateMode.value = !isCreateMode.value;
            error.value = null;
          }}
          className={styles.switchBtn}
        >
          {isCreateMode.value ? "Already have a key? Unlock" : "Create new identity key"}
        </button>
      </div>
    </div>
  );
}

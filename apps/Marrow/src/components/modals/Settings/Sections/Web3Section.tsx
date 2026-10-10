import { useSignal } from "@preact/signals";

import styles from "@/styles/Settings/SettingsModal.module.css";

type EthereumProvider = {
  request(args: {
    method: string;
    params?: unknown[];
  }): Promise<unknown>;
  on?: (event: string, listener: (...args: unknown[]) => void) => void;
  removeListener?: (
    event: string,
    listener: (...args: unknown[]) => void,
  ) => void;
};

declare global {
  interface Window {
    ethereum?: EthereumProvider;
  }
}

const CHAINS: Record<string, string> = {
  "0x1": "Ethereum Mainnet",
  "0xaa36a7": "Sepolia Testnet",
  "0x89": "Polygon",
  "0x2105": "Base",
  "0xa4b1": "Arbitrum One",
};

function getErrorMessage(error: unknown): string {
  if (error && typeof error === "object" && "code" in error) {
    const code = (error as { code: number }).code;

    if (code === 4001) return "Request rejected by user.";
  }

  return error instanceof Error ? error.message : "Wallet request failed.";
}

export function Web3Section() {
  const address = useSignal("");
  const chainId = useSignal("");
  const signature = useSignal("");
  const status = useSignal("Not connected");
  const busy = useSignal(false);

  const provider = () => window.ethereum;

  async function connectWallet() {
    const wallet = provider();

    if (!wallet) {
      status.value =
        "No injected wallet found. Open Marrow in a compatible browser or configure WalletConnect.";
      return;
    }

    busy.value = true;
    signature.value = "";

    try {
      const accounts = (await wallet.request({
        method: "eth_requestAccounts",
      })) as string[];

      if (!accounts?.length) {
        throw new Error("The wallet returned no accounts.");
      }

      const network = (await wallet.request({
        method: "eth_chainId",
      })) as string;

      address.value = accounts[0];
      chainId.value = network;
      status.value = "Wallet connected";
    } catch (error) {
      status.value = getErrorMessage(error);
    } finally {
      busy.value = false;
    }
  }

  async function verifyOwnership() {
    const wallet = provider();

    if (!wallet || !address.value) {
      status.value = "Connect a wallet first.";
      return;
    }

    busy.value = true;
    signature.value = "";

    try {
      const nonce = crypto.randomUUID();
      const message = [
        "Marrow Wallet Verification",
        "",
        `Address: ${address.value}`,
        `Nonce: ${nonce}`,
        `Issued At: ${new Date().toISOString()}`,
        "",
        "Sign to prove control of this address. This does not authorize a transaction.",
      ].join("\n");

      const result = await wallet.request({
        method: "personal_sign",
        params: [message, address.value],
      });

      if (typeof result !== "string") {
        throw new Error("Wallet returned an invalid signature.");
      }

      signature.value = result;
      status.value =
        "Signature received. Backend verification is still required.";
    } catch (error) {
      status.value = getErrorMessage(error);
    } finally {
      busy.value = false;
    }
  }

  function disconnectWallet() {
    address.value = "";
    chainId.value = "";
    signature.value = "";
    status.value = "Disconnected from Marrow";
  }

  return (
    <section className={styles.web3Section}>
      <p className={styles.web3Description}>
        Connect an Ethereum-compatible wallet to associate a public address
        with your Marrow profile. Your wallet's private key stays with the
        wallet provider.
      </p>

      <div className={styles.web3Card}>
        <div className={styles.web3Heading}>
          <strong>Wallet connection</strong>
          <span>{address.value ? "Connected" : "Disconnected"}</span>
        </div>

        <p className={styles.web3Status} role="status">
          {status.value}
        </p>

        {address.value && (
          <>
            <label className={styles.web3Label}>Wallet address</label>
            <code className={styles.web3Address}>
              {address.value}
            </code>

            <label className={styles.web3Label}>Network</label>
            <p>
              {CHAINS[chainId.value] ?? `Chain ID: ${chainId.value}`}
            </p>
          </>
        )}

        <div className={styles.web3Actions}>
          {!address.value ? (
            <button
              type="button"
              disabled={busy.value}
              onClick={connectWallet}
            >
              {busy.value ? "Connecting…" : "Connect wallet"}
            </button>
          ) : (
            <>
              <button
                type="button"
                disabled={busy.value}
                onClick={verifyOwnership}
              >
                {busy.value ? "Waiting…" : "Verify ownership"}
              </button>

              <button type="button" onClick={disconnectWallet}>
                Disconnect
              </button>
            </>
          )}
        </div>

        {signature.value && (
          <details>
            <summary>View signature</summary>
            <code className={styles.web3Address}>
              {signature.value}
            </code>
          </details>
        )}
      </div>

      <p className={styles.web3Warning}>
        Never enter a seed phrase or private key into Marrow. Verify
        signatures on the backend before treating a wallet as authenticated.
      </p>
    </section>
  );
}
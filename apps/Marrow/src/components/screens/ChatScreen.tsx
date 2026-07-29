import { useSignal } from "@preact/signals";
import { useEffect } from "preact/hooks";
import { api, Session, DecryptedMessageDto } from "../../api/tauri";
import styles from "../../styles/ChatScreen.module.css";

interface Props {
  session: Session;
  onBack: () => void;
}

export function ChatScreen({ session, onBack }: Props) {
  const messages = useSignal<DecryptedMessageDto[]>([]);
  const payloadHex = useSignal("");

  const loadMessages = async () => {
    try {
      const msgs = await api.getSessionMessages(session.id);
      messages.value = msgs;
    } catch (err) {
      console.error("Failed to load messages:", err);
    }
  };

  useEffect(() => {
    loadMessages();
  }, [session.id]);

  const handleSimulateIncoming = async (e: Event) => {
    e.preventDefault();
    if (!payloadHex.value) return;

    try {
      const rawFrame = Array.from(new TextEncoder().encode(payloadHex.value));
      await api.processIncomingFrame(
        rawFrame,
        session.id,
        session.peer_pubkey_hex,
        Date.now()
      );
      payloadHex.value = "";
      loadMessages();
    } catch (err) {
      alert("Failed to process frame: " + err);
    }
  };

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <button onClick={onBack} className={styles.backBtn}>← Back</button>
        <div>
          <div className={styles.title}>Session: {session.id.slice(0, 8)}...</div>
          <div className={styles.subtitle}>Peer: {session.peer_pubkey_hex}</div>
        </div>
      </header>

      <div className={styles.messageList}>
        {messages.value.map((m, idx) => {
          const bubbleClass = m.direction === "Inbound" ? styles.inbound : styles.outbound;
          return (
            <div key={idx} className={`${styles.messageBubble} ${bubbleClass}`}>
              <div className={styles.msgPayload}>{m.payload_hex}</div>
              <div className={styles.msgMeta}>
                #{m.sequence_number} | {new Date(m.timestamp * 1000).toLocaleTimeString()}
              </div>
            </div>
          );
        })}
      </div>

      <form onSubmit={handleSimulateIncoming} className={styles.inputArea}>
        <input
          placeholder="Hex Payload or Text..."
          value={payloadHex.value}
          onInput={(e) => (payloadHex.value = (e.target as HTMLInputElement).value)}
          className={styles.input}
        />
        <button type="submit" className={styles.sendBtn}>Inbound Frame</button>
      </form>
    </div>
  );
}
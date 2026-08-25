import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface PublicIdentityDto {
  pubkey_hex: string;
}

export interface KeyFileInfoDto {
  filename: string;
  path: string;
}

export interface Contact {
  pubkey_hex: string;
  alias: string;
  added_at: number;
}

export interface Session {
  id: string;
  peer_pubkey_hex: string;
  created_at: number;
  last_activity: number;
}

export interface DecryptedMessageDto {
  session_id: string;
  sender_pubkey_hex: string;
  payload_hex: string;
  timestamp: number;
  direction: "Inbound" | "Outbound";
  sequence_number: number;
}

export interface NetworkEventPayload {
  peer_id: string;
  data_hex?: string;
}

export const api = {
  initStorage: () => invoke<void>("init_storage"),
  listIdentityFiles: () => invoke<KeyFileInfoDto[]>("list_identity_files"),
  createIdentity: (password: string, alias?: string) =>
    invoke<PublicIdentityDto>("create_identity", { password, alias }),
  unlockIdentityFromFile: (filePath: string, password: string) =>
    invoke<PublicIdentityDto>("unlock_identity_from_file", {
      filePath,
      password,
    }),
  importIdentityFile: (sourcePath: string) =>
    invoke<KeyFileInfoDto>("import_identity_file", { sourcePath }),
  getCurrentIdentity: () =>
    invoke<PublicIdentityDto | null>("get_current_identity"),
  logoutIdentity: () => invoke<void>("logout_identity"),
  saveContact: (pubkey_hex: string, alias: string) =>
    invoke<Contact>("save_contact", { pubkeyHex: pubkey_hex, alias }),
  listContacts: () => invoke<Contact[]>("list_contacts"),
  deleteContact: (pubkey_hex: string) =>
    invoke<boolean>("delete_contact", { pubkeyHex: pubkey_hex }),
  createSession: (peer_pubkey_hex: string) =>
    invoke<Session>("create_session", { peerPubkeyHex: peer_pubkey_hex }),
  listSessions: () => invoke<Session[]>("list_sessions"),
  deleteSession: (session_id: string) =>
    invoke<boolean>("delete_session", { sessionId: session_id }),
  sendMessage: (sessionId: string, peerPubkeyHex: string, text: string) =>
    invoke<DecryptedMessageDto>("send_chat_message", {
      sessionId,
      peerPubkeyHex,
      text,
    }),
  getSessionMessages: (session_id: string) =>
    invoke<DecryptedMessageDto[]>("get_session_messages", {
      sessionId: session_id,
    }),
  onFrameReceived: (cb: (payload: NetworkEventPayload) => void): Promise<UnlistenFn> =>
    listen<NetworkEventPayload>("network://frame_received", (e) => cb(e.payload)),
};

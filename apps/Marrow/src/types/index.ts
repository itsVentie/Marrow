export type Screen = 'auth' | 'dashboard' | 'chat' | 'settings';

export interface UserProfile {
  alias: string;
  pubkey: string;
}

export interface Contact {
  pubkey: string;
  alias: string;
  status: 'online' | 'offline';
  lastSeen?: string;
  unreadCount: number;
}

export interface Message {
  id: string;
  senderPubkey: string;
  recipientPubkey: string;
  content: string;
  timestamp: number;
  isOutgoing: boolean;
  status: 'pending' | 'sent' | 'delivered' | 'failed';
}

export type NetworkStatus = 'connected' | 'connecting' | 'disconnected';
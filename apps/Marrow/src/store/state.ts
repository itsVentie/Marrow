import { signal, computed } from '@preact/signals';
import type { Screen, UserProfile, Contact, Message, NetworkStatus } from '../types';

export const currentScreen = signal<Screen>('auth');
export const networkStatus = signal<NetworkStatus>('disconnected');

export const currentUser = signal<UserProfile | null>(null);

export const activePeerPubkey = signal<string | null>(null);
export const contacts = signal<Contact[]>([]);
export const messages = signal<Record<string, Message[]>>({});

export const activeChatMessages = computed(() => {
  const peer = activePeerPubkey.value;
  if (!peer) return [];
  return messages.value[peer] || [];
});

export const activeContact = computed(() => {
  return contacts.value.find((c) => c.pubkey === activePeerPubkey.value) || null;
});
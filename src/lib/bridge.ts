import { Channel, invoke, isTauri } from '@tauri-apps/api/core';

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}
export interface Session {
  id: string;
  title: string;
  updatedAt: number;
}
export interface Preferences {
  baseUrl: string;
  model: string;
}
export interface Bootstrap {
  sessions: Session[];
  preferences: Preferences;
  connected: boolean;
  plugins: { id: string; kind: string; apiVersion: number }[];
}
export interface DeltaEvent {
  runId: string;
  delta: string;
}
export const desktop = isTauri();

// Browser mode previews the UI only. It never stores secrets or sends model requests.
const previewSessions: Session[] = [];
const previewMessages = new Map<string, Message[]>();
let previewPreferences: Preferences = {
  baseUrl: 'http://localhost:11434/v1',
  model: '',
};

export const bridge = {
  async bootstrap(): Promise<Bootstrap> {
    if (desktop) return invoke('bootstrap');
    return {
      sessions: [...previewSessions],
      preferences: previewPreferences,
      connected: false,
      plugins: [],
    };
  },
  async createSession(): Promise<Session> {
    if (desktop) return invoke('create_session');
    const session = {
      id: crypto.randomUUID(),
      title: 'New conversation',
      updatedAt: Date.now(),
    };
    previewSessions.unshift(session);
    previewMessages.set(session.id, []);
    return session;
  },
  async messages(sessionId: string): Promise<Message[]> {
    if (desktop) return invoke('get_messages', { sessionId });
    return [...(previewMessages.get(sessionId) ?? [])];
  },
  async deleteSession(sessionId: string): Promise<void> {
    if (desktop) return invoke('delete_session', { sessionId });
    const index = previewSessions.findIndex((s) => s.id === sessionId);
    if (index >= 0) previewSessions.splice(index, 1);
    previewMessages.delete(sessionId);
  },
  async configure(preferences: Preferences, apiKey: string): Promise<void> {
    if (desktop) return invoke('configure_connection', { preferences, apiKey });
    previewPreferences = { ...preferences };
  },
  async generate(
    sessionId: string,
    runId: string,
    input: string,
    onDelta: (event: DeltaEvent) => void,
  ): Promise<void> {
    if (!desktop)
      throw new Error(
        'Open the desktop app to connect a model. This browser view is a UI preview.',
      );
    const onEvent = new Channel<DeltaEvent>();
    onEvent.onmessage = onDelta;
    return invoke('generate', { sessionId, runId, input, onEvent });
  },
  async cancel(runId: string): Promise<void> {
    if (desktop) return invoke('cancel_run', { runId });
  },
};

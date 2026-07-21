import type {
  AddCardioEntryPayload,
  AddStrengthSetPayload,
  CreateWorkoutSessionPayload,
  WorkoutSession
} from './apiClient';

export type SyncStatus = 'pending' | 'synced' | 'failed';

export type LocalWorkoutSession = {
  clientId: string;
  serverId: number | null;
  started_at: string;
  completed_at: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  syncStatus: SyncStatus;
};

export type LocalStrengthEntry = {
  kind: 'strength';
  clientId: string;
  clientSessionId: string;
  serverId: number | null;
  exerciseName: string;
  detail: string;
  payload: AddStrengthSetPayload;
  created_at: string;
  syncStatus: SyncStatus;
};

export type LocalCardioEntry = {
  kind: 'cardio';
  clientId: string;
  clientSessionId: string;
  serverId: number | null;
  exerciseName: string;
  detail: string;
  payload: AddCardioEntryPayload;
  created_at: string;
  syncStatus: SyncStatus;
};

export type LocalWorkoutEntry = LocalStrengthEntry | LocalCardioEntry;

export type LocalMutation =
  | {
      id: string;
      type: 'create_session';
      clientSessionId: string;
      payload: CreateWorkoutSessionPayload;
      created_at: string;
    }
  | {
      id: string;
      type: 'add_strength_set';
      clientSessionId: string;
      clientEntryId: string;
      payload: AddStrengthSetPayload;
      created_at: string;
    }
  | {
      id: string;
      type: 'add_cardio_entry';
      clientSessionId: string;
      clientEntryId: string;
      payload: AddCardioEntryPayload;
      created_at: string;
    };

export type LocalWorkoutState = {
  version: 1;
  sessions: LocalWorkoutSession[];
  entries: LocalWorkoutEntry[];
  queue: LocalMutation[];
};

export type TodayLocalSummary = {
  session: LocalWorkoutSession | null;
  strengthSetCount: number;
  cardioEntryCount: number;
  pendingCount: number;
};

export type LocalSyncSnapshot = {
  failedCount: number;
  pendingCount: number;
  unsyncedCount: number;
};

const storageVersion = 1;
const keyPrefix = 'splitstreak.local-workouts.v1';

export function loadLocalWorkoutState(userSub: string): LocalWorkoutState {
  if (!canUseLocalStorage()) {
    return emptyState();
  }

  const raw = window.localStorage.getItem(storageKey(userSub));
  if (!raw) {
    return emptyState();
  }

  try {
    const parsed = JSON.parse(raw) as Partial<LocalWorkoutState>;
    if (parsed.version !== storageVersion) {
      return emptyState();
    }

    return reconcileLocalWorkoutState({
      version: storageVersion,
      sessions: Array.isArray(parsed.sessions) ? parsed.sessions : [],
      entries: Array.isArray(parsed.entries) ? parsed.entries : [],
      queue: Array.isArray(parsed.queue) ? parsed.queue : []
    });
  } catch {
    return emptyState();
  }
}

export function saveLocalWorkoutState(userSub: string, state: LocalWorkoutState) {
  if (!canUseLocalStorage()) {
    return;
  }

  window.localStorage.setItem(storageKey(userSub), JSON.stringify(state));
  window.dispatchEvent(new CustomEvent('splitstreak-local-workouts-updated'));
}

export function updateLocalWorkoutState(
  userSub: string,
  updater: (state: LocalWorkoutState) => LocalWorkoutState
) {
  const nextState = reconcileLocalWorkoutState(updater(loadLocalWorkoutState(userSub)));
  saveLocalWorkoutState(userSub, nextState);
  return nextState;
}

export function reconcileStoredWorkoutState(userSub: string) {
  return updateLocalWorkoutState(userSub, (current) => current);
}

export function ensureTodayLocalSession(userSub: string): {
  session: LocalWorkoutSession;
  state: LocalWorkoutState;
} {
  const current = loadLocalWorkoutState(userSub);
  const existing = findTodaySession(current);
  if (existing) {
    return { session: existing, state: current };
  }

  const now = new Date().toISOString();
  const session: LocalWorkoutSession = {
    clientId: createClientId('session'),
    serverId: null,
    started_at: now,
    completed_at: null,
    notes: null,
    created_at: now,
    updated_at: now,
    syncStatus: 'pending'
  };
  const state = {
    ...current,
    sessions: [session, ...current.sessions],
    queue: [
      ...current.queue,
      {
        id: createClientId('mutation'),
        type: 'create_session' as const,
        clientSessionId: session.clientId,
        payload: { started_at: session.started_at },
        created_at: now
      }
    ]
  };

  saveLocalWorkoutState(userSub, state);
  return { session, state };
}

export function addLocalStrengthEntry(
  userSub: string,
  clientSessionId: string,
  exerciseName: string,
  detail: string,
  payload: AddStrengthSetPayload
) {
  const now = new Date().toISOString();
  const entry: LocalStrengthEntry = {
    kind: 'strength',
    clientId: createClientId('strength'),
    clientSessionId,
    serverId: null,
    exerciseName,
    detail,
    payload,
    created_at: now,
    syncStatus: 'pending'
  };

  updateLocalWorkoutState(userSub, (current) => ({
    ...current,
    entries: [entry, ...current.entries],
    queue: [
      ...current.queue,
      {
        id: createClientId('mutation'),
        type: 'add_strength_set',
        clientSessionId,
        clientEntryId: entry.clientId,
        payload,
        created_at: now
      }
    ]
  }));

  return entry;
}

export function addLocalCardioEntry(
  userSub: string,
  clientSessionId: string,
  exerciseName: string,
  detail: string,
  payload: AddCardioEntryPayload
) {
  const now = new Date().toISOString();
  const entry: LocalCardioEntry = {
    kind: 'cardio',
    clientId: createClientId('cardio'),
    clientSessionId,
    serverId: null,
    exerciseName,
    detail,
    payload,
    created_at: now,
    syncStatus: 'pending'
  };

  updateLocalWorkoutState(userSub, (current) => ({
    ...current,
    entries: [entry, ...current.entries],
    queue: [
      ...current.queue,
      {
        id: createClientId('mutation'),
        type: 'add_cardio_entry',
        clientSessionId,
        clientEntryId: entry.clientId,
        payload,
        created_at: now
      }
    ]
  }));

  return entry;
}

export function applySyncedSession(
  userSub: string,
  clientSessionId: string,
  serverSession: WorkoutSession,
  mutationId: string
) {
  updateLocalWorkoutState(userSub, (current) => ({
    ...current,
    sessions: current.sessions.map((session) =>
      session.clientId === clientSessionId
        ? {
            ...session,
            serverId: serverSession.id,
            started_at: serverSession.started_at,
            completed_at: serverSession.completed_at,
            notes: serverSession.notes,
            updated_at: serverSession.updated_at,
            syncStatus: 'synced'
          }
        : session
    ),
    queue: current.queue.filter((item) => {
      if (item.id === mutationId) {
        return false;
      }
      if (item.type === 'create_session') {
        return item.clientSessionId !== clientSessionId;
      }

      return true;
    })
  }));
}

export function applySyncedEntry(
  userSub: string,
  clientEntryId: string,
  serverId: number,
  mutationId: string
) {
  updateLocalWorkoutState(userSub, (current) => ({
    ...current,
    entries: current.entries.map((entry) =>
      entry.clientId === clientEntryId
        ? { ...entry, serverId, syncStatus: 'synced' }
        : entry
    ),
    queue: current.queue.filter((item) => {
      if (item.id === mutationId) {
        return false;
      }
      if (item.type === 'create_session') {
        return true;
      }

      return item.clientEntryId !== clientEntryId;
    })
  }));
}

export function markMutationFailed(userSub: string, mutationId: string) {
  updateLocalWorkoutState(userSub, (current) => {
    const mutation = current.queue.find((item) => item.id === mutationId);
    if (!mutation || mutation.type === 'create_session') {
      return current;
    }

    return {
      ...current,
      entries: current.entries.map((entry) =>
        entry.clientId === mutation.clientEntryId
          ? { ...entry, syncStatus: 'failed' }
          : entry
      )
    };
  });
}

export function getTodayLocalSummary(userSub: string): TodayLocalSummary {
  const state = loadLocalWorkoutState(userSub);
  const session = findTodaySession(state);
  if (!session) {
    return {
      session: null,
      strengthSetCount: 0,
      cardioEntryCount: 0,
      pendingCount: state.queue.length
    };
  }

  const entries = state.entries.filter(
    (entry) => entry.clientSessionId === session.clientId
  );

  return {
    session,
    strengthSetCount: entries.filter((entry) => entry.kind === 'strength').length,
    cardioEntryCount: entries.filter((entry) => entry.kind === 'cardio').length,
    pendingCount: state.queue.length
  };
}

export function getTodayLocalEntries(userSub: string) {
  const state = loadLocalWorkoutState(userSub);
  const session = findTodaySession(state);
  if (!session) {
    return [];
  }

  return state.entries.filter((entry) => entry.clientSessionId === session.clientId);
}

export function getPendingMutationCount(userSub: string) {
  return loadLocalWorkoutState(userSub).queue.length;
}

export function getLocalSyncSnapshot(userSub: string): LocalSyncSnapshot {
  const state = loadLocalWorkoutState(userSub);
  const failedCount = state.entries.filter(
    (entry) => entry.syncStatus === 'failed'
  ).length;
  const pendingCount = state.queue.length;
  const unsyncedSessionCount = state.sessions.filter(
    (session) => session.syncStatus !== 'synced'
  ).length;
  const unsyncedEntryCount = state.entries.filter(
    (entry) => entry.syncStatus !== 'synced'
  ).length;

  return {
    failedCount,
    pendingCount,
    unsyncedCount: unsyncedSessionCount + unsyncedEntryCount
  };
}

function findTodaySession(state: LocalWorkoutState) {
  return state.sessions.find((session) => isToday(session.started_at)) ?? null;
}

function isToday(value: string) {
  return new Date(value).toDateString() === new Date().toDateString();
}

function emptyState(): LocalWorkoutState {
  return {
    version: storageVersion,
    sessions: [],
    entries: [],
    queue: []
  };
}

function reconcileLocalWorkoutState(state: LocalWorkoutState): LocalWorkoutState {
  const sessions = dedupeSessions(state.sessions);
  const entries = dedupeEntries(state.entries);
  const sessionByClientId = new Map(
    sessions.map((session) => [session.clientId, session])
  );
  const entryByClientId = new Map(entries.map((entry) => [entry.clientId, entry]));
  const seenMutations = new Set<string>();
  const queue = state.queue.filter((mutation) => {
    const key = mutationKey(mutation);
    if (seenMutations.has(key)) {
      return false;
    }
    seenMutations.add(key);

    if (mutation.type === 'create_session') {
      const session = sessionByClientId.get(mutation.clientSessionId);
      return Boolean(session && session.syncStatus !== 'synced');
    }

    const entry = entryByClientId.get(mutation.clientEntryId);
    return Boolean(entry && entry.syncStatus !== 'synced');
  });

  return {
    version: storageVersion,
    sessions,
    entries,
    queue
  };
}

function dedupeSessions(sessions: LocalWorkoutSession[]) {
  const deduped: LocalWorkoutSession[] = [];
  const indexByClientId = new Map<string, number>();
  for (const session of sessions) {
    const existingIndex = indexByClientId.get(session.clientId);
    if (existingIndex === undefined) {
      indexByClientId.set(session.clientId, deduped.length);
      deduped.push(session);
    } else {
      deduped[existingIndex] = mergeSession(deduped[existingIndex], session);
    }
  }

  return deduped;
}

function dedupeEntries(entries: LocalWorkoutEntry[]) {
  const deduped: LocalWorkoutEntry[] = [];
  const indexByClientId = new Map<string, number>();
  for (const entry of entries) {
    const existingIndex = indexByClientId.get(entry.clientId);
    if (existingIndex === undefined) {
      indexByClientId.set(entry.clientId, deduped.length);
      deduped.push(entry);
    } else {
      deduped[existingIndex] = mergeEntry(deduped[existingIndex], entry);
    }
  }

  return deduped;
}

function mergeSession(
  existing: LocalWorkoutSession,
  duplicate: LocalWorkoutSession
): LocalWorkoutSession {
  const serverId = existing.serverId ?? duplicate.serverId;
  return {
    ...existing,
    serverId,
    completed_at: existing.completed_at ?? duplicate.completed_at,
    notes: existing.notes ?? duplicate.notes,
    syncStatus: serverId ? 'synced' : mergeSyncStatus(existing, duplicate)
  };
}

function mergeEntry(
  existing: LocalWorkoutEntry,
  duplicate: LocalWorkoutEntry
): LocalWorkoutEntry {
  const serverId = existing.serverId ?? duplicate.serverId;
  return {
    ...existing,
    serverId,
    syncStatus: serverId ? 'synced' : mergeSyncStatus(existing, duplicate)
  };
}

function mergeSyncStatus(
  existing: { syncStatus: SyncStatus },
  duplicate: { syncStatus: SyncStatus }
): SyncStatus {
  if (existing.syncStatus === 'synced' || duplicate.syncStatus === 'synced') {
    return 'synced';
  }
  if (existing.syncStatus === 'pending' || duplicate.syncStatus === 'pending') {
    return 'pending';
  }

  return 'failed';
}

function mutationKey(mutation: LocalMutation) {
  if (mutation.type === 'create_session') {
    return `${mutation.type}:${mutation.clientSessionId}`;
  }

  return `${mutation.type}:${mutation.clientEntryId}`;
}

function storageKey(userSub: string) {
  return `${keyPrefix}.${encodeURIComponent(userSub)}`;
}

function canUseLocalStorage() {
  return typeof window !== 'undefined' && Boolean(window.localStorage);
}

function createClientId(prefix: string) {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return `${prefix}-${crypto.randomUUID()}`;
  }

  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

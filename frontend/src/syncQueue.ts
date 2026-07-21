import { ApiError, syncOfflineBatch, type SyncBatchPayload } from './apiClient';
import {
  applySyncedEntry,
  applySyncedSession,
  loadLocalWorkoutState,
  markMutationFailed,
  reconcileStoredWorkoutState
} from './localStore';

export type SyncQueueResult = {
  attempted: number;
  synced: number;
  pending: number;
};

export const syncQueueStatusEvent = 'splitstreak-sync-status-changed';

type SyncQueueStatus = 'idle' | 'syncing';

const inFlightByUser = new Map<string, Promise<SyncQueueResult>>();

export async function syncQueuedMutations(userSub: string): Promise<SyncQueueResult> {
  const existing = inFlightByUser.get(userSub);
  if (existing) {
    return existing;
  }

  const inFlight = syncQueuedMutationsInternal(userSub).finally(() => {
    inFlightByUser.delete(userSub);
  });
  inFlightByUser.set(userSub, inFlight);

  return inFlight;
}

async function syncQueuedMutationsInternal(userSub: string): Promise<SyncQueueResult> {
  let attempted = 0;
  let synced = 0;
  dispatchSyncQueueStatus(userSub, 'syncing');

  try {
    reconcileStoredWorkoutState(userSub);

    while (true) {
      const state = loadLocalWorkoutState(userSub);
      const [mutation] = state.queue;
      if (!mutation) {
        return { attempted, synced, pending: 0 };
      }
      const batch = buildSyncBatch(state);
      const batchSize =
        batch.payload.sessions.length +
        batch.payload.strength_sets.length +
        batch.payload.cardio_entries.length;

      if (batchSize === 0) {
        reconcileStoredWorkoutState(userSub);
        const pending = loadLocalWorkoutState(userSub).queue.length;
        if (pending === 0) {
          return { attempted, synced, pending };
        }

        markMutationFailed(userSub, mutation.id);
        return { attempted, synced, pending };
      }

      attempted += batchSize;

      try {
        let syncedThisBatch = 0;
        const response = await syncOfflineBatch(batch.payload);
        for (const session of response.sessions) {
          const queuedSession = batch.sessionMutations.get(session.client_id);
          if (queuedSession) {
            applySyncedSession(
              userSub,
              session.client_id,
              session.session,
              queuedSession.id
            );
            synced += 1;
            syncedThisBatch += 1;
          }
        }
        for (const strengthSet of response.strength_sets) {
          const queuedEntry = batch.entryMutations.get(strengthSet.client_id);
          if (queuedEntry) {
            applySyncedEntry(
              userSub,
              strengthSet.client_id,
              strengthSet.server_id,
              queuedEntry.id
            );
            synced += 1;
            syncedThisBatch += 1;
          }
        }
        for (const cardioEntry of response.cardio_entries) {
          const queuedEntry = batch.entryMutations.get(cardioEntry.client_id);
          if (queuedEntry) {
            applySyncedEntry(
              userSub,
              cardioEntry.client_id,
              cardioEntry.server_id,
              queuedEntry.id
            );
            synced += 1;
            syncedThisBatch += 1;
          }
        }

        if (syncedThisBatch === 0 && loadLocalWorkoutState(userSub).queue.length > 0) {
          markMutationFailed(userSub, mutation.id);
          return {
            attempted,
            synced,
            pending: loadLocalWorkoutState(userSub).queue.length
          };
        }
      } catch (caught) {
        if (caught instanceof ApiError && caught.status === 401) {
          throw caught;
        }

        markMutationFailed(userSub, mutation.id);
        return {
          attempted,
          synced,
          pending: loadLocalWorkoutState(userSub).queue.length
        };
      }
    }
  } finally {
    dispatchSyncQueueStatus(userSub, 'idle');
  }
}

function buildSyncBatch(state: ReturnType<typeof loadLocalWorkoutState>) {
  const payload: SyncBatchPayload = {
    sessions: [],
    strength_sets: [],
    cardio_entries: []
  };
  const sessionMutations = new Map<string, { id: string }>();
  const entryMutations = new Map<string, { id: string }>();

  for (const mutation of state.queue) {
    if (mutation.type === 'create_session') {
      const session = state.sessions.find(
        (item) => item.clientId === mutation.clientSessionId
      );
      if (session) {
        payload.sessions.push({
          client_id: session.clientId,
          started_at: session.started_at,
          completed_at: session.completed_at,
          notes: session.notes
        });
        sessionMutations.set(session.clientId, { id: mutation.id });
      }
    } else if (mutation.type === 'add_strength_set') {
      payload.strength_sets.push({
        ...mutation.payload,
        client_id: mutation.clientEntryId,
        client_session_id: mutation.clientSessionId
      });
      entryMutations.set(mutation.clientEntryId, { id: mutation.id });
    } else {
      payload.cardio_entries.push({
        ...mutation.payload,
        client_id: mutation.clientEntryId,
        client_session_id: mutation.clientSessionId
      });
      entryMutations.set(mutation.clientEntryId, { id: mutation.id });
    }
  }

  return {
    entryMutations,
    payload,
    sessionMutations
  };
}

function dispatchSyncQueueStatus(userSub: string, status: SyncQueueStatus) {
  if (typeof window === 'undefined') {
    return;
  }

  window.dispatchEvent(
    new CustomEvent(syncQueueStatusEvent, {
      detail: {
        status,
        userSub
      }
    })
  );
}

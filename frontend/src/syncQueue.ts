import {
  ApiError,
  addCardioEntry,
  addStrengthSet,
  createWorkoutSession
} from './apiClient';
import {
  applySyncedEntry,
  applySyncedSession,
  loadLocalWorkoutState,
  markMutationFailed
} from './localStore';

export type SyncQueueResult = {
  attempted: number;
  synced: number;
  pending: number;
};

export async function syncQueuedMutations(userSub: string): Promise<SyncQueueResult> {
  let attempted = 0;
  let synced = 0;

  while (true) {
    const state = loadLocalWorkoutState(userSub);
    const [mutation] = state.queue;
    if (!mutation) {
      return { attempted, synced, pending: 0 };
    }

    attempted += 1;

    try {
      if (mutation.type === 'create_session') {
        const session = state.sessions.find(
          (item) => item.clientId === mutation.clientSessionId
        );
        if (!session) {
          markMutationFailed(userSub, mutation.id);
          return {
            attempted,
            synced,
            pending: loadLocalWorkoutState(userSub).queue.length
          };
        }

        if (session.serverId !== null) {
          applySyncedSession(
            userSub,
            session.clientId,
            {
              id: session.serverId,
              user_sub: userSub,
              started_at: session.started_at,
              completed_at: session.completed_at,
              notes: session.notes,
              created_at: session.created_at,
              updated_at: session.updated_at
            },
            mutation.id
          );
        } else {
          const response = await createWorkoutSession(mutation.payload);
          applySyncedSession(
            userSub,
            mutation.clientSessionId,
            response.session,
            mutation.id
          );
        }
      } else {
        const session = state.sessions.find(
          (item) => item.clientId === mutation.clientSessionId
        );
        if (!session?.serverId) {
          return {
            attempted,
            synced,
            pending: loadLocalWorkoutState(userSub).queue.length
          };
        }

        if (mutation.type === 'add_strength_set') {
          const response = await addStrengthSet(session.serverId, mutation.payload);
          applySyncedEntry(
            userSub,
            mutation.clientEntryId,
            response.strength_set.id,
            mutation.id
          );
        } else {
          const response = await addCardioEntry(session.serverId, mutation.payload);
          applySyncedEntry(
            userSub,
            mutation.clientEntryId,
            response.cardio_entry.id,
            mutation.id
          );
        }
      }

      synced += 1;
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
}

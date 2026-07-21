import React from 'react';

import { ApiError, type ExerciseSearchItem, redirectToLogin } from '../apiClient';
import {
  addLocalCardioEntry,
  addLocalStrengthEntry,
  ensureTodayLocalSession,
  getPendingMutationCount,
  getTodayLocalEntries,
  type LocalWorkoutEntry
} from '../localStore';
import { syncQueuedMutations } from '../syncQueue';
import { CardioEntry, type CardioEntrySubmission } from './CardioEntry';
import { ExerciseSearch } from './ExerciseSearch';
import { SetEntry, type SetDraft } from './SetEntry';

type EntryMode = 'strength' | 'cardio';

type RecentEntry = {
  detail: string;
  id: string;
  label: string;
  syncStatus: string;
};

const initialDraft: SetDraft = {
  reps: 8,
  setNumber: 1,
  weightKg: 20
};

type LogScreenProps = {
  userSub: string;
};

export function LogScreen({ userSub }: LogScreenProps) {
  const [sessionClientId, setSessionClientId] = React.useState<string | null>(null);
  const [selectedExercise, setSelectedExercise] =
    React.useState<ExerciseSearchItem | null>(null);
  const [entryMode, setEntryMode] = React.useState<EntryMode>('strength');
  const [draft, setDraft] = React.useState<SetDraft>(initialDraft);
  const [recentEntries, setRecentEntries] = React.useState<RecentEntry[]>([]);
  const [message, setMessage] = React.useState<string | null>(null);
  const [isSaving, setIsSaving] = React.useState(false);

  const refreshLocalEntries = React.useCallback(() => {
    const localEntries = getTodayLocalEntries(userSub);
    setRecentEntries(localEntries.map(toRecentEntry));
    setSessionClientId(localEntries[0]?.clientSessionId ?? null);
  }, [userSub]);

  const attemptSync = React.useCallback(
    async (successMessage?: string) => {
      try {
        const result = await syncQueuedMutations(userSub);
        refreshLocalEntries();
        const pending = result.pending;
        if (successMessage) {
          setMessage(
            pending > 0 ? `${successMessage} ${pending} pending sync.` : successMessage
          );
        } else if (pending > 0) {
          setMessage(`${pending} pending sync.`);
        }
      } catch (caught) {
        if (caught instanceof ApiError && caught.status === 401) {
          redirectToLogin(caught.loginUrl);
          return;
        }

        const pending = getPendingMutationCount(userSub);
        setMessage(
          pending > 0
            ? `${pending} pending sync.`
            : caught instanceof Error
              ? caught.message
              : 'Sync failed.'
        );
      }
    },
    [refreshLocalEntries, userSub]
  );

  React.useEffect(() => {
    refreshLocalEntries();
    void attemptSync();

    function handleOnline() {
      void attemptSync();
    }

    window.addEventListener('online', handleOnline);
    window.addEventListener('splitstreak-local-workouts-updated', refreshLocalEntries);
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener(
        'splitstreak-local-workouts-updated',
        refreshLocalEntries
      );
    };
  }, [attemptSync, refreshLocalEntries]);

  function getActiveSession() {
    const { session } = ensureTodayLocalSession(userSub);
    if (sessionClientId !== session.clientId) {
      setSessionClientId(session.clientId);
    }

    return session;
  }

  async function handleAddSet() {
    if (!selectedExercise) {
      setMessage('Choose an exercise first.');
      return;
    }

    setIsSaving(true);
    setMessage(null);
    try {
      const activeSession = await getActiveSession();
      const payload = {
        exercise_id: selectedExercise.id,
        reps: draft.reps,
        set_number: draft.setNumber,
        weight_kg: draft.weightKg
      };
      addLocalStrengthEntry(
        userSub,
        activeSession.clientId,
        selectedExercise.name,
        formatStrengthPayload(payload),
        payload
      );

      refreshLocalEntries();
      setDraft((current) => ({
        ...current,
        setNumber: Math.min(200, current.setNumber + 1)
      }));
      setMessage('Set saved locally.');
      await attemptSync('Set logged.');
    } catch (caught) {
      if (caught instanceof ApiError && caught.status === 401) {
        redirectToLogin(caught.loginUrl);
        return;
      }

      setMessage(caught instanceof Error ? caught.message : 'Set could not be logged.');
    } finally {
      setIsSaving(false);
    }
  }

  async function handleAddCardio(submission: CardioEntrySubmission) {
    setIsSaving(true);
    setMessage(null);
    try {
      const activeSession = await getActiveSession();
      addLocalCardioEntry(
        userSub,
        activeSession.clientId,
        submission.exerciseName,
        formatCardioPayload(submission.payload),
        submission.payload
      );

      refreshLocalEntries();
      setMessage('Cardio saved locally.');
      await attemptSync('Cardio logged.');
    } catch (caught) {
      if (caught instanceof ApiError && caught.status === 401) {
        redirectToLogin(caught.loginUrl);
        return;
      }

      setMessage(
        caught instanceof Error ? caught.message : 'Cardio could not be logged.'
      );
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <section className="log-layout" aria-labelledby="log-heading">
      <div className="log-workspace">
        <div className="section-heading">
          <p className="eyebrow">Workout log</p>
          <h2 id="log-heading">Today</h2>
        </div>

        <div className="segmented-control log-mode-tabs" aria-label="Entry type">
          <button
            aria-pressed={entryMode === 'strength'}
            className={entryMode === 'strength' ? 'segment segment--active' : 'segment'}
            onClick={() => {
              setEntryMode('strength');
              setMessage(null);
            }}
            type="button"
          >
            Strength
          </button>
          <button
            aria-pressed={entryMode === 'cardio'}
            className={entryMode === 'cardio' ? 'segment segment--active' : 'segment'}
            onClick={() => {
              setEntryMode('cardio');
              setMessage(null);
            }}
            type="button"
          >
            Cardio
          </button>
        </div>

        {entryMode === 'strength' ? (
          <>
            <ExerciseSearch
              disabled={isSaving}
              onSelect={(exercise) => {
                setSelectedExercise(exercise);
                setDraft(initialDraft);
                setMessage(null);
              }}
              selectedExercise={selectedExercise}
            />

            <SetEntry
              disabled={isSaving || !selectedExercise}
              draft={draft}
              onChange={setDraft}
              onSubmit={handleAddSet}
            />
          </>
        ) : (
          <CardioEntry disabled={isSaving} onSubmit={handleAddCardio} />
        )}

        {message && <p className="form-message">{message}</p>}
      </div>

      <aside className="recent-panel" aria-labelledby="recent-heading">
        <div className="section-heading">
          <p className="eyebrow">Logged</p>
          <h2 id="recent-heading">Entries</h2>
        </div>
        {recentEntries.length === 0 ? (
          <p className="empty-state">No entries logged in this session.</p>
        ) : (
          <ol className="recent-sets">
            {recentEntries.map((item) => (
              <li key={item.id}>
                <span>
                  {item.label}
                  {item.syncStatus !== 'synced' && (
                    <small className="sync-status">Sync pending</small>
                  )}
                </span>
                <strong>{item.detail}</strong>
              </li>
            ))}
          </ol>
        )}
      </aside>
    </section>
  );
}

function toRecentEntry(entry: LocalWorkoutEntry): RecentEntry {
  return {
    detail: entry.detail,
    id: entry.clientId,
    label: entry.exerciseName,
    syncStatus: entry.syncStatus
  };
}

function formatStrengthPayload(payload: {
  set_number: number;
  reps: number;
  weight_kg: number;
}) {
  return `${payload.set_number} x ${payload.reps} @ ${payload.weight_kg} kg`;
}

function formatCardioPayload(payload: CardioEntrySubmission['payload']) {
  const parts = [`${Math.round(payload.duration_seconds / 60)} min`];
  if (payload.distance_meters !== undefined) {
    parts.push(`${payload.distance_meters / 1000} km`);
  }
  if (payload.intensity_level !== undefined) {
    parts.push(`RPE ${payload.intensity_level}`);
  }
  if (payload.speed_kph !== undefined) {
    parts.push(`${payload.speed_kph} kph`);
  }
  if (payload.incline_percent !== undefined) {
    parts.push(`${payload.incline_percent}% incline`);
  }

  return parts.join(', ');
}

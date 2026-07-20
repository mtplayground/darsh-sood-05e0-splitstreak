import React from 'react';

import {
  ApiError,
  type CardioEntry as CardioRecord,
  type ExerciseSearchItem,
  type StrengthSet,
  type WorkoutSession,
  addCardioEntry,
  addStrengthSet,
  createWorkoutSession,
  redirectToLogin
} from '../apiClient';
import { CardioEntry, type CardioEntrySubmission } from './CardioEntry';
import { ExerciseSearch } from './ExerciseSearch';
import { SetEntry, type SetDraft } from './SetEntry';

type EntryMode = 'strength' | 'cardio';

type RecentEntry = {
  detail: string;
  id: string;
  label: string;
};

const initialDraft: SetDraft = {
  reps: 8,
  setNumber: 1,
  weightKg: 20
};

export function LogScreen() {
  const [session, setSession] = React.useState<WorkoutSession | null>(null);
  const [selectedExercise, setSelectedExercise] =
    React.useState<ExerciseSearchItem | null>(null);
  const [entryMode, setEntryMode] = React.useState<EntryMode>('strength');
  const [draft, setDraft] = React.useState<SetDraft>(initialDraft);
  const [recentEntries, setRecentEntries] = React.useState<RecentEntry[]>([]);
  const [message, setMessage] = React.useState<string | null>(null);
  const [isSaving, setIsSaving] = React.useState(false);

  async function getActiveSession() {
    const activeSession = session ?? (await createWorkoutSession()).session;
    if (!session) {
      setSession(activeSession);
    }

    return activeSession;
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
      const response = await addStrengthSet(activeSession.id, {
        exercise_id: selectedExercise.id,
        reps: draft.reps,
        set_number: draft.setNumber,
        weight_kg: draft.weightKg
      });

      setRecentEntries((entries) => [
        {
          detail: formatStrengthSet(response.strength_set),
          id: `strength-${response.strength_set.id}`,
          label: selectedExercise.name
        },
        ...entries
      ]);
      setDraft((current) => ({
        ...current,
        setNumber: Math.min(200, current.setNumber + 1)
      }));
      setMessage('Set logged.');
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
      const response = await addCardioEntry(activeSession.id, submission.payload);

      setRecentEntries((entries) => [
        {
          detail: formatCardioEntry(response.cardio_entry),
          id: `cardio-${response.cardio_entry.id}`,
          label: submission.exerciseName
        },
        ...entries
      ]);
      setMessage('Cardio logged.');
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
                <span>{item.label}</span>
                <strong>{item.detail}</strong>
              </li>
            ))}
          </ol>
        )}
      </aside>
    </section>
  );
}

function formatStrengthSet(set: StrengthSet) {
  return `${set.set_number} x ${set.reps} @ ${set.weight_kg} kg`;
}

function formatCardioEntry(entry: CardioRecord) {
  const parts = [`${Math.round(entry.duration_seconds / 60)} min`];
  if (entry.distance_meters !== null) {
    parts.push(`${entry.distance_meters / 1000} km`);
  }
  if (entry.intensity_level !== null) {
    parts.push(`RPE ${entry.intensity_level}`);
  }
  if (entry.speed_kph !== null) {
    parts.push(`${entry.speed_kph} kph`);
  }
  if (entry.incline_percent !== null) {
    parts.push(`${entry.incline_percent}% incline`);
  }

  return parts.join(', ');
}

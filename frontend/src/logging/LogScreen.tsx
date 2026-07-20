import React from 'react';

import {
  ApiError,
  type ExerciseSearchItem,
  type StrengthSet,
  type WorkoutSession,
  addStrengthSet,
  createWorkoutSession,
  redirectToLogin
} from '../apiClient';
import { ExerciseSearch } from './ExerciseSearch';
import { SetEntry, type SetDraft } from './SetEntry';

type RecentLoggedSet = {
  exerciseName: string;
  set: StrengthSet;
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
  const [draft, setDraft] = React.useState<SetDraft>(initialDraft);
  const [recentSets, setRecentSets] = React.useState<RecentLoggedSet[]>([]);
  const [message, setMessage] = React.useState<string | null>(null);
  const [isSaving, setIsSaving] = React.useState(false);

  async function handleAddSet() {
    if (!selectedExercise) {
      setMessage('Choose an exercise first.');
      return;
    }

    setIsSaving(true);
    setMessage(null);
    try {
      const activeSession = session ?? (await createWorkoutSession()).session;
      if (!session) {
        setSession(activeSession);
      }

      const response = await addStrengthSet(activeSession.id, {
        exercise_id: selectedExercise.id,
        reps: draft.reps,
        set_number: draft.setNumber,
        weight_kg: draft.weightKg
      });

      setRecentSets((sets) => [
        {
          exerciseName: selectedExercise.name,
          set: response.strength_set
        },
        ...sets
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

  return (
    <section className="log-layout" aria-labelledby="log-heading">
      <div className="log-workspace">
        <div className="section-heading">
          <p className="eyebrow">Workout log</p>
          <h2 id="log-heading">Today</h2>
        </div>

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

        {message && <p className="form-message">{message}</p>}
      </div>

      <aside className="recent-panel" aria-labelledby="recent-heading">
        <div className="section-heading">
          <p className="eyebrow">Logged</p>
          <h2 id="recent-heading">Sets</h2>
        </div>
        {recentSets.length === 0 ? (
          <p className="empty-state">No sets logged in this session.</p>
        ) : (
          <ol className="recent-sets">
            {recentSets.map((item) => (
              <li key={item.set.id}>
                <span>{item.exerciseName}</span>
                <strong>
                  {item.set.set_number} x {item.set.reps} @ {item.set.weight_kg} kg
                </strong>
              </li>
            ))}
          </ol>
        )}
      </aside>
    </section>
  );
}

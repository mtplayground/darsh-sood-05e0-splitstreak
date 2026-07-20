import React from 'react';

import type { ExerciseSearchItem } from '../apiClient';
import { searchExercises } from '../apiClient';

type ExerciseSearchProps = {
  disabled?: boolean;
  onSelect: (exercise: ExerciseSearchItem | null) => void;
  selectedExercise: ExerciseSearchItem | null;
};

export function ExerciseSearch({
  disabled = false,
  onSelect,
  selectedExercise
}: ExerciseSearchProps) {
  const [query, setQuery] = React.useState(selectedExercise?.name ?? '');
  const [results, setResults] = React.useState<ExerciseSearchItem[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (selectedExercise) {
      setQuery(selectedExercise.name);
    }
  }, [selectedExercise]);

  React.useEffect(() => {
    const normalized = query.trim();
    if (normalized.length < 2 || selectedExercise?.name === query) {
      setResults([]);
      setIsLoading(false);
      setError(null);
      return;
    }

    const abortController = new AbortController();
    const timer = window.setTimeout(() => {
      setIsLoading(true);
      setError(null);
      searchExercises(normalized)
        .then((response) => {
          if (!abortController.signal.aborted) {
            setResults(response.exercises);
          }
        })
        .catch((caught) => {
          if (!abortController.signal.aborted) {
            setResults([]);
            setError(caught instanceof Error ? caught.message : 'Search failed');
          }
        })
        .finally(() => {
          if (!abortController.signal.aborted) {
            setIsLoading(false);
          }
        });
    }, 160);

    return () => {
      abortController.abort();
      window.clearTimeout(timer);
    };
  }, [query, selectedExercise]);

  return (
    <div className="exercise-search">
      <label htmlFor="exercise-search">Exercise</label>
      <input
        autoComplete="off"
        disabled={disabled}
        id="exercise-search"
        onChange={(event) => {
          setQuery(event.target.value);
          if (selectedExercise && event.target.value !== selectedExercise.name) {
            onSelect(null);
          }
        }}
        placeholder="Bench, squat, row..."
        type="search"
        value={query}
      />

      <div className="search-status" aria-live="polite">
        {isLoading ? 'Searching...' : error}
      </div>

      {results.length > 0 && (
        <div className="autocomplete-list" role="listbox">
          {results.map((exercise) => (
            <button
              className="autocomplete-option"
              key={exercise.id}
              onClick={() => {
                onSelect(exercise);
                setQuery(exercise.name);
                setResults([]);
              }}
              role="option"
              type="button"
            >
              <span>{exercise.name}</span>
              <small>{exercise.equipment ?? 'bodyweight'}</small>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

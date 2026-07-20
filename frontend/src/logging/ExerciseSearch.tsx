import React from 'react';

import type { ExerciseSearchItem } from '../apiClient';
import { searchExercises } from '../apiClient';

type ExerciseSearchProps = {
  disabled?: boolean;
  inputId?: string;
  label?: string;
  modality?: ExerciseSearchItem['modality'];
  onSelect: (exercise: ExerciseSearchItem | null) => void;
  placeholder?: string;
  selectedExercise: ExerciseSearchItem | null;
};

export function ExerciseSearch({
  disabled = false,
  inputId = 'exercise-search',
  label = 'Exercise',
  modality = 'strength',
  onSelect,
  placeholder = 'Bench, squat, row...',
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
      searchExercises(normalized, modality)
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
  }, [modality, query, selectedExercise]);

  return (
    <div className="exercise-search">
      <label htmlFor={inputId}>{label}</label>
      <input
        autoComplete="off"
        disabled={disabled}
        id={inputId}
        onChange={(event) => {
          setQuery(event.target.value);
          if (selectedExercise && event.target.value !== selectedExercise.name) {
            onSelect(null);
          }
        }}
        placeholder={placeholder}
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

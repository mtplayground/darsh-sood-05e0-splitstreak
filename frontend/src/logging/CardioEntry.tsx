import React from 'react';

import type { AddCardioEntryPayload, ExerciseSearchItem } from '../apiClient';
import { ExerciseSearch } from './ExerciseSearch';
import { NumberStepper } from './SetEntry';

type CardioType = {
  label: string;
  value: string;
};

export type CardioEntrySubmission = {
  exerciseName: string;
  payload: AddCardioEntryPayload;
};

type CardioEntryProps = {
  disabled?: boolean;
  onSubmit: (submission: CardioEntrySubmission) => void;
};

const cardioTypes: CardioType[] = [
  { label: 'Treadmill', value: 'treadmill' },
  { label: 'Outdoor run', value: 'outdoor_run' },
  { label: 'Walk', value: 'walk' },
  { label: 'Cycling', value: 'cycling' },
  { label: 'Stationary bike', value: 'stationary_bike' },
  { label: 'Rowing', value: 'rowing' },
  { label: 'Elliptical', value: 'elliptical' },
  { label: 'Stair climber', value: 'stair_climber' },
  { label: 'Swimming', value: 'swimming' },
  { label: 'Jump rope', value: 'jump_rope' }
];

const machineCardioTypes = new Set([
  'treadmill',
  'stationary_bike',
  'rowing',
  'elliptical',
  'stair_climber'
]);

export function CardioEntry({ disabled = false, onSubmit }: CardioEntryProps) {
  const [selectedExercise, setSelectedExercise] =
    React.useState<ExerciseSearchItem | null>(null);
  const [cardioType, setCardioType] = React.useState('treadmill');
  const [durationMinutes, setDurationMinutes] = React.useState(20);
  const [distanceKm, setDistanceKm] = React.useState(3);
  const [intensityLevel, setIntensityLevel] = React.useState(5);
  const [speedKph, setSpeedKph] = React.useState(9);
  const [inclinePercent, setInclinePercent] = React.useState(1);
  const [notes, setNotes] = React.useState('');
  const [message, setMessage] = React.useState<string | null>(null);
  const isMachineCardio = machineCardioTypes.has(cardioType);

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedExercise) {
      setMessage('Choose a cardio exercise first.');
      return;
    }

    setMessage(null);
    onSubmit({
      exerciseName: selectedExercise.name,
      payload: {
        cardio_type: cardioType,
        distance_meters: Math.round(distanceKm * 1000),
        duration_seconds: Math.round(durationMinutes * 60),
        exercise_id: selectedExercise.id,
        incline_percent: isMachineCardio ? inclinePercent : undefined,
        intensity_level: intensityLevel,
        notes: notes.trim() || undefined,
        speed_kph: isMachineCardio ? speedKph : undefined
      }
    });
  }

  return (
    <form className="cardio-entry" onSubmit={handleSubmit}>
      <label className="cardio-type-field" htmlFor="cardio-type">
        <span>Type</span>
        <select
          disabled={disabled}
          id="cardio-type"
          onChange={(event) => setCardioType(event.target.value)}
          value={cardioType}
        >
          {cardioTypes.map((type) => (
            <option key={type.value} value={type.value}>
              {type.label}
            </option>
          ))}
        </select>
      </label>

      <ExerciseSearch
        disabled={disabled}
        inputId="cardio-exercise-search"
        label="Cardio exercise"
        modality="cardio"
        onSelect={(exercise) => {
          setSelectedExercise(exercise);
          setMessage(null);
          if (exercise) {
            setCardioType(inferCardioType(exercise));
          }
        }}
        placeholder="Run, bike, row..."
        selectedExercise={selectedExercise}
      />

      <div className="cardio-grid">
        <NumberStepper
          label="Minutes"
          max={1440}
          min={1}
          onChange={setDurationMinutes}
          step={5}
          value={durationMinutes}
        />
        <NumberStepper
          label="Distance km"
          max={1000}
          min={0}
          onChange={setDistanceKm}
          step={0.25}
          value={distanceKm}
        />
        <NumberStepper
          label="Intensity"
          max={10}
          min={1}
          onChange={setIntensityLevel}
          step={1}
          value={intensityLevel}
        />
      </div>

      {isMachineCardio && (
        <div className="cardio-grid cardio-grid--machine">
          <NumberStepper
            label="Speed kph"
            max={80}
            min={0}
            onChange={setSpeedKph}
            step={0.5}
            value={speedKph}
          />
          <NumberStepper
            label="Incline %"
            max={40}
            min={-20}
            onChange={setInclinePercent}
            step={0.5}
            value={inclinePercent}
          />
        </div>
      )}

      <label className="notes-field" htmlFor="cardio-notes">
        <span>Notes</span>
        <input
          disabled={disabled}
          id="cardio-notes"
          onChange={(event) => setNotes(event.target.value)}
          placeholder="Optional"
          value={notes}
        />
      </label>

      <button
        className="primary-action set-entry__submit"
        disabled={disabled}
        type="submit"
      >
        Add cardio
      </button>
      {message && <p className="form-message form-message--error">{message}</p>}
    </form>
  );
}

function inferCardioType(exercise: ExerciseSearchItem) {
  switch (exercise.slug) {
    case 'treadmill-run':
      return 'treadmill';
    case 'outdoor-run':
      return 'outdoor_run';
    case 'walk':
      return 'walk';
    case 'stationary-bike':
      return 'stationary_bike';
    case 'rowing-machine':
      return 'rowing';
    case 'elliptical':
      return 'elliptical';
    case 'stair-climber':
      return 'stair_climber';
    case 'swimming':
      return 'swimming';
    case 'jump-rope':
      return 'jump_rope';
    default:
      return exercise.equipment === 'bike' ? 'cycling' : 'outdoor_run';
  }
}

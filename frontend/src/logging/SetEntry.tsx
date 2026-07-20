import React from 'react';

export type SetDraft = {
  reps: number;
  setNumber: number;
  weightKg: number;
};

type SetEntryProps = {
  disabled?: boolean;
  draft: SetDraft;
  onChange: (draft: SetDraft) => void;
  onSubmit: () => void;
};

export function SetEntry({
  disabled = false,
  draft,
  onChange,
  onSubmit
}: SetEntryProps) {
  return (
    <form
      className="set-entry"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <NumberStepper
        label="Set"
        max={200}
        min={1}
        onChange={(setNumber) => onChange({ ...draft, setNumber })}
        step={1}
        value={draft.setNumber}
      />
      <NumberStepper
        label="Reps"
        max={1000}
        min={1}
        onChange={(reps) => onChange({ ...draft, reps })}
        step={1}
        value={draft.reps}
      />
      <NumberStepper
        label="Weight kg"
        max={2000}
        min={0}
        onChange={(weightKg) => onChange({ ...draft, weightKg })}
        step={2.5}
        value={draft.weightKg}
      />
      <button
        className="primary-action set-entry__submit"
        disabled={disabled}
        type="submit"
      >
        Add set
      </button>
    </form>
  );
}

type NumberStepperProps = {
  label: string;
  max: number;
  min: number;
  onChange: (value: number) => void;
  step: number;
  value: number;
};

function NumberStepper({ label, max, min, onChange, step, value }: NumberStepperProps) {
  function commit(nextValue: number) {
    const rounded = Math.round(nextValue * 100) / 100;
    onChange(Math.min(max, Math.max(min, rounded)));
  }

  return (
    <label className="number-stepper">
      <span>{label}</span>
      <div className="stepper-controls">
        <button
          aria-label={`Decrease ${label}`}
          onClick={() => commit(value - step)}
          type="button"
        >
          -
        </button>
        <input
          inputMode="decimal"
          max={max}
          min={min}
          onChange={(event) => commit(Number(event.target.value))}
          step={step}
          type="number"
          value={Number.isInteger(value) ? value.toString() : value.toFixed(1)}
        />
        <button
          aria-label={`Increase ${label}`}
          onClick={() => commit(value + step)}
          type="button"
        >
          +
        </button>
      </div>
    </label>
  );
}

import type { StreakSummary } from './apiClient';

type StreakIndicatorProps = {
  isLoading: boolean;
  summary: StreakSummary | null;
};

export function StreakIndicator({ isLoading, summary }: StreakIndicatorProps) {
  const currentDays = summary?.current_days ?? 0;

  return (
    <div className="streak-counter" aria-live="polite">
      <div>
        <strong>{isLoading ? '-' : currentDays}</strong>
        <span>{currentDays === 1 ? 'day' : 'days'}</span>
      </div>
      <p>{streakMessage(isLoading, summary)}</p>
    </div>
  );
}

function streakMessage(isLoading: boolean, summary: StreakSummary | null) {
  if (isLoading) {
    return 'Loading streak...';
  }

  if (!summary || summary.status === 'no_active_split') {
    return 'Select a split to start rest-day-aware streak tracking.';
  }

  if (summary.current_days === 0) {
    return summary.today_is_training_day
      ? 'Log today to start your streak.'
      : 'Log your next workout to start your streak.';
  }

  if (summary.today_is_training_day && !summary.current_streak_started_on) {
    return 'Today is a training day.';
  }

  if (summary.today_is_training_day) {
    return summary.today_schedule_item
      ? `Today: ${summary.today_schedule_item}`
      : 'Today is a training day.';
  }

  return summary.today_schedule_item
    ? `Today: ${summary.today_schedule_item}`
    : 'Rest day is protected.';
}

import React from 'react';

import {
  ApiError,
  type StreakCalendarResponse,
  type TodayDashboardResponse,
  fetchStreakCalendar,
  fetchTodayDashboard,
  redirectToLogin
} from './apiClient';
import { getTodayLocalSummary, type TodayLocalSummary } from './localStore';
import { StreakCalendar } from './StreakCalendar';
import { StreakIndicator } from './StreakIndicator';

type HomeDashboardProps = {
  onQuickStart: () => void;
  userSub: string;
};

export function HomeDashboard({ onQuickStart, userSub }: HomeDashboardProps) {
  const [dashboard, setDashboard] = React.useState<TodayDashboardResponse | null>(null);
  const [streakCalendar, setStreakCalendar] =
    React.useState<StreakCalendarResponse | null>(null);
  const [localSummary, setLocalSummary] = React.useState<TodayLocalSummary>(() =>
    getTodayLocalSummary(userSub)
  );
  const [message, setMessage] = React.useState<string | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);

  const refreshLocalSummary = React.useCallback(() => {
    setLocalSummary(getTodayLocalSummary(userSub));
  }, [userSub]);

  const loadDashboard = React.useCallback(async () => {
    setIsLoading(true);
    setMessage(null);
    refreshLocalSummary();
    try {
      const [dashboardResponse, streakResponse] = await Promise.all([
        fetchTodayDashboard(),
        fetchStreakCalendar()
      ]);
      setDashboard(dashboardResponse);
      setStreakCalendar(streakResponse);
    } catch (caught) {
      if (caught instanceof ApiError && caught.status === 401) {
        redirectToLogin(caught.loginUrl);
        return;
      }

      setMessage(
        caught instanceof Error ? caught.message : 'Dashboard could not be loaded.'
      );
    } finally {
      setIsLoading(false);
    }
  }, [refreshLocalSummary]);

  React.useEffect(() => {
    void loadDashboard();
    window.addEventListener('splitstreak-local-workouts-updated', refreshLocalSummary);
    return () => {
      window.removeEventListener(
        'splitstreak-local-workouts-updated',
        refreshLocalSummary
      );
    };
  }, [loadDashboard, refreshLocalSummary]);

  const workout = dashboard?.workout ?? null;
  const strengthCount = Math.max(
    workout?.strength_set_count ?? 0,
    localSummary.strengthSetCount
  );
  const cardioCount = Math.max(
    workout?.cardio_entry_count ?? 0,
    localSummary.cardioEntryCount
  );
  const hasWorkout = Boolean(workout || localSummary.session);
  const totalEntries = strengthCount + cardioCount;
  const streakSummary = streakCalendar?.summary ?? dashboard?.streak ?? null;

  return (
    <section className="dashboard-layout" aria-labelledby="dashboard-heading">
      <div className="dashboard-main">
        <div className="section-heading">
          <p className="eyebrow">Today</p>
          <h2 id="dashboard-heading">Workout</h2>
        </div>

        {isLoading ? (
          <p className="empty-state" aria-live="polite">
            Loading today...
          </p>
        ) : hasWorkout ? (
          <div className="today-workout">
            <div>
              <p className="dashboard-kicker">
                Started{' '}
                {formatTime(
                  workout?.session.started_at ??
                    localSummary.session?.started_at ??
                    new Date().toISOString()
                )}
              </p>
              <h3>Workout in progress</h3>
              <p>{entrySummary(strengthCount, cardioCount)}</p>
            </div>
            <div className="dashboard-metrics" aria-label="Today workout totals">
              <Metric label="Entries" value={totalEntries.toString()} />
              <Metric label="Strength" value={strengthCount.toString()} />
              <Metric label="Cardio" value={cardioCount.toString()} />
            </div>
          </div>
        ) : (
          <div className="today-workout">
            <div>
              <p className="dashboard-kicker">Ready</p>
              <h3>No workout started today</h3>
              <p>Start a session and add your first set or cardio entry.</p>
            </div>
            <div className="dashboard-metrics" aria-label="Today workout totals">
              <Metric label="Entries" value="0" />
              <Metric label="Strength" value="0" />
              <Metric label="Cardio" value="0" />
            </div>
          </div>
        )}

        <div className="dashboard-actions">
          <button className="primary-action" onClick={onQuickStart} type="button">
            {hasWorkout ? 'Continue logging' : 'Start workout'}
          </button>
          <button disabled={isLoading} onClick={loadDashboard} type="button">
            Refresh
          </button>
        </div>

        {localSummary.pendingCount > 0 && (
          <p className="form-message">{localSummary.pendingCount} pending sync.</p>
        )}
        {message && <p className="form-message form-message--error">{message}</p>}
      </div>

      <aside className="streak-panel" aria-labelledby="streak-heading">
        <div className="section-heading">
          <p className="eyebrow">Streak</p>
          <h2 id="streak-heading">Current run</h2>
        </div>
        <StreakIndicator isLoading={isLoading} summary={streakSummary} />
        <StreakCalendar days={streakCalendar?.days ?? []} isLoading={isLoading} />
      </aside>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function entrySummary(strengthCount: number, cardioCount: number) {
  if (strengthCount === 0 && cardioCount === 0) {
    return 'No entries logged yet.';
  }

  const parts = [];
  if (strengthCount > 0) {
    parts.push(`${strengthCount} strength ${strengthCount === 1 ? 'set' : 'sets'}`);
  }
  if (cardioCount > 0) {
    parts.push(`${cardioCount} cardio ${cardioCount === 1 ? 'entry' : 'entries'}`);
  }

  return parts.join(', ');
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    hour: 'numeric',
    minute: '2-digit'
  }).format(new Date(value));
}

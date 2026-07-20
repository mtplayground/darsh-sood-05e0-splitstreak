import React from 'react';

import {
  ApiError,
  type TodayDashboardResponse,
  fetchTodayDashboard,
  redirectToLogin
} from './apiClient';

type HomeDashboardProps = {
  onQuickStart: () => void;
};

export function HomeDashboard({ onQuickStart }: HomeDashboardProps) {
  const [dashboard, setDashboard] = React.useState<TodayDashboardResponse | null>(null);
  const [message, setMessage] = React.useState<string | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);

  const loadDashboard = React.useCallback(async () => {
    setIsLoading(true);
    setMessage(null);
    try {
      setDashboard(await fetchTodayDashboard());
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
  }, []);

  React.useEffect(() => {
    void loadDashboard();
  }, [loadDashboard]);

  const workout = dashboard?.workout ?? null;
  const totalEntries =
    (workout?.strength_set_count ?? 0) + (workout?.cardio_entry_count ?? 0);

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
        ) : workout ? (
          <div className="today-workout">
            <div>
              <p className="dashboard-kicker">
                Started {formatTime(workout.session.started_at)}
              </p>
              <h3>Workout in progress</h3>
              <p>
                {entrySummary(workout.strength_set_count, workout.cardio_entry_count)}
              </p>
            </div>
            <div className="dashboard-metrics" aria-label="Today workout totals">
              <Metric label="Entries" value={totalEntries.toString()} />
              <Metric label="Strength" value={workout.strength_set_count.toString()} />
              <Metric label="Cardio" value={workout.cardio_entry_count.toString()} />
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
            {workout ? 'Continue logging' : 'Start workout'}
          </button>
          <button disabled={isLoading} onClick={loadDashboard} type="button">
            Refresh
          </button>
        </div>

        {message && <p className="form-message form-message--error">{message}</p>}
      </div>

      <aside className="streak-panel" aria-labelledby="streak-heading">
        <div className="section-heading">
          <p className="eyebrow">Streak</p>
          <h2 id="streak-heading">Current run</h2>
        </div>
        <div className="streak-placeholder">
          <strong>{dashboard?.streak.current_days ?? 0}</strong>
          <span>days</span>
        </div>
        <p className="empty-state">Pending</p>
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

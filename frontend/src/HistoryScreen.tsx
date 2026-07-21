import React from 'react';

import {
  ApiError,
  type HistoryCardioEntry,
  type HistorySession,
  type HistorySessionsResponse,
  type HistoryStrengthSet,
  fetchHistorySessions,
  redirectToLogin
} from './apiClient';

const PAGE_SIZE = 10;

export function HistoryScreen() {
  const [history, setHistory] = React.useState<HistorySessionsResponse | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [isLoadingMore, setIsLoadingMore] = React.useState(false);
  const [message, setMessage] = React.useState<string | null>(null);

  const loadHistory = React.useCallback(
    async ({ append, offset }: { append: boolean; offset: number }) => {
      if (append) {
        setIsLoadingMore(true);
      } else {
        setIsLoading(true);
      }
      setMessage(null);

      try {
        const response = await fetchHistorySessions(PAGE_SIZE, offset);
        setHistory((current) =>
          append && current
            ? {
                page: response.page,
                sessions: [...current.sessions, ...response.sessions]
              }
            : response
        );
      } catch (caught) {
        if (caught instanceof ApiError && caught.status === 401) {
          redirectToLogin(caught.loginUrl);
          return;
        }

        setMessage(
          caught instanceof Error ? caught.message : 'History could not be loaded.'
        );
      } finally {
        if (append) {
          setIsLoadingMore(false);
        } else {
          setIsLoading(false);
        }
      }
    },
    []
  );

  React.useEffect(() => {
    void loadHistory({ append: false, offset: 0 });
  }, [loadHistory]);

  const sessions = history?.sessions ?? [];

  return (
    <section className="history-screen" aria-labelledby="history-heading">
      <div className="history-toolbar">
        <div className="section-heading">
          <p className="eyebrow">History</p>
          <h2 id="history-heading">Past sessions</h2>
        </div>
        <button
          disabled={isLoading || isLoadingMore}
          onClick={() => void loadHistory({ append: false, offset: 0 })}
          type="button"
        >
          Refresh
        </button>
      </div>

      {isLoading ? (
        <p className="empty-state" aria-live="polite">
          Loading history...
        </p>
      ) : sessions.length === 0 ? (
        <p className="empty-state">No logged sessions yet.</p>
      ) : (
        <ol className="history-list">
          {sessions.map((session) => (
            <HistoryCard key={session.id} session={session} />
          ))}
        </ol>
      )}

      {history?.page.has_more && (
        <button
          className="history-load-more"
          disabled={isLoadingMore}
          onClick={() =>
            void loadHistory({
              append: true,
              offset: history.page.next_offset ?? sessions.length
            })
          }
          type="button"
        >
          {isLoadingMore ? 'Loading...' : 'Load more'}
        </button>
      )}

      {message && <p className="form-message form-message--error">{message}</p>}
    </section>
  );
}

function HistoryCard({ session }: { session: HistorySession }) {
  const strengthGroups = groupStrengthSets(session.strength_sets);
  const entryCount = session.strength_sets.length + session.cardio_entries.length;

  return (
    <li className="history-card">
      <div className="history-card__header">
        <div>
          <p className="dashboard-kicker">{formatDate(session.date)}</p>
          <h3>{formatTimeRange(session.started_at, session.completed_at)}</h3>
        </div>
        <div className="history-card__metrics" aria-label="Session totals">
          <Metric label="Entries" value={entryCount.toString()} />
          <Metric label="Strength" value={session.strength_sets.length.toString()} />
          <Metric label="Cardio" value={session.cardio_entries.length.toString()} />
        </div>
      </div>

      {session.notes && <p className="history-card__notes">{session.notes}</p>}

      <div className="history-card__body">
        {strengthGroups.length > 0 && (
          <section aria-label="Strength exercises">
            <h4>Strength</h4>
            <div className="history-entry-list">
              {strengthGroups.map((group) => (
                <div className="history-entry" key={group.exerciseName}>
                  <strong>{group.exerciseName}</strong>
                  <span>{formatStrengthSets(group.sets)}</span>
                </div>
              ))}
            </div>
          </section>
        )}

        {session.cardio_entries.length > 0 && (
          <section aria-label="Cardio entries">
            <h4>Cardio</h4>
            <div className="history-entry-list">
              {session.cardio_entries.map((entry) => (
                <div className="history-entry" key={entry.id}>
                  <strong>{entry.exercise_name}</strong>
                  <span>{formatCardioEntry(entry)}</span>
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </li>
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

function groupStrengthSets(sets: HistoryStrengthSet[]) {
  const groups = new Map<
    string,
    { exerciseName: string; sets: HistoryStrengthSet[] }
  >();

  for (const set of sets) {
    const existing = groups.get(set.exercise_slug);
    if (existing) {
      existing.sets.push(set);
    } else {
      groups.set(set.exercise_slug, {
        exerciseName: set.exercise_name,
        sets: [set]
      });
    }
  }

  return [...groups.values()];
}

function formatStrengthSets(sets: HistoryStrengthSet[]) {
  return sets.map((set) => `${set.reps} x ${formatWeight(set.weight_kg)}`).join(', ');
}

function formatCardioEntry(entry: HistoryCardioEntry) {
  const parts = [formatDuration(entry.duration_seconds)];

  if (entry.distance_meters !== null) {
    parts.push(formatDistance(entry.distance_meters));
  }

  if (entry.speed_kph !== null) {
    parts.push(`${formatNumber(entry.speed_kph)} kph`);
  }

  if (entry.incline_percent !== null) {
    parts.push(`${formatNumber(entry.incline_percent)}% incline`);
  }

  if (entry.intensity_level !== null) {
    parts.push(`RPE ${entry.intensity_level}`);
  }

  return parts.join(' · ');
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    day: 'numeric',
    month: 'short',
    weekday: 'short'
  }).format(new Date(`${value}T00:00:00Z`));
}

function formatTimeRange(startedAt: string, completedAt: string | null) {
  const start = formatTime(startedAt);
  if (!completedAt) {
    return `${start} session`;
  }

  return `${start} - ${formatTime(completedAt)}`;
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    hour: 'numeric',
    minute: '2-digit'
  }).format(new Date(value));
}

function formatDuration(seconds: number) {
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) {
    return `${minutes} min`;
  }

  const hours = Math.floor(minutes / 60);
  const remainder = minutes % 60;
  return remainder === 0 ? `${hours} hr` : `${hours} hr ${remainder} min`;
}

function formatDistance(meters: number) {
  if (meters < 1000) {
    return `${Math.round(meters)} m`;
  }

  return `${formatNumber(meters / 1000)} km`;
}

function formatWeight(weightKg: number) {
  return `${formatNumber(weightKg)} kg`;
}

function formatNumber(value: number) {
  return new Intl.NumberFormat(undefined, {
    maximumFractionDigits: 1,
    minimumFractionDigits: 0
  }).format(value);
}

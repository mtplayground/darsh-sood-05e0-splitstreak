import type { CSSProperties } from 'react';

import type { HistorySession, StreakCalendarDay } from './apiClient';
import { StreakCalendar } from './StreakCalendar';

type TrendChartsProps = {
  isLoading: boolean;
  sessions: HistorySession[];
  streakDays: StreakCalendarDay[];
};

type VolumePoint = {
  date: string;
  label: string;
  sessionId: number;
  volumeKg: number;
};

export function TrendCharts({ isLoading, sessions, streakDays }: TrendChartsProps) {
  return (
    <div className="trend-grid" aria-label="Training trends">
      <WeightTrendChart isLoading={isLoading} sessions={sessions} />
      <section className="trend-card" aria-labelledby="streak-trend-heading">
        <div className="section-heading">
          <p className="eyebrow">Streak</p>
          <h3 id="streak-trend-heading">Calendar</h3>
        </div>
        <StreakCalendar days={streakDays} isLoading={isLoading} />
      </section>
    </div>
  );
}

function WeightTrendChart({
  isLoading,
  sessions
}: {
  isLoading: boolean;
  sessions: HistorySession[];
}) {
  const points = buildVolumePoints(sessions);
  const maxVolume = Math.max(...points.map((point) => point.volumeKg), 0);

  return (
    <section className="trend-card" aria-labelledby="weight-trend-heading">
      <div className="section-heading">
        <p className="eyebrow">Strength</p>
        <h3 id="weight-trend-heading">Weight lifted</h3>
      </div>

      {isLoading ? (
        <p className="empty-state">Loading trend...</p>
      ) : points.length === 0 || maxVolume === 0 ? (
        <p className="empty-state">Log weighted sets to build this trend.</p>
      ) : (
        <>
          <div className="weight-chart" aria-label="Weight lifted over recent sessions">
            {points.map((point) => (
              <div className="weight-chart__bar-wrap" key={point.sessionId}>
                <span
                  className="weight-chart__bar"
                  style={
                    {
                      '--bar-height': `${Math.max(
                        8,
                        (point.volumeKg / maxVolume) * 100
                      )}%`
                    } as CSSProperties
                  }
                  title={`${point.label}: ${formatWeight(point.volumeKg)}`}
                />
                <span>{point.label}</span>
              </div>
            ))}
          </div>
          <p className="trend-caption">
            Peak recent session: {formatWeight(maxVolume)}
          </p>
        </>
      )}
    </section>
  );
}

function buildVolumePoints(sessions: HistorySession[]): VolumePoint[] {
  return sessions
    .slice(0, 8)
    .map((session) => ({
      date: session.date,
      label: formatShortDate(session.date),
      sessionId: session.id,
      volumeKg: session.strength_sets.reduce(
        (total, set) => total + set.reps * set.weight_kg,
        0
      )
    }))
    .reverse();
}

function formatShortDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    day: 'numeric',
    month: 'short'
  }).format(new Date(`${value}T00:00:00Z`));
}

function formatWeight(weightKg: number) {
  return `${formatNumber(weightKg)} kg`;
}

function formatNumber(value: number) {
  return new Intl.NumberFormat(undefined, {
    maximumFractionDigits: 0,
    minimumFractionDigits: 0
  }).format(value);
}

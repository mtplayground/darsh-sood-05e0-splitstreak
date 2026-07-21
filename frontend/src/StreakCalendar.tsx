import type { StreakCalendarDay, StreakCalendarDayStatus } from './apiClient';

type StreakCalendarProps = {
  days: StreakCalendarDay[];
  isLoading: boolean;
};

export function StreakCalendar({ days, isLoading }: StreakCalendarProps) {
  if (isLoading) {
    return <p className="empty-state">Loading calendar...</p>;
  }

  if (days.length === 0) {
    return <p className="empty-state">No streak calendar yet.</p>;
  }

  return (
    <div className="streak-calendar" aria-label="Streak calendar">
      <ol className="streak-calendar__grid">
        {days.map((day) => (
          <li
            aria-label={calendarDayLabel(day)}
            className={`streak-calendar__day streak-calendar__day--${day.status}`}
            key={day.date}
            title={calendarDayLabel(day)}
          >
            <span>{dayNumber(day.date)}</span>
          </li>
        ))}
      </ol>
      <div className="streak-calendar__legend" aria-label="Calendar legend">
        <LegendItem label="Logged" status="logged" />
        <LegendItem label="Rest" status="rest" />
        <LegendItem label="Missed" status="missed" />
      </div>
    </div>
  );
}

function LegendItem({
  label,
  status
}: {
  label: string;
  status: StreakCalendarDayStatus;
}) {
  return (
    <span>
      <i className={`streak-calendar__key streak-calendar__key--${status}`} />
      {label}
    </span>
  );
}

function calendarDayLabel(day: StreakCalendarDay) {
  const date = new Intl.DateTimeFormat(undefined, {
    day: 'numeric',
    month: 'short',
    weekday: 'short'
  }).format(new Date(`${day.date}T00:00:00Z`));
  const schedule = day.schedule_item ? `, ${day.schedule_item}` : '';
  return `${date}: ${statusLabel(day.status)}${schedule}`;
}

function statusLabel(status: StreakCalendarDayStatus) {
  switch (status) {
    case 'before_active_split':
      return 'before active split';
    case 'logged':
      return 'logged';
    case 'missed':
      return 'missed training day';
    case 'no_active_split':
      return 'no active split';
    case 'pending':
      return 'pending training day';
    case 'rest':
      return 'rest day';
  }
}

function dayNumber(date: string) {
  return new Date(`${date}T00:00:00Z`).getUTCDate().toString();
}

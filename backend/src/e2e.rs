use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, NaiveDate};

#[test]
fn core_flow_log_offline_sync_streak_history_and_trends() {
    let active_split = ActiveSplitSchedule {
        selected_on: date("2026-07-19"),
        days: vec![
            ScheduleDay::training("Strength"),
            ScheduleDay::rest(),
            ScheduleDay::training("Cardio"),
        ],
    };
    let today = date("2026-07-21");
    let mut queue = OfflineQueue::default();

    queue.capture_session(OfflineSession {
        client_id: "client-session-cardio-001".to_owned(),
        date: today,
        notes: Some("Hotel gym session".to_owned()),
    });
    queue.capture_cardio(CardioEntry {
        client_id: "client-cardio-001".to_owned(),
        client_session_id: "client-session-cardio-001".to_owned(),
        exercise_name: "Treadmill".to_owned(),
        duration_seconds: 1_200,
    });
    queue.capture_strength_set(StrengthSet {
        client_id: "client-set-001".to_owned(),
        client_session_id: "client-session-cardio-001".to_owned(),
        exercise_name: "Dumbbell bench press".to_owned(),
        set_number: 1,
        reps: 10,
        weight_kg: 30,
    });

    assert_eq!(queue.pending_count(), 3);

    let offline_batch = queue.pending_batch();
    let mut sync = ReconciliationHarness::default();
    let first_receipt = match sync.reconcile(&offline_batch) {
        Ok(receipt) => receipt,
        Err(error) => panic!("first sync should reconcile offline batch: {error}"),
    };
    queue.apply_receipt(&first_receipt);

    assert_eq!(queue.pending_count(), 0);
    assert_eq!(first_receipt.synced_sessions, 1);
    assert_eq!(first_receipt.synced_strength_sets, 1);
    assert_eq!(first_receipt.synced_cardio_entries, 1);

    let retry_receipt = match sync.reconcile(&offline_batch) {
        Ok(receipt) => receipt,
        Err(error) => panic!("retry sync should remain idempotent: {error}"),
    };

    assert_eq!(retry_receipt.session_ids, first_receipt.session_ids);
    assert_eq!(retry_receipt.strength_set_ids, first_receipt.strength_set_ids);
    assert_eq!(retry_receipt.cardio_entry_ids, first_receipt.cardio_entry_ids);
    assert_eq!(sync.session_count(), 1);
    assert_eq!(sync.strength_set_count(), 1);
    assert_eq!(sync.cardio_entry_count(), 1);

    let logged_days = sync.logged_days();
    let streak = compute_streak(&active_split, today, &logged_days);
    assert_eq!(streak.current_days, 1);
    assert_eq!(streak.today_status, DayStatus::Logged);
    assert_eq!(streak.days.last().map(|day| day.status), Some(DayStatus::Logged));
    assert!(
        streak
            .days
            .iter()
            .any(|day| day.date == date("2026-07-20") && day.status == DayStatus::Rest)
    );

    let history = sync.history_page(10);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].date, today);
    assert_eq!(history[0].exercise_names, vec!["Dumbbell bench press", "Treadmill"]);
    assert!(history[0].summary.contains("1 strength set"));
    assert!(history[0].summary.contains("20 min cardio"));

    let trends = sync.weight_trends();
    assert_eq!(
        trends,
        vec![TrendPoint {
            date: today,
            total_weight_kg: 300
        }]
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveSplitSchedule {
    selected_on: NaiveDate,
    days: Vec<ScheduleDay>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScheduleDay {
    label: String,
    is_training_day: bool,
}

impl ScheduleDay {
    fn training(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            is_training_day: true,
        }
    }

    fn rest() -> Self {
        Self {
            label: "Rest".to_owned(),
            is_training_day: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OfflineSession {
    client_id: String,
    date: NaiveDate,
    notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StrengthSet {
    client_id: String,
    client_session_id: String,
    exercise_name: String,
    set_number: i32,
    reps: i32,
    weight_kg: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CardioEntry {
    client_id: String,
    client_session_id: String,
    exercise_name: String,
    duration_seconds: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueuedMutation {
    Session(OfflineSession),
    StrengthSet(StrengthSet),
    Cardio(CardioEntry),
}

#[derive(Debug, Default)]
struct OfflineQueue {
    pending: Vec<QueuedMutation>,
}

impl OfflineQueue {
    fn capture_session(&mut self, session: OfflineSession) {
        self.pending.push(QueuedMutation::Session(session));
    }

    fn capture_strength_set(&mut self, strength_set: StrengthSet) {
        self.pending.push(QueuedMutation::StrengthSet(strength_set));
    }

    fn capture_cardio(&mut self, cardio_entry: CardioEntry) {
        self.pending.push(QueuedMutation::Cardio(cardio_entry));
    }

    fn pending_count(&self) -> usize {
        self.pending.len()
    }

    fn pending_batch(&self) -> Vec<QueuedMutation> {
        self.pending.clone()
    }

    fn apply_receipt(&mut self, receipt: &SyncReceipt) {
        self.pending.retain(|mutation| match mutation {
            QueuedMutation::Session(session) => {
                !receipt.session_ids.contains_key(&session.client_id)
            }
            QueuedMutation::StrengthSet(strength_set) => {
                !receipt.strength_set_ids.contains_key(&strength_set.client_id)
            }
            QueuedMutation::Cardio(cardio_entry) => {
                !receipt.cardio_entry_ids.contains_key(&cardio_entry.client_id)
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncedSession {
    server_id: i64,
    client_id: String,
    date: NaiveDate,
    notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncedStrengthSet {
    server_id: i64,
    session_server_id: i64,
    exercise_name: String,
    set_number: i32,
    reps: i32,
    weight_kg: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncedCardioEntry {
    server_id: i64,
    session_server_id: i64,
    exercise_name: String,
    duration_seconds: i32,
}

#[derive(Debug, Default)]
struct ReconciliationHarness {
    next_session_id: i64,
    next_strength_set_id: i64,
    next_cardio_entry_id: i64,
    sessions: BTreeMap<String, SyncedSession>,
    strength_sets: BTreeMap<String, SyncedStrengthSet>,
    cardio_entries: BTreeMap<String, SyncedCardioEntry>,
}

impl ReconciliationHarness {
    fn reconcile(&mut self, batch: &[QueuedMutation]) -> Result<SyncReceipt, String> {
        let mut receipt = SyncReceipt::default();

        for mutation in batch {
            if let QueuedMutation::Session(session) = mutation {
                if session.client_id.trim().is_empty() {
                    return Err("session client id cannot be blank".to_owned());
                }

                let synced = self.sessions.entry(session.client_id.clone()).or_insert_with(|| {
                    self.next_session_id += 1;
                    SyncedSession {
                        server_id: self.next_session_id,
                        client_id: session.client_id.clone(),
                        date: session.date,
                        notes: session.notes.clone(),
                    }
                });
                receipt
                    .session_ids
                    .insert(session.client_id.clone(), synced.server_id);
            }
        }

        for mutation in batch {
            match mutation {
                QueuedMutation::Session(_) => {}
                QueuedMutation::StrengthSet(strength_set) => {
                    let session_id = self.resolve_session_id(&strength_set.client_session_id)?;
                    let synced = self
                        .strength_sets
                        .entry(strength_set.client_id.clone())
                        .or_insert_with(|| {
                            self.next_strength_set_id += 1;
                            SyncedStrengthSet {
                                server_id: self.next_strength_set_id,
                                session_server_id: session_id,
                                exercise_name: strength_set.exercise_name.clone(),
                                set_number: strength_set.set_number,
                                reps: strength_set.reps,
                                weight_kg: strength_set.weight_kg,
                            }
                        });
                    receipt
                        .strength_set_ids
                        .insert(strength_set.client_id.clone(), synced.server_id);
                }
                QueuedMutation::Cardio(cardio_entry) => {
                    let session_id = self.resolve_session_id(&cardio_entry.client_session_id)?;
                    let synced = self
                        .cardio_entries
                        .entry(cardio_entry.client_id.clone())
                        .or_insert_with(|| {
                            self.next_cardio_entry_id += 1;
                            SyncedCardioEntry {
                                server_id: self.next_cardio_entry_id,
                                session_server_id: session_id,
                                exercise_name: cardio_entry.exercise_name.clone(),
                                duration_seconds: cardio_entry.duration_seconds,
                            }
                        });
                    receipt
                        .cardio_entry_ids
                        .insert(cardio_entry.client_id.clone(), synced.server_id);
                }
            }
        }

        receipt.synced_sessions = receipt.session_ids.len();
        receipt.synced_strength_sets = receipt.strength_set_ids.len();
        receipt.synced_cardio_entries = receipt.cardio_entry_ids.len();

        Ok(receipt)
    }

    fn resolve_session_id(&self, client_session_id: &str) -> Result<i64, String> {
        self.sessions
            .get(client_session_id)
            .map(|session| session.server_id)
            .ok_or_else(|| format!("client session was not found: {client_session_id}"))
    }

    fn logged_days(&self) -> BTreeSet<NaiveDate> {
        self.sessions
            .values()
            .filter(|session| {
                self.strength_sets
                    .values()
                    .any(|set| set.session_server_id == session.server_id)
                    || self
                        .cardio_entries
                        .values()
                        .any(|entry| entry.session_server_id == session.server_id)
            })
            .map(|session| session.date)
            .collect()
    }

    fn history_page(&self, limit: usize) -> Vec<HistoryRow> {
        let mut sessions = self.sessions.values().cloned().collect::<Vec<_>>();
        sessions.sort_by(|left, right| {
            right
                .date
                .cmp(&left.date)
                .then(right.server_id.cmp(&left.server_id))
        });

        sessions
            .into_iter()
            .take(limit)
            .map(|session| {
                let strength_sets = self
                    .strength_sets
                    .values()
                    .filter(|set| set.session_server_id == session.server_id)
                    .collect::<Vec<_>>();
                let cardio_entries = self
                    .cardio_entries
                    .values()
                    .filter(|entry| entry.session_server_id == session.server_id)
                    .collect::<Vec<_>>();
                let mut exercise_names = strength_sets
                    .iter()
                    .map(|set| set.exercise_name.as_str())
                    .chain(cardio_entries.iter().map(|entry| entry.exercise_name.as_str()))
                    .collect::<Vec<_>>();
                exercise_names.sort_unstable();

                let cardio_minutes = cardio_entries
                    .iter()
                    .map(|entry| entry.duration_seconds / 60)
                    .sum::<i32>();
                HistoryRow {
                    date: session.date,
                    exercise_names: exercise_names.into_iter().map(str::to_owned).collect(),
                    summary: format!(
                        "{} strength set, {cardio_minutes} min cardio",
                        strength_sets.len()
                    ),
                }
            })
            .collect()
    }

    fn weight_trends(&self) -> Vec<TrendPoint> {
        let session_dates = self
            .sessions
            .values()
            .map(|session| (session.server_id, session.date))
            .collect::<BTreeMap<_, _>>();
        let mut totals = BTreeMap::<NaiveDate, i32>::new();

        for strength_set in self.strength_sets.values() {
            if let Some(date) = session_dates.get(&strength_set.session_server_id) {
                let volume = strength_set.reps * strength_set.weight_kg;
                *totals.entry(*date).or_default() += volume;
            }
        }

        totals
            .into_iter()
            .map(|(date, total_weight_kg)| TrendPoint {
                date,
                total_weight_kg,
            })
            .collect()
    }

    fn session_count(&self) -> usize {
        self.sessions.len()
    }

    fn strength_set_count(&self) -> usize {
        self.strength_sets.len()
    }

    fn cardio_entry_count(&self) -> usize {
        self.cardio_entries.len()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SyncReceipt {
    synced_sessions: usize,
    synced_strength_sets: usize,
    synced_cardio_entries: usize,
    session_ids: BTreeMap<String, i64>,
    strength_set_ids: BTreeMap<String, i64>,
    cardio_entry_ids: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StreakSnapshot {
    current_days: i64,
    today_status: DayStatus,
    days: Vec<CalendarDay>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CalendarDay {
    date: NaiveDate,
    status: DayStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DayStatus {
    Logged,
    Missed,
    Pending,
    Rest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryRow {
    date: NaiveDate,
    exercise_names: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrendPoint {
    date: NaiveDate,
    total_weight_kg: i32,
}

fn compute_streak(
    active_split: &ActiveSplitSchedule,
    today: NaiveDate,
    logged_days: &BTreeSet<NaiveDate>,
) -> StreakSnapshot {
    let mut days = Vec::new();
    let mut current_days = 0;
    let mut cursor = active_split.selected_on;

    while cursor <= today {
        let schedule_day = schedule_day_on(active_split, cursor);
        let logged = logged_days.contains(&cursor);
        let status = if logged {
            DayStatus::Logged
        } else if !schedule_day.is_training_day {
            DayStatus::Rest
        } else if cursor == today {
            DayStatus::Pending
        } else {
            DayStatus::Missed
        };

        if status == DayStatus::Missed {
            current_days = 0;
        } else if status == DayStatus::Logged || current_days > 0 {
            current_days += 1;
        }

        days.push(CalendarDay { date: cursor, status });
        cursor = match cursor.checked_add_signed(Duration::days(1)) {
            Some(next) => next,
            None => break,
        };
    }

    StreakSnapshot {
        current_days,
        today_status: days
            .last()
            .map(|day| day.status)
            .unwrap_or(DayStatus::Pending),
        days,
    }
}

fn schedule_day_on(active_split: &ActiveSplitSchedule, date: NaiveDate) -> &ScheduleDay {
    let days_since_selected = date
        .signed_duration_since(active_split.selected_on)
        .num_days()
        .rem_euclid(active_split.days.len() as i64);

    &active_split.days[days_since_selected as usize]
}

fn date(value: &str) -> NaiveDate {
    match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        Ok(date) => date,
        Err(error) => panic!("test date should parse: {error}"),
    }
}

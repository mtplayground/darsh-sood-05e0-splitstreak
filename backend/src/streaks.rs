use std::collections::HashSet;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::active_splits::{self, ActiveSplit};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreakSummary {
    pub status: StreakStatus,
    pub current_days: i64,
    pub current_streak_started_on: Option<NaiveDate>,
    pub last_missed_training_day: Option<NaiveDate>,
    pub today: NaiveDate,
    pub today_schedule_item: Option<String>,
    pub today_is_training_day: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreakStatus {
    Active,
    NoActiveSplit,
}

pub async fn compute_current_streak(
    pool: &PgPool,
    user_sub: &str,
) -> Result<StreakSummary, sqlx::Error> {
    let today = Utc::now().date_naive();
    compute_current_streak_on(pool, user_sub, today).await
}

async fn compute_current_streak_on(
    pool: &PgPool,
    user_sub: &str,
    today: NaiveDate,
) -> Result<StreakSummary, sqlx::Error> {
    let active_split = active_splits::find_active_split(pool, user_sub).await?;
    let Some(active_split) = active_split else {
        return Ok(StreakSummary::no_active_split(today));
    };

    let selected_on = active_split.selected_at.date_naive();
    let logged_days = logged_training_days(pool, user_sub, selected_on, today).await?;

    Ok(compute_from_active_split(
        &active_split,
        selected_on,
        today,
        &logged_days,
    ))
}

async fn logged_training_days(
    pool: &PgPool,
    user_sub: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<HashSet<NaiveDate>, sqlx::Error> {
    let days = sqlx::query_scalar::<_, NaiveDate>(
        r#"
        SELECT DISTINCT (ws.started_at AT TIME ZONE 'UTC')::DATE AS workout_date
        FROM workout_sessions ws
        WHERE
            ws.user_sub = $1
            AND (ws.started_at AT TIME ZONE 'UTC')::DATE >= $2
            AND (ws.started_at AT TIME ZONE 'UTC')::DATE <= $3
            AND (
                EXISTS (
                    SELECT 1
                    FROM strength_sets ss
                    WHERE ss.session_id = ws.id
                )
                OR EXISTS (
                    SELECT 1
                    FROM cardio_entries ce
                    WHERE ce.session_id = ws.id
                )
            )
        "#,
    )
    .bind(user_sub)
    .bind(start_date)
    .bind(end_date)
    .persistent(false)
    .fetch_all(pool)
    .await?;

    Ok(days.into_iter().collect())
}

fn compute_from_active_split(
    active_split: &ActiveSplit,
    selected_on: NaiveDate,
    today: NaiveDate,
    logged_days: &HashSet<NaiveDate>,
) -> StreakSummary {
    if active_split.schedule.is_empty() || today < selected_on {
        return StreakSummary::active(today, None, false, 0, None, None);
    }

    let today_schedule = schedule_item_on(&active_split.schedule, selected_on, today);
    let today_is_training = today_schedule
        .as_deref()
        .map(is_training_schedule_item)
        .unwrap_or(false);

    let end_date = if today_is_training && !logged_days.contains(&today) {
        today.pred_opt()
    } else {
        Some(today)
    };

    let Some(end_date) = end_date else {
        return StreakSummary::active(today, today_schedule, today_is_training, 0, None, None);
    };

    if end_date < selected_on {
        return StreakSummary::active(today, today_schedule, today_is_training, 0, None, None);
    }

    let mut cursor = selected_on;
    let mut last_missed_training_day = None;
    while cursor <= end_date {
        let scheduled_item = schedule_item_on(&active_split.schedule, selected_on, cursor);
        let is_training_day = scheduled_item
            .as_deref()
            .map(is_training_schedule_item)
            .unwrap_or(false);

        if is_training_day && !logged_days.contains(&cursor) {
            last_missed_training_day = Some(cursor);
        }

        cursor = match cursor.succ_opt() {
            Some(next) => next,
            None => break,
        };
    }

    let window_start = last_missed_training_day
        .and_then(|day| day.succ_opt())
        .unwrap_or(selected_on);
    let streak_started_on = logged_days
        .iter()
        .copied()
        .filter(|day| *day >= window_start && *day <= end_date)
        .min();
    let current_days = streak_started_on
        .map(|started_on| end_date.signed_duration_since(started_on).num_days() + 1)
        .unwrap_or(0);

    StreakSummary::active(
        today,
        today_schedule,
        today_is_training,
        current_days,
        streak_started_on,
        last_missed_training_day,
    )
}

fn schedule_item_on(
    schedule: &[String],
    selected_on: NaiveDate,
    date: NaiveDate,
) -> Option<String> {
    if schedule.is_empty() || date < selected_on {
        return None;
    }

    let days_since_start = date.signed_duration_since(selected_on).num_days();
    let schedule_index = days_since_start.rem_euclid(schedule.len() as i64) as usize;
    schedule.get(schedule_index).cloned()
}

fn is_training_schedule_item(item: &str) -> bool {
    !item.trim().eq_ignore_ascii_case("rest")
}

impl StreakSummary {
    fn active(
        today: NaiveDate,
        today_schedule_item: Option<String>,
        today_is_training_day: bool,
        current_days: i64,
        current_streak_started_on: Option<NaiveDate>,
        last_missed_training_day: Option<NaiveDate>,
    ) -> Self {
        Self {
            status: StreakStatus::Active,
            current_days,
            current_streak_started_on,
            last_missed_training_day,
            today,
            today_schedule_item,
            today_is_training_day: Some(today_is_training_day),
        }
    }

    fn no_active_split(today: NaiveDate) -> Self {
        Self {
            status: StreakStatus::NoActiveSplit,
            current_days: 0,
            current_streak_started_on: None,
            last_missed_training_day: None,
            today,
            today_schedule_item: None,
            today_is_training_day: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn counts_rest_days_inside_an_active_streak() {
        let selected_on = date("2026-07-14");
        let today = date("2026-07-17");
        let active_split = active_split(
            selected_on,
            vec![
                "Full body".to_owned(),
                "Rest".to_owned(),
                "Full body".to_owned(),
                "Rest".to_owned(),
            ],
        );
        let logged_days = HashSet::from([date("2026-07-14"), date("2026-07-16")]);

        let summary =
            compute_from_active_split(&active_split, selected_on, today, &logged_days);

        assert_eq!(summary.status, StreakStatus::Active);
        assert_eq!(summary.current_days, 4);
        assert_eq!(summary.current_streak_started_on, Some(selected_on));
        assert_eq!(summary.last_missed_training_day, None);
        assert_eq!(summary.today_schedule_item, Some("Rest".to_owned()));
        assert_eq!(summary.today_is_training_day, Some(false));
    }

    #[test]
    fn breaks_on_past_missed_training_day() {
        let selected_on = date("2026-07-14");
        let today = date("2026-07-18");
        let active_split = active_split(
            selected_on,
            vec![
                "Train".to_owned(),
                "Rest".to_owned(),
                "Train".to_owned(),
                "Rest".to_owned(),
            ],
        );
        let logged_days = HashSet::from([date("2026-07-14"), date("2026-07-18")]);

        let summary =
            compute_from_active_split(&active_split, selected_on, today, &logged_days);

        assert_eq!(summary.current_days, 1);
        assert_eq!(summary.current_streak_started_on, Some(today));
        assert_eq!(summary.last_missed_training_day, Some(date("2026-07-16")));
    }

    #[test]
    fn does_not_break_on_unlogged_training_day_until_today_passes() {
        let selected_on = date("2026-07-14");
        let today = date("2026-07-17");
        let active_split = active_split(
            selected_on,
            vec!["Train".to_owned(), "Rest".to_owned(), "Rest".to_owned()],
        );
        let logged_days = HashSet::from([date("2026-07-14")]);

        let summary =
            compute_from_active_split(&active_split, selected_on, today, &logged_days);

        assert_eq!(summary.current_days, 3);
        assert_eq!(summary.current_streak_started_on, Some(selected_on));
        assert_eq!(summary.last_missed_training_day, None);
        assert_eq!(summary.today_is_training_day, Some(true));
    }

    #[test]
    fn rest_days_do_not_create_streak_before_first_logged_day() {
        let selected_on = date("2026-07-14");
        let today = date("2026-07-15");
        let active_split = active_split(
            selected_on,
            vec!["Train".to_owned(), "Rest".to_owned()],
        );
        let logged_days = HashSet::new();

        let summary =
            compute_from_active_split(&active_split, selected_on, today, &logged_days);

        assert_eq!(summary.current_days, 0);
        assert_eq!(summary.current_streak_started_on, None);
        assert_eq!(summary.last_missed_training_day, Some(selected_on));
    }

    #[test]
    fn logged_rest_day_can_start_a_streak() {
        let selected_on = date("2026-07-14");
        let today = date("2026-07-15");
        let active_split = active_split(
            selected_on,
            vec!["Rest".to_owned(), "Train".to_owned()],
        );
        let logged_days = HashSet::from([selected_on]);

        let summary =
            compute_from_active_split(&active_split, selected_on, today, &logged_days);

        assert_eq!(summary.current_days, 1);
        assert_eq!(summary.current_streak_started_on, Some(selected_on));
        assert_eq!(summary.today_is_training_day, Some(true));
    }

    fn active_split(selected_on: NaiveDate, schedule: Vec<String>) -> ActiveSplit {
        let selected_at = match DateTime::parse_from_rfc3339(&format!(
            "{selected_on}T00:00:00Z"
        )) {
            Ok(value) => value.with_timezone(&Utc),
            Err(error) => panic!("test active split timestamp should parse: {error}"),
        };

        ActiveSplit {
            user_sub: "user-sub".to_owned(),
            split_template_id: 1,
            template_slug: "test-split".to_owned(),
            template_name: "Test split".to_owned(),
            depth_level: "simple".to_owned(),
            training_days_per_cycle: schedule
                .iter()
                .filter(|item| is_training_schedule_item(item))
                .count() as i32,
            rest_days_per_cycle: schedule
                .iter()
                .filter(|item| !is_training_schedule_item(item))
                .count() as i32,
            schedule,
            selected_at,
            updated_at: selected_at,
        }
    }

    fn date(value: &str) -> NaiveDate {
        match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            Ok(date) => date,
            Err(error) => panic!("test date should parse: {error}"),
        }
    }
}

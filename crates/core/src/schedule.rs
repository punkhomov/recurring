//! Calendar-based recurrence rules.
//!
//! Dates are intentionally handled without a time zone here. The time zone only
//! matters when an occurrence is turned into a notification timestamp, which is
//! a storage/notification concern.

use std::num::NonZeroU16;

use chrono::{Datelike, Days, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScheduleError {
    #[error("repeat interval must be at least 1")]
    ZeroInterval,
    #[error("at least one weekday is required")]
    EmptyWeekdays,
    #[error("day of month must be between 1 and 31, got {0}")]
    InvalidDayOfMonth(u8),
    #[error("month must be between 1 and 12, got {0}")]
    InvalidMonth(u8),
}

/// A day of the month. `0` is the internal representation of "last day".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8")]
pub struct MonthDay(u8);

impl MonthDay {
    pub fn day(day: u8) -> Result<Self, ScheduleError> {
        if (1..=31).contains(&day) {
            Ok(Self(day))
        } else {
            Err(ScheduleError::InvalidDayOfMonth(day))
        }
    }

    pub fn last() -> Self {
        Self(0)
    }

    pub fn is_last(self) -> bool {
        self.0 == 0
    }

    pub fn number(self) -> Option<u8> {
        (self.0 != 0).then_some(self.0)
    }

    pub fn resolve(self, year: i32, month: u32) -> Option<NaiveDate> {
        if self.0 == 0 {
            return last_day_of_month(year, month);
        }
        NaiveDate::from_ymd_opt(year, month, u32::from(self.0))
            .or_else(|| last_day_of_month(year, month))
    }
}

impl TryFrom<u8> for MonthDay {
    type Error = ScheduleError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            Ok(Self::last())
        } else {
            Self::day(value)
        }
    }
}

/// A validated month number (1-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8")]
pub struct MonthOfYear(u8);

impl MonthOfYear {
    pub fn new(month: u8) -> Result<Self, ScheduleError> {
        if (1..=12).contains(&month) {
            Ok(Self(month))
        } else {
            Err(ScheduleError::InvalidMonth(month))
        }
    }

    pub fn number(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for MonthOfYear {
    type Error = ScheduleError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// A non-empty, de-duplicated set of weekdays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Weekday>")]
pub struct Weekdays(Vec<Weekday>);

impl Weekdays {
    pub fn new(weekdays: impl IntoIterator<Item = Weekday>) -> Result<Self, ScheduleError> {
        let mut weekdays: Vec<_> = weekdays.into_iter().collect();
        weekdays.sort_by_key(|weekday| weekday.num_days_from_monday());
        weekdays.dedup();
        if weekdays.is_empty() {
            Err(ScheduleError::EmptyWeekdays)
        } else {
            Ok(Self(weekdays))
        }
    }

    pub fn contains(&self, weekday: Weekday) -> bool {
        self.0.contains(&weekday)
    }

    pub fn iter(&self) -> impl Iterator<Item = Weekday> + '_ {
        self.0.iter().copied()
    }
}

impl TryFrom<Vec<Weekday>> for Weekdays {
    type Error = ScheduleError;

    fn try_from(value: Vec<Weekday>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// The repeating part of a schedule. The first occurrence is constrained by
/// [`Schedule::anchor`], which is stored separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Recurrence {
    Once,
    Daily {
        interval: NonZeroU16,
    },
    Weekly {
        interval: NonZeroU16,
        weekdays: Weekdays,
    },
    Monthly {
        interval: NonZeroU16,
        day: MonthDay,
    },
    Yearly {
        interval: NonZeroU16,
        month: MonthOfYear,
        day: MonthDay,
    },
}

impl Recurrence {
    pub fn once() -> Self {
        Self::Once
    }

    pub fn daily(interval: u16) -> Result<Self, ScheduleError> {
        Ok(Self::Daily {
            interval: nonzero(interval)?,
        })
    }

    pub fn weekly(
        interval: u16,
        weekdays: impl IntoIterator<Item = Weekday>,
    ) -> Result<Self, ScheduleError> {
        Ok(Self::Weekly {
            interval: nonzero(interval)?,
            weekdays: Weekdays::new(weekdays)?,
        })
    }

    pub fn monthly(interval: u16, day: u8) -> Result<Self, ScheduleError> {
        Ok(Self::Monthly {
            interval: nonzero(interval)?,
            day: MonthDay::day(day)?,
        })
    }

    pub fn monthly_last(interval: u16) -> Result<Self, ScheduleError> {
        Ok(Self::Monthly {
            interval: nonzero(interval)?,
            day: MonthDay::last(),
        })
    }

    pub fn yearly(interval: u16, month: u8, day: u8) -> Result<Self, ScheduleError> {
        Ok(Self::Yearly {
            interval: nonzero(interval)?,
            month: MonthOfYear::new(month)?,
            day: MonthDay::day(day)?,
        })
    }
}

fn nonzero(value: u16) -> Result<NonZeroU16, ScheduleError> {
    NonZeroU16::new(value).ok_or(ScheduleError::ZeroInterval)
}

/// A recurrence anchored at a concrete date.
///
/// All constructors go through validated building blocks, so a decoded
/// schedule is always safe to evaluate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    anchor: NaiveDate,
    recurrence: Recurrence,
}

impl Schedule {
    pub fn new(anchor: NaiveDate, recurrence: Recurrence) -> Self {
        Self { anchor, recurrence }
    }

    pub fn once(date: NaiveDate) -> Self {
        Self::new(date, Recurrence::once())
    }

    pub fn daily(anchor: NaiveDate, interval: u16) -> Result<Self, ScheduleError> {
        Ok(Self::new(anchor, Recurrence::daily(interval)?))
    }

    pub fn weekly(
        anchor: NaiveDate,
        interval: u16,
        weekdays: impl IntoIterator<Item = Weekday>,
    ) -> Result<Self, ScheduleError> {
        Ok(Self::new(anchor, Recurrence::weekly(interval, weekdays)?))
    }

    pub fn monthly(anchor: NaiveDate, interval: u16, day: u8) -> Result<Self, ScheduleError> {
        Ok(Self::new(anchor, Recurrence::monthly(interval, day)?))
    }

    pub fn monthly_last(anchor: NaiveDate, interval: u16) -> Result<Self, ScheduleError> {
        Ok(Self::new(anchor, Recurrence::monthly_last(interval)?))
    }

    pub fn yearly(
        anchor: NaiveDate,
        interval: u16,
        month: u8,
        day: u8,
    ) -> Result<Self, ScheduleError> {
        Ok(Self::new(anchor, Recurrence::yearly(interval, month, day)?))
    }

    pub fn anchor(&self) -> NaiveDate {
        self.anchor
    }

    pub fn recurrence(&self) -> &Recurrence {
        &self.recurrence
    }

    /// First occurrence on or after `from`.
    pub fn next_on_or_after(&self, from: NaiveDate) -> Option<NaiveDate> {
        match &self.recurrence {
            Recurrence::Once => (self.anchor >= from).then_some(self.anchor),
            Recurrence::Daily { interval } => self.next_daily(from, i64::from(interval.get())),
            Recurrence::Weekly { interval, weekdays } => {
                self.next_weekly(from, i64::from(interval.get()), weekdays)
            }
            Recurrence::Monthly { interval, day } => {
                self.next_monthly(from, i64::from(interval.get()), *day)
            }
            Recurrence::Yearly {
                interval,
                month,
                day,
            } => self.next_yearly(from, i64::from(interval.get()), month.number(), *day),
        }
    }

    /// First occurrence strictly after `after`.
    pub fn next_after(&self, after: NaiveDate) -> Option<NaiveDate> {
        self.next_on_or_after(after.checked_add_days(Days::new(1))?)
    }

    /// Materializes occurrences in the closed interval `[from, to]`.
    ///
    /// The result is capped so a wide range cannot allocate without bound.
    pub fn occurrences_between(&self, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        const LIMIT: usize = 512;
        if from > to {
            return Vec::new();
        }

        let mut occurrences = Vec::new();
        let mut cursor = from;
        while occurrences.len() < LIMIT {
            let Some(next) = self.next_on_or_after(cursor) else {
                break;
            };
            if next > to {
                break;
            }
            occurrences.push(next);
            let Some(after) = next.checked_add_days(Days::new(1)) else {
                break;
            };
            cursor = after;
        }
        occurrences
    }

    fn next_daily(&self, from: NaiveDate, interval: i64) -> Option<NaiveDate> {
        let base = from.max(self.anchor);
        let offset = (base - self.anchor).num_days();
        let steps = offset / interval + i64::from(offset % interval != 0);
        let days = steps.checked_mul(interval)?;
        self.anchor
            .checked_add_days(Days::new(u64::try_from(days).ok()?))
    }

    fn next_weekly(
        &self,
        from: NaiveDate,
        interval: i64,
        weekdays: &Weekdays,
    ) -> Option<NaiveDate> {
        let base = from.max(self.anchor);
        let anchor_monday = start_of_week(self.anchor)?;
        let mut best = None;

        for weekday in weekdays.iter() {
            let ahead = (i64::from(weekday.num_days_from_monday())
                - i64::from(base.weekday().num_days_from_monday()))
            .rem_euclid(7);
            let mut candidate = base.checked_add_days(Days::new(ahead as u64))?;
            let week = (start_of_week(candidate)? - anchor_monday).num_days() / 7;
            let remainder = week.rem_euclid(interval);
            if remainder != 0 {
                let days = (interval - remainder).checked_mul(7)?;
                candidate = candidate.checked_add_days(Days::new(u64::try_from(days).ok()?))?;
            }
            best = Some(best.map_or(candidate, |current: NaiveDate| current.min(candidate)));
        }

        best
    }

    fn next_monthly(&self, from: NaiveDate, interval: i64, day: MonthDay) -> Option<NaiveDate> {
        let base = from.max(self.anchor);
        let anchor_index = month_index(self.anchor.year(), self.anchor.month())?;
        let base_index = month_index(base.year(), base.month())?;
        let mut months = (base_index - anchor_index).max(0);
        let remainder = months.rem_euclid(interval);
        if remainder != 0 {
            months = months.checked_add(interval - remainder)?;
        }

        loop {
            let index = anchor_index.checked_add(months)?;
            let year = i32::try_from(index.div_euclid(12)).ok()?;
            let month = u32::try_from(index.rem_euclid(12) + 1).ok()?;
            let candidate = day.resolve(year, month)?;
            if candidate >= base {
                return Some(candidate);
            }
            months = months.checked_add(interval)?;
        }
    }

    fn next_yearly(
        &self,
        from: NaiveDate,
        interval: i64,
        month: u8,
        day: MonthDay,
    ) -> Option<NaiveDate> {
        let base = from.max(self.anchor);
        let mut years = i64::from(base.year()) - i64::from(self.anchor.year());
        let remainder = years.rem_euclid(interval);
        if remainder != 0 {
            years = years.checked_add(interval - remainder)?;
        }

        loop {
            let year = self.anchor.year().checked_add(i32::try_from(years).ok()?)?;
            let candidate = day.resolve(year, u32::from(month))?;
            if candidate >= base {
                return Some(candidate);
            }
            years = years.checked_add(interval)?;
        }
    }
}

fn month_index(year: i32, month: u32) -> Option<i64> {
    i64::from(year)
        .checked_mul(12)?
        .checked_add(i64::from(month) - 1)
}

fn start_of_week(date: NaiveDate) -> Option<NaiveDate> {
    date.checked_sub_days(Days::new(u64::from(date.weekday().num_days_from_monday())))
}

fn last_day_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    if !(1..=12).contains(&month) {
        return None;
    }
    let (next_year, next_month) = if month == 12 {
        (year.checked_add(1)?, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)?.pred_opt()
}

#[cfg(test)]
mod tests {
    use chrono::Weekday;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn daily_interval_respects_anchor() {
        let schedule = Schedule::daily(date(2026, 1, 1), 3).unwrap();
        assert_eq!(
            schedule.next_on_or_after(date(2026, 1, 1)),
            Some(date(2026, 1, 1))
        );
        assert_eq!(
            schedule.next_on_or_after(date(2026, 1, 2)),
            Some(date(2026, 1, 4))
        );
        assert_eq!(
            schedule.next_after(date(2026, 1, 4)),
            Some(date(2026, 1, 7))
        );
    }

    #[test]
    fn weekly_interval_and_selected_weekdays() {
        // Monday, 2 November 2026.
        let schedule =
            Schedule::weekly(date(2026, 11, 2), 2, [Weekday::Mon, Weekday::Wed]).unwrap();

        assert_eq!(
            schedule.next_on_or_after(date(2026, 11, 1)),
            Some(date(2026, 11, 2))
        );
        assert_eq!(
            schedule.next_after(date(2026, 11, 2)),
            Some(date(2026, 11, 4))
        );
        assert_eq!(
            schedule.next_after(date(2026, 11, 4)),
            Some(date(2026, 11, 16))
        );
        assert_eq!(
            schedule.next_after(date(2026, 11, 16)),
            Some(date(2026, 11, 18))
        );
    }

    #[test]
    fn weekly_anchor_does_not_emit_dates_before_anchor() {
        // Anchor is Wednesday; Monday of the same week is in the past.
        let schedule =
            Schedule::weekly(date(2026, 11, 4), 1, [Weekday::Mon, Weekday::Fri]).unwrap();

        assert_eq!(
            schedule.next_on_or_after(date(2026, 11, 1)),
            Some(date(2026, 11, 6))
        );
    }

    #[test]
    fn monthly_31_clamps_to_last_day_without_drifting() {
        let schedule = Schedule::monthly(date(2026, 1, 31), 1, 31).unwrap();
        assert_eq!(
            schedule.next_on_or_after(date(2026, 2, 1)),
            Some(date(2026, 2, 28))
        );
        assert_eq!(
            schedule.next_after(date(2026, 2, 28)),
            Some(date(2026, 3, 31))
        );
        assert_eq!(
            schedule.next_after(date(2026, 3, 31)),
            Some(date(2026, 4, 30))
        );
        assert_eq!(
            schedule.next_after(date(2026, 4, 30)),
            Some(date(2026, 5, 31))
        );
    }

    #[test]
    fn monthly_last_day() {
        let schedule = Schedule::monthly_last(date(2026, 1, 31), 1).unwrap();
        assert_eq!(
            schedule.next_on_or_after(date(2026, 2, 1)),
            Some(date(2026, 2, 28))
        );
        assert_eq!(
            schedule.next_after(date(2026, 2, 28)),
            Some(date(2026, 3, 31))
        );
        assert_eq!(
            schedule.next_after(date(2026, 3, 31)),
            Some(date(2026, 4, 30))
        );
    }

    #[test]
    fn monthly_first_occurrence_is_not_before_anchor() {
        let schedule = Schedule::monthly(date(2026, 11, 15), 1, 10).unwrap();
        assert_eq!(
            schedule.next_on_or_after(date(2026, 11, 1)),
            Some(date(2026, 12, 10))
        );
    }

    #[test]
    fn yearly_leap_day_clamps_in_common_years() {
        let schedule = Schedule::yearly(date(2024, 2, 29), 1, 2, 29).unwrap();
        assert_eq!(
            schedule.next_on_or_after(date(2025, 1, 1)),
            Some(date(2025, 2, 28))
        );
        assert_eq!(
            schedule.next_after(date(2025, 2, 28)),
            Some(date(2026, 2, 28))
        );
        assert_eq!(
            schedule.next_after(date(2026, 2, 28)),
            Some(date(2027, 2, 28))
        );
        assert_eq!(
            schedule.next_after(date(2027, 2, 28)),
            Some(date(2028, 2, 29))
        );
    }

    #[test]
    fn once_is_strict_after() {
        let schedule = Schedule::once(date(2026, 12, 1));
        assert_eq!(
            schedule.next_on_or_after(date(2026, 11, 30)),
            Some(date(2026, 12, 1))
        );
        assert_eq!(schedule.next_after(date(2026, 12, 1)), None);
        assert_eq!(schedule.next_on_or_after(date(2026, 12, 2)), None);
    }

    #[test]
    fn occurrence_ranges_are_bounded_and_sorted() {
        let schedule = Schedule::daily(date(2026, 1, 1), 1).unwrap();
        let occurrences = schedule.occurrences_between(date(2026, 1, 1), date(2028, 1, 1));
        assert_eq!(occurrences.len(), 512);
        assert!(occurrences.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn rejects_invalid_parts() {
        assert_eq!(
            Schedule::monthly(date(2026, 1, 1), 0, 1),
            Err(ScheduleError::ZeroInterval)
        );
        assert_eq!(
            Schedule::monthly(date(2026, 1, 1), 1, 0),
            Err(ScheduleError::InvalidDayOfMonth(0))
        );
        assert_eq!(
            Schedule::yearly(date(2026, 1, 1), 1, 13, 1),
            Err(ScheduleError::InvalidMonth(13))
        );
        assert_eq!(
            Schedule::weekly(date(2026, 1, 1), 1, []),
            Err(ScheduleError::EmptyWeekdays)
        );
    }

    #[test]
    fn json_round_trip() {
        let schedule =
            Schedule::weekly(date(2026, 11, 2), 2, [Weekday::Mon, Weekday::Fri]).unwrap();
        let json = serde_json::to_value(&schedule).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "anchor": "2026-11-02",
                "recurrence": {
                    "kind": "weekly",
                    "interval": 2,
                    "weekdays": ["Mon", "Fri"]
                }
            })
        );
        assert_eq!(serde_json::from_value::<Schedule>(json).unwrap(), schedule);
    }

    #[test]
    fn invalid_json_is_rejected() {
        let invalid = serde_json::json!({
            "anchor": "2026-11-02",
            "recurrence": { "kind": "weekly", "interval": 1, "weekdays": [] }
        });
        assert!(serde_json::from_value::<Schedule>(invalid).is_err());
    }
}

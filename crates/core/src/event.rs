use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Amount, Schedule};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Subscription,
    Bill,
    Meter,
    Task,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub notes: String,
    pub category: Category,
    pub amount: Option<Amount>,
    pub schedule: Schedule,
    pub timezone: Tz,
    pub remind_at: NaiveTime,
    pub lead_days: u16,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Event {
    pub fn new(
        title: impl Into<String>,
        category: Category,
        schedule: Schedule,
        timezone: Tz,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            title: title.into(),
            notes: String::new(),
            category,
            amount: None,
            schedule,
            timezone,
            remind_at: NaiveTime::from_hms_opt(9, 0, 0).expect("09:00 is a valid time"),
            lead_days: 3,
            active: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn next_on_or_after(&self, date: NaiveDate) -> Option<NaiveDate> {
        self.schedule.next_on_or_after(date)
    }

    pub fn next_after(&self, date: NaiveDate) -> Option<NaiveDate> {
        self.schedule.next_after(date)
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

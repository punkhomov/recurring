//! Pure domain layer shared by the desktop app and (later) the mobile client.
//!
//! It has no database, network, UI, or platform dependencies.

pub mod event;
pub mod money;
pub mod schedule;

pub use event::{Category, Event};
pub use money::{Amount, AmountError};
pub use schedule::{MonthDay, MonthOfYear, Recurrence, Schedule, ScheduleError, Weekdays};

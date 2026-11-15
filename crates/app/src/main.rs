mod app;
mod demo;
mod format;
mod theme;

use chrono::{Local, NaiveDate};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let screenshot = std::env::args().any(|argument| argument == "--screenshot");
    let today = std::env::var("RECURRING_TODAY")
        .ok()
        .and_then(|value| NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok())
        .unwrap_or_else(|| Local::now().date_naive());

    tracing::info!(%today, screenshot, "starting Recurring");
    app::run(app::Options { today, screenshot });
}

use chrono::{Datelike, NaiveDate};
use recurring_core::{Amount, Category, MonthDay, Recurrence, Schedule, Weekdays};

const MONTHS_GENITIVE: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];

const MONTHS_SHORT: [&str; 12] = [
    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];

const WEEKDAYS: [&str; 7] = [
    "понедельник",
    "вторник",
    "среда",
    "четверг",
    "пятница",
    "суббота",
    "воскресенье",
];

const WEEKDAYS_SHORT: [&str; 7] = ["пн", "вт", "ср", "чт", "пт", "сб", "вс"];

pub fn long_date(date: NaiveDate) -> String {
    format!(
        "{}, {} {}",
        WEEKDAYS[date.weekday().num_days_from_monday() as usize],
        date.day(),
        month_genitive(date.month())
    )
}

pub fn short_date(date: NaiveDate) -> String {
    format!("{} {}", date.day(), MONTHS_SHORT[date.month0() as usize])
}

pub fn due_label(today: NaiveDate, due: NaiveDate) -> String {
    let delta = (due - today).num_days();
    match delta {
        days if days < 0 => format!("Просрочено · {}", days_phrase(-days)),
        0 => "Сегодня".to_owned(),
        1 => "Завтра".to_owned(),
        2..=6 => format!("Через {}", days_phrase(delta)),
        _ => {
            if due.year() == today.year() {
                short_date(due)
            } else {
                format!("{} {}", short_date(due), due.year())
            }
        }
    }
}

pub fn schedule_label(schedule: &Schedule) -> String {
    match schedule.recurrence() {
        Recurrence::Once => format!("Один раз · {}", short_date(schedule.anchor())),
        Recurrence::Daily { interval } => {
            let interval = interval.get();
            if interval == 1 {
                "Каждый день".to_owned()
            } else {
                format!("Каждые {} {}", interval, plural_days(i64::from(interval)))
            }
        }
        Recurrence::Weekly { interval, weekdays } => {
            let interval = interval.get();
            let base = if interval == 1 {
                "Каждую неделю".to_owned()
            } else {
                format!("Каждые {} {}", interval, plural_weeks(interval))
            };
            let days = format_weekdays(weekdays);
            if days.is_empty() {
                base
            } else {
                format!("{base} · {days}")
            }
        }
        Recurrence::Monthly { interval, day } => {
            let interval = interval.get();
            let base = if interval == 1 {
                "Каждый месяц".to_owned()
            } else {
                format!("Каждые {} {}", interval, plural_months(interval))
            };
            format!("{base} · {}", day_label(*day))
        }
        Recurrence::Yearly {
            interval,
            month,
            day,
        } => {
            let interval = interval.get();
            let base = if interval == 1 {
                "Каждый год".to_owned()
            } else {
                format!("Каждые {} {}", interval, plural_years(interval))
            };
            let day_text = day
                .number()
                .map_or_else(|| "последний день".to_owned(), |day| day.to_string());
            format!(
                "{base} · {day_text} {}",
                month_genitive(u32::from(month.number()))
            )
        }
    }
}

pub fn amount(amount: &Amount) -> String {
    let major = group_digits(amount.major_units());
    let fraction = amount.fraction();
    let number = if fraction == 0 {
        major
    } else {
        format!("{major},{fraction:02}")
    };

    match amount.currency() {
        "RUB" => format!("{number} ₽"),
        "USD" => format!("${number}"),
        "EUR" => format!("€{number}"),
        currency => format!("{number} {currency}"),
    }
}

pub fn events_count(count: usize) -> String {
    format!(
        "{count} {}",
        plural(
            i64::try_from(count).unwrap_or(i64::MAX),
            "событие",
            "события",
            "событий"
        )
    )
}

pub fn category_label(category: Category) -> &'static str {
    match category {
        Category::Subscription => "Подписка",
        Category::Bill => "Платёж",
        Category::Meter => "Счётчик",
        Category::Task => "Задача",
        Category::Other => "Другое",
    }
}

fn format_weekdays(weekdays: &Weekdays) -> String {
    let days: Vec<_> = weekdays
        .iter()
        .map(|weekday| WEEKDAYS_SHORT[weekday.num_days_from_monday() as usize])
        .collect();
    if days.len() == 7 {
        String::new()
    } else {
        days.join(", ")
    }
}

fn day_label(day: MonthDay) -> String {
    if day.is_last() {
        "последний день".to_owned()
    } else {
        format!("{} числа", day.number().unwrap_or(1))
    }
}

fn month_genitive(month: u32) -> &'static str {
    MONTHS_GENITIVE[(month - 1) as usize]
}

fn days_phrase(days: i64) -> String {
    format!("{days} {}", plural_days(days))
}

fn plural_days(count: i64) -> &'static str {
    plural(count, "день", "дня", "дней")
}

fn plural_weeks(count: u16) -> &'static str {
    plural(i64::from(count), "неделя", "недели", "недель")
}

fn plural_months(count: u16) -> &'static str {
    plural(i64::from(count), "месяц", "месяца", "месяцев")
}

fn plural_years(count: u16) -> &'static str {
    plural(i64::from(count), "год", "года", "лет")
}

fn plural(count: i64, one: &'static str, few: &'static str, many: &'static str) -> &'static str {
    let count = count.rem_euclid(100);
    if (11..=14).contains(&count) {
        return many;
    }
    match count.rem_euclid(10) {
        1 => one,
        2..=4 => few,
        _ => many,
    }
}

fn group_digits(value: i64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push('\u{00A0}');
        }
        grouped.push(character);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use chrono::Weekday;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn formats_russian_day_plurals() {
        assert_eq!(days_phrase(1), "1 день");
        assert_eq!(days_phrase(2), "2 дня");
        assert_eq!(days_phrase(5), "5 дней");
        assert_eq!(days_phrase(11), "11 дней");
        assert_eq!(days_phrase(21), "21 день");
        assert_eq!(days_phrase(24), "24 дня");
    }

    #[test]
    fn formats_due_states() {
        let today = date(2026, 11, 15);
        assert_eq!(due_label(today, date(2026, 11, 14)), "Просрочено · 1 день");
        assert_eq!(due_label(today, today), "Сегодня");
        assert_eq!(due_label(today, date(2026, 11, 16)), "Завтра");
        assert_eq!(due_label(today, date(2026, 11, 18)), "Через 3 дня");
        assert_eq!(due_label(today, date(2026, 12, 1)), "1 дек");
        assert_eq!(due_label(today, date(2027, 3, 14)), "14 мар 2027");
    }

    #[test]
    fn formats_event_counts() {
        assert_eq!(events_count(1), "1 событие");
        assert_eq!(events_count(2), "2 события");
        assert_eq!(events_count(5), "5 событий");
        assert_eq!(events_count(21), "21 событие");
    }

    #[test]
    fn formats_amounts() {
        assert_eq!(amount(&Amount::rub(79_900)), "799 ₽");
        assert_eq!(amount(&Amount::rub(129_900)), "1\u{00A0}299 ₽");
        assert_eq!(amount(&Amount::new(5, "USD").unwrap()), "$0,05");
    }

    #[test]
    fn formats_schedules() {
        let anchor = date(2026, 1, 1);
        assert_eq!(
            schedule_label(&Schedule::monthly(anchor, 1, 15).unwrap()),
            "Каждый месяц · 15 числа"
        );
        assert_eq!(
            schedule_label(&Schedule::yearly(anchor, 1, 3, 14).unwrap()),
            "Каждый год · 14 марта"
        );
        assert_eq!(
            schedule_label(&Schedule::weekly(anchor, 2, [Weekday::Mon, Weekday::Fri]).unwrap()),
            "Каждые 2 недели · пн, пт"
        );
    }
}

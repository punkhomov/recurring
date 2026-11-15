use chrono::{Datelike, Days, Months, NaiveDate};
use chrono_tz::Europe::Moscow;
use recurring_core::{Amount, Category, Event, Schedule};
use uuid::Uuid;

pub struct Seed {
    pub event: Event,
    pub due_on: NaiveDate,
}

/// Local, deterministic sample data used until the SQLite layer lands.
///
/// Relative dates keep the Today/Upcoming views useful on any run. Screenshot
/// runs pin `RECURRING_TODAY` so the output is stable.
pub fn seeds(today: NaiveDate) -> Vec<Seed> {
    let yesterday = today.pred_opt().unwrap_or(today);
    let in_two_days = today.checked_add_days(Days::new(2)).unwrap_or(today);
    let in_five_days = today.checked_add_days(Days::new(5)).unwrap_or(today);

    let credit_card = monthly_event(
        1,
        "Кредитная карта",
        Category::Bill,
        None,
        yesterday,
        "Погасить полную задолженность до льготного периода.",
    );

    let netflix = monthly_event(
        2,
        "Netflix",
        Category::Subscription,
        Some(Amount::rub(79_900)),
        today,
        "",
    );

    let yandex_plus = monthly_event(
        3,
        "Яндекс Плюс",
        Category::Subscription,
        Some(Amount::rub(39_900)),
        in_two_days,
        "",
    );

    let meters = monthly_event(
        4,
        "Показания счётчиков",
        Category::Meter,
        None,
        in_five_days,
        "Электричество и вода: передать до 25-го числа.",
    );

    let rent_due = first_day_of_next_month(today);
    let rent_anchor = rent_due
        .checked_sub_months(Months::new(12))
        .unwrap_or(rent_due);
    let rent = event(
        5,
        "Аренда квартиры",
        Category::Bill,
        Some(Amount::rub(6_500_000)),
        Schedule::monthly(rent_anchor, 1, 1).expect("valid monthly schedule"),
        rent_due,
        "Перевод собственнику.",
    );

    let domain_schedule =
        Schedule::yearly(date(2020, 3, 14), 1, 3, 14).expect("valid yearly schedule");
    let domain_due = domain_schedule.next_on_or_after(today).unwrap_or(today);
    let domain = event(
        6,
        "Домен recurring.app",
        Category::Subscription,
        Some(Amount::rub(149_000)),
        domain_schedule,
        domain_due,
        "",
    );

    let filter_anchor = today.checked_sub_days(Days::new(70)).unwrap_or(today);
    let filter_schedule = Schedule::daily(filter_anchor, 90).expect("valid daily schedule");
    let filter_due = filter_schedule.next_on_or_after(today).unwrap_or(today);
    let filter = event(
        7,
        "Фильтр для воды",
        Category::Task,
        Some(Amount::rub(240_000)),
        filter_schedule,
        filter_due,
        "Заказать сменный картридж.",
    );

    [
        credit_card,
        netflix,
        yandex_plus,
        meters,
        rent,
        domain,
        filter,
    ]
    .into_iter()
    .collect()
}

fn monthly_event(
    id: u128,
    title: &str,
    category: Category,
    amount: Option<Amount>,
    due_on: NaiveDate,
    notes: &str,
) -> Seed {
    let anchor = due_on.checked_sub_months(Months::new(12)).unwrap_or(due_on);
    let day = u8::try_from(due_on.day()).expect("day of month fits in u8");
    let schedule = Schedule::monthly(anchor, 1, day).expect("valid monthly schedule");
    event(id, title, category, amount, schedule, due_on, notes)
}

fn event(
    id: u128,
    title: &str,
    category: Category,
    amount: Option<Amount>,
    schedule: Schedule,
    due_on: NaiveDate,
    notes: &str,
) -> Seed {
    let mut event = Event::new(title, category, schedule, Moscow);
    event.id = Uuid::from_u128(id);
    event.amount = amount;
    event.notes = notes.to_owned();
    event.lead_days = 3;

    Seed { event, due_on }
}

fn first_day_of_next_month(today: NaiveDate) -> NaiveDate {
    let next = today.checked_add_months(Months::new(1)).unwrap_or(today);
    date(next.year(), next.month(), 1)
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid demo date")
}

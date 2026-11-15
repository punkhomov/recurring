use chrono::{Days, NaiveDate};
use gpui::{
    AnyElement, App, Application, Bounds, Context, Div, ElementId, FontWeight, Render,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions, div, point, prelude::*, px,
    size,
};
use recurring_core::Event;
use uuid::Uuid;

use crate::{demo, format, theme};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub today: NaiveDate,
    pub screenshot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Today,
    Upcoming,
    All,
    Settings,
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Self::Today => "Сегодня",
            Self::Upcoming => "Ближайшие",
            Self::All => "Все события",
            Self::Settings => "Настройки",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Today => "Всё, что требует внимания",
            Self::Upcoming => "Следующие 30 дней",
            Self::All => "Полный список",
            Self::Settings => "Приложение и синхронизация",
        }
    }
}

#[derive(Clone)]
struct Item {
    event: Event,
    due_on: NaiveDate,
    done: bool,
}

pub struct RecurringApp {
    today: NaiveDate,
    section: Section,
    items: Vec<Item>,
}

impl RecurringApp {
    fn new(today: NaiveDate) -> Self {
        let items = demo::seeds(today)
            .into_iter()
            .map(|seed| Item {
                event: seed.event,
                due_on: seed.due_on,
                done: false,
            })
            .collect();

        Self {
            today,
            section: Section::Today,
            items,
        }
    }

    fn toggle_done(&mut self, id: Uuid, due_on: NaiveDate, cx: &mut Context<Self>) {
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.event.id == id && item.due_on == due_on)
        {
            item.done = !item.done;
        }
        cx.notify();
    }

    fn visible_items(&self) -> Vec<Item> {
        let horizon = self
            .today
            .checked_add_days(Days::new(30))
            .unwrap_or(self.today);

        let mut items: Vec<_> = self
            .items
            .iter()
            .filter(|item| match self.section {
                Section::Today => item.due_on <= self.today,
                Section::Upcoming => item.due_on > self.today && item.due_on <= horizon,
                Section::All => true,
                Section::Settings => false,
            })
            .cloned()
            .collect();

        items.sort_by(|left, right| {
            left.due_on
                .cmp(&right.due_on)
                .then_with(|| left.event.title.cmp(&right.event.title))
        });
        items
    }

    fn nav_item(&self, section: Section, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.section == section;
        let listener = cx.listener(move |this, _, _, cx| {
            this.section = section;
            cx.notify();
        });
        let dot_color = if selected {
            theme::accent()
        } else {
            theme::faint()
        };

        div()
            .id(ElementId::Name(format!("nav-{}", section.title()).into()))
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .when(selected, |element| {
                element.bg(theme::accent_soft()).text_color(theme::accent())
            })
            .when(!selected, |element| element.text_color(theme::muted()))
            .hover(|style| style.bg(theme::surface_hover()))
            .on_click(listener)
            .child(div().size(px(6.0)).rounded_full().bg(dot_color))
            .child(div().text_sm().child(section.title()))
            .into_any_element()
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> Div {
        let today_label = format::long_date(self.today);

        div()
            .flex()
            .flex_col()
            .w(px(236.0))
            .h_full()
            .flex_shrink_0()
            .bg(theme::sidebar())
            .border_r_1()
            .border_color(theme::border())
            .p_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_2()
                    .py_2()
                    .mb_2()
                    .child(
                        div()
                            .size(px(32.0))
                            .rounded_lg()
                            .bg(theme::accent())
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme::bg())
                            .font_weight(FontWeight::BOLD)
                            .child("R"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Recurring"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::muted())
                                    .child("подписки и платежи"),
                            ),
                    ),
            )
            .child(divider())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(self.nav_item(Section::Today, cx))
                    .child(self.nav_item(Section::Upcoming, cx))
                    .child(self.nav_item(Section::All, cx)),
            )
            .child(divider())
            .child(self.nav_item(Section::Settings, cx))
            .child(
                div().mt_auto().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_3()
                        .rounded_lg()
                        .bg(theme::surface())
                        .border_1()
                        .border_color(theme::border())
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::warning())
                                .child("Локальный режим"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::muted())
                                .child("Синхронизация ещё не подключена"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::faint())
                                .child(today_label),
                        ),
                ),
            )
    }

    fn header(&self) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(84.0))
            .px_8()
            .border_b_1()
            .border_color(theme::border())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.section.title()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::muted())
                            .child(self.section.subtitle()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_full()
                            .bg(theme::surface())
                            .border_1()
                            .border_color(theme::border())
                            .text_xs()
                            .text_color(theme::muted())
                            .child(format::long_date(self.today)),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_full()
                            .bg(theme::warning_soft())
                            .text_xs()
                            .text_color(theme::warning())
                            .child("Демо-данные"),
                    ),
            )
    }

    fn content(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.section == Section::Settings {
            return settings_view().into_any_element();
        }

        let items = self.visible_items();
        let item_count = items.len();
        let today = self.today;
        let horizon = today.checked_add_days(Days::new(7)).unwrap_or(today);
        let overdue = self
            .items
            .iter()
            .filter(|item| !item.done && item.due_on < today)
            .count();
        let due_today = self
            .items
            .iter()
            .filter(|item| !item.done && item.due_on == today)
            .count();
        let this_week = self
            .items
            .iter()
            .filter(|item| !item.done && item.due_on >= today && item.due_on <= horizon)
            .count();

        let mut rows = Vec::with_capacity(items.len());
        for item in items {
            rows.push(event_row(item, today, cx));
        }

        let list = if rows.is_empty() {
            empty_state()
        } else {
            div().flex().flex_col().gap_2().children(rows)
        };

        div()
            .id("content")
            .flex_1()
            .overflow_y_scroll()
            .p_8()
            .child(
                div()
                    .flex()
                    .gap_3()
                    .mb_6()
                    .child(stat_card("Просрочено", overdue, theme::danger()))
                    .child(stat_card("Сегодня", due_today, theme::accent()))
                    .child(stat_card("На неделе", this_week, theme::warning())),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::muted())
                            .child(format::events_count(item_count)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::faint())
                            .child("Локально · без синхронизации"),
                    ),
            )
            .child(list)
            .into_any_element()
    }
}

impl Render for RecurringApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme::bg())
            .text_color(theme::text())
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .child(self.header())
                    .child(self.content(cx)),
            )
    }
}

pub fn run(options: Options) {
    Application::new().run(move |cx: &mut App| {
        let bounds = if options.screenshot {
            Bounds::new(point(px(0.0), px(0.0)), size(px(1280.0), px(800.0)))
        } else {
            Bounds::centered(None, size(px(1180.0), px(760.0)), cx)
        };

        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("Recurring")),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                app_id: Some("dev.punkhomov.recurring".to_owned()),
                window_min_size: Some(size(px(900.0), px(600.0))),
                ..Default::default()
            },
            move |_window, cx| cx.new(|_| RecurringApp::new(options.today)),
        )
        .expect("failed to open the main window");

        cx.activate(true);
    });
}

fn stat_card(label: &'static str, value: usize, color: gpui::Rgba) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .p_4()
        .rounded_lg()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border())
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child(value.to_string()),
        )
        .child(div().text_sm().text_color(theme::muted()).child(label))
}

fn event_row(item: Item, today: NaiveDate, cx: &mut Context<RecurringApp>) -> Div {
    let id = item.event.id;
    let due_on = item.due_on;
    let done = item.done;
    let listener = cx.listener(move |this, _, _, cx| this.toggle_done(id, due_on, cx));

    let overdue = !done && item.due_on < today;
    let is_today = !done && item.due_on == today;
    let (due_color, due_background) = if done {
        (theme::success(), theme::surface_hover())
    } else if overdue {
        (theme::danger(), theme::danger_soft())
    } else if is_today {
        (theme::accent(), theme::accent_soft())
    } else {
        (theme::muted(), theme::surface_hover())
    };

    let due_text = if done {
        "Готово".to_owned()
    } else {
        format::due_label(today, item.due_on)
    };
    let title_color = if done { theme::faint() } else { theme::text() };
    let subtitle_color = if done { theme::faint() } else { theme::muted() };
    let amount = item
        .event
        .amount
        .as_ref()
        .map(format::amount)
        .unwrap_or_default();
    let mut meta = vec![
        format::category_label(item.event.category).to_owned(),
        format::schedule_label(&item.event.schedule),
    ];
    if !amount.is_empty() {
        meta.push(amount);
    }

    div()
        .flex()
        .items_center()
        .gap_4()
        .p_4()
        .rounded_lg()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border())
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(36.0))
                .flex_shrink_0()
                .rounded_lg()
                .bg(theme::surface_hover())
                .child(
                    div()
                        .size(px(10.0))
                        .rounded_full()
                        .bg(theme::category(item.event.category)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(title_color)
                        .child(item.event.title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(subtitle_color)
                        .child(meta.join("  ·  ")),
                ),
        )
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_md()
                .bg(due_background)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(due_color)
                .child(due_text),
        )
        .child(
            div()
                .id(ElementId::Name(
                    format!("toggle-{}-{}", item.event.id, item.due_on).into(),
                ))
                .px_3()
                .py_1p5()
                .rounded_md()
                .border_1()
                .border_color(if done {
                    theme::success()
                } else {
                    theme::border()
                })
                .text_xs()
                .text_color(if done {
                    theme::success()
                } else {
                    theme::muted()
                })
                .cursor_pointer()
                .hover(|style| style.bg(theme::surface_hover()))
                .on_click(listener)
                .child(if done {
                    "Вернуть"
                } else {
                    "Отметить"
                }),
        )
}

fn empty_state() -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .h(px(220.0))
        .rounded_lg()
        .border_1()
        .border_color(theme::border())
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Здесь пока пусто"),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme::muted())
                .child("Новые события появятся в этом списке."),
        )
}

fn settings_view() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::muted())
                .child("ТЕКУЩАЯ КОНФИГУРАЦИЯ"),
        )
        .child(setting_row("Хранилище", "SQLite · запланировано"))
        .child(setting_row("Облако", "Supabase · не подключено"))
        .child(setting_row("Уведомления", "Не подключены"))
        .child(
            div()
                .mt_2()
                .p_4()
                .rounded_lg()
                .bg(theme::surface())
                .border_1()
                .border_color(theme::border())
                .text_sm()
                .text_color(theme::muted())
                .child(
                    "Следующий шаг: локальная база, вход через Supabase и фоновая синхронизация. \
                     Схема облака уже лежит в supabase/migrations.",
                ),
        )
}

fn setting_row(label: &'static str, value: &'static str) -> Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_3()
        .rounded_lg()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border())
        .child(div().text_sm().child(label))
        .child(div().text_sm().text_color(theme::muted()).child(value))
}

fn divider() -> Div {
    div().h(px(1.0)).bg(theme::border()).my_4()
}

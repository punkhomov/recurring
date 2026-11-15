use gpui::{Rgba, rgb};
use recurring_core::Category;

pub fn bg() -> Rgba {
    rgb(0x0B0E14)
}

pub fn sidebar() -> Rgba {
    rgb(0x0F131A)
}

pub fn surface() -> Rgba {
    rgb(0x151A23)
}

pub fn surface_hover() -> Rgba {
    rgb(0x1B2230)
}

pub fn border() -> Rgba {
    rgb(0x232C3A)
}

pub fn text() -> Rgba {
    rgb(0xE7EDF7)
}

pub fn muted() -> Rgba {
    rgb(0x8894A7)
}

pub fn faint() -> Rgba {
    rgb(0x5C697C)
}

pub fn accent() -> Rgba {
    rgb(0x7AA2F7)
}

pub fn accent_soft() -> Rgba {
    rgb(0x1B2740)
}

pub fn danger() -> Rgba {
    rgb(0xF7768E)
}

pub fn danger_soft() -> Rgba {
    rgb(0x2A1820)
}

pub fn warning() -> Rgba {
    rgb(0xE0AF68)
}

pub fn warning_soft() -> Rgba {
    rgb(0x2A2418)
}

pub fn success() -> Rgba {
    rgb(0x9ECE6A)
}

pub fn purple() -> Rgba {
    rgb(0xBB9AF7)
}

pub fn teal() -> Rgba {
    rgb(0x7DCFFF)
}

pub fn category(category: Category) -> Rgba {
    match category {
        Category::Subscription => accent(),
        Category::Bill => warning(),
        Category::Meter => teal(),
        Category::Task => purple(),
        Category::Other => muted(),
    }
}

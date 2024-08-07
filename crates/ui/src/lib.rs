mod clickable;
mod colors;
mod disableable;
mod event;
mod focusable;
mod icon;

mod selectable;
mod stack;
mod styled_ext;
mod svg_img;
mod time;

pub mod button;
pub mod checkbox;
pub mod clipboard;
pub mod context_menu;
pub mod divider;
pub mod dropdown;
pub mod indicator;
pub mod input;
pub mod label;
pub mod link;
pub mod list;
pub mod popover;
pub mod popup_menu;
pub mod prelude;
pub mod progress;
pub mod radio;
pub mod resizable;
pub mod scroll;
pub mod slider;
pub mod switch;
pub mod tab;
pub mod table;
pub mod theme;
pub mod tooltip;
pub mod webview;

use std::ops::Deref;

// re-export
pub use wry;

pub use clickable::Clickable;
pub use disableable::Disableable;
pub use event::InteractiveElementExt;
pub use focusable::FocusableCycle;
pub use selectable::{Selectable, Selection};
pub use styled_ext::{Sizable, Size, StyledExt};
pub use time::*;

pub use colors::*;
pub use icon::*;
pub use stack::*;
pub use svg_img::*;

rust_i18n::i18n!("locales", fallback = "en");

/// Initialize the UI module.
pub fn init(cx: &mut gpui::AppContext) {
    input::init(cx);
    list::init(cx);
    dropdown::init(cx);
    popover::init(cx);
    popup_menu::init(cx);
    context_menu::init(cx);
    table::init(cx);
    webview::init(cx)
}

pub fn locale() -> impl Deref<Target = str> {
    rust_i18n::locale()
}

pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale)
}

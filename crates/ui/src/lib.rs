mod clickable;
mod disableable;
mod event;
mod icon;
mod platform;
mod prelude;
mod scrollbar;
mod selectable;
mod stock;
mod styled_ext;

pub mod button;
pub mod checkbox;
pub mod empty;
pub mod label;
pub mod theme;
pub mod title_bar;
pub use styled_ext::StyledExt;
pub mod divider;
pub mod dock;
pub mod dropdown;
pub mod input;
pub mod list;
pub mod picker;
pub mod popover;
pub mod switch;
pub mod tab;
pub mod tooltip;

pub use clickable::Clickable;
pub use disableable::Disableable;
pub use event::InterativeElementExt;
pub use selectable::{Selectable, Selection};

pub use icon::*;
pub use stock::*;

/// Initialize the UI module.
pub fn init(cx: &mut gpui::AppContext) {
    input::init(cx);
    picker::init(cx);
    dropdown::init(cx);
    popover::init(cx);
}

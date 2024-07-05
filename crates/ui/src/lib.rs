mod clickable;
mod disableable;
mod event;
mod focusable;
mod icon;
mod scrollbar;
mod selectable;
mod stock;
mod styled_ext;
mod svg_img;

pub mod button;
pub mod checkbox;
pub mod empty;
pub mod label;
pub mod prelude;
pub mod theme;
pub use styled_ext::StyledExt;
pub mod divider;
pub mod dropdown;
pub mod input;
pub mod list;
pub mod picker;
pub mod popover;
pub mod popup_menu;
pub mod switch;
pub mod tab;
pub mod table;
pub mod tooltip;

pub use clickable::Clickable;
pub use disableable::Disableable;
pub use event::InterativeElementExt;
pub use focusable::FocusableCycle;
pub use selectable::{Selectable, Selection};

pub use icon::*;
pub use stock::*;
pub use svg_img::*;

/// Initialize the UI module.
pub fn init(cx: &mut gpui::AppContext) {
    input::init(cx);
    picker::init(cx);
    dropdown::init(cx);
    popover::init(cx);
    popup_menu::init(cx);
    table::init(cx);
}

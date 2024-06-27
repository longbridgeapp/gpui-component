mod clickable;
mod disableable;
mod event;
mod icon;
mod platform;
mod prelude;
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
// pub mod dropdown;
pub mod input;
pub mod list;
pub mod picker;
pub mod switch;
pub mod tab;

pub use clickable::Clickable;
pub use disableable::Disableable;
pub use selectable::*;

pub use icon::*;
pub use stock::*;

/// Initialize the UI module.
pub fn init(cx: &mut gpui::AppContext) {
    input::init(cx);
    picker::init(cx);
}

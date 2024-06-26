use crate::StyledExt as _;
use gpui::{div, Div, Styled};

/// Horizontally stacks elements. Sets `flex()`, `flex_row()`, `items_center()`
#[track_caller]
pub fn h_flex() -> Div {
    div().h_flex()
}

/// Vertically stacks elements. Sets `flex()`, `flex_col()`
#[track_caller]
pub fn v_flex() -> Div {
    div().v_flex()
}

/// A horizontal divider. Sets `h_0.5()`, `bg()`, `bg_gray()`
pub fn span() -> Div {
    div().w_auto()
}

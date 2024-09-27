use gpui::{px, WindowContext};

use crate::{
    button::{Button, ButtonStyled as _},
    IconName, Sizable as _,
};

pub(crate) struct ClearButton {}

impl ClearButton {
    pub fn new(_: &mut WindowContext) -> Button {
        Button::new("clean")
            .icon(IconName::CircleX)
            .ghost()
            .with_size(px(14.))
    }
}

use gpui::{px, Styled as _, WindowContext};

use crate::{button::Button, IconName, Sizable as _};

pub(crate) struct ClearButton {}

impl ClearButton {
    pub fn new(cx: &mut WindowContext) -> Button {
        Button::new("clean", cx)
            .icon(IconName::CircleX)
            .ghost()
            .with_size(px(14.))
            .cursor_pointer()
    }
}

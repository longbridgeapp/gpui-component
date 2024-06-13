use gpui::{ClickEvent, WindowContext};

pub trait Disableable {
    fn disabled(self, disabled: bool) -> Self;
}

pub trait Selectable {
    fn selected(self, selected: bool) -> Self;
}

pub trait Clickable {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self;
}

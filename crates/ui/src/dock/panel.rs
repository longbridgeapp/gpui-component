use gpui::{AnyView, EventEmitter, FocusableView, SharedString, View, WindowContext};
use rust_i18n::t;

use super::PanelEvent;

pub trait Panel: EventEmitter<PanelEvent> + FocusableView {
    /// The title of the panel, default is `None`.
    fn title(&self, _cx: &WindowContext) -> SharedString {
        t!("Dock.Unnamed").into()
    }

    /// Whether the panel can be closed, default is `true`.
    fn closeable(&self, _cx: &WindowContext) -> bool {
        true
    }
}

pub trait PanelView: 'static + Send + Sync {
    /// The title of the panel, default is `None`.
    fn title(&self, _cx: &WindowContext) -> SharedString {
        t!("Dock.Unnamed").into()
    }

    fn view(&self) -> AnyView;
}

impl<T: Panel> PanelView for View<T> {
    fn title(&self, cx: &WindowContext) -> SharedString {
        self.read(cx).title(cx)
    }

    fn view(&self) -> AnyView {
        self.clone().into()
    }
}

impl From<&dyn PanelView> for AnyView {
    fn from(handle: &dyn PanelView) -> Self {
        handle.view()
    }
}

impl<T: Panel> From<&dyn PanelView> for View<T> {
    fn from(value: &dyn PanelView) -> Self {
        value.view().downcast::<T>().unwrap()
    }
}

impl PartialEq for dyn PanelView {
    fn eq(&self, other: &Self) -> bool {
        self.view() == other.view()
    }
}

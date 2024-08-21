use gpui::{px, AnyView, EntityId, FocusableView, Pixels, SharedString, View, WindowContext};

use crate::Placement;

pub trait Panel: FocusableView {
    /// The title of the panel, default is `None`.
    fn title(&self, cx: &WindowContext) -> SharedString {
        "Unnamed".into()
    }
    /// The size of the panel, default is `50px`.
    fn size(&self, cx: &WindowContext) -> Pixels {
        px(50.)
    }
    /// Called when the size of the panel is changed.
    fn set_size(&mut self, size: Pixels, cx: &mut WindowContext) {}

    /// Whether the panel can be closed, default is `true`.
    fn closeable(&self, cx: &WindowContext) -> bool {
        true
    }
}

pub trait PanelView: Send + Sync {
    /// The title of the panel, default is `None`.
    fn title(&self, cx: &WindowContext) -> SharedString {
        "Unnamed".into()
    }
    /// The size of the panel, default is `50px`.
    fn size(&self, cx: &WindowContext) -> Pixels {
        px(50.)
    }
    /// Called when the size of the panel is changed.
    fn set_size(&mut self, size: Pixels, cx: &mut WindowContext);

    fn view(&self) -> AnyView;
}

impl<T: Panel> PanelView for View<T> {
    fn title(&self, cx: &WindowContext) -> SharedString {
        self.read(cx).title(cx)
    }

    fn size(&self, cx: &WindowContext) -> Pixels {
        self.read(cx).size(cx)
    }

    fn set_size(&mut self, size: Pixels, cx: &mut WindowContext) {
        self.update(cx, |view, cx| {
            view.set_size(size, cx);
        })
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

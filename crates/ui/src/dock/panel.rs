use gpui::{
    px, AnyView, FocusableView, Pixels, Render, SharedString, View, ViewContext, WindowContext,
};

use crate::Placement;

pub trait Panel: FocusableView {
    /// The title of the panel, default is `None`.
    fn title(&self, cx: &WindowContext) -> Option<SharedString> {
        None
    }
    /// The size of the panel, default is `50px`.
    fn size(&self, cx: &WindowContext) -> Pixels {
        px(50.)
    }
    /// Called when the size of the panel is changed.
    fn set_size(&mut self, size: Pixels, cx: &mut WindowContext);
    /// The placement of the panel, default is `Placement::Left`.
    fn placement(&self, cx: &WindowContext) -> Placement {
        Placement::Left
    }
    /// Called when the placement of the panel is changed.
    fn set_placement(&mut self, placement: Placement, cx: &mut WindowContext);
}

pub(crate) trait PanelView: Send + Sync {
    /// The title of the panel, default is `None`.
    fn title(&self, cx: &WindowContext) -> Option<SharedString> {
        None
    }
    /// The size of the panel, default is `50px`.
    fn size(&self, cx: &WindowContext) -> Pixels {
        px(50.)
    }
    /// Called when the size of the panel is changed.
    fn set_size(&mut self, size: Pixels, cx: &mut WindowContext);
    /// The placement of the panel, default is `Placement::Left`.
    fn placement(&self, cx: &WindowContext) -> Placement {
        Placement::Left
    }
    /// Called when the placement of the panel is changed.
    fn set_placement(&mut self, placement: Placement, cx: &mut WindowContext);

    fn into_any(&self) -> AnyView;
}

impl<T: Panel> PanelView for View<T> {
    fn title(&self, cx: &WindowContext) -> Option<SharedString> {
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

    fn set_placement(&mut self, placement: Placement, cx: &mut WindowContext) {
        self.update(cx, |view, cx| {
            view.set_placement(placement, cx);
        })
    }

    fn placement(&self, cx: &WindowContext) -> Placement {
        self.read(cx).placement(cx)
    }

    fn into_any(&self) -> AnyView {
        self.clone().into()
    }
}

impl From<&dyn PanelView> for AnyView {
    fn from(handle: &dyn PanelView) -> Self {
        handle.into_any()
    }
}

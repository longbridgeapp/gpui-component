use std::time::Duration;

use gpui::{
    div, Animation, AnimationExt, Div, Element, InteractiveElement, Interactivity, IntoElement,
    ParentElement as _, RenderOnce, Stateful, Styled,
};

pub struct Skeleton {
    base: Stateful<Div>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            base: div().id("skeleton").w_full().h_4(),
        }
    }
}

impl Styled for Skeleton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl IntoElement for Skeleton {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Skeleton {
    type RequestLayoutState = ();

    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let (layout_id, _) = self.base.request_layout(id, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        self.base.prepaint(id, bounds, request_layout, cx);
        ()
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        todo!()
    }
}

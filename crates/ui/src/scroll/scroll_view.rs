use std::{cell::Cell, rc::Rc};

use gpui::{
    canvas, div, relative, AnyElement, AnyView, Element, ElementId, GlobalElementId,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Position, ScrollHandle,
    SharedString, Size, StatefulInteractiveElement, Style, Styled as _, WindowContext,
};

use crate::StyledExt;

use super::{Scrollbar, ScrollbarAxis, ScrollbarState};

pub fn scroll_view(id: impl Into<ElementId>, view: impl Into<AnyView>) -> ScrollView {
    ScrollView::new(id, view, ScrollbarAxis::Both)
}

/// A scroll view is a container that allows the user to scroll through a large amount of content.
pub struct ScrollView {
    id: ElementId,
    view: AnyView,
    axix: ScrollbarAxis,
    content: Option<Box<dyn Fn(&mut WindowContext) -> AnyElement + 'static>>,
}

impl ScrollView {
    fn new(id: impl Into<ElementId>, view: impl Into<AnyView>, axis: ScrollbarAxis) -> Self {
        let view: AnyView = view.into();
        Self {
            id: ElementId::Name(SharedString::from(format!(
                "{}-{}",
                view.entity_id(),
                id.into()
            ))),
            view,
            axix: axis,
            content: None,
        }
    }

    /// Set only a vertical scrollbar.
    pub fn vertical(mut self) -> Self {
        self.set_axis(ScrollbarAxis::Vertical);
        self
    }

    /// Set only a horizontal scrollbar.
    pub fn horizontal(mut self) -> Self {
        self.set_axis(ScrollbarAxis::Horizontal);
        self
    }

    /// Set the content render of the scroll view.
    #[must_use]
    pub fn content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut WindowContext) -> E + 'static,
        E: IntoElement,
    {
        self.content = Some(Box::new(move |cx| builder(cx).into_any_element()));
        self
    }

    /// Set the axis of the scroll view.
    pub fn set_axis(&mut self, axis: ScrollbarAxis) {
        self.axix = axis;
    }

    fn with_element_state<R>(
        &mut self,
        id: &GlobalElementId,
        cx: &mut WindowContext,
        f: impl FnOnce(&mut Self, &mut ScrollViewState, &mut WindowContext) -> R,
    ) -> R {
        cx.with_optional_element_state::<ScrollViewState, _>(Some(id), |element_state, cx| {
            let mut element_state = element_state.unwrap().unwrap_or_default();
            let result = f(self, &mut element_state, cx);
            (result, Some(element_state))
        })
    }
}

pub struct ScrollViewState {
    scroll_size: Rc<Cell<Size<Pixels>>>,
    state: Rc<Cell<ScrollbarState>>,
    handle: ScrollHandle,
}

impl Default for ScrollViewState {
    fn default() -> Self {
        Self {
            handle: ScrollHandle::new(),
            scroll_size: Rc::new(Cell::new(Size::default())),
            state: Rc::new(Cell::new(ScrollbarState::default())),
        }
    }
}

impl IntoElement for ScrollView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ScrollView {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ScrollViewState;

    fn id(&self) -> Option<gpui::ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 1.0;
        style.position = Position::Relative;
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();

        let axix = self.axix;
        let view = self.view.clone();

        let scroll_id = self.id.clone();
        let content = self.content.as_ref().map(|c| c(cx));

        self.with_element_state(id.unwrap(), cx, |_, element_state, cx| {
            let handle = element_state.handle.clone();
            let state = element_state.state.clone();
            let scroll_size = element_state.scroll_size.clone();

            let mut element = div()
                .relative()
                .size_full()
                .child(
                    div()
                        .id(scroll_id)
                        .track_scroll(&handle)
                        .overflow_scroll()
                        .relative()
                        .size_full()
                        .child(div().debug_pink().children(content).child({
                            let scroll_size = element_state.scroll_size.clone();
                            canvas(move |b, _| scroll_size.set(b.size), |_, _, _| {})
                                .absolute()
                                .size_full()
                        })),
                )
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .child(
                            Scrollbar::both(view, state, handle.clone(), scroll_size.get())
                                .axis(axix),
                        ),
                )
                .into_any_element();
            let element_id = element.request_layout(cx);

            let layout_id = cx.request_layout(style, vec![element_id]);

            (layout_id, element)
        })
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        element.prepaint(cx);
        // do nothing
        ScrollViewState::default()
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        element.paint(cx)
    }
}

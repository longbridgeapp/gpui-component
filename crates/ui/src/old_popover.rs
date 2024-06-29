use std::{cell::RefCell, rc::Rc};

use crate::theme::ActiveTheme;
use crate::{Clickable, Selectable, StyledExt};
use gpui::{
    actions, anchored, deferred, div, prelude::FluentBuilder as _, AnchorCorner, AnyElement,
    AppContext, Bounds, DismissEvent, Element, ElementId, EventEmitter, FocusHandle, FocusableView,
    HitboxId, InteractiveElement, IntoElement, LayoutId, ManagedView, ParentElement, Pixels,
    Render, Style, Styled, View, ViewContext, VisualContext, WindowContext,
};
use gpui::{point, px, DispatchPhase, MouseDownEvent, Point};

actions!(popover, [Dismiss]);

pub trait Triggerable: IntoElement + Clickable + Selectable + 'static {}
impl<T: IntoElement + Clickable + Selectable + 'static> Triggerable for T {}

pub fn init(_cx: &mut AppContext) {}

pub struct PopoverContent {
    focus_handle: FocusHandle,
    content: Rc<dyn Fn(&mut WindowContext) -> AnyElement>,
}

impl PopoverContent {
    pub fn new<B>(content: B, cx: &mut WindowContext) -> View<Self>
    where
        B: Fn(&mut WindowContext) -> AnyElement + 'static,
    {
        cx.new_view(|cx| {
            let focus_handle = cx.focus_handle();

            Self {
                focus_handle,
                content: Rc::new(content),
            }
        })
    }
}
impl EventEmitter<DismissEvent> for PopoverContent {}

impl FocusableView for PopoverContent {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopoverContent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("test")
            // .on_action(cx.listener(Self::dissmiss))
            .absolute()
            .mt_2()
            .elevation_2(cx)
            .bg(cx.theme().popover)
            .border_1()
            .border_color(cx.theme().border)
            .p_4()
            .max_w_128()
            .w_80()
            .occlude()
            .child(self.content.clone()(cx))
            .on_mouse_down_out(cx.listener(|_, _, cx| cx.emit(DismissEvent)))
    }
}

pub struct Popover<M: ManagedView> {
    id: ElementId,
    anchor: AnchorCorner,
    trigger_builder: Option<
        Box<
            dyn FnOnce(
                    Rc<RefCell<Option<View<M>>>>,
                    Option<Rc<dyn Fn(&mut WindowContext) -> View<M> + 'static>>,
                ) -> AnyElement
                + 'static,
        >,
    >,
    content_builder: Option<Rc<dyn Fn(&mut WindowContext) -> View<M> + 'static>>,
}

impl<M: ManagedView> Popover<M> {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger_builder: None,
            content_builder: None,
            anchor: AnchorCorner::TopLeft,
        }
    }

    pub fn trigger<T: Triggerable>(mut self, trigger: T) -> Self {
        self.trigger_builder = Some(Box::new(|popover, builder| {
            let open = popover.borrow_mut().is_some();
            trigger
                .selected(open)
                .when_some(builder, |this, builder| {
                    this.on_click(move |_, cx| Self::show_popover(&builder, &popover, cx))
                })
                .into_any_element()
        }));

        self
    }

    pub fn content(
        mut self,
        content_builder: impl Fn(&mut WindowContext) -> View<M> + 'static,
    ) -> Self {
        self.content_builder = Some(Rc::new(content_builder));
        self
    }

    /// anchor defines which corner of the menu to anchor to the attachment point
    /// (by default the cursor position, but see attach)
    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor = anchor;
        self
    }

    fn show_popover(
        builder: &Rc<dyn Fn(&mut WindowContext) -> View<M> + 'static>,
        popover: &Rc<RefCell<Option<View<M>>>>,
        cx: &mut WindowContext,
    ) {
        let new_popover = (builder)(cx);

        let popover_cloned = popover.clone();
        let prev_focus_handle = cx.focused();

        cx.subscribe(&new_popover, move |modal, _: &DismissEvent, cx| {
            if modal.focus_handle(cx).contains_focused(cx) {
                if let Some(prev_focus_handle) = prev_focus_handle.as_ref() {
                    cx.focus(prev_focus_handle);
                }
            }
            *popover_cloned.borrow_mut() = None;
            cx.refresh();
        })
        .detach();

        cx.focus_view(&new_popover);
        *popover.borrow_mut() = Some(new_popover);
        cx.refresh();
    }

    fn resolved_corner(&self, bounds: Bounds<Pixels>) -> Point<Pixels> {
        match self.anchor {
            AnchorCorner::TopLeft => AnchorCorner::BottomLeft,
            AnchorCorner::TopRight => AnchorCorner::BottomRight,
            AnchorCorner::BottomLeft => AnchorCorner::TopLeft,
            AnchorCorner::BottomRight => AnchorCorner::TopRight,
        }
        .corner(bounds)
    }

    fn resolved_offset(&self, _cx: &WindowContext) -> Point<Pixels> {
        let offset = px(0.);
        match self.anchor {
            AnchorCorner::TopRight | AnchorCorner::BottomRight => point(offset, px(0.)),
            AnchorCorner::TopLeft | AnchorCorner::BottomLeft => point(-offset, px(0.)),
        }
    }
}

pub struct PopoverElementState<M> {
    trigger_bounds: Option<Bounds<Pixels>>,
    popover: Rc<RefCell<Option<View<M>>>>,
}

impl<M> Clone for PopoverElementState<M> {
    fn clone(&self) -> Self {
        Self {
            popover: Rc::clone(&self.popover),
            trigger_bounds: self.trigger_bounds,
        }
    }
}

impl<M> Default for PopoverElementState<M> {
    fn default() -> Self {
        Self {
            popover: Rc::default(),
            trigger_bounds: None,
        }
    }
}

pub struct PopoverFrameState {
    trigger_layout_id: Option<LayoutId>,
    trigger_element: Option<AnyElement>,
    popover_element: Option<AnyElement>,
}

impl<M: ManagedView> IntoElement for Popover<M> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<M: ManagedView> Element for Popover<M> {
    type RequestLayoutState = PopoverFrameState;
    type PrepaintState = Option<HitboxId>;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        cx.with_element_state(
            global_id.unwrap(),
            |element_state: Option<PopoverElementState<M>>, cx| {
                let element_state = element_state.unwrap_or_default();

                let mut popover_layout_id = None;
                let popover_element = element_state.popover.borrow_mut().as_mut().map(|popover| {
                    let mut anchored = anchored().snap_to_window().anchor(self.anchor);
                    if let Some(trigger_bounds) = element_state.trigger_bounds {
                        anchored = anchored.position(
                            self.resolved_corner(trigger_bounds) + self.resolved_offset(cx),
                        );
                    }
                    let mut element = deferred(anchored.child(popover.clone()))
                        .with_priority(1)
                        .into_any();

                    popover_layout_id = Some(element.request_layout(cx));
                    element
                });

                let mut trigger_element = self.trigger_builder.take().map(|builder| {
                    builder(element_state.popover.clone(), self.content_builder.clone())
                });

                let trigger_layout_id = trigger_element
                    .as_mut()
                    .map(|trigger_element| trigger_element.request_layout(cx));

                let layout_id = cx.request_layout(
                    Style::default(),
                    popover_layout_id.into_iter().chain(trigger_layout_id),
                );

                (
                    (
                        layout_id,
                        PopoverFrameState {
                            trigger_layout_id,
                            trigger_element,
                            popover_element,
                        },
                    ),
                    element_state,
                )
            },
        )
    }

    fn prepaint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        if let Some(element) = &mut request_layout.trigger_element {
            element.prepaint(cx);
        }

        if let Some(element) = &mut request_layout.popover_element {
            element.prepaint(cx);
        }

        request_layout.trigger_layout_id.map(|layout_id| {
            let bounds = cx.layout_bounds(layout_id);
            cx.with_element_state(global_id.unwrap(), |element_state, _cx| {
                let mut element_state: PopoverElementState<M> = element_state.unwrap();
                element_state.trigger_bounds = Some(bounds);
                ((), element_state)
            });

            cx.insert_hitbox(bounds, false).id
        })
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        trigger_hitbox: &mut Option<HitboxId>,
        cx: &mut WindowContext,
    ) {
        if let Some(element) = &mut request_layout.trigger_element {
            element.paint(cx);
        }

        if let Some(element) = &mut request_layout.popover_element {
            element.paint(cx);

            if let Some(hitbox) = *trigger_hitbox {
                // Mouse-downing outside the menu dismisses it, so we don't
                // want a click on the toggle to re-open it.
                cx.on_mouse_event(move |_: &MouseDownEvent, phase, cx| {
                    if phase == DispatchPhase::Bubble && hitbox.is_hovered(cx) {
                        cx.stop_propagation()
                    }
                })
            }
        }
    }
}

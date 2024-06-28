use std::borrow::Borrow;
use std::{alloc::Layout, cell::RefCell, rc::Rc};

use crate::theme::ActiveTheme;
use crate::{Clickable, Selectable, StyledExt};
use gpui::{
    actions, anchored, deferred, div, prelude::FluentBuilder as _, AnchorCorner, AnyElement,
    AnyView, AppContext, Bounds, DismissEvent, Div, Element, ElementId, EventEmitter, FocusHandle,
    FocusableView, HitboxId, InteractiveElement, IntoElement, KeyBinding, LayoutId, ManagedView,
    MouseButton, ParentElement, Pixels, Render, RenderOnce, StatefulInteractiveElement as _, Style,
    StyleRefinement, Styled, View, ViewContext, VisualContext, WindowContext,
};
use gpui::{px, SharedString};

actions!(popover, [Dismiss]);

pub trait Triggerable: IntoElement + Clickable + Selectable + 'static {}
impl<T: IntoElement + Clickable + Selectable + 'static> Triggerable for T {}

pub fn init(_cx: &mut AppContext) {}

pub struct PopoverContent {
    focus_handle: FocusHandle,
    content: SharedString,
}

impl PopoverContent {
    pub fn new(content: impl Into<SharedString>, cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let focus_handle = cx.focus_handle();

            Self {
                focus_handle,
                content: content.into(),
            }
        })
    }
}
impl EventEmitter<DismissEvent> for PopoverContent {}

impl FocusableView for PopoverContent {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
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
            .child(self.content.clone())
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
    open: bool,
}

impl<M: ManagedView> Popover<M> {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger_builder: None,
            content_builder: None,
            anchor: AnchorCorner::BottomRight,
            open: false,
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
        let previous_focus_handle = cx.focused();

        cx.subscribe(&new_popover, move |modal, _: &DismissEvent, cx| {
            if modal.focus_handle(cx).contains_focused(cx) {
                if let Some(previous_focus_handle) = previous_focus_handle.as_ref() {
                    cx.focus(previous_focus_handle);
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
}

pub struct PopoverElementState<M> {
    child_bounds: Option<Bounds<Pixels>>,
    popover: Rc<RefCell<Option<View<M>>>>,
}

impl<M> Clone for PopoverElementState<M> {
    fn clone(&self) -> Self {
        Self {
            popover: Rc::clone(&self.popover),
            child_bounds: self.child_bounds,
        }
    }
}

impl<M> Default for PopoverElementState<M> {
    fn default() -> Self {
        Self {
            popover: Rc::default(),
            child_bounds: None,
        }
    }
}

pub struct PopoverFrameState {
    trigger_layour_id: Option<LayoutId>,
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
                let style = Style::default();

                let mut popover_layout_id = None;
                let popover_element = element_state.popover.borrow_mut().as_mut().map(|popover| {
                    let mut anchored = anchored().snap_to_window().anchor(self.anchor);
                    if let Some(child_bounds) = element_state.child_bounds {
                        anchored = anchored.position(gpui::Point {
                            x: child_bounds.origin.x + px(40.),
                            y: child_bounds.origin.y,
                        });
                    }
                    let mut element =
                        deferred(anchored.child(div().occlude().child(popover.clone())))
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
                    style,
                    popover_layout_id.into_iter().chain(trigger_layout_id),
                );

                (
                    (
                        layout_id,
                        PopoverFrameState {
                            trigger_layour_id: trigger_layout_id,
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
        if let Some(trigger_element) = &mut request_layout.trigger_element {
            trigger_element.prepaint(cx);
        }

        if let Some(popover_element) = &mut request_layout.popover_element {
            popover_element.prepaint(cx);
        }

        let hitbox_id = request_layout.trigger_layour_id.map(|layout_id| {
            let bounds = cx.layout_bounds(layout_id);
            // cx.with_element_state(global_id.unwrap(), |element_state, _cx| {
            //     let mut element_state: PopoverElementState<M> = element_state.unwrap();
            //     element_state.child_bounds = Some(bounds);
            //     ((), element_state)
            // });

            cx.insert_hitbox(bounds, false).id
        });

        hitbox_id
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        if let Some(trigger_element) = &mut request_layout.trigger_element {
            trigger_element.paint(cx);
        }

        if let Some(popover_element) = &mut request_layout.popover_element {
            popover_element.paint(cx);
        }
    }
}

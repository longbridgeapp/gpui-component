use gpui::{
    actions, anchored, deferred, div, prelude::FluentBuilder as _, px, AnchorCorner, AnyElement,
    AppContext, Bounds, DismissEvent, DispatchPhase, Element, ElementId, EventEmitter, FocusHandle,
    FocusableView, GlobalElementId, Hitbox, InteractiveElement as _, IntoElement, KeyBinding,
    LayoutId, ManagedView, MouseButton, MouseDownEvent, ParentElement, Pixels, Point, Render,
    Style, Styled, View, ViewContext, VisualContext, WindowContext,
};
use std::{cell::RefCell, rc::Rc};

use crate::{Selectable, StyledExt as _};

const CONTEXT: &str = "Popover";

actions!(popover, [Escape]);

pub fn init(cx: &mut AppContext) {
    cx.bind_keys([KeyBinding::new("escape", Escape, Some(CONTEXT))])
}

pub struct PopoverContent {
    focus_handle: FocusHandle,
    content: Rc<dyn Fn(&mut ViewContext<Self>) -> AnyElement>,
    max_width: Option<Pixels>,
}

impl PopoverContent {
    pub fn new<B>(cx: &mut WindowContext, content: B) -> Self
    where
        B: Fn(&mut ViewContext<Self>) -> AnyElement + 'static,
    {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            content: Rc::new(content),
            max_width: None,
        }
    }

    pub fn max_w(mut self, max_width: Pixels) -> Self {
        self.max_width = Some(max_width);
        self
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
            .track_focus(&self.focus_handle)
            .key_context(CONTEXT)
            .on_action(cx.listener(|_, _: &Escape, cx| cx.emit(DismissEvent)))
            .p_2()
            .when_some(self.max_width, |this, v| this.max_w(v))
            .child(self.content.clone()(cx))
    }
}

pub struct Popover<M: ManagedView> {
    id: ElementId,
    anchor: AnchorCorner,
    trigger: Option<Box<dyn FnOnce(bool, &WindowContext) -> AnyElement + 'static>>,
    content: Option<Rc<dyn Fn(&mut WindowContext) -> View<M> + 'static>>,
    mouse_button: MouseButton,
    no_style: bool,
}

impl<M> Popover<M>
where
    M: ManagedView,
{
    /// Create a new Popover with `view` mode.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            anchor: AnchorCorner::TopLeft,
            trigger: None,
            content: None,
            mouse_button: MouseButton::Left,
            no_style: false,
        }
    }

    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set the mouse button to trigger the popover, default is `MouseButton::Left`.
    pub fn mouse_button(mut self, mouse_button: MouseButton) -> Self {
        self.mouse_button = mouse_button;
        self
    }

    pub fn trigger<T>(mut self, trigger: T) -> Self
    where
        T: Selectable + IntoElement + 'static,
    {
        self.trigger = Some(Box::new(|is_open, _| {
            trigger.selected(is_open).into_any_element()
        }));
        self
    }

    /// Set the content of the popover.
    ///
    /// The `content` is a closure that returns an `AnyElement`.
    pub fn content<C>(mut self, content: C) -> Self
    where
        C: Fn(&mut WindowContext) -> View<M> + 'static,
    {
        self.content = Some(Rc::new(content));
        self
    }

    /// Set whether the popover no style, default is `false`.
    ///
    /// If no style:
    ///
    /// - The popover will not have a bg, border, shadow, or padding.
    /// - The click out of the popover will not dismiss it.
    pub fn no_style(mut self) -> Self {
        self.no_style = true;
        self
    }

    fn render_trigger(&mut self, is_open: bool, cx: &mut WindowContext) -> impl IntoElement {
        let base = div().id("popover-trigger");

        if self.trigger.is_none() {
            return base;
        }

        let trigger = self.trigger.take().unwrap();

        base.child((trigger)(is_open, cx)).into_element()
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

    fn with_element_state<R>(
        &mut self,
        id: &GlobalElementId,
        cx: &mut WindowContext,
        f: impl FnOnce(&mut Self, &mut PopoverElementState<M>, &mut WindowContext) -> R,
    ) -> R {
        cx.with_optional_element_state::<PopoverElementState<M>, _>(
            Some(id),
            |element_state, cx| {
                let mut element_state = element_state.unwrap().unwrap_or_default();
                let result = f(self, &mut element_state, cx);
                (result, Some(element_state))
            },
        )
    }
}

impl<M> IntoElement for Popover<M>
where
    M: ManagedView,
{
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct PopoverElementState<M> {
    trigger_layout_id: Option<LayoutId>,
    popover_layout_id: Option<LayoutId>,
    popover_element: Option<AnyElement>,
    trigger_element: Option<AnyElement>,
    content_view: Rc<RefCell<Option<View<M>>>>,
    /// Trigger bounds for positioning the popover.
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl<M> Default for PopoverElementState<M> {
    fn default() -> Self {
        Self {
            trigger_layout_id: None,
            popover_layout_id: None,
            popover_element: None,
            trigger_element: None,
            content_view: Rc::new(RefCell::new(None)),
            trigger_bounds: None,
        }
    }
}

pub struct PrepaintState {
    hitbox: Hitbox,
    /// Trigger bounds for limit a rect to handle mouse click.
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl<M: ManagedView> Element for Popover<M> {
    type RequestLayoutState = PopoverElementState<M>;
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        self.with_element_state(id.unwrap(), cx, |view, element_state, cx| {
            let mut popover_layout_id = None;
            let mut popover_element = None;
            let mut is_open = false;

            if let Some(content_view) = element_state.content_view.borrow_mut().as_mut() {
                is_open = true;

                let mut anchored = anchored()
                    .snap_to_window_with_margin(px(8.))
                    .anchor(view.anchor);
                if let Some(trigger_bounds) = element_state.trigger_bounds {
                    anchored = anchored.position(view.resolved_corner(trigger_bounds));
                }

                let mut element = {
                    let content_view_mut = element_state.content_view.clone();
                    let anchor = view.anchor;
                    let no_style = view.no_style;
                    deferred(
                        anchored.child(
                            div()
                                .size_full()
                                .occlude()
                                .when(!no_style, |this| this.popover_style(cx))
                                .map(|this| match anchor {
                                    AnchorCorner::TopLeft | AnchorCorner::TopRight => {
                                        this.top_1p5()
                                    }
                                    AnchorCorner::BottomLeft | AnchorCorner::BottomRight => {
                                        this.bottom_1p5()
                                    }
                                })
                                .child(content_view.clone())
                                .when(!no_style, |this| {
                                    this.on_mouse_down_out(move |_, cx| {
                                        // Update the element_state.content_view to `None`,
                                        // so that the `paint`` method will not paint it.
                                        *content_view_mut.borrow_mut() = None;
                                        cx.refresh();
                                    })
                                }),
                        ),
                    )
                    .with_priority(1)
                    .into_any()
                };

                popover_layout_id = Some(element.request_layout(cx));
                popover_element = Some(element);
            }

            let mut trigger_element = view.render_trigger(is_open, cx).into_any_element();
            let trigger_layout_id = trigger_element.request_layout(cx);

            let layout_id = cx.request_layout(
                Style::default(),
                popover_layout_id.into_iter().chain(Some(trigger_layout_id)),
            );

            (
                layout_id,
                PopoverElementState {
                    trigger_layout_id: Some(trigger_layout_id),
                    popover_layout_id,
                    popover_element,
                    trigger_element: Some(trigger_element),
                    ..Default::default()
                },
            )
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        if let Some(element) = &mut request_layout.trigger_element {
            element.prepaint(cx);
        }
        if let Some(element) = &mut request_layout.popover_element {
            element.prepaint(cx);
        }

        let trigger_bounds = request_layout
            .trigger_layout_id
            .map(|id| cx.layout_bounds(id));

        // Prepare the popover, for get the bounds of it for open window size.
        let _ = request_layout
            .popover_layout_id
            .map(|id| cx.layout_bounds(id));

        let hitbox = cx.insert_hitbox(trigger_bounds.unwrap_or_default(), false);

        PrepaintState {
            trigger_bounds,
            hitbox,
        }
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        self.with_element_state(id.unwrap(), cx, |this, element_state, cx| {
            element_state.trigger_bounds = prepaint.trigger_bounds;

            if let Some(mut element) = request_layout.trigger_element.take() {
                element.paint(cx);
            }

            if let Some(mut element) = request_layout.popover_element.take() {
                element.paint(cx);
                return;
            }

            // When mouse click down in the trigger bounds, open the popover.
            let Some(content_build) = this.content.take() else {
                return;
            };
            let old_content_view = element_state.content_view.clone();
            let hitbox_id = prepaint.hitbox.id;
            let mouse_button = this.mouse_button;
            cx.on_mouse_event(move |event: &MouseDownEvent, phase, cx| {
                if phase == DispatchPhase::Bubble
                    && event.button == mouse_button
                    && hitbox_id.is_hovered(cx)
                {
                    cx.stop_propagation();
                    cx.prevent_default();

                    let new_content_view = (content_build)(cx);
                    let old_content_view1 = old_content_view.clone();

                    let previous_focus_handle = cx.focused();

                    cx.subscribe(&new_content_view, move |modal, _: &DismissEvent, cx| {
                        if modal.focus_handle(cx).contains_focused(cx) {
                            if let Some(previous_focus_handle) = previous_focus_handle.as_ref() {
                                cx.focus(previous_focus_handle);
                            }
                        }
                        *old_content_view1.borrow_mut() = None;

                        cx.refresh();
                    })
                    .detach();

                    cx.focus_view(&new_content_view);
                    *old_content_view.borrow_mut() = Some(new_content_view);
                    cx.refresh();
                }
            });
        });
    }
}

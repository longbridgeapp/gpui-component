use std::{cell::RefCell, rc::Rc};

use crate::{theme::ActiveTheme, Selectable, StyledExt as _};
use gpui::{
    actions, anchored, deferred, div, AnchorCorner, AnyElement, AppContext, Bounds, DismissEvent,
    DispatchPhase, Element, ElementId, EventEmitter, FocusHandle, FocusableView, GlobalElementId,
    Hitbox, InteractiveElement, IntoElement, LayoutId, ManagedView, MouseButton, MouseDownEvent,
    ParentElement as _, Pixels, Point, Render, Style, Styled as _, View, ViewContext,
    VisualContext, WindowContext,
};

actions!(popover, [Open, Dismiss]);

pub fn init(_cx: &AppContext) {}

pub struct PopoverContent {
    focus_handle: FocusHandle,
    content: Rc<dyn Fn(&mut WindowContext) -> AnyElement>,
}

impl PopoverContent {
    pub fn new<B>(cx: &mut WindowContext, content: B) -> View<Self>
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
        div().child(self.content.clone()(cx))
    }
}

pub struct Popover<M: ManagedView> {
    id: ElementId,
    anchor: AnchorCorner,
    trigger: Option<Box<dyn FnOnce(&WindowContext) -> AnyElement + 'static>>,
    content: Option<Rc<dyn Fn(&mut WindowContext) -> View<M> + 'static>>,
    mouse_button: MouseButton,
}

impl<M> Popover<M>
where
    M: ManagedView,
{
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            anchor: AnchorCorner::TopLeft,
            trigger: None,
            content: None,
            mouse_button: MouseButton::Left,
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
        self.trigger = Some(Box::new(|_| trigger.into_any_element()));
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

    fn render_trigger(&mut self, cx: &mut WindowContext) -> impl IntoElement {
        let base = div().id("popover-trigger");

        if self.trigger.is_none() {
            return base;
        }

        let trigger = self.trigger.take().unwrap();

        base.child((trigger)(cx)).into_element()
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
        self.with_element_state(id.unwrap(), cx, |this, element_state, cx| {
            let mut trigger_element = this.render_trigger(cx).into_any_element();
            let trigger_layout_id = trigger_element.request_layout(cx);

            let mut popover_layout_id = None;
            let mut popover_element = None;

            if let Some(content_view) = element_state.content_view.borrow_mut().as_mut() {
                let content_view_mut = element_state.content_view.clone();

                let mut anchored = anchored().snap_to_window().anchor(this.anchor);
                if let Some(trigger_bounds) = element_state.trigger_bounds {
                    anchored = anchored.position(this.resolved_corner(trigger_bounds));
                }

                let mut element = deferred(
                    anchored.child(
                        div()
                            .occlude()
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
                            .on_mouse_down_out(move |_, cx| {
                                *content_view_mut.borrow_mut() = None;
                                cx.refresh();
                            })
                            .child(content_view.clone()),
                    ),
                )
                .with_priority(1)
                .into_any();

                popover_layout_id = Some(element.request_layout(cx));
                popover_element = Some(element);
            }

            let layout_id = cx.request_layout(
                Style::default(),
                popover_layout_id.into_iter().chain(Some(trigger_layout_id)),
            );

            (
                layout_id,
                PopoverElementState {
                    trigger_layout_id: Some(trigger_layout_id),
                    popover_element: popover_element,
                    trigger_element: Some(trigger_element),
                    ..Default::default()
                },
            )
        })
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
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
        let hitbox = cx.insert_hitbox(trigger_bounds.unwrap_or_default(), false);

        PrepaintState {
            trigger_bounds,
            hitbox,
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
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

            let Some(content_build) = this.content.take() else {
                return;
            };

            let old_content_view = element_state.content_view.clone();
            let hitbox_id = prepaint.hitbox.id;
            let mouse_button = this.mouse_button;
            // When mouse click down in the trigger bounds, open the popover.
            cx.on_mouse_event(move |event: &MouseDownEvent, phase, cx| {
                if phase == DispatchPhase::Bubble
                    && event.button == mouse_button
                    && hitbox_id.is_hovered(cx)
                {
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

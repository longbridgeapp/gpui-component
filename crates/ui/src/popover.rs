use anyhow::Result;
use gpui::{
    actions, anchored, deferred, div, point, prelude::FluentBuilder as _, px, size, AnchorCorner,
    AnyElement, AppContext, Bounds, Context, DismissEvent, DispatchPhase, Element, ElementId,
    EventEmitter, FocusHandle, FocusableView, Global, GlobalElementId, Hitbox,
    InteractiveElement as _, IntoElement, LayoutId, ManagedView, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, Render, Style, Styled, Subscription, View, ViewContext,
    VisualContext, WindowBackgroundAppearance, WindowContext, WindowId, WindowOptions,
};
use std::{cell::RefCell, rc::Rc};

use crate::{theme::ActiveTheme, Selectable};

actions!(popover, [Open, Dismiss]);

pub fn init(cx: &mut AppContext) {
    cx.set_global(PopoverWindowState { window_id: None });
}

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
        div().p_4().max_w_128().child(self.content.clone()(cx))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupMode {
    View,
    Window,
}

pub struct Popover<M: ManagedView> {
    id: ElementId,
    anchor: AnchorCorner,
    trigger: Option<Box<dyn FnOnce(bool, &WindowContext) -> AnyElement + 'static>>,
    content: Option<Rc<dyn Fn(&mut WindowContext) -> View<M> + 'static>>,
    mouse_button: MouseButton,
    mode: PopupMode,
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
            mode: PopupMode::View,
        }
    }

    /// Set Popover to use Window mode
    pub fn window_mode(mut self) -> Self {
        self.mode = PopupMode::Window;
        self
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
    popover_bounds: Option<Bounds<Pixels>>,
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
        let mode = self.mode;

        self.with_element_state(id.unwrap(), cx, |view, element_state, cx| {
            let mut popover_layout_id = None;
            let mut popover_element = None;
            let mut is_open = false;

            if let Some(content_view) = element_state.content_view.borrow_mut().as_mut() {
                is_open = true;

                let mut anchored = anchored().snap_to_window().anchor(view.anchor);
                if let Some(trigger_bounds) = element_state.trigger_bounds {
                    anchored = anchored.position(view.resolved_corner(trigger_bounds));
                }

                let mut element = if mode == PopupMode::Window {
                    // layout the content view, to let the popover know the size of the content for window size.
                    anchored
                        .child(
                            div()
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(content_view.clone()),
                        )
                        .into_any()
                } else {
                    let content_view_mut = element_state.content_view.clone();
                    let bg_color = cx.theme().popover;
                    let anchor = view.anchor;
                    deferred(
                        anchored.child(
                            div()
                                .size_full()
                                .occlude()
                                .border_1()
                                .border_color(cx.theme().border)
                                .shadow_lg()
                                .rounded_lg()
                                .bg(bg_color)
                                .map(|this| match anchor {
                                    AnchorCorner::TopLeft | AnchorCorner::TopRight => this.top_2(),
                                    AnchorCorner::BottomLeft | AnchorCorner::BottomRight => {
                                        this.bottom_2()
                                    }
                                })
                                .child(content_view.clone())
                                .on_mouse_down_out(move |_, cx| {
                                    // Update the element_state.content_view to `None`,
                                    // so that the `paint`` method will not paint it.
                                    *content_view_mut.borrow_mut() = None;
                                    cx.refresh();
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
        let popover_bounds = request_layout
            .popover_layout_id
            .map(|id| cx.layout_bounds(id));

        let hitbox = cx.insert_hitbox(trigger_bounds.unwrap_or_default(), false);

        PrepaintState {
            trigger_bounds,
            popover_bounds,
            hitbox,
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let anchor = self.anchor;
        let mode = self.mode;
        self.with_element_state(id.unwrap(), cx, |this, element_state, cx| {
            element_state.trigger_bounds = prepaint.trigger_bounds;

            if let Some(mut element) = request_layout.trigger_element.take() {
                element.paint(cx);
            }

            if mode == PopupMode::Window {
                if let Some(content_view) = element_state.content_view.take() {
                    let popover_bounds = prepaint.popover_bounds.unwrap();
                    let trigger_bounds = prepaint.trigger_bounds.unwrap();

                    PopoverWindow::open_popover(
                        content_view,
                        trigger_bounds,
                        popover_bounds,
                        anchor,
                        cx,
                    )
                    .expect("BUG: failed to open popover window.");

                    return;
                }
            } else {
                if let Some(mut element) = request_layout.popover_element.take() {
                    element.paint(cx);
                    return;
                }
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
                        close_popover(cx);

                        cx.refresh();
                    })
                    .detach();

                    cx.focus_view(&new_content_view);
                    *old_content_view.borrow_mut() = Some(new_content_view);
                    cx.refresh();
                }
            });

            // Click parent window to dimiss popover
            if mode == PopupMode::Window {
                let content_view = element_state.content_view.clone();
                cx.on_mouse_event(move |_: &MouseDownEvent, _, cx| {
                    *content_view.borrow_mut() = None;
                    close_popover(cx);
                });
            }
        });
    }
}

struct PopoverWindowState {
    window_id: Option<WindowId>,
}

impl Global for PopoverWindowState {}

impl PopoverWindowState {
    fn window_id(cx: &AppContext) -> Option<WindowId> {
        cx.try_global::<Self>().and_then(|state| state.window_id)
    }

    fn set_window_id(window_id: WindowId, cx: &mut WindowContext) {
        cx.set_global(PopoverWindowState {
            window_id: Some(window_id),
        });
    }

    fn close_window(cx: &mut AppContext) {
        if let Some(window) = cx
            .windows()
            .into_iter()
            .find(|window| Some(window.window_id()) == PopoverWindowState::window_id(cx))
        {
            cx.update_window(window, |_, cx| {
                cx.remove_window();
            })
            .ok();
        }
    }
}

pub struct PopoverWindow<M: ManagedView> {
    view: View<M>,
    anchor: AnchorCorner,
    close_when_deactivate: bool,
    _subscriptions: Vec<Subscription>,
}

pub fn close_popover(cx: &mut AppContext) {
    PopoverWindowState::close_window(cx);
}

impl<M> PopoverWindow<M>
where
    M: ManagedView,
{
    pub fn open_popover(
        view: View<M>,
        trigger_bounds: Bounds<Pixels>,
        bounds: Bounds<Pixels>,
        anchor: AnchorCorner,
        cx: &mut WindowContext,
    ) -> Result<()> {
        // Every open_popover will close the existing one
        PopoverWindowState::close_window(cx);

        let display = cx.display();
        let window_bounds = cx.bounds();

        // TODO: avoid out of the screen bounds

        let border_bounds = if cfg!(target_os = "windows") {
            Bounds {
                origin: point(px(-8.0), px(5.0)),
                size: size(px(16.0), px(8.0)),
            }
        } else {
            Bounds {
                origin: point(px(-8.0), px(0.0)),
                size: size(px(20.0), px(20.0)),
            }
        };

        let trigger_screen_bounds = Bounds {
            origin: window_bounds.origin + trigger_bounds.origin + border_bounds.origin,
            size: trigger_bounds.size,
        };

        let popover_offset = px(2.);
        let popover_origin = match anchor {
            AnchorCorner::TopLeft => {
                trigger_screen_bounds.lower_left() + point(px(0.), popover_offset)
            }
            AnchorCorner::TopRight => {
                trigger_screen_bounds.lower_right() + point(-bounds.size.width, popover_offset)
            }
            AnchorCorner::BottomLeft => {
                trigger_screen_bounds.origin
                    - point(
                        px(0.0),
                        bounds.size.height + border_bounds.size.height + popover_offset,
                    )
            }
            AnchorCorner::BottomRight => {
                trigger_screen_bounds.upper_right()
                    - point(
                        bounds.size.width,
                        bounds.size.height + border_bounds.size.height + popover_offset,
                    )
            }
        };

        let bounds = Bounds {
            origin: popover_origin,
            size: size(
                bounds.size.width + border_bounds.size.width,
                bounds.size.height + border_bounds.size.height,
            ),
        };

        let view = view.clone();
        cx.spawn(|mut cx| async move {
            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                        window_background: WindowBackgroundAppearance::Transparent,
                        // NOTE: on Windows in currently must use PopUp kind, otherwise the window will be sizeable.
                        // And the PopUp kind can fast open.
                        kind: gpui::WindowKind::PopUp,
                        is_movable: false,
                        focus: true,
                        show: true,
                        display_id: display.map(|d| d.id()),
                        ..Default::default()
                    },
                    |cx| {
                        let mut _subscriptions = Vec::new();

                        let view = cx.new_view(|cx| {
                            // Listen to window diactivation to close window
                            _subscriptions.push(
                                cx.observe_window_activation(Self::window_activation_changed),
                            );

                            PopoverWindow {
                                view,
                                anchor,
                                close_when_deactivate: true,
                                _subscriptions,
                            }
                        });

                        view
                    },
                )
                .expect("BUG: faild to create a new window.");

            cx.update(|cx| {
                PopoverWindowState::set_window_id(window.window_id(), cx);
            })
            .expect("BUG: failed to set window id.")
        })
        .detach();

        Ok(())
    }
}

impl<M> PopoverWindow<M>
where
    M: ManagedView,
{
    fn window_activation_changed(&mut self, cx: &mut ViewContext<Self>) {
        if self.close_when_deactivate {
            if !cx.is_window_active() {
                self.dismiss(cx);
            }
        }
    }

    fn dismiss(&mut self, cx: &mut ViewContext<Self>) {
        cx.remove_window();
    }
}

impl<M> FocusableView for PopoverWindow<M>
where
    M: ManagedView,
{
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.view.focus_handle(cx)
    }
}

impl<M> Render for PopoverWindow<M>
where
    M: ManagedView,
{
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let is_windows = cfg!(target_os = "windows");

        div()
            .id("PopoverWindow")
            .size_full()
            .when(!is_windows, |this| this.p_2())
            .when(is_windows, |this| this.bg(cx.theme().popover))
            .text_color(cx.theme().popover_foreground)
            // Leave margin for show window shadow
            .map(|d| match self.anchor {
                AnchorCorner::TopLeft | AnchorCorner::TopRight => d.mt_8(),
                AnchorCorner::BottomLeft | AnchorCorner::BottomRight => d.mb_8(),
            })
            .child(
                div()
                    .when(!is_windows, |this| {
                        this.bg(cx.theme().popover)
                            .border_1()
                            .border_color(cx.theme().border)
                            .shadow_lg()
                            .rounded_lg()
                    })
                    .bg(cx.theme().popover)
                    .child(self.view.clone())
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|_, _, cx| {
                            cx.stop_propagation();

                            PopoverWindowState::close_window(cx);
                        }),
                    ),
            )
    }
}

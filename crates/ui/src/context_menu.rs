use std::{cell::RefCell, rc::Rc};

use gpui::{
    anchored, deferred, div, prelude::FluentBuilder, relative, AnchorCorner, AnyElement,
    AppContext, DismissEvent, DispatchPhase, Element, ElementId, Focusable, GlobalElementId,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement, Pixels, Point,
    Position, Stateful, Style, Styled as _, View, WindowContext,
};

use crate::{popup_menu::PopupMenu, theme::ActiveTheme};

pub fn init(_cx: &mut AppContext) {}

pub trait ContextMenuExt: ParentElement + Sized {
    fn context_menu(
        self,
        f: impl Fn(PopupMenu, &mut WindowContext) -> PopupMenu + 'static,
    ) -> Self {
        self.child(ContextMenu::new("context_menu").menu(f))
    }
}

impl<E> ContextMenuExt for Stateful<E> where E: ParentElement {}
impl<E> ContextMenuExt for Focusable<E> where E: ParentElement {}

pub struct ContextMenu {
    id: ElementId,
    menu: Option<Box<dyn Fn(PopupMenu, &mut WindowContext) -> PopupMenu + 'static>>,
    anchor: AnchorCorner,
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            menu: None,
            anchor: AnchorCorner::TopLeft,
        }
    }

    #[must_use]
    pub fn menu<F>(mut self, builder: F) -> Self
    where
        F: Fn(PopupMenu, &mut WindowContext) -> PopupMenu + 'static,
    {
        self.menu = Some(Box::new(builder));
        self
    }

    fn with_element_state<R>(
        &mut self,
        id: &GlobalElementId,
        cx: &mut WindowContext,
        f: impl FnOnce(&mut Self, &mut ContextMenuState, &mut WindowContext) -> R,
    ) -> R {
        cx.with_optional_element_state::<ContextMenuState, _>(Some(id), |element_state, cx| {
            let mut element_state = element_state.unwrap().unwrap_or_default();
            let result = f(self, &mut element_state, cx);
            (result, Some(element_state))
        })
    }
}

impl IntoElement for ContextMenu {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct ContextMenuState {
    menu_view: Rc<RefCell<Option<View<PopupMenu>>>>,
    menu_element: Option<AnyElement>,
    open: Rc<RefCell<bool>>,
    position: Rc<RefCell<Point<Pixels>>>,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            menu_view: Rc::new(RefCell::new(None)),
            menu_element: None,
            open: Rc::new(RefCell::new(false)),
            position: Default::default(),
        }
    }
}

impl Element for ContextMenu {
    type RequestLayoutState = ContextMenuState;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        // Set the layout style relative to the table view to get same size.
        style.position = Position::Absolute;
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();

        let anchor = self.anchor;

        self.with_element_state(id.unwrap(), cx, |_, state: &mut ContextMenuState, cx| {
            let position = state.position.clone();
            let position = position.borrow();
            let open = state.open.clone();
            let menu_view = state.menu_view.borrow().clone();

            let (menu_element, menu_layout_id) = if *open.borrow() {
                let mut menu_element = deferred(
                    anchored()
                        .position(*position)
                        .snap_to_window()
                        .anchor(anchor)
                        .when_some(menu_view, |this, menu| {
                            // Focus the menu, so that can be handle the action.
                            menu.focus_handle(cx).focus(cx);

                            this.child(
                                div()
                                    .bg(cx.theme().popover)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .shadow_lg()
                                    .rounded_lg()
                                    .child(menu)
                                    .on_mouse_down_out(move |_, cx| {
                                        *open.borrow_mut() = false;
                                        cx.refresh();
                                    }),
                            )
                        }),
                )
                .with_priority(1)
                .into_any();

                let menu_layout_id = menu_element.request_layout(cx);
                (Some(menu_element), Some(menu_layout_id))
            } else {
                (None, None)
            };

            let mut layout_ids = vec![];
            if let Some(menu_layout_id) = menu_layout_id {
                layout_ids.push(menu_layout_id);
            }

            let layout_id = cx.request_layout(style, layout_ids);

            (
                layout_id,
                ContextMenuState {
                    menu_element,

                    ..Default::default()
                },
            )
        })
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        if let Some(menu_element) = &mut request_layout.menu_element {
            menu_element.prepaint(cx);
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        if let Some(menu_element) = &mut request_layout.menu_element {
            menu_element.paint(cx);
        }

        let Some(builder) = self.menu.take() else {
            return;
        };

        self.with_element_state(
            id.unwrap(),
            cx,
            |_view, state: &mut ContextMenuState, cx| {
                let position = state.position.clone();
                let open = state.open.clone();
                let menu_view = state.menu_view.clone();

                // When right mouse click, to build content menu, and show it at the mouse position.
                cx.on_mouse_event(move |event: &MouseDownEvent, phase, cx| {
                    if phase == DispatchPhase::Bubble
                        && event.button == MouseButton::Right
                        && bounds.contains(&event.position)
                    {
                        cx.prevent_default();
                        cx.stop_propagation();

                        *position.borrow_mut() = event.position;
                        *open.borrow_mut() = true;

                        let menu =
                            PopupMenu::build(cx, |menu, cx| (builder)(menu, cx)).into_element();

                        let open = open.clone();
                        cx.subscribe(&menu, move |_, _: &DismissEvent, cx| {
                            *open.borrow_mut() = false;
                            cx.refresh();
                        })
                        .detach();

                        *menu_view.borrow_mut() = Some(menu);

                        cx.refresh();
                    }
                });
            },
        );
    }
}

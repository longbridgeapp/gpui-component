use std::{cell::RefCell, rc::Rc};

use gpui::{
    anchored, deferred, div, AnchorCorner, AnyElement, AppContext, Div, Element, ElementId,
    GlobalElementId, InteractiveElement as _, IntoElement, KeyBinding, MouseButton, MouseDownEvent,
    MouseEvent, ParentElement, Pixels, Point, Position, Render, Stateful, Style, View,
    WindowContext,
};
use smallvec::SmallVec;

use crate::popup_menu::PopupMenu;

pub fn init(cx: &mut AppContext) {}

pub struct ContextMenu {
    id: ElementId,
    menu: View<PopupMenu>,
    anchor: AnchorCorner,
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>, cx: &mut WindowContext) -> Self {
        let menu = PopupMenu::build(cx, |this, _cx| this);
        Self {
            id: id.into(),
            menu,
            anchor: AnchorCorner::TopLeft,
        }
    }

    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor = anchor;
        self
    }

    #[must_use]
    pub fn menu(mut self, menu: View<PopupMenu>) -> Self {
        self.menu = menu;
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
    menu_element: Option<AnyElement>,
    anchor: AnchorCorner,
    position: Rc<RefCell<Point<Pixels>>>,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            menu_element: None,
            anchor: AnchorCorner::TopLeft,
            position: Default::default(),
        }
    }
}

impl Element for ContextMenu {
    type RequestLayoutState = ContextMenuState;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        self.with_element_state(
            id.unwrap(),
            cx,
            |_view, state: &mut ContextMenuState, cx| {
                let position = state.position.clone();
                let anchor = state.anchor;
                let position = position.borrow().clone();

                let mut menu_element = deferred(
                    anchored()
                        .position(position)
                        .anchor(gpui::AnchorCorner::TopLeft)
                        .child(self.menu.take()),
                )
                .with_priority(1)
                .into_any_element();

                let menu_layout_id = menu_element.request_layout(cx);

                let layout_id = cx.request_layout(Style::default(), vec![menu_layout_id]);

                (
                    layout_id,
                    ContextMenuState {
                        menu_element: Some(menu_element),
                        anchor,
                        ..Default::default()
                    },
                )
            },
        )
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
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
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        if let Some(menu_element) = &mut request_layout.menu_element {
            menu_element.paint(cx);
        }

        self.with_element_state(
            id.unwrap(),
            cx,
            |_view, state: &mut ContextMenuState, cx| {
                let position = state.position.clone();
                // When right mouse click, to build content menu, and show it at the mouse position.
                cx.on_mouse_event(move |event: &MouseDownEvent, phase, cx| {
                    if event.button != MouseButton::Right {
                        return;
                    }

                    let posision = event.position;
                    *position.borrow_mut() = posision;
                });
            },
        );
    }
}

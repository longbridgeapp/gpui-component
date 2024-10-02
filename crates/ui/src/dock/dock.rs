//! Dock is a fixed container that places at left, bottom, right of the Windows.

use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder as _, px, Axis, Element, InteractiveElement as _, IntoElement,
    MouseMoveEvent, MouseUpEvent, ParentElement as _, Pixels, Point, Render,
    StatefulInteractiveElement, Style, Styled as _, View, ViewContext, VisualContext as _,
    WeakView,
};

use crate::{
    resizable::{HANDLE_PADDING, HANDLE_SIZE, PANEL_MIN_SIZE},
    theme::ActiveTheme as _,
    AxisExt as _, StyledExt,
};

use super::{DockArea, PanelView, TabPanel};

#[derive(Clone, Render)]
struct ResizePanel;

#[derive(Debug, Clone, Copy)]
pub enum DockPlacement {
    Left,
    Bottom,
    Right,
}

impl DockPlacement {
    fn axis(&self) -> Axis {
        match self {
            Self::Left | Self::Right => Axis::Horizontal,
            Self::Bottom => Axis::Vertical,
        }
    }

    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }

    pub fn is_bottom(&self) -> bool {
        matches!(self, Self::Bottom)
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right)
    }
}

/// The Dock is a fixed container that places at left, bottom, right of the Windows.
///
/// This is unlike Panel, it can't be move or add any other panel.
pub struct Dock {
    placement: DockPlacement,
    dock_area: WeakView<DockArea>,
    pub(crate) panel: View<TabPanel>,
    /// The size is means the width or height of the Dock, if the placement is left or right, the size is width, otherwise the size is height.
    size: Pixels,
    open: bool,
    resizeable: bool,
    is_resizing: bool,
}

impl Dock {
    fn new(
        dock_area: WeakView<DockArea>,
        placement: DockPlacement,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let panel = cx.new_view(|cx| {
            let mut tab = TabPanel::new(None, dock_area.clone(), cx);
            tab.closeable = false;
            tab.zoomable = false;
            tab
        });

        Self {
            placement,
            dock_area,
            panel,
            open: true,
            resizeable: true,
            size: px(200.0),
            is_resizing: false,
        }
    }

    pub fn left(dock_area: WeakView<DockArea>, cx: &mut ViewContext<Self>) -> Self {
        Self::new(dock_area, DockPlacement::Left, cx)
    }

    pub fn bottom(dock_area: WeakView<DockArea>, cx: &mut ViewContext<Self>) -> Self {
        Self::new(dock_area, DockPlacement::Bottom, cx)
    }

    pub fn right(dock_area: WeakView<DockArea>, cx: &mut ViewContext<Self>) -> Self {
        Self::new(dock_area, DockPlacement::Right, cx)
    }

    pub fn set_panels(&mut self, panels: Vec<Arc<dyn PanelView>>, cx: &mut ViewContext<Self>) {
        self.panel.update(cx, |tab_panel, _| {
            tab_panel.panels = panels;
            tab_panel.active_ix = 0;
        });
        cx.notify();
    }

    /// Set the Dock to be resizeable, default: true
    pub fn resizeable(mut self, resizeable: bool) -> Self {
        self.resizeable = resizeable;
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn toggle_open(&mut self, cx: &mut ViewContext<Self>) {
        self.open = !self.open;
        cx.notify();
    }

    /// Returns the size of the Dock, the size is means the width or height of
    /// the Dock, if the placement is left or right, the size is width,
    /// otherwise the size is height.
    pub fn size(&self) -> Pixels {
        self.size
    }

    /// Set the size of the Dock.
    pub fn set_size(&mut self, size: Pixels, cx: &mut ViewContext<Self>) {
        self.size = size.max(PANEL_MIN_SIZE);
        cx.notify();
    }

    /// Set the open state of the Dock.
    pub fn set_open(&mut self, open: bool, cx: &mut ViewContext<Self>) {
        self.open = open;
        cx.notify();
    }

    fn render_resize_handle(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let axis = self.placement.axis();
        let neg_offset = -HANDLE_PADDING;
        let view = cx.view().clone();

        div()
            .id("resize-handle")
            .occlude()
            .absolute()
            .flex_shrink_0()
            .when(self.placement.is_left(), |this| {
                // FIXME: Improve this to let the scroll bar have px(HANDLE_PADDING)
                this.cursor_col_resize()
                    .top_0()
                    .right(px(1.))
                    .h_full()
                    .w(HANDLE_SIZE)
                    .pl(HANDLE_PADDING)
            })
            .when(self.placement.is_right(), |this| {
                this.cursor_col_resize()
                    .top_0()
                    .left(neg_offset)
                    .h_full()
                    .w(HANDLE_SIZE)
                    .px(HANDLE_PADDING)
            })
            .when(self.placement.is_bottom(), |this| {
                this.cursor_row_resize()
                    .top(neg_offset)
                    .left_0()
                    .w_full()
                    .h(HANDLE_SIZE)
                    .py(HANDLE_PADDING)
            })
            .child(
                div()
                    .bg(cx.theme().border)
                    .when(axis.is_horizontal(), |this| this.h_full().w(HANDLE_SIZE))
                    .when(axis.is_vertical(), |this| this.w_full().h(HANDLE_SIZE)),
            )
            .on_drag(ResizePanel {}, move |info, cx| {
                cx.stop_propagation();
                view.update(cx, |view, _| {
                    view.is_resizing = true;
                });
                cx.new_view(|_| info.clone())
            })
    }

    fn resize(&mut self, mouse_position: Point<Pixels>, cx: &mut ViewContext<Self>) {
        if !self.is_resizing {
            return;
        }

        let area_bounds = self
            .dock_area
            .upgrade()
            .expect("DockArea is missing")
            .read(cx)
            .bounds;

        let size = match self.placement {
            DockPlacement::Left => mouse_position.x - area_bounds.left(),
            DockPlacement::Right => area_bounds.right() - mouse_position.x,
            DockPlacement::Bottom => area_bounds.bottom() - mouse_position.y,
        };

        self.size = size.max(PANEL_MIN_SIZE);
        cx.notify();
    }

    fn done_resizing(&mut self, _: &mut ViewContext<Self>) {
        self.is_resizing = false;
    }
}

impl Render for Dock {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        if !self.open && !self.placement.is_bottom() {
            return div();
        }

        div()
            .relative()
            .overflow_hidden()
            .map(|this| match self.placement {
                DockPlacement::Left | DockPlacement::Right => this.h_flex().h_full().w(self.size),
                DockPlacement::Bottom => this.w_full().h(self.size),
            })
            // Bottom Dock should keep the title bar, then user can click the Toggle button
            .when(!self.open && self.placement.is_bottom(), |this| {
                this.h(px(30.))
            })
            .child(self.panel.clone())
            .child(self.render_resize_handle(cx))
            .child(DockElement {
                view: cx.view().clone(),
            })
    }
}

struct DockElement {
    view: View<Dock>,
}

impl IntoElement for DockElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for DockElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        (cx.request_layout(Style::default(), None), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        cx.on_mouse_event({
            let view = self.view.clone();
            move |e: &MouseMoveEvent, phase, cx| {
                if phase.bubble() {
                    view.update(cx, |view, cx| view.resize(e.position, cx))
                }
            }
        });

        // When any mouse up, stop dragging
        cx.on_mouse_event({
            let view = self.view.clone();
            move |_: &MouseUpEvent, phase, cx| {
                if phase.bubble() {
                    view.update(cx, |view, cx| view.done_resizing(cx));
                }
            }
        })
    }
}

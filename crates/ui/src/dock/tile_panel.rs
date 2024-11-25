use std::sync::Arc;

use crate::{h_flex, theme::ActiveTheme, v_flex, Placement};

use super::{CanvasPanelState, DockItemInfo, DockItemState, Panel, PanelEvent, PanelView};
use gpui::{
    div, px, AppContext, Bounds, DismissEvent, DragMoveEvent, EntityId, EventEmitter, FocusHandle,
    FocusableView, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, StatefulInteractiveElement, Styled, ViewContext,
    VisualContext,
};

const MINIMUM_WIDTH: f32 = 100.;
const MINIMUM_HEIGHT: f32 = 100.;
const DRAG_BAR_HEIGHT: f32 = 30.;

#[derive(Clone, Render)]
pub struct DragBar(EntityId);

#[derive(Clone, Render)]
pub struct DragResizing(EntityId);

#[derive(Clone)]
struct ResizeDragData {
    axis: ResizeAxis,
    initial_mouse_position: Point<Pixels>,
    initial_panel_bounds: Bounds<Pixels>,
}

#[derive(Clone, PartialEq)]
enum ResizeAxis {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Clone)]
pub struct TilesItem {
    pub(crate) panel: Arc<dyn PanelView>,
    bounds: Bounds<Pixels>,
}

pub struct TilePanel {
    focus_handle: FocusHandle,
    pub(crate) panels: Vec<TilesItem>,
    dragging_panel_index: Option<usize>,
    dragging_initial_mouse: Point<Pixels>,
    dragging_initial_bounds: Bounds<Pixels>,
    resizing_panel_index: Option<usize>,
    resizing_drag_data: Option<ResizeDragData>,
}

impl Panel for TilePanel {
    fn panel_name(&self) -> &'static str {
        "CanvasPanel"
    }

    fn title(&self, _cx: &gpui::WindowContext) -> gpui::AnyElement {
        "CanvasPanel".into_any_element()
    }

    fn dump(&self, cx: &AppContext) -> DockItemState {
        let panels_state = self
            .panels
            .iter()
            .map(|item: &TilesItem| {
                let panel_state = item.panel.dump(cx);
                CanvasPanelState {
                    panel_state,
                    x: item.bounds.origin.x,
                    y: item.bounds.origin.y,
                    w: item.bounds.size.width,
                    h: item.bounds.size.height,
                }
            })
            .collect();

        let mut state = DockItemState::new(self);
        state.info = DockItemInfo::Canvas {
            panels: panels_state,
        };
        state
    }
}

impl TilePanel {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            panels: vec![],
            dragging_panel_index: None,
            dragging_initial_mouse: Point::default(),
            dragging_initial_bounds: Bounds::default(),
            resizing_panel_index: None,
            resizing_drag_data: None,
        }
    }

    pub(super) fn panels_len(&self) -> usize {
        self.panels.len()
    }

    /// Return the index of the panel.
    pub(crate) fn index_of_panel(&self, panel: Arc<dyn PanelView>) -> Option<usize> {
        self.panels.iter().position(|p| &p.panel == &panel)
    }

    /// Add a panel at the end of the canvas.
    pub fn add_panel(
        &mut self,
        panel: Arc<dyn PanelView>,
        bounds: Bounds<Pixels>,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel(panel, self.panels.len(), bounds, cx);
    }

    pub fn add_panel_at(
        &mut self,
        panel: Arc<dyn PanelView>,
        bounds: Bounds<Pixels>,
        placement: Placement,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel_at(panel, bounds, self.panels_len(), placement, cx);
    }

    pub fn insert_panel_at(
        &mut self,
        panel: Arc<dyn PanelView>,
        bounds: Bounds<Pixels>,
        ix: usize,
        placement: Placement,
        cx: &mut ViewContext<Self>,
    ) {
        match placement {
            Placement::Top | Placement::Left => self.insert_panel_before(panel, bounds, ix, cx),
            Placement::Right | Placement::Bottom => self.insert_panel_after(panel, bounds, ix, cx),
        }
    }

    /// Insert a panel at the index.
    pub fn insert_panel_before(
        &mut self,
        panel: Arc<dyn PanelView>,
        bounds: Bounds<Pixels>,
        ix: usize,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel(panel, ix, bounds, cx);
    }

    /// Insert a panel after the index.
    pub fn insert_panel_after(
        &mut self,
        panel: Arc<dyn PanelView>,
        bounds: Bounds<Pixels>,
        ix: usize,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel(panel, ix + 1, bounds, cx);
    }

    fn insert_panel(
        &mut self,
        panel: Arc<dyn PanelView>,
        ix: usize,
        bounds: Bounds<Pixels>,
        cx: &mut ViewContext<Self>,
    ) {
        // If the panel is already in the canvas, return.
        if let Some(_) = self.index_of_panel(panel.clone()) {
            return;
        }

        let ix = if ix > self.panels.len() {
            self.panels.len()
        } else {
            ix
        };

        self.panels.insert(
            ix,
            TilesItem {
                panel: panel.clone(),
                bounds: bounds,
            },
        );

        cx.emit(PanelEvent::LayoutChanged);
        cx.notify();
    }

    /// Remove panel from the canvas.
    pub fn remove_panel(&mut self, panel: Arc<dyn PanelView>, cx: &mut ViewContext<Self>) {
        if let Some(ix) = self.index_of_panel(panel.clone()) {
            self.panels.remove(ix);

            cx.emit(PanelEvent::LayoutChanged);
        } else {
            println!("Panel not found in canvas panel.");
        }
    }

    fn update_initial_position(
        &mut self,
        position: Point<Pixels>,
        cx: &mut ViewContext<'_, TilePanel>,
    ) {
        for (index, item) in self.panels.iter().enumerate() {
            if item.bounds.contains(&position) {
                self.dragging_panel_index = Some(index);
                self.dragging_initial_mouse = position;
                self.dragging_initial_bounds = item.bounds;
                cx.notify();
                return;
            }
        }
    }

    fn update_position(
        &mut self,
        current_mouse_position: Point<Pixels>,
        cx: &mut ViewContext<'_, TilePanel>,
    ) {
        if let Some(index) = self.dragging_panel_index {
            if let Some(item) = self.panels.get_mut(index) {
                let delta = current_mouse_position - self.dragging_initial_mouse;
                let new_origin = self.dragging_initial_bounds.origin + delta;
                item.bounds.origin = round_point_to_nearest_ten(new_origin);
                cx.notify();
            }
        }
    }

    fn update_resizing_drag(
        &mut self,
        drag_data: ResizeDragData,
        cx: &mut ViewContext<'_, TilePanel>,
    ) {
        for (index, item) in self.panels.iter().enumerate() {
            if item.bounds == drag_data.initial_panel_bounds {
                self.resizing_panel_index = Some(index);
                self.resizing_drag_data = Some(drag_data);
                cx.notify();
                return;
            }
        }
    }

    fn resize_panel_width(&mut self, new_width: Pixels, cx: &mut ViewContext<'_, TilePanel>) {
        if let Some(index) = self.resizing_panel_index {
            if let Some(item) = self.panels.get_mut(index) {
                item.bounds.size.width = round_to_nearest_ten(new_width);
                cx.notify();
            }
        }
    }

    fn resize_panel_height(
        &mut self,
        new_height: Pixels,
        cx: &mut ViewContext<'_, TilePanel>,
    ) {
        if let Some(index) = self.resizing_panel_index {
            if let Some(item) = self.panels.get_mut(index) {
                item.bounds.size.height = round_to_nearest_ten(new_height);
                cx.notify();
            }
        }
    }
}

fn round_to_nearest_ten(value: Pixels) -> Pixels {
    px((value.0 / 10.0).round() * 10.0)
}

fn round_point_to_nearest_ten(point: Point<Pixels>) -> Point<Pixels> {
    Point::new(round_to_nearest_ten(point.x), round_to_nearest_ten(point.y))
}

impl FocusableView for TilePanel {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl EventEmitter<PanelEvent> for TilePanel {}
impl EventEmitter<DismissEvent> for TilePanel {}
impl Render for TilePanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let entity_id = cx.entity_id();

        h_flex()
            .size_full()
            .overflow_hidden()
            .relative()
            .bg(cx.theme().background)
            .children(self.panels.clone().into_iter().map(|item| {
                let panel = item.panel.clone();
                let panel_view = panel.view();

                v_flex()
                    .border_1()
                    .rounded_md()
                    .border_color(cx.theme().border)
                    .absolute()
                    .left(item.bounds.origin.x)
                    .top(item.bounds.origin.y)
                    .w(item.bounds.size.width)
                    .h(item.bounds.size.height)
                    .child(
                        // Panel content
                        h_flex()
                            .w_full()
                            .h_full()
                            .overflow_hidden()
                            .child(panel_view),
                    )
                    // Resize handles
                    .child(
                        // Right edge resize handle
                        div()
                            .id("right-resize-handle")
                            .cursor_col_resize()
                            .absolute()
                            .top(px(0.0))
                            .right(px(-5.0))
                            .w(px(10.0))
                            .h(item.bounds.size.height)
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _event: &MouseUpEvent, cx| {
                                    this.resizing_panel_index = None;
                                    this.resizing_drag_data = None;
                                    cx.emit(PanelEvent::LayoutChanged);
                                    cx.notify();
                                }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, cx| {
                                    let initial_mouse_position = event.position;
                                    let panel_bounds = item.bounds;
                                    let drag_data = ResizeDragData {
                                        axis: ResizeAxis::Horizontal,
                                        initial_mouse_position,
                                        initial_panel_bounds: panel_bounds,
                                    };
                                    this.update_resizing_drag(drag_data, cx);
                                }),
                            )
                            .on_drag(DragResizing(entity_id), |drag, _, cx| {
                                cx.stop_propagation();
                                cx.new_view(|_| drag.clone())
                            })
                            .on_drag_move(cx.listener(
                                move |this, e: &DragMoveEvent<DragResizing>, cx| match e.drag(cx) {
                                    DragResizing(id) => {
                                        if *id != entity_id {
                                            return;
                                        }

                                        if let Some(ref drag_data) = this.resizing_drag_data {
                                            if drag_data.axis != ResizeAxis::Horizontal {
                                                return;
                                            }
                                            let current_mouse_position = e.event.position;
                                            let delta = current_mouse_position.x
                                                - drag_data.initial_mouse_position.x;
                                            let new_width =
                                                (drag_data.initial_panel_bounds.size.width + delta)
                                                    .max(px(MINIMUM_WIDTH));
                                            this.resize_panel_width(new_width, cx);
                                        }
                                    }
                                },
                            )),
                    )
                    .child(
                        // Bottom edge resize handle
                        div()
                            .id("bottom-resize-handle")
                            .cursor_row_resize()
                            .absolute()
                            .left(px(0.0))
                            .bottom(px(-5.0))
                            .w(item.bounds.size.width)
                            .h(px(10.0))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _event: &MouseUpEvent, cx| {
                                    this.resizing_panel_index = None;
                                    this.resizing_drag_data = None;
                                    cx.emit(PanelEvent::LayoutChanged);
                                    cx.notify();
                                }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, cx| {
                                    let initial_mouse_position = event.position;
                                    let panel_bounds = item.bounds;
                                    let drag_data = ResizeDragData {
                                        axis: ResizeAxis::Vertical,
                                        initial_mouse_position,
                                        initial_panel_bounds: panel_bounds,
                                    };
                                    this.update_resizing_drag(drag_data, cx);
                                }),
                            )
                            .on_drag(DragResizing(entity_id), |drag, _, cx| {
                                cx.stop_propagation();
                                cx.new_view(|_| drag.clone())
                            })
                            .on_drag_move(cx.listener(
                                move |this, e: &DragMoveEvent<DragResizing>, cx| match e.drag(cx) {
                                    DragResizing(id) => {
                                        if *id != entity_id {
                                            return;
                                        }

                                        if let Some(ref drag_data) = this.resizing_drag_data {
                                            let current_mouse_position = e.event.position;
                                            let delta = current_mouse_position.y
                                                - drag_data.initial_mouse_position.y;
                                            let new_height =
                                                (drag_data.initial_panel_bounds.size.height
                                                    + delta)
                                                    .max(px(MINIMUM_HEIGHT));
                                            this.resize_panel_height(new_height, cx);
                                        }
                                    }
                                },
                            )),
                    )
                    .child(
                        // Corner resize handle
                        div()
                            .id("corner-resize-handle")
                            .cursor_crosshair()
                            .absolute()
                            .right(px(-5.0))
                            .bottom(px(-5.0))
                            .w(px(10.0))
                            .h(px(10.0))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _event: &MouseUpEvent, cx| {
                                    this.resizing_panel_index = None;
                                    this.resizing_drag_data = None;
                                    cx.emit(PanelEvent::LayoutChanged);
                                    cx.notify();
                                }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, cx| {
                                    let initial_mouse_position = event.position;
                                    let panel_bounds = item.bounds;
                                    let drag_data = ResizeDragData {
                                        axis: ResizeAxis::Both,
                                        initial_mouse_position,
                                        initial_panel_bounds: panel_bounds,
                                    };
                                    this.update_resizing_drag(drag_data, cx);
                                }),
                            )
                            .on_drag(DragResizing(entity_id), |drag, _, cx| {
                                cx.stop_propagation();
                                cx.new_view(|_| drag.clone())
                            })
                            .on_drag_move(cx.listener(
                                move |this, e: &DragMoveEvent<DragResizing>, cx| match e.drag(cx) {
                                    DragResizing(id) => {
                                        if *id != entity_id {
                                            return;
                                        }

                                        if let Some(ref drag_data) = this.resizing_drag_data {
                                            if drag_data.axis != ResizeAxis::Both {
                                                return;
                                            }
                                            let current_mouse_position = e.event.position;
                                            let delta_x = current_mouse_position.x
                                                - drag_data.initial_mouse_position.x;
                                            let delta_y = current_mouse_position.y
                                                - drag_data.initial_mouse_position.y;
                                            let new_width =
                                                (drag_data.initial_panel_bounds.size.width
                                                    + delta_x)
                                                    .max(px(MINIMUM_WIDTH));
                                            let new_height =
                                                (drag_data.initial_panel_bounds.size.height
                                                    + delta_y)
                                                    .max(px(MINIMUM_HEIGHT));
                                            this.resize_panel_height(new_height, cx);
                                            this.resize_panel_width(new_width, cx);
                                        }
                                    }
                                },
                            )),
                    )
                    // Drag bar
                    .child(
                        h_flex()
                            .id("drag-bar")
                            .cursor_grab()
                            .absolute()
                            .w_full()
                            .h(px(DRAG_BAR_HEIGHT))
                            .bg(cx.theme().transparent)
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _event: &MouseUpEvent, cx| {
                                    this.dragging_panel_index = None;
                                    cx.emit(PanelEvent::LayoutChanged);
                                    cx.notify();
                                }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, cx| {
                                    let initial_mouse_position = event.position;
                                    this.update_initial_position(initial_mouse_position, cx);
                                }),
                            )
                            .on_drag(DragBar(entity_id), |drag, _, cx| {
                                cx.stop_propagation();
                                cx.new_view(|_| drag.clone())
                            })
                            .on_drag_move(cx.listener(
                                move |this, e: &DragMoveEvent<DragBar>, cx| match e.drag(cx) {
                                    DragBar(id) => {
                                        if *id != entity_id {
                                            return;
                                        }

                                        this.update_position(e.event.position, cx);
                                    }
                                },
                            )),
                    )
            }))
    }
}

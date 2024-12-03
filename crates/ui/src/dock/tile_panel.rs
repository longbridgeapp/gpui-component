use std::sync::Arc;

use crate::{h_flex, theme::ActiveTheme, v_flex, Placement};

use super::{DockItemInfo, DockItemState, Panel, PanelEvent, PanelView, TilePanelState};
use gpui::{
    canvas, div, px, AppContext, Bounds, DismissEvent, DragMoveEvent, EntityId, EventEmitter,
    FocusHandle, FocusableView, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Pixels, Point, Render, StatefulInteractiveElement, Styled,
    ViewContext, VisualContext,
};

const MINIMUM_WIDTH: f32 = 100.;
const MINIMUM_HEIGHT: f32 = 100.;
const DRAG_BAR_HEIGHT: f32 = 30.;

#[derive(Clone, Render)]
pub struct DragMoving(EntityId);

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
    z_index: usize,
}

pub struct TilePanel {
    focus_handle: FocusHandle,
    pub(crate) panels: Vec<TilesItem>,
    dragging_panel_index: Option<usize>,
    dragging_initial_mouse: Point<Pixels>,
    dragging_initial_bounds: Bounds<Pixels>,
    resizing_panel_index: Option<usize>,
    resizing_drag_data: Option<ResizeDragData>,
    bounds: Bounds<Pixels>,
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
                TilePanelState {
                    panel_state,
                    x: item.bounds.origin.x,
                    y: item.bounds.origin.y,
                    w: item.bounds.size.width,
                    h: item.bounds.size.height,
                    z_index: item.z_index,
                }
            })
            .collect();

        let mut state = DockItemState::new(self);
        state.info = DockItemInfo::Tiles {
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
            bounds: Bounds::default(),
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
                z_index: self
                    .panels
                    .iter()
                    .map(|item| item.z_index)
                    .max()
                    .unwrap_or(0)
                    + 1,
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
        let adjusted_position = position - self.bounds.origin;
        for (index, item) in self.panels.iter().enumerate() {
            if item.bounds.contains(&adjusted_position) {
                self.dragging_panel_index = Some(index);
                self.dragging_initial_mouse = adjusted_position;
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
                let adjusted_position = current_mouse_position - self.bounds.origin;
                let delta = adjusted_position - self.dragging_initial_mouse;
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

    fn resize_panel_height(&mut self, new_height: Pixels, cx: &mut ViewContext<'_, TilePanel>) {
        if let Some(index) = self.resizing_panel_index {
            if let Some(item) = self.panels.get_mut(index) {
                item.bounds.size.height = round_to_nearest_ten(new_height);
                cx.notify();
            }
        }
    }

    pub fn add_panel_with_z_index(
        &mut self,
        panel: Arc<dyn PanelView>,
        bounds: Bounds<Pixels>,
        z_index: usize,
        cx: &mut ViewContext<Self>,
    ) {
        self.panels.push(TilesItem {
            panel: panel.clone(),
            bounds,
            z_index,
        });

        cx.emit(PanelEvent::LayoutChanged);
        cx.notify();
    }

    /// Bring the panel of target_index to front by updating its z_index
    fn bring_panel_to_front(&mut self, target_index: Option<usize>) {
        if let Some(index) = target_index {
            let max_z_index = self
                .panels
                .iter()
                .map(|item| item.z_index)
                .max()
                .unwrap_or(0);

            if let Some(item) = self.panels.get_mut(index) {
                if item.z_index == max_z_index {
                    return;
                }
                item.z_index = (max_z_index + 1) % usize::MAX;
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

        // Sort panels by z_index
        let mut panels = self.panels.clone();
        panels.sort_by_key(|item| item.z_index);

        h_flex()
            .size_full()
            .overflow_hidden()
            .relative()
            .bg(cx.theme().background)
            .children(panels.into_iter().map(|item| {
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
                                    this.bring_panel_to_front(this.resizing_panel_index);
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
                                    this.bring_panel_to_front(this.resizing_panel_index);
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
                                    this.bring_panel_to_front(this.resizing_panel_index);
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, cx| {
                                    let initial_mouse_position = event.position;
                                    this.update_initial_position(initial_mouse_position, cx);
                                    this.bring_panel_to_front(this.dragging_panel_index);
                                }),
                            )
                            .on_drag(DragMoving(entity_id), |drag, _, cx| {
                                cx.stop_propagation();
                                cx.new_view(|_| drag.clone())
                            })
                            .on_drag_move(cx.listener(
                                move |this, e: &DragMoveEvent<DragMoving>, cx| match e.drag(cx) {
                                    DragMoving(id) => {
                                        if *id != entity_id {
                                            return;
                                        }

                                        this.update_position(e.event.position, cx);
                                    }
                                },
                            )),
                    )
            }))
            .child({
                let view = cx.view().clone();
                canvas(
                    move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |this, _event: &MouseUpEvent, cx| {
                    this.dragging_panel_index = None;
                    this.resizing_panel_index = None;
                    this.resizing_drag_data = None;
                    cx.emit(PanelEvent::LayoutChanged);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, cx| {
                    if this.resizing_panel_index.is_none() && this.dragging_panel_index.is_none() {
                        let position = event.position;
                        let adjusted_position = position - this.bounds.origin;
                        for (index, item) in this.panels.iter().enumerate() {
                            if item.bounds.contains(&adjusted_position) {
                                this.bring_panel_to_front(Some(index));
                                cx.notify();
                                break;
                            }
                        }
                    }
                }),
            )
    }
}

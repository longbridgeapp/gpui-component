use std::rc::Rc;

use gpui::{
    canvas, div, prelude::FluentBuilder, px, Along, AnyElement, AnyView, Axis, Bounds, Element,
    Entity, EntityId, EventEmitter, InteractiveElement as _, IntoElement, MouseMoveEvent,
    MouseUpEvent, ParentElement, Pixels, Render, StatefulInteractiveElement, Style, Styled, View,
    ViewContext, VisualContext as _, WindowContext,
};

use crate::{h_flex, theme::ActiveTheme, v_flex, AxisExt};

const PANEL_MIN_SIZE: Pixels = px(100.);
const HANDLE_PADDING: Pixels = px(4.);

pub enum ResizablePanelEvent {
    Resized,
}

#[derive(Clone, Render)]
pub struct DragPanel(pub (EntityId, usize, Axis));

#[derive(Clone)]
pub struct ResizablePanelGroup {
    panels: Vec<View<ResizablePanel>>,
    sizes: Vec<Pixels>,
    axis: Axis,
    handle_size: Pixels,
    size: Option<Pixels>,
    bounds: Bounds<Pixels>,
    resizing_panel_ix: Option<usize>,
}

impl ResizablePanelGroup {
    pub(super) fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            axis: Axis::Horizontal,
            sizes: Vec::new(),
            panels: Vec::new(),
            handle_size: px(1.),
            size: None,
            bounds: Bounds::default(),
            resizing_panel_ix: None,
        }
    }

    pub fn load(&mut self, sizes: Vec<Pixels>, panels: Vec<View<ResizablePanel>>) {
        self.sizes = sizes;
        self.panels = panels;
    }

    /// Set the axis of the resizable panel group, default is horizontal.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub(crate) fn set_axis(&mut self, axis: Axis, cx: &mut ViewContext<Self>) {
        self.axis = axis;
        cx.notify();
    }

    /// Set the size of the resize handle, default is 3px.
    ///
    /// The handle size will inherit the parent group handle size, if you insert a group into another group.
    pub fn handle_size(mut self, size: Pixels) -> Self {
        self.handle_size = size;
        self
    }

    /// Add a resizable panel to the group.
    pub fn child(mut self, panel: ResizablePanel, cx: &mut ViewContext<Self>) -> Self {
        self.add_child(panel, cx);
        self
    }

    /// Add a ResizablePanelGroup as a child to the group.
    pub fn group(self, group: ResizablePanelGroup, cx: &mut ViewContext<Self>) -> Self {
        let mut group: ResizablePanelGroup = group;
        group.handle_size = self.handle_size;
        let size = group.size;
        let panel = ResizablePanel::new()
            .content_view(cx.new_view(|_| group).into())
            .when_some(size, |this, size| this.size(size));
        self.child(panel, cx)
    }

    /// Set size of the resizable panel group
    ///
    /// - When the axis is horizontal, the size is the height of the group.
    /// - When the axis is vertical, the size is the width of the group.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    /// Returns the sizes of the resizable panels.
    pub(crate) fn sizes(&self) -> Vec<Pixels> {
        self.sizes.clone()
    }

    pub fn add_child(&mut self, panel: ResizablePanel, cx: &mut ViewContext<Self>) {
        let mut panel = panel;
        panel.axis = self.axis;
        panel.group = Some(cx.view().clone());
        self.sizes.push(panel.initial_size.unwrap_or_default());
        self.panels.push(cx.new_view(|_| panel));
    }

    pub fn insert_child(&mut self, panel: ResizablePanel, ix: usize, cx: &mut ViewContext<Self>) {
        let mut panel = panel;
        panel.axis = self.axis;
        panel.group = Some(cx.view().clone());

        self.sizes
            .insert(ix, panel.initial_size.unwrap_or_default());
        self.panels.insert(ix, cx.new_view(|_| panel));
        cx.notify()
    }

    /// Replace a child panel with a new panel at the given index.
    pub(crate) fn replace_child(
        &mut self,
        panel: ResizablePanel,
        ix: usize,
        cx: &mut ViewContext<Self>,
    ) {
        let mut panel = panel;

        let old_panel = self.panels[ix].clone();
        let old_panel_initial_size = old_panel.read(cx).initial_size;

        panel.initial_size = old_panel_initial_size;
        panel.axis = self.axis;
        panel.group = Some(cx.view().clone());
        self.sizes[ix] = panel.initial_size.unwrap_or_default();
        self.panels[ix] = cx.new_view(|_| panel);
        cx.notify()
    }

    pub fn remove_child(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        self.sizes.remove(ix);
        self.panels.remove(ix);
        cx.notify()
    }

    pub(crate) fn remove_all_children(&mut self, cx: &mut ViewContext<Self>) {
        self.sizes.clear();
        self.panels.clear();
        cx.notify()
    }

    fn render_resize_handle(&self, ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let axis = self.axis;
        let neg_offset = -HANDLE_PADDING;
        let view = cx.view().clone();

        div()
            .id(("resizable-handle", ix))
            .occlude()
            .absolute()
            .flex_shrink_0()
            .when(self.axis.is_horizontal(), |this| {
                this.cursor_col_resize()
                    .top_0()
                    .left(neg_offset)
                    .h_full()
                    .w(px(1.))
                    .px(HANDLE_PADDING)
            })
            .when(self.axis.is_vertical(), |this| {
                this.cursor_row_resize()
                    .top(neg_offset)
                    .left_0()
                    .w_full()
                    .h(px(1.))
                    .py(HANDLE_PADDING)
            })
            .child(
                div()
                    .bg(cx.theme().border)
                    .when(self.axis.is_horizontal(), |this| {
                        this.h_full().w(self.handle_size)
                    })
                    .when(self.axis.is_vertical(), |this| {
                        this.w_full().h(self.handle_size)
                    }),
            )
            .on_drag(
                DragPanel((cx.entity_id(), ix, axis)),
                move |drag_panel, cx| {
                    cx.stop_propagation();
                    // Set current resizing panel ix
                    view.update(cx, |view, _| {
                        view.resizing_panel_ix = Some(ix);
                    });
                    cx.new_view(|_| drag_panel.clone())
                },
            )
    }

    fn done_resizing(&mut self, cx: &mut ViewContext<Self>) {
        cx.emit(ResizablePanelEvent::Resized);
        self.resizing_panel_ix = None;
    }

    fn sync_real_panel_sizes(&mut self, cx: &WindowContext) {
        for (i, panel) in self.panels.iter().enumerate() {
            self.sizes[i] = panel.read(cx).bounds.size.along(self.axis)
        }
    }

    /// The `ix`` is the index of the panel to resize,
    /// and the `size` is the new size for the panel.
    fn resize_panels(&mut self, ix: usize, size: Pixels, cx: &mut ViewContext<Self>) {
        let mut ix = ix;
        // Only resize the left panels.
        if ix >= self.panels.len() - 1 {
            return;
        }
        let size = size.floor();
        let container_size = self.bounds.size.along(self.axis);

        self.sync_real_panel_sizes(cx);

        let mut changed = size - self.sizes[ix];
        let is_expand = changed > px(0.);

        let main_ix = ix;
        let mut new_sizes = self.sizes.clone();

        if is_expand {
            new_sizes[ix] = size;

            // Now to expand logic is correct.
            while changed > px(0.) && ix < self.panels.len() - 1 {
                ix += 1;
                let available_size = (new_sizes[ix] - PANEL_MIN_SIZE).max(px(0.));
                let to_reduce = changed.min(available_size);
                new_sizes[ix] -= to_reduce;
                changed -= to_reduce;
            }
        } else {
            let new_size = size.max(PANEL_MIN_SIZE);
            new_sizes[ix] = new_size;
            changed = size - PANEL_MIN_SIZE;
            new_sizes[ix + 1] += self.sizes[ix] - new_size;

            while changed < px(0.) && ix > 0 {
                ix -= 1;
                let available_size = self.sizes[ix] - PANEL_MIN_SIZE;
                let to_increase = (changed).min(available_size);
                new_sizes[ix] += to_increase;
                changed += to_increase;
            }
        }

        // If total size exceeds container size, adjust the main panel
        let total_size: Pixels = new_sizes.iter().map(|s| s.0).sum::<f32>().into();
        if total_size > container_size {
            let overflow = total_size - container_size;
            new_sizes[main_ix] = (new_sizes[main_ix] - overflow).max(PANEL_MIN_SIZE);
        }

        self.sizes = new_sizes;
        for (i, panel) in self.panels.iter().enumerate() {
            let size = self.sizes[i];
            if size > px(0.) {
                panel.update(cx, |this, _| this.size = Some(size));
            }
        }
    }
}
impl EventEmitter<ResizablePanelEvent> for ResizablePanelGroup {}
impl Render for ResizablePanelGroup {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();
        let container = if self.axis.is_horizontal() {
            h_flex()
        } else {
            v_flex()
        };

        container
            .size_full()
            .children(self.panels.iter().enumerate().map(|(ix, panel)| {
                if ix > 0 {
                    let handle = self.render_resize_handle(ix - 1, cx);
                    panel.update(cx, |view, _| {
                        view.resize_handle = Some(handle.into_any_element())
                    });
                }

                panel.clone()
            }))
            .child({
                canvas(
                    move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .child(ResizePanelGroupElement {
                view: cx.view().clone(),
                axis: self.axis,
            })
    }
}

pub struct ResizablePanel {
    group: Option<View<ResizablePanelGroup>>,
    /// Initial size is the size that the panel has when it is created.
    initial_size: Option<Pixels>,
    /// size is the size that the panel has when it is resized or ajusted by flex layout.
    size: Option<Pixels>,
    axis: Axis,
    content_builder: Option<Rc<dyn Fn(&mut WindowContext) -> AnyElement>>,
    content_view: Option<AnyView>,
    /// The bounds of the resizable panel, when render the bounds will be updated.
    bounds: Bounds<Pixels>,
    resize_handle: Option<AnyElement>,
}

impl ResizablePanel {
    pub(super) fn new() -> Self {
        Self {
            group: None,
            initial_size: None,
            size: None,
            axis: Axis::Horizontal,
            content_builder: None,
            content_view: None,
            bounds: Bounds::default(),
            resize_handle: None,
        }
    }

    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&mut WindowContext) -> AnyElement + 'static,
    {
        self.content_builder = Some(Rc::new(content));
        self
    }

    pub fn content_view(mut self, content: AnyView) -> Self {
        self.content_view = Some(content);
        self
    }

    /// Set the initial size of the panel.
    pub fn size(mut self, size: Pixels) -> Self {
        self.initial_size = Some(size);
        self
    }

    /// Save the real panel size, and update group sizes
    fn update_size(&mut self, bounds: Bounds<Pixels>, cx: &mut ViewContext<Self>) {
        let new_size = bounds.size.along(self.axis);
        self.bounds = bounds;
        self.size = Some(new_size);

        let panel_view = cx.view().clone();
        if let Some(group) = self.group.as_ref() {
            group.update(cx, |view, _| {
                if let Some(ix) = view
                    .panels
                    .iter()
                    .position(|v| v.entity_id() == panel_view.entity_id())
                {
                    view.sizes[ix] = new_size;
                }
            })
        }
        cx.notify();
    }
}

impl FluentBuilder for ResizablePanel {}

impl Render for ResizablePanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();

        div()
            .flex()
            .flex_grow()
            .size_full()
            .relative()
            .when(self.initial_size.is_none(), |this| this.flex_shrink())
            .when(self.axis.is_vertical(), |this| this.min_h(PANEL_MIN_SIZE))
            .when(self.axis.is_horizontal(), |this| this.min_w(PANEL_MIN_SIZE))
            .when_some(self.initial_size, |this, size| {
                // The `self.size` is None, that mean the initial size for the panel, so we need set flex_shrink_0
                // To let it keep the initial size.
                this.when(self.size.is_none(), |this| this.flex_shrink_0())
                    .flex_basis(size)
            })
            .when_some(self.size, |this, size| this.flex_basis(size))
            .child({
                canvas(
                    move |bounds, cx| view.update(cx, |r, cx| r.update_size(bounds, cx)),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .when_some(self.content_builder.clone(), |this, c| this.child(c(cx)))
            .when_some(self.content_view.clone(), |this, c| this.child(c))
            .when_some(self.resize_handle.take(), |this, c| this.child(c))
    }
}

struct ResizePanelGroupElement {
    axis: Axis,
    view: View<ResizablePanelGroup>,
}

impl IntoElement for ResizePanelGroupElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ResizePanelGroupElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        (cx.request_layout(Style::default(), None), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut WindowContext,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        cx.on_mouse_event({
            let view = self.view.clone();
            let axis = self.axis;
            let current_ix = view.read(cx).resizing_panel_ix;
            move |e: &MouseMoveEvent, phase, cx| {
                if phase.bubble() {
                    if let Some(ix) = current_ix {
                        view.update(cx, |view, cx| {
                            let panel = view
                                .panels
                                .get(ix)
                                .expect("BUG: invalid panel index")
                                .read(cx);

                            match axis {
                                Axis::Horizontal => {
                                    view.resize_panels(ix, e.position.x - panel.bounds.left(), cx)
                                }
                                Axis::Vertical => {
                                    view.resize_panels(ix, e.position.y - panel.bounds.top(), cx);
                                }
                            }
                        })
                    }
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

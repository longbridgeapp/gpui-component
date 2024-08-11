use std::rc::Rc;

use gpui::{
    canvas, div, prelude::FluentBuilder as _, px, AnyElement, AnyView, Axis, Bounds, DragMoveEvent,
    EntityId, InteractiveElement as _, IntoElement, MouseButton, ParentElement, Pixels, Render,
    StatefulInteractiveElement, Styled, View, ViewContext, VisualContext as _, WindowContext,
};

use crate::{h_flex, theme::ActiveTheme, v_flex, AxisExt};

#[derive(Clone, Render)]
pub struct DragPanel(pub (EntityId, usize, Axis));

#[derive(Clone)]
pub struct ResizablePanelGroup {
    panels: Vec<View<ResizablePanel>>,
    sizes: Vec<Pixels>,
    axis: Axis,
    handle_size: Pixels,
    size: Pixels,
    resizing_panel_ix: Option<usize>,
}

impl ResizablePanelGroup {
    pub(super) fn new() -> Self {
        Self {
            axis: Axis::Horizontal,
            sizes: Vec::new(),
            panels: Vec::new(),
            handle_size: px(3.),
            size: px(20.),
            resizing_panel_ix: None,
        }
    }

    /// Set the axis of the resizable panel group, default is horizontal.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Set the size of the resize handle, default is 3px.
    ///
    /// The handle size will inherit the parent group handle size, if you insert a group into another group.
    pub fn handle_size(mut self, size: Pixels) -> Self {
        self.handle_size = size;
        self
    }

    /// Add a resizable panel to the group.
    pub fn child(mut self, panel: ResizablePanel, cx: &mut WindowContext) -> Self {
        let mut panel = panel;
        panel.axis = self.axis;
        self.sizes.push(panel.size);
        self.panels.push(cx.new_view(|_| panel));
        self
    }

    /// Add a ResizablePanelGroup as a child to the group.
    pub fn group(self, group: ResizablePanelGroup, cx: &mut WindowContext) -> Self {
        let mut group: ResizablePanelGroup = group;
        group.handle_size = self.handle_size;
        let size = group.size;
        let panel = ResizablePanel::new()
            .content_view(cx.new_view(|_| group).into())
            .size(size);
        self.child(panel, cx)
    }

    /// Set size of the resizable panel group
    ///
    /// - When the axis is horizontal, the size is the height of the group.
    /// - When the axis is vertical, the size is the width of the group.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    fn render_resize_handle(&self, ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let axis = self.axis;
        let handle_size = self.handle_size;
        let is_resizing = self.resizing_panel_ix == Some(ix);

        div()
            .id(("resizable-handle", ix))
            .occlude()
            .hover(|this| this.bg(cx.theme().drag_border))
            .when(is_resizing, |this| this.bg(cx.theme().drag_border))
            .when(self.axis.is_horizontal(), |this| {
                this.cursor_col_resize().top_0().h_full().w(handle_size)
            })
            .when(self.axis.is_vertical(), |this| {
                this.cursor_row_resize().left_0().w_full().h(handle_size)
            })
            .on_drag_move(cx.listener(
                move |view, e: &DragMoveEvent<DragPanel>, cx| match e.drag(cx) {
                    DragPanel((entity_id, ix, axis)) => {
                        if cx.entity_id() != *entity_id {
                            return;
                        }

                        let ix = *ix;
                        view.resizing_panel_ix = Some(ix);
                        let panel = view
                            .panels
                            .get(ix)
                            .expect("BUG: invalid panel index")
                            .read(cx);

                        view.sync_real_panel_sizes(cx);
                        match axis {
                            Axis::Horizontal => {
                                view.resize_panels(ix, e.event.position.x - panel.bounds.left(), cx)
                            }
                            Axis::Vertical => {
                                view.resize_panels(ix, e.event.position.y - panel.bounds.top(), cx);
                            }
                        }
                    }
                },
            ))
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|view, _, _| {
                    if view.resizing_panel_ix.is_none() {
                        return;
                    }

                    view.resizing_panel_ix = None;
                }),
            )
            .on_drag(DragPanel((cx.entity_id(), ix, axis)), |drag_panel, cx| {
                cx.stop_propagation();
                cx.new_view(|_| drag_panel.clone())
            })
    }

    fn sync_real_panel_sizes(&mut self, cx: &WindowContext) {
        for (i, panel) in self.panels.iter_mut().enumerate() {
            if self.axis.is_horizontal() {
                self.sizes[i] = panel.read(cx).bounds.size.width;
            } else {
                self.sizes[i] = panel.read(cx).bounds.size.height;
            }
        }
    }

    /// The `ix`` is the index of the panel to resize,
    /// and the `size` is the new size for the panel.
    fn resize_panels(&mut self, ix: usize, size: Pixels, cx: &mut ViewContext<Self>) {
        // Only resize the left panels.
        if ix == self.panels.len() - 1 {
            return;
        }
        let size = size.floor();

        let old_size = self.sizes[ix];
        let new_size = self.panels[ix].read(cx).limit_size(size);
        if new_size < size {
            return;
        }
        let changed_size = new_size - old_size;

        // If change size is less than 1px, do nothing.
        if changed_size > px(-1.0) && changed_size < px(1.0) {
            return;
        }
        self.sizes[ix] = new_size;

        let next_size = self.sizes[ix + 1];
        self.sizes[ix + 1] = self.panels[ix + 1]
            .read(cx)
            .limit_size(next_size - changed_size);

        for (i, panel) in self.panels.iter_mut().enumerate() {
            let size = self.sizes[i];
            panel.update(cx, |this, _| this.size = size);
        }
    }
}

impl Render for ResizablePanelGroup {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let mut children: Vec<AnyElement> = vec![];
        for (ix, panel) in self.panels.iter().enumerate() {
            children.push(panel.clone().into_any_element());
            if ix < self.panels.len() - 1 {
                children.push(self.render_resize_handle(ix, cx).into_any_element());
            }
        }

        let container = if self.axis.is_horizontal() {
            h_flex()
        } else {
            v_flex()
        };

        container.size_full().children(children)
    }
}

pub struct ResizablePanel {
    size: Pixels,
    max_size: Option<Pixels>,
    min_size: Option<Pixels>,
    axis: Axis,
    content_builder: Option<Rc<dyn Fn(&mut WindowContext) -> AnyElement>>,
    content_view: Option<AnyView>,
    /// The bounds of the resizable panel, when render the bounds will be updated.
    bounds: Bounds<Pixels>,

    grow: bool,
}

impl ResizablePanel {
    pub(super) fn new() -> Self {
        Self {
            size: px(20.),
            axis: Axis::Horizontal,
            max_size: None,
            min_size: None,
            content_builder: None,
            content_view: None,
            bounds: Bounds::default(),
            grow: false,
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

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn max_size(mut self, max_size: Pixels) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn min_size(mut self, min_size: Pixels) -> Self {
        self.min_size = Some(min_size);
        self
    }

    fn limit_size(&self, size: Pixels) -> Pixels {
        if let Some(max_size) = self.max_size {
            if size > max_size {
                return max_size;
            }
        }

        if let Some(min_size) = self.min_size {
            if size < min_size {
                return min_size;
            }
        }

        size
    }

    /// Set the panel to grow to fill the remaining space.
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }
}

impl Render for ResizablePanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();
        let size = self.limit_size(self.size);

        div()
            .size_full()
            .relative()
            .when(self.grow, |this| this.flex_grow())
            .when(self.axis.is_vertical(), |this| this.h(size))
            .when(self.axis.is_horizontal(), |this| this.w(size))
            .child({
                canvas(
                    move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .when_some(self.content_builder.clone(), |this, c| this.child(c(cx)))
            .when_some(self.content_view.clone(), |this, c| this.child(c))
    }
}

use std::rc::Rc;

use gpui::{
    canvas, deferred, div, prelude::FluentBuilder as _, px, AnyElement, AnyView, Axis, Bounds, Div,
    DragMoveEvent, Element, ElementId, InteractiveElement as _, IntoElement, MouseDownEvent,
    ParentElement, Pixels, Render, RenderOnce, StatefulInteractiveElement, Styled, View,
    ViewContext, VisualContext as _, WindowContext,
};
use wry::WebContext;

use crate::{theme::ActiveTheme, StyledExt};

#[derive(Clone, Render)]
pub struct DragPanel(pub Axis);

pub struct ResizablePanelGroup {
    id: ElementId,
    panels: Vec<View<ResizablePanel>>,
    sizes: Vec<Pixels>,
    axis: Axis,
    /// The bounds of the resizable panel, when render the bounds will be updated.
    bounds: Bounds<Pixels>,
}

impl ResizablePanelGroup {
    pub fn new() -> Self {
        Self {
            id: "".into(),
            axis: Axis::Horizontal,
            sizes: Vec::new(),
            panels: Vec::new(),
            bounds: Bounds::default(),
        }
    }

    pub fn id(&mut self, id: impl Into<ElementId>) -> &mut Self {
        self.id = id.into();
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn panel(mut self, panel: ResizablePanel, cx: &mut WindowContext) -> Self {
        let mut panel = panel;
        panel.axis = self.axis;
        self.sizes.push(panel.size);
        self.panels.push(cx.new_view(|_| panel));
        self
    }

    fn render_resize_handle(&self, ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let axis = self.axis;
        let handle_size = px(4.);
        let offset = px(0.) - handle_size / 2.;

        deferred(
            div()
                .id("resizable-handle")
                .occlude()
                .hover(|this| this.bg(cx.theme().drag_border))
                .on_drag_move(cx.listener(move |view, e: &DragMoveEvent<DragPanel>, cx| {
                    match e.drag(cx) {
                        DragPanel(axis) => match axis {
                            Axis::Horizontal => {
                                let size = e.event.position.x - view.bounds.left();
                                view.resize_panels(ix, size, cx)
                            }
                            Axis::Vertical => {
                                let size = e.event.position.y - view.bounds.top();
                                view.resize_panels(ix, size, cx);
                            }
                        },
                    }
                }))
                .when(self.axis == Axis::Horizontal, |this| {
                    this.cursor_col_resize().top_0().w(handle_size).h_full()
                })
                .when(self.axis == Axis::Vertical, |this| {
                    this.cursor_row_resize().left_0().w_full().h(handle_size)
                })
                .on_drag(DragPanel(axis), |drag_panel, cx| {
                    cx.stop_propagation();
                    cx.new_view(|_| drag_panel.clone())
                }),
        )
    }

    fn container_size(&self) -> Pixels {
        if self.axis == Axis::Horizontal {
            self.bounds.size.width
        } else {
            self.bounds.size.height
        }
    }

    /// ix is the index of the panel to resize.
    /// size is the new size of the panel.
    ///
    /// Other panels will be resized to fill the remaining space.
    fn resize_panels(&mut self, ix: usize, size: Pixels, cx: &mut ViewContext<Self>) {
        // Only resize the middle panels.
        if ix == 0 || ix == self.panels.len() - 1 {
            return;
        }

        let old_size = self.sizes[ix];
        let size = self.panels[ix].read(cx).limit_size(size);
        self.sizes[ix] = size;
        let changed_size = size - old_size;

        let next_size = self.sizes[ix + 1];
        self.sizes[ix + 1] = self.panels[ix + 1]
            .read(cx)
            .limit_size(next_size - changed_size);

        for (i, panel) in self.panels.iter_mut().enumerate() {
            let size = self.sizes[i];
            panel.update(cx, |this, _| this.size = size);
        }
        cx.notify();
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

        div()
            .size_full()
            .flex()
            .items_center()
            .children(children)
            .child({
                let view = cx.view().clone();
                canvas(
                    move |bounds, cx| view.update(cx, |this, _cx| this.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
    }
}

pub struct ResizablePanel {
    size: Pixels,
    max_size: Option<Pixels>,
    min_size: Option<Pixels>,
    axis: Axis,
    content_builder: Option<Rc<dyn Fn(&mut WindowContext) -> AnyElement>>,
    content_view: Option<AnyView>,
}

impl ResizablePanel {
    pub fn new() -> Self {
        Self {
            size: px(20.),
            axis: Axis::Horizontal,
            max_size: None,
            min_size: None,
            content_builder: None,
            content_view: None,
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
}

impl Render for ResizablePanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let size = self.limit_size(self.size);

        div()
            .size_full()
            .relative()
            .when(self.axis == Axis::Vertical, |this| this.h(size))
            .when(self.axis == Axis::Horizontal, |this| this.w(size))
            .border_1()
            .border_color(cx.theme().border)
            .when_some(self.content_builder.clone(), |this, c| this.child(c(cx)))
            .when_some(self.content_view.clone(), |this, c| this.child(c))
    }
}

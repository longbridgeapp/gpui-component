use gpui::{
    canvas, deferred, div, prelude::FluentBuilder as _, px, Axis, Bounds, DragMoveEvent, ElementId,
    InteractiveElement as _, IntoElement, ParentElement, Pixels, Render,
    StatefulInteractiveElement, Styled, ViewContext, VisualContext as _,
};

use crate::{theme::ActiveTheme, StyledExt};

#[derive(Clone, Render)]
pub struct DragPanel(pub Axis);

pub struct ResizablePanel {
    id: ElementId,
    axis: Axis,
    size: Pixels,
    /// The bounds of the resizable panel, when render the bounds will be updated.
    bounds: Bounds<Pixels>,
}

impl ResizablePanel {
    pub fn new(id: impl Into<ElementId>, size: Pixels) -> Self {
        let axis = Axis::Horizontal;

        Self {
            id: id.into(),
            bounds: Bounds::default(),
            axis,
            size,
        }
    }

    pub fn axis(&mut self, axis: Axis) -> &mut Self {
        self.axis = axis;
        self
    }

    fn resize_panel(&mut self, size: Pixels, _cx: &mut ViewContext<Self>) {
        self.size = size;
    }

    fn render_resize_handle(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let axis = self.axis;
        let handle_size = px(3.);
        let offset = px(0.);

        deferred(
            div()
                .id("resizable-handle")
                .occlude()
                .absolute()
                .flex_none()
                .hover(|this| this.bg(cx.theme().drag_border))
                .on_drag_move(cx.listener(
                    |panel, e: &DragMoveEvent<DragPanel>, cx| match e.drag(cx) {
                        DragPanel(axis) => match axis {
                            Axis::Horizontal => {
                                let size = e.event.position.x - panel.bounds.left();
                                panel.resize_panel(size, cx);
                            }
                            Axis::Vertical => {
                                let size = panel.bounds.top() + e.event.position.y;
                                panel.resize_panel(size, cx);
                            }
                        },
                    },
                ))
                .when(self.axis == Axis::Horizontal, |this| {
                    this.cursor_col_resize()
                        .top_0()
                        .w(handle_size)
                        .right(offset)
                        .h_full()
                })
                .when(self.axis == Axis::Vertical, |this| {
                    this.cursor_row_resize()
                        .left_0()
                        .bottom(offset)
                        .w_full()
                        .h(handle_size)
                })
                .on_drag(DragPanel(axis), |panel, cx| {
                    cx.stop_propagation();
                    cx.new_view(|_| panel.clone())
                }),
        )
    }
}

impl Render for ResizablePanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let axis = self.axis;
        let size = self.size;

        div()
            .id(self.id.clone())
            .size_full()
            .relative()
            .when(axis == Axis::Vertical, |this| this.h(size))
            .when(axis == Axis::Horizontal, |this| this.w(size))
            .child({
                let view = cx.view().clone();
                canvas(
                    move |bounds, cx| view.update(cx, |this, _cx| this.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .debug_pink()
                .size_full()
            })
            .child(format!("Size: {:?}", self.bounds))
            .child(self.render_resize_handle(cx))
    }
}

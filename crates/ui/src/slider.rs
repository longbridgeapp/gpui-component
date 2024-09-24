use crate::{theme::ActiveTheme, tooltip::Tooltip};
use gpui::{
    canvas, div, prelude::FluentBuilder as _, px, relative, Axis, Bounds, DragMoveEvent, EntityId,
    EventEmitter, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement as _,
    Pixels, Point, Render, StatefulInteractiveElement as _, Styled, ViewContext,
    VisualContext as _,
};

#[derive(Clone, Render)]
pub struct DragThumb(EntityId);

pub enum SliderEvent {
    Change(f32),
}

/// A slider component.
pub struct Slider {
    axis: Axis,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    bounds: Bounds<Pixels>,
}

impl Slider {
    fn new(axis: Axis) -> Self {
        Self {
            axis,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            value: 0.0,
            bounds: Bounds::default(),
        }
    }

    pub fn horizontal() -> Self {
        Self::new(Axis::Horizontal)
    }

    /// Set the minimum value of the slider, default: 0.0
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum value of the slider, default: 100.0
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Set the step value of the slider, default: 1.0
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Set the default value of the slider, default: 0.0
    pub fn default_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Set the value of the slider.
    pub fn set_value(&mut self, value: f32, cx: &mut gpui::ViewContext<Self>) {
        self.value = value;
        cx.notify();
    }

    /// Return percentage value of the slider, range of 0.0..1.0
    fn relative_value(&self) -> f32 {
        let step = self.step;
        let value = self.value;
        let min = self.min;
        let max = self.max;

        let relative_value = (value - min) / (max - min);
        let relative_step = step / (max - min);

        let relative_value = (relative_value / relative_step).round() * relative_step;
        relative_value.clamp(0.0, 1.0)
    }

    /// Update value by mouse position
    fn update_value_by_position(
        &mut self,
        position: Point<Pixels>,
        cx: &mut gpui::ViewContext<Self>,
    ) {
        let bounds = self.bounds;
        let axis = self.axis;
        let min = self.min;
        let max = self.max;
        let step = self.step;

        let value = match axis {
            Axis::Horizontal => {
                let relative = (position.x - bounds.left()) / bounds.size.width;
                min + (max - min) * relative
            }
            Axis::Vertical => {
                let relative = (position.y - bounds.top()) / bounds.size.height;
                max - (max - min) * relative
            }
        };

        let value = (value / step).round() * step;

        self.value = value.clamp(self.min, self.max);
        cx.emit(SliderEvent::Change(self.value));
        cx.notify();
    }

    fn render_thumb(&self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        let value = self.value;
        let entity_id = cx.entity_id();

        div()
            .id("slider-thumb")
            .on_drag(DragThumb(entity_id), |drag, cx| {
                cx.stop_propagation();
                cx.new_view(|_| drag.clone())
            })
            .on_drag_move(cx.listener(
                move |view, e: &DragMoveEvent<DragThumb>, cx| match e.drag(cx) {
                    DragThumb(id) => {
                        if *id != entity_id {
                            return;
                        }

                        // set value by mouse position
                        view.update_value_by_position(e.event.position, cx)
                    }
                },
            ))
            .absolute()
            .top(px(-5.))
            .left(relative(self.relative_value()))
            .ml(-px(8.))
            .size_4()
            .rounded_full()
            .border_1()
            .border_color(cx.theme().slider_bar.opacity(0.9))
            .when(cx.theme().shadow, |this| this.shadow_md())
            .bg(cx.theme().slider_thumb)
            .tooltip(move |cx| Tooltip::new(format!("{}", value), cx))
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut gpui::ViewContext<Self>) {
        self.update_value_by_position(event.position, cx);
    }
}

impl EventEmitter<SliderEvent> for Slider {}

impl Render for Slider {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("slider")
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .h_5()
            .child(
                div()
                    .id("slider-bar")
                    .relative()
                    .w_full()
                    .my_1p5()
                    .h_1p5()
                    .bg(cx.theme().slider_bar.opacity(0.2))
                    .active(|this| this.bg(cx.theme().slider_bar.opacity(0.4)))
                    .rounded(px(3.))
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .h_full()
                            .w(relative(self.relative_value()))
                            .bg(cx.theme().slider_bar)
                            .rounded_l(px(3.)),
                    )
                    .child(self.render_thumb(cx))
                    .child({
                        let view = cx.view().clone();
                        canvas(
                            move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                            |_, _, _| {},
                        )
                        .absolute()
                        .size_full()
                    }),
            )
    }
}

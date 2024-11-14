use gpui::{
    div, prelude::FluentBuilder as _, Div, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, Stateful, Styled, WeakView, WindowContext,
};

use crate::{
    button::{Button, ButtonStyle, ButtonStyled},
    button_group::ButtonGroup,
    IconName, Selectable as _, Sizable, Size,
};

use super::{DockArea, DockPlacement};

#[derive(IntoElement)]
pub struct ToggleButtons {
    base: Stateful<Div>,
    dock_area: WeakView<DockArea>,
    size: Size,
    style: ButtonStyle,
}

impl ToggleButtons {
    /// Create a new instance of the toggle buttons.
    pub fn new(dock_area: WeakView<DockArea>) -> Self {
        Self {
            dock_area,
            base: div().id("dock-toggle-buttons"),
            style: ButtonStyle::Outline,
            size: Size::Medium,
        }
    }
}

impl Sizable for ToggleButtons {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}
impl ButtonStyled for ToggleButtons {
    fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
}
impl Styled for ToggleButtons {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ToggleButtons {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let Some(dock_area) = self.dock_area.upgrade() else {
            return self.base;
        };

        let left_dock: Option<bool> = dock_area
            .read(cx)
            .has_dock(DockPlacement::Left)
            .then(|| dock_area.read(cx).is_dock_open(DockPlacement::Left, cx));
        let right_dock: Option<bool> = dock_area
            .read(cx)
            .has_dock(DockPlacement::Right)
            .then(|| dock_area.read(cx).is_dock_open(DockPlacement::Right, cx));
        let bottom_dock: Option<bool> = dock_area
            .read(cx)
            .has_dock(DockPlacement::Bottom)
            .then(|| dock_area.read(cx).is_dock_open(DockPlacement::Bottom, cx));

        self.base.child(
            ButtonGroup::new("toggle-docks")
                .with_style(self.style)
                .with_size(self.size)
                .when_some(left_dock, |this, open| {
                    this.child(
                        Button::new("toggle-left-dock")
                            .icon(IconName::PanelLeft)
                            .selected(open),
                    )
                })
                .when_some(bottom_dock, |this, open| {
                    this.child(
                        Button::new("toggle-bottom-dock")
                            .icon(IconName::PanelBottom)
                            .selected(open),
                    )
                })
                .when_some(right_dock, |this, open| {
                    this.child(
                        Button::new("toggle-right-dock")
                            .icon(IconName::PanelRight)
                            .selected(open),
                    )
                })
                .on_click(move |indexes, cx| {
                    if let Some(ix) = indexes.first() {
                        let placement = match ix {
                            0 => DockPlacement::Left,
                            1 => DockPlacement::Bottom,
                            2 => DockPlacement::Right,
                            _ => DockPlacement::Left,
                        };

                        dock_area.update(cx, |this, cx| {
                            this.toggle_dock(placement, cx);
                        })
                    }
                }),
        )
    }
}

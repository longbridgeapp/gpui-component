use std::time::Duration;

use crate::{
    h_flex,
    theme::{ActiveTheme, Colorize},
    Disableable, Sizable, Size,
};
use gpui::{
    div, prelude::FluentBuilder as _, px, Animation, AnimationExt as _, Div, ElementId,
    InteractiveElement, IntoElement, ParentElement as _, RenderOnce, SharedString, Stateful,
    Styled as _, WindowContext,
};

type OnClick = Box<dyn Fn(&bool, &mut WindowContext) + 'static>;

pub enum LabelSide {
    Left,
    Right,
}

impl LabelSide {
    fn left(&self) -> bool {
        matches!(self, Self::Left)
    }
}

#[derive(IntoElement)]
pub struct Switch {
    id: SharedString,
    base: Stateful<Div>,
    checked: bool,
    disabled: bool,
    label: Option<SharedString>,
    label_side: LabelSide,
    on_click: Option<OnClick>,
    size: Size,
}

impl Switch {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id: SharedString = id.into();

        Self {
            id: id.clone(),
            base: div().id(id),
            checked: false,
            disabled: false,
            label: None,
            on_click: None,
            label_side: LabelSide::Right,
            size: Size::Medium,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&bool, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn label_side(mut self, label_side: LabelSide) -> Self {
        self.label_side = label_side;
        self
    }
}

impl Sizable for Switch {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Disableable for Switch {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        let theme = cx.theme();
        let group_id = format!("switch_group_{:?}", self.id);
        let checked = self.checked;

        let (bg, toggle_bg) = match self.checked {
            true => (theme.primary, theme.background),
            false => (theme.input, theme.background),
        };

        let (bg, toggle_bg) = match self.disabled {
            true => (bg.opacity(0.3), toggle_bg.opacity(0.8)),
            false => (bg, toggle_bg),
        };

        let (bg_width, bg_height) = match self.size {
            Size::XSmall | Size::Small => (px(28.), px(16.)),
            _ => (px(36.), px(20.)),
        };
        let bar_width = match self.size {
            Size::XSmall | Size::Small => px(12.),
            _ => px(16.),
        };
        let inset = px(2.);

        h_flex()
            .id(self.id)
            .group(group_id)
            .items_center()
            .gap_2()
            .when(self.label_side.left(), |this| this.flex_row_reverse())
            .child(
                // Switch Bar
                self.base
                    .w(bg_width)
                    .h(bg_height)
                    .rounded(bg_height / 2.)
                    .flex()
                    .items_center()
                    .border(inset)
                    .border_color(theme.transparent)
                    .bg(bg)
                    .when(!self.disabled, |this| this.cursor_pointer())
                    .child(
                        // Switch Toggle
                        div()
                            .rounded_full()
                            .bg(toggle_bg)
                            .size(bar_width)
                            .with_animation(
                                ElementId::NamedInteger("move".into(), checked as usize),
                                Animation::new(Duration::from_secs_f64(0.15)),
                                move |this, delta| {
                                    let max_x = bg_width - bar_width - inset * 2;
                                    let x = if checked {
                                        max_x * delta
                                    } else {
                                        max_x - max_x * delta
                                    };
                                    this.left(x)
                                },
                            ),
                    ),
            )
            .when_some(self.label, |this, label| {
                this.child(div().child(label).map(|this| match self.size {
                    Size::XSmall | Size::Small => this.text_sm(),
                    _ => this.text_base(),
                }))
            })
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_mouse_down(gpui::MouseButton::Left, move |_, cx| {
                        cx.stop_propagation();
                        on_click(&!self.checked, cx);
                    })
                },
            )
    }
}

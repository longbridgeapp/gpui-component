use std::sync::Arc;
use std::time::Duration;

use crate::{h_flex, theme::ActiveTheme, Disableable, Sizable, Size};
use gpui::{
    div, prelude::FluentBuilder as _, px, Animation, AnimationExt as _, Div, ElementId,
    InteractiveElement, IntoElement, ParentElement as _, RenderOnce, SharedString, Stateful,
    Styled as _, WindowContext,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SwitchState {
    checked: bool,
    prev_checked: Option<bool>,
}

impl SwitchState {
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            prev_checked: None,
        }
    }

    pub fn get(&self) -> bool {
        self.checked
    }

    pub fn set(&mut self, checked: bool) {
        self.prev_checked = Some(self.checked);
        self.checked = checked;
    }

    pub fn toggle(&mut self) {
        self.prev_checked = Some(self.checked);
        self.checked = !self.checked;
    }

    fn toggled(mut self) -> Self {
        self.prev_checked = Some(self.checked);
        self.checked = !self.checked;
        self
    }

    fn refreshed(mut self) -> Self {
        self.prev_checked = Some(self.checked);
        self
    }
}

impl From<bool> for SwitchState {
    fn from(checked: bool) -> Self {
        SwitchState::new(checked)
    }
}
impl From<SwitchState> for bool {
    fn from(state: SwitchState) -> Self {
        state.checked
    }
}

impl From<&SwitchState> for bool {
    fn from(state: &SwitchState) -> Self {
        state.checked
    }
}

type OnClick = Arc<dyn Fn(&SwitchState, &mut WindowContext) + 'static>;

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
    id: ElementId,
    base: Stateful<Div>,
    state: SwitchState,
    disabled: bool,
    label: Option<SharedString>,
    label_side: LabelSide,
    on_click: Option<OnClick>,
    size: Size,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id: ElementId = id.into();
        Self {
            id: id.clone(),
            base: div().id(id),
            state: SwitchState::new(false),
            disabled: false,
            label: None,
            on_click: None,
            label_side: LabelSide::Right,
            size: Size::Medium,
        }
    }

    pub fn checked(mut self, state: SwitchState) -> Self {
        self.state = state;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&SwitchState, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
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
        let checked = self.state.get();
        let prev_checked = self.state.prev_checked;
        eprintln!("prev_checked={:?}, checked={:?}", prev_checked, checked);

        let (bg, toggle_bg) = match checked {
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
                            .map(|this| {
                                if prev_checked.map_or(false, |prev| prev != checked) {
                                    this.with_animation(
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
                                    )
                                    .into_any_element()
                                } else {
                                    let max_x = bg_width - bar_width - inset * 2;
                                    let x = if checked { max_x } else { px(0.) };
                                    this.left(x).into_any_element()
                                }
                            }),
                    ),
            )
            .when_some(self.label, |this, label| {
                this.child(div().child(label).map(|this| match self.size {
                    Size::XSmall | Size::Small => this.text_sm(),
                    _ => this.text_base(),
                }))
            })
            .when_some(
                self.on_click.clone().filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_mouse_down(gpui::MouseButton::Left, move |_, cx| {
                        cx.stop_propagation();
                        on_click(&self.state.toggled(), cx);
                    })
                },
            )
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_mouse_move(move |_, cx| {
                        cx.stop_propagation();
                        on_click(&self.state.refreshed(), cx);
                    })
                },
            )
    }
}

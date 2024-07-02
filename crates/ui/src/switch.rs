use crate::{
    button::ButtonSize,
    stock::h_flex,
    theme::{ActiveTheme, Colorize},
    Disableable,
};
use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, Div, InteractiveElement, IntoElement,
    ParentElement as _, RenderOnce, SharedString, Stateful, StatefulInteractiveElement,
    Styled as _, WindowContext,
};

type OnClick = Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>;

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
    size: ButtonSize,
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
            size: ButtonSize::Medium,
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

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn label_side(mut self, label_side: LabelSide) -> Self {
        self.label_side = label_side;
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

        let (bg, toggle_bg) = match self.checked {
            true => (theme.primary, theme.background),
            false => (theme.input, theme.background),
        };

        let (bg, toggle_bg) = match self.disabled {
            true => (bg.opacity(0.3), toggle_bg.opacity(0.8)),
            false => (bg, toggle_bg),
        };

        h_flex()
            .id(self.id)
            .group(group_id)
            .items_center()
            .gap_2()
            .when(self.label_side.left(), |this| this.flex_row_reverse())
            .child(
                self.base
                    .map(|this| match self.size {
                        ButtonSize::Medium => this.w_11().h_6().rounded_xl(),
                        ButtonSize::XSmall | ButtonSize::Small => this.w_8().h_4().rounded_lg(),
                    })
                    .flex()
                    .items_center()
                    .border_2()
                    .border_color(theme.transparent)
                    .bg(bg)
                    .when(!self.disabled, |this| this.cursor_pointer())
                    .map(|this| match self.checked {
                        true => this.flex_row_reverse(),
                        false => this,
                    })
                    .child(
                        div()
                            .rounded_full()
                            .bg(toggle_bg)
                            .map(|this| match self.size {
                                ButtonSize::Medium => this.w_5().h_5(),
                                ButtonSize::XSmall | ButtonSize::Small => this.w_3().h_3(),
                            }),
                    ),
            )
            .when_some(self.label, |this, label| {
                this.child(div().child(label).map(|this| match self.size {
                    ButtonSize::Medium => this.text_base(),
                    ButtonSize::XSmall | ButtonSize::Small => this.text_sm(),
                }))
            })
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_click(move |ev, cx| {
                        cx.stop_propagation();
                        on_click(ev, cx);
                    })
                },
            )
    }
}

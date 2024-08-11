use gpui::{
    px, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};

use ui::{
    button::{Button, ButtonCustomStyle},
    checkbox::Checkbox,
    h_flex,
    prelude::FluentBuilder,
    theme::ActiveTheme,
    v_flex, Disableable as _, Icon, IconName, Selectable as _, Sizable as _,
};

use crate::section;

pub struct ButtonStory {
    disabled: bool,
    loading: bool,
    selected: bool,
    compact: bool,
}

impl ButtonStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self {
            disabled: false,
            loading: false,
            selected: false,
            compact: false,
        })
    }

    fn on_click(ev: &ClickEvent, _: &mut WindowContext) {
        println!("Button clicked! {:?}", ev);
    }
}

impl Render for ButtonStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let disabled = self.disabled;
        let loading = self.loading;
        let selected = self.selected;
        let compact = self.compact;

        v_flex()
            .gap_6()
            .child(
                h_flex()
                    .gap_3()
                    .child("State")
                    .child(
                        Checkbox::new("disabled-button")
                            .label("Disabled")
                            .checked(self.disabled)
                            .on_click(cx.listener(|view, _, cx| {
                                view.disabled = !view.disabled;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new("loading-button")
                            .label("Loading")
                            .checked(self.loading)
                            .on_click(cx.listener(|view, _, cx| {
                                view.loading = !view.loading;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new("selected-button")
                            .label("Selected")
                            .checked(self.selected)
                            .on_click(cx.listener(|view, _, cx| {
                                view.selected = !view.selected;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new("compact-button")
                            .label("Compact")
                            .checked(self.compact)
                            .on_click(cx.listener(|view, _, cx| {
                                view.compact = !view.compact;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        section("Normal Button", cx)
                            .child(
                                Button::new("button-1", cx)
                                    .primary()
                                    .label("Primary Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-2", cx)
                                    .label("Secondary Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-4", cx)
                                    .danger()
                                    .label("Danger Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-5", cx)
                                    .outline()
                                    .label("Outline Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-5-ghost", cx)
                                    .ghost()
                                    .label("Ghost Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-5-link", cx)
                                    .link()
                                    .label("Link Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-5-text", cx)
                                    .text()
                                    .label("Text Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-6-custom", cx)
                                    .custom(
                                        ButtonCustomStyle::new(cx)
                                            .color(cx.theme().muted)
                                            .foreground(cx.theme().destructive)
                                            .border(cx.theme().scrollbar)
                                            .hover(cx.theme().tab_active_foreground)
                                            .active(cx.theme().selection),
                                    )
                                    .label("Custom Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            ),
                    )
                    .child(
                        section("Button with Icon", cx)
                            .child(
                                Button::new("button-icon-1", cx)
                                    .primary()
                                    .label("Confirm")
                                    .icon(IconName::Check)
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-2", cx)
                                    .label("Abort")
                                    .icon(IconName::Close)
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-3", cx)
                                    .label("Maximize")
                                    .icon(Icon::new(IconName::Maximize))
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-4", cx)
                                    .primary()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child("Custom Child")
                                            .child(IconName::ChevronDown)
                                            .child(IconName::Eye),
                                    )
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-5-ghost", cx)
                                    .ghost()
                                    .icon(IconName::Check)
                                    .label("Confirm")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-6-link", cx)
                                    .link()
                                    .icon(IconName::Check)
                                    .label("Link")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-6-text", cx)
                                    .text()
                                    .icon(IconName::Check)
                                    .label("Text Button")
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        section("Small Size", cx)
                            .child(
                                Button::new("button-6", cx)
                                    .label("Primary Button")
                                    .primary()
                                    .small()
                                    .loading(true)
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-7", cx)
                                    .label("Secondary Button")
                                    .small()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8", cx)
                                    .label("Danger Button")
                                    .danger()
                                    .small()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8-outline", cx)
                                    .label("Outline Button")
                                    .outline()
                                    .small()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8-ghost", cx)
                                    .label("Ghost Button")
                                    .ghost()
                                    .small()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8-link", cx)
                                    .label("Link Button")
                                    .link()
                                    .small()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            ),
                    )
                    .child(
                        section("XSmall Size", cx)
                            .child(
                                Button::new("button-xs-1", cx)
                                    .label("Primary Button")
                                    .primary()
                                    .small()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-2", cx)
                                    .label("Secondary Button")
                                    .xsmall()
                                    .loading(true)
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3", cx)
                                    .label("Danger Button")
                                    .danger()
                                    .xsmall()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3-ghost", cx)
                                    .label("Ghost Button")
                                    .ghost()
                                    .xsmall()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3-outline", cx)
                                    .label("Outline Button")
                                    .outline()
                                    .xsmall()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3-link", cx)
                                    .label("Link Button")
                                    .link()
                                    .xsmall()
                                    .disabled(disabled)
                                    .selected(selected)
                                    .loading(loading)
                                    .when(compact, |this| this.compact())
                                    .on_click(Self::on_click),
                            ),
                    ),
            )
            .child(
                section("Icon Button", cx)
                    .child(
                        Button::new("icon-button-primary", cx)
                            .icon(IconName::Search)
                            .primary()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-secondary", cx)
                            .icon(IconName::Info)
                            .loading(true)
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-danger", cx)
                            .icon(IconName::Close)
                            .danger()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-small-primary", cx)
                            .icon(IconName::Search)
                            .small()
                            .primary()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-outline", cx)
                            .icon(IconName::Search)
                            .outline()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-ghost", cx)
                            .icon(IconName::ArrowLeft)
                            .ghost()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    ),
            )
            .child(
                section("Icon Button", cx)
                    .child(
                        Button::new("icon-button-4", cx)
                            .icon(IconName::Info)
                            .small()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-5", cx)
                            .icon(IconName::Close)
                            .small()
                            .danger()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-6", cx)
                            .icon(IconName::Search)
                            .small()
                            .primary()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-7", cx)
                            .icon(IconName::Info)
                            .xsmall()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-8", cx)
                            .icon(IconName::Close)
                            .xsmall()
                            .danger()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    )
                    .child(
                        Button::new("icon-button-9", cx)
                            .icon(IconName::Heart)
                            .size(px(24.))
                            .ghost()
                            .disabled(disabled)
                            .selected(selected)
                            .loading(loading)
                            .when(compact, |this| this.compact()),
                    ),
            )
    }
}

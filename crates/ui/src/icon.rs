use std::ops::Deref;

use crate::{button::ButtonSize, theme::ActiveTheme};
use gpui::{
    prelude::FluentBuilder as _, svg, AnyElement, AnyView, Hsla, IntoElement, Render, RenderOnce,
    SharedString, StyleRefinement, Styled, Svg, View, VisualContext, WindowContext,
};

#[derive(IntoElement)]
pub enum IconName {
    Check,
    Minus,
    Dash,
    Maximize,
    Minimize,
    Close,
    ChevronDown,
    ChevronsUpDown,
    Plus,
    Info,
    Ellipsis,
    EllipsisVertical,
    Search,
    Delete,
    CicleX,
}

impl IconName {
    pub fn path(self) -> SharedString {
        match self {
            IconName::Check => "icons/check.svg",
            IconName::Minus => "icons/minus.svg",
            IconName::Dash => "icons/dash.svg",
            IconName::Maximize => "icons/maximize.svg",
            IconName::Minimize => "icons/minimize.svg",
            IconName::Close => "icons/close.svg",
            IconName::ChevronDown => "icons/chevron-down.svg",
            IconName::ChevronsUpDown => "icons/chevrons-up-down.svg",
            IconName::Plus => "icons/plus.svg",
            IconName::Info => "icons/info.svg",
            IconName::Ellipsis => "icons/ellipsis.svg",
            IconName::EllipsisVertical => "icons/ellipsis-vertical.svg",
            IconName::Search => "icons/search.svg",
            IconName::Delete => "icons/delete.svg",
            IconName::CicleX => "icons/circle-x.svg",
        }
        .into()
    }

    /// Return the icon as a View<Icon>
    pub fn view(self, cx: &mut WindowContext) -> View<Icon> {
        Icon::new(self).view(cx)
    }
}

impl From<IconName> for Icon {
    fn from(val: IconName) -> Self {
        Icon::new(val)
    }
}

impl From<IconName> for AnyElement {
    fn from(val: IconName) -> Self {
        Icon::new(val).into_any_element()
    }
}

impl RenderOnce for IconName {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        Icon::new(self)
    }
}

#[derive(IntoElement)]
pub struct Icon {
    base: Svg,
    path: SharedString,
    text_color: Option<Hsla>,
    size: ButtonSize,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            base: svg().flex_none().size_4(),
            path: name.path(),
            text_color: None,
            size: ButtonSize::Medium,
        }
    }

    /// Set the icon path of the Assets bundle
    ///
    /// For example: `icons/foo.svg`
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;

        self
    }

    /// Create a new view for the icon
    pub fn view(self, cx: &mut WindowContext) -> View<Icon> {
        cx.new_view(|_| self)
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }

    fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let text_color = self.text_color.unwrap_or_else(|| cx.theme().foreground);

        self.base
            .text_color(text_color)
            .map(|this| match self.size {
                ButtonSize::XSmall => this.size_3(),
                ButtonSize::Small => this.size_3p5(),
                ButtonSize::Medium => this.size_4(),
            })
            .path(self.path)
    }
}

impl From<Icon> for AnyElement {
    fn from(val: Icon) -> Self {
        val.into_any_element()
    }
}

impl Render for Icon {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let text_color = self.text_color.unwrap_or_else(|| cx.theme().foreground);

        svg()
            .flex_none()
            .size_4()
            .text_color(text_color)
            .map(|this| match self.size {
                ButtonSize::XSmall => this.size_3(),
                ButtonSize::Small => this.size_3p5(),
                ButtonSize::Medium => this.size_4(),
            })
            .path(self.path.clone())
    }
}

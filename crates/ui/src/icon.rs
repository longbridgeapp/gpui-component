use crate::{theme::ActiveTheme, Size};
use gpui::{
    prelude::FluentBuilder as _, svg, AnyElement, Hsla, IntoElement, Render, RenderOnce,
    SharedString, StyleRefinement, Styled, Svg, View, VisualContext, WindowContext,
};

#[derive(IntoElement, Clone)]
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
    CircleX,
    Loader,
    LoaderCircle,
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
            IconName::CircleX => "icons/circle-x.svg",
            IconName::Loader => "icons/loader.svg",
            IconName::LoaderCircle => "icons/loader-circle.svg",
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
    size: Size,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            base: svg().flex_none().size_4(),
            path: "".into(),
            text_color: None,
            size: Size::Medium,
        }
    }
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        Self::default().path(self.path.clone()).size(self.size)
    }
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self::default().path(name.path())
    }

    /// Set the icon path of the Assets bundle
    ///
    /// For example: `icons/foo.svg`
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
        self
    }

    /// Set the size of the icon, default is `IconSize::Medium`
    ///
    /// Also can receive a `ButtonSize` to convert to `IconSize`,
    /// Or a `Pixels` to set a custom size: `px(30.)`
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();

        self
    }

    /// Create a new view for the icon
    pub fn view(self, cx: &mut WindowContext) -> View<Icon> {
        cx.new_view(|_| self)
    }

    pub fn transform(mut self, transformation: gpui::Transformation) -> Self {
        self.base = self.base.with_transformation(transformation);
        self
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
                Size::Size(px) => this.size(px),
                Size::XSmall => this.size_3(),
                Size::Small => this.size_3p5(),
                Size::Medium => this.size_4(),
                Size::Large => this.size_6(),
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
                Size::Size(px) => this.size(px),
                Size::XSmall => this.size_3(),
                Size::Small => this.size_3p5(),
                Size::Medium => this.size_4(),
                Size::Large => this.size_6(),
            })
            .path(self.path.clone())
    }
}

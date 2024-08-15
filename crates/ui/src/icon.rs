use crate::{theme::ActiveTheme, Sizable, Size};
use gpui::{
    prelude::FluentBuilder as _, svg, AnyElement, Hsla, IntoElement, Render, RenderOnce,
    SharedString, StyleRefinement, Styled, Svg, View, VisualContext, WindowContext,
};

#[derive(IntoElement, Clone)]
pub enum IconName {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Asterisk,
    Bell,
    Calendar,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronsUpDown,
    CircleCheck,
    CircleX,
    Close,
    Copy,
    Dash,
    Delete,
    Ellipsis,
    EllipsisVertical,
    Eye,
    EyeOff,
    GitHub,
    Globe,
    Heart,
    HeartOff,
    Inbox,
    Info,
    Loader,
    LoaderCircle,
    Maximize,
    Menu,
    Minimize,
    Minus,
    Moon,
    Plus,
    Search,
    SortAscending,
    SortDescending,
    Star,
    StarOff,
    Sun,
    ThumbsDown,
    ThumbsUp,
    TriangleAlert,
}

impl IconName {
    pub fn path(self) -> SharedString {
        match self {
            IconName::ArrowDown => "icons/arrow-down.svg",
            IconName::ArrowLeft => "icons/arrow-left.svg",
            IconName::ArrowRight => "icons/arrow-right.svg",
            IconName::ArrowUp => "icons/arrow-up.svg",
            IconName::Asterisk => "icons/asterisk.svg",
            IconName::Bell => "icons/bell.svg",
            IconName::Calendar => "icons/calendar.svg",
            IconName::Check => "icons/check.svg",
            IconName::ChevronDown => "icons/chevron-down.svg",
            IconName::ChevronLeft => "icons/chevron-left.svg",
            IconName::ChevronRight => "icons/chevron-right.svg",
            IconName::ChevronUp => "icons/chevron-up.svg",
            IconName::ChevronsUpDown => "icons/chevrons-up-down.svg",
            IconName::CircleCheck => "icons/circle-check.svg",
            IconName::CircleX => "icons/circle-x.svg",
            IconName::Close => "icons/close.svg",
            IconName::Copy => "icons/copy.svg",
            IconName::Dash => "icons/dash.svg",
            IconName::Delete => "icons/delete.svg",
            IconName::Ellipsis => "icons/ellipsis.svg",
            IconName::EllipsisVertical => "icons/ellipsis-vertical.svg",
            IconName::Eye => "icons/eye.svg",
            IconName::EyeOff => "icons/eye-off.svg",
            IconName::GitHub => "icons/github.svg",
            IconName::Globe => "icons/globe.svg",
            IconName::Heart => "icons/heart.svg",
            IconName::HeartOff => "icons/heart-off.svg",
            IconName::Inbox => "icons/inbox.svg",
            IconName::Info => "icons/info.svg",
            IconName::Loader => "icons/loader.svg",
            IconName::LoaderCircle => "icons/loader-circle.svg",
            IconName::Maximize => "icons/maximize.svg",
            IconName::Menu => "icons/menu.svg",
            IconName::Minimize => "icons/minimize.svg",
            IconName::Minus => "icons/minus.svg",
            IconName::Moon => "icons/moon.svg",
            IconName::Plus => "icons/plus.svg",
            IconName::Search => "icons/search.svg",
            IconName::SortAscending => "icons/sort-ascending.svg",
            IconName::SortDescending => "icons/sort-descending.svg",
            IconName::Star => "icons/star.svg",
            IconName::StarOff => "icons/star-off.svg",
            IconName::Sun => "icons/sun.svg",
            IconName::ThumbsDown => "icons/thumbs-down.svg",
            IconName::ThumbsUp => "icons/thumbs-up.svg",
            IconName::TriangleAlert => "icons/triangle-alert.svg",
        }
        .into()
    }

    /// Return the icon as a View<Icon>
    pub fn view(self, cx: &mut WindowContext) -> View<Icon> {
        Icon::build(self).view(cx)
    }
}

impl From<IconName> for Icon {
    fn from(val: IconName) -> Self {
        Icon::build(val)
    }
}

impl From<IconName> for AnyElement {
    fn from(val: IconName) -> Self {
        Icon::build(val).into_any_element()
    }
}

impl RenderOnce for IconName {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        Icon::build(self)
    }
}

#[derive(IntoElement)]
pub struct Icon {
    base: Svg,
    path: SharedString,
    text_color: Option<Hsla>,
    size: Option<Size>,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            base: svg().flex_none().size_4(),
            path: "".into(),
            text_color: None,
            size: None,
        }
    }
}

impl Clone for Icon {
    fn clone(&self) -> Self {
        let mut this = Self::default().path(self.path.clone());
        if let Some(size) = self.size {
            this = this.with_size(size);
        }
        this
    }
}

pub trait IconNamed {
    fn path(&self) -> SharedString;
}

impl Icon {
    pub fn new(icon: impl Into<Icon>) -> Self {
        icon.into()
    }

    fn build(name: IconName) -> Self {
        Self::default().path(name.path())
    }

    /// Set the icon path of the Assets bundle
    ///
    /// For example: `icons/foo.svg`
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
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

    pub fn empty() -> Self {
        Self::default()
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

impl Sizable for Icon {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let text_color = self.text_color.unwrap_or_else(|| cx.text_style().color);

        self.base
            .text_color(text_color)
            .when_some(self.size, |this, size| match size {
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
            .text_color(text_color)
            .when_some(self.size, |this, size| match size {
                Size::Size(px) => this.size(px),
                Size::XSmall => this.size_3(),
                Size::Small => this.size_3p5(),
                Size::Medium => this.size_4(),
                Size::Large => this.size_6(),
            })
            .path(self.path.clone())
    }
}

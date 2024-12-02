use crate::{theme::ActiveTheme, Sizable, Size};
use gpui::{
    prelude::FluentBuilder as _, svg, AnyElement, Hsla, IntoElement, Radians, Render, RenderOnce,
    SharedString, StyleRefinement, Styled, Svg, Transformation, View, VisualContext, WindowContext,
};

#[derive(IntoElement, Clone)]
pub enum IconName {
    ALargeSmall,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Asterisk,
    Bell,
    BookOpen,
    Bot,
    Calendar,
    ChartPie,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronsUpDown,
    CircleCheck,
    CircleUser,
    CircleX,
    Close,
    Copy,
    Dash,
    Delete,
    Ellipsis,
    EllipsisVertical,
    Eye,
    EyeOff,
    Frame,
    GalleryVerticalEnd,
    GitHub,
    Globe,
    Heart,
    HeartOff,
    Inbox,
    Info,
    LayoutDashboard,
    Loader,
    LoaderCircle,
    Map,
    Maximize,
    Menu,
    Minimize,
    Minus,
    Moon,
    Palette,
    PanelBottom,
    PanelBottomOpen,
    PanelLeft,
    PanelLeftClose,
    PanelLeftOpen,
    PanelRight,
    PanelRightClose,
    PanelRightOpen,
    Plus,
    Search,
    Settings,
    Settings2,
    SortAscending,
    SortDescending,
    SquareTerminal,
    Star,
    StarOff,
    Sun,
    ThumbsDown,
    ThumbsUp,
    TriangleAlert,
    WindowClose,
    WindowMaximize,
    WindowMinimize,
    WindowRestore,
}

impl IconName {
    pub fn path(self) -> SharedString {
        match self {
            IconName::ALargeSmall => "icons/a-large-small.svg",
            IconName::ArrowDown => "icons/arrow-down.svg",
            IconName::ArrowLeft => "icons/arrow-left.svg",
            IconName::ArrowRight => "icons/arrow-right.svg",
            IconName::ArrowUp => "icons/arrow-up.svg",
            IconName::Asterisk => "icons/asterisk.svg",
            IconName::Bell => "icons/bell.svg",
            IconName::BookOpen => "icons/book-open.svg",
            IconName::Bot => "icons/bot.svg",
            IconName::Calendar => "icons/calendar.svg",
            IconName::ChartPie => "icons/chart-pie.svg",
            IconName::Check => "icons/check.svg",
            IconName::ChevronDown => "icons/chevron-down.svg",
            IconName::ChevronLeft => "icons/chevron-left.svg",
            IconName::ChevronRight => "icons/chevron-right.svg",
            IconName::ChevronUp => "icons/chevron-up.svg",
            IconName::ChevronsUpDown => "icons/chevrons-up-down.svg",
            IconName::CircleCheck => "icons/circle-check.svg",
            IconName::CircleUser => "icons/circle-user.svg",
            IconName::CircleX => "icons/circle-x.svg",
            IconName::Close => "icons/close.svg",
            IconName::Copy => "icons/copy.svg",
            IconName::Dash => "icons/dash.svg",
            IconName::Delete => "icons/delete.svg",
            IconName::Ellipsis => "icons/ellipsis.svg",
            IconName::EllipsisVertical => "icons/ellipsis-vertical.svg",
            IconName::Eye => "icons/eye.svg",
            IconName::EyeOff => "icons/eye-off.svg",
            IconName::Frame => "icons/frame.svg",
            IconName::GalleryVerticalEnd => "icons/gallery-vertical-end.svg",
            IconName::GitHub => "icons/github.svg",
            IconName::Globe => "icons/globe.svg",
            IconName::Heart => "icons/heart.svg",
            IconName::HeartOff => "icons/heart-off.svg",
            IconName::Inbox => "icons/inbox.svg",
            IconName::Info => "icons/info.svg",
            IconName::LayoutDashboard => "icons/layout-dashboard.svg",
            IconName::Loader => "icons/loader.svg",
            IconName::LoaderCircle => "icons/loader-circle.svg",
            IconName::Map => "icons/map.svg",
            IconName::Maximize => "icons/maximize.svg",
            IconName::Menu => "icons/menu.svg",
            IconName::Minimize => "icons/minimize.svg",
            IconName::Minus => "icons/minus.svg",
            IconName::Moon => "icons/moon.svg",
            IconName::Palette => "icons/palette.svg",
            IconName::PanelBottom => "icons/panel-bottom.svg",
            IconName::PanelBottomOpen => "icons/panel-bottom-open.svg",
            IconName::PanelLeft => "icons/panel-left.svg",
            IconName::PanelLeftClose => "icons/panel-left-close.svg",
            IconName::PanelLeftOpen => "icons/panel-left-open.svg",
            IconName::PanelRight => "icons/panel-right.svg",
            IconName::PanelRightClose => "icons/panel-right-close.svg",
            IconName::PanelRightOpen => "icons/panel-right-open.svg",
            IconName::Plus => "icons/plus.svg",
            IconName::Search => "icons/search.svg",
            IconName::Settings => "icons/settings.svg",
            IconName::Settings2 => "icons/settings-2.svg",
            IconName::SortAscending => "icons/sort-ascending.svg",
            IconName::SortDescending => "icons/sort-descending.svg",
            IconName::SquareTerminal => "icons/square-terminal.svg",
            IconName::Star => "icons/star.svg",
            IconName::StarOff => "icons/star-off.svg",
            IconName::Sun => "icons/sun.svg",
            IconName::ThumbsDown => "icons/thumbs-down.svg",
            IconName::ThumbsUp => "icons/thumbs-up.svg",
            IconName::TriangleAlert => "icons/triangle-alert.svg",
            IconName::WindowClose => "icons/window-close.svg",
            IconName::WindowMaximize => "icons/window-maximize.svg",
            IconName::WindowMinimize => "icons/window-minimize.svg",
            IconName::WindowRestore => "icons/window-restore.svg",
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
    rotation: Option<Radians>,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            base: svg().flex_none().size_4(),
            path: "".into(),
            text_color: None,
            size: None,
            rotation: None,
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

    /// Rotate the icon by the given angle
    pub fn rotate(mut self, radians: impl Into<Radians>) -> Self {
        self.base = self
            .base
            .with_transformation(Transformation::rotate(radians));
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
            .when_some(self.rotation, |this, rotation| {
                this.with_transformation(Transformation::rotate(rotation))
            })
    }
}

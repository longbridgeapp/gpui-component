use gpui::{
    div, rgb, svg, Div, InteractiveElement, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StyleRefinement, Styled, Svg, TextStyle, TextStyleRefinement, WindowContext,
};

use crate::theme::ActiveTheme;

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
        }
        .into()
    }
}

impl Into<Icon> for IconName {
    fn into(self) -> Icon {
        Icon::new(self)
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
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            base: svg().flex_none().size_4().text_color(rgb(0x000000)),
            path: name.path(),
        }
    }

    /// Set the icon path of the Assets bundle
    ///
    /// For example: `icons/foo.svg`
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
        self
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Icon {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        self.base.path(self.path)
    }
}

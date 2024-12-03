use crate::{theme::ActiveTheme as _, Sizable, Size};
use gpui::{
    div, prelude::FluentBuilder as _, relative, Div, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, Styled,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Destructive,
    Custom {
        color: Hsla,
        foreground: Hsla,
        border: Hsla,
    },
}
impl BadgeVariant {
    fn bg(&self, cx: &gpui::WindowContext) -> Hsla {
        match self {
            Self::Primary => cx.theme().primary,
            Self::Secondary => cx.theme().secondary,
            Self::Outline => gpui::transparent_black(),
            Self::Destructive => cx.theme().destructive,
            Self::Custom { color, .. } => *color,
        }
    }

    fn border(&self, cx: &gpui::WindowContext) -> Hsla {
        match self {
            Self::Primary => cx.theme().primary,
            Self::Secondary => cx.theme().secondary,
            Self::Outline => cx.theme().border,
            Self::Destructive => cx.theme().destructive,
            Self::Custom { border, .. } => *border,
        }
    }

    fn fg(&self, cx: &gpui::WindowContext) -> Hsla {
        match self {
            Self::Primary => cx.theme().primary_foreground,
            Self::Secondary => cx.theme().secondary_foreground,
            Self::Outline => cx.theme().foreground,
            Self::Destructive => cx.theme().destructive_foreground,
            Self::Custom { foreground, .. } => *foreground,
        }
    }
}

/// Badge is a small status indicator for UI elements.
///
/// Only support: Medium, Small
#[derive(IntoElement)]
pub struct Badge {
    base: Div,
    veriant: BadgeVariant,
    size: Size,
}
impl Badge {
    fn new() -> Self {
        Self {
            base: div().flex().items_center().rounded_md().border_1(),
            veriant: BadgeVariant::default(),
            size: Size::Medium,
        }
    }

    pub fn with_variant(mut self, variant: BadgeVariant) -> Self {
        self.veriant = variant;
        self
    }

    pub fn primary() -> Self {
        Self::new().with_variant(BadgeVariant::Primary)
    }

    pub fn secondary() -> Self {
        Self::new().with_variant(BadgeVariant::Secondary)
    }

    pub fn outline() -> Self {
        Self::new().with_variant(BadgeVariant::Outline)
    }

    pub fn destructive() -> Self {
        Self::new().with_variant(BadgeVariant::Destructive)
    }

    pub fn custom(color: Hsla, foreground: Hsla, border: Hsla) -> Self {
        Self::new().with_variant(BadgeVariant::Custom {
            color,
            foreground,
            border,
        })
    }
}
impl Sizable for Badge {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}
impl ParentElement for Badge {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}
impl RenderOnce for Badge {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        self.base
            .line_height(relative(1.3))
            .map(|this| match self.size {
                Size::XSmall | Size::Small => this.text_xs().px_1p5().py_0(),
                _ => this.text_xs().px_2p5().py_0p5(),
            })
            .bg(self.veriant.bg(cx))
            .text_color(self.veriant.fg(cx))
            .border_color(self.veriant.border(cx))
            .hover(|this| this.opacity(0.9))
    }
}

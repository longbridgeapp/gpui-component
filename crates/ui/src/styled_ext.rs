use crate::theme::{hsl, ActiveTheme};
use gpui::{hsla, point, px, BoxShadow, FocusHandle, Styled, WindowContext};
use smallvec::{smallvec, SmallVec};

pub enum ElevationIndex {
    Surface,
    PopoverSurface,
    ModalSurface,
}

impl ElevationIndex {
    pub fn shadow(self) -> SmallVec<[BoxShadow; 2]> {
        match self {
            ElevationIndex::Surface => smallvec![],

            ElevationIndex::PopoverSurface => smallvec![BoxShadow {
                color: hsla(0., 0., 0., 0.12),
                offset: point(px(0.), px(2.)),
                blur_radius: px(3.),
                spread_radius: px(0.),
            }],

            ElevationIndex::ModalSurface => smallvec![
                BoxShadow {
                    color: hsla(0., 0., 0., 0.1),
                    offset: point(px(0.), px(4.)),
                    blur_radius: px(6.),
                    spread_radius: px(-1.),
                },
                BoxShadow {
                    color: hsla(0., 0., 0., 0.1),
                    offset: point(px(0.), px(2.)),
                    blur_radius: px(4.),
                    spread_radius: px(-2.),
                }
            ],
        }
    }
}

fn elevated<E: Styled>(this: E, cx: &WindowContext, index: ElevationIndex) -> E {
    this.bg(cx.theme().popover)
        .rounded(px(8.))
        .border_1()
        .border_color(cx.theme().border)
        .shadow(index.shadow())
}

/// Extends [`gpui::Styled`] with specific styling methods.
pub trait StyledExt: Styled + Sized {
    /// Horizontally stacks elements.
    ///
    /// Sets `flex()`, `flex_row()`, `items_center()`
    fn h_flex(self) -> Self {
        self.flex().flex_row().items_center()
    }

    /// Vertically stacks elements.
    ///
    /// Sets `flex()`, `flex_col()`
    fn v_flex(self) -> Self {
        self.flex().flex_col()
    }

    /// Located above the app background
    fn elevation_1(self, cx: &WindowContext) -> Self {
        elevated(self, cx, ElevationIndex::Surface)
    }

    /// Appear above most UI elements
    fn elevation_2(self, cx: &WindowContext) -> Self {
        elevated(self, cx, ElevationIndex::PopoverSurface)
    }

    // Above all other UI elements and are located above the wash layer
    fn elevation_3(self, cx: &WindowContext) -> Self {
        elevated(self, cx, ElevationIndex::ModalSurface)
    }

    /// Render a border with a width of 1px, color blue
    fn debug_blue(self) -> Self {
        self.border_1().border_color(gpui::blue())
    }

    /// Render a border with a width of 1px, color yellow
    fn debug_yellow(self) -> Self {
        self.border_1().border_color(gpui::yellow())
    }

    /// Render a border with a width of 1px, color green
    fn debug_green(self) -> Self {
        self.border_1().border_color(gpui::green())
    }

    /// Render a border with a width of 1px, color pink
    fn debug_pink(self) -> Self {
        self.border_1().border_color(hsl(300., 100., 47.))
    }

    /// Render a 1px blue border, when if the element is focused
    fn debug_focused(self, focus_handle: &FocusHandle, cx: &WindowContext) -> Self {
        if focus_handle.is_focused(cx) {
            self.debug_blue()
        } else {
            self
        }
    }

    /// Render a border with a width of 1px, color ring color
    ///
    /// Please ensure this after the shadow
    fn outline(self, cx: &WindowContext) -> Self {
        self.shadow(smallvec![BoxShadow {
            color: cx.theme().ring,
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.1),
            spread_radius: px(1.),
        }])
    }
}

impl<E: Styled> StyledExt for E {}

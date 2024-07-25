use crate::{
    scroll::{Scrollable, ScrollbarAxis},
    theme::ActiveTheme,
};
use gpui::{
    hsla, point, px, rems, AnyView, Axis, BoxShadow, Element, FocusHandle, Pixels, Styled,
    WindowContext,
};
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

macro_rules! font_weight {
    ($fn:ident, $const:ident) => {
        /// [docs](https://tailwindcss.com/docs/font-weight)
        fn $fn(self) -> Self {
            self.font_weight(gpui::FontWeight::$const)
        }
    };
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

    /// Render a border with a width of 1px, color red
    fn debug_red(self) -> Self {
        if cfg!(debug_assertions) {
            self.border_1().border_color(crate::red_500())
        } else {
            self
        }
    }

    /// Render a border with a width of 1px, color blue
    fn debug_blue(self) -> Self {
        if cfg!(debug_assertions) {
            self.border_1().border_color(crate::blue_500())
        } else {
            self
        }
    }

    /// Render a border with a width of 1px, color yellow
    fn debug_yellow(self) -> Self {
        if cfg!(debug_assertions) {
            self.border_1().border_color(crate::yellow_500())
        } else {
            self
        }
    }

    /// Render a border with a width of 1px, color green
    fn debug_green(self) -> Self {
        if cfg!(debug_assertions) {
            self.border_1().border_color(crate::green_500())
        } else {
            self
        }
    }

    /// Render a border with a width of 1px, color pink
    fn debug_pink(self) -> Self {
        if cfg!(debug_assertions) {
            self.border_1().border_color(crate::pink_500())
        } else {
            self
        }
    }

    /// Render a 1px blue border, when if the element is focused
    fn debug_focused(self, focus_handle: &FocusHandle, cx: &WindowContext) -> Self {
        if cfg!(debug_assertions) {
            if focus_handle.is_focused(cx) {
                self.debug_blue()
            } else {
                self
            }
        } else {
            self
        }
    }

    /// Render a border with a width of 1px, color ring color
    fn outline(self, cx: &WindowContext) -> Self {
        self.border_color(cx.theme().ring)
    }

    /// Wraps the element in a ScrollView.
    ///
    /// Current this is only have a vertical scrollbar.
    fn scrollable(self, view: impl Into<AnyView>, axis: ScrollbarAxis) -> Scrollable<Self>
    where
        Self: Element,
    {
        Scrollable::new(self, view, axis)
    }

    font_weight!(font_thin, THIN);
    font_weight!(font_extralight, EXTRA_LIGHT);
    font_weight!(font_light, LIGHT);
    font_weight!(font_normal, NORMAL);
    font_weight!(font_medium, MEDIUM);
    font_weight!(font_semibold, SEMIBOLD);
    font_weight!(font_bold, BOLD);
    font_weight!(font_extrabold, EXTRA_BOLD);
    font_weight!(font_black, BLACK);
}

impl<E: Styled> StyledExt for E {}

/// A size for elements.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Size(Pixels),
    XSmall,
    Small,
    Medium,
    Large,
}

impl From<Pixels> for Size {
    fn from(size: Pixels) -> Self {
        Size::Size(size)
    }
}

#[allow(unused)]
pub trait StyleSized<T: Styled> {
    fn input_size(self, size: Size) -> Self;
    fn input_pl(self, size: Size) -> Self;
    fn input_pr(self, size: Size) -> Self;
    fn input_px(self, size: Size) -> Self;
    fn input_py(self, size: Size) -> Self;
    fn input_h(self, size: Size) -> Self;
}

impl<T: Styled> StyleSized<T> for T {
    fn input_size(self, size: Size) -> Self {
        self.input_px(size).input_py(size).input_h(size)
    }

    fn input_pl(self, size: Size) -> Self {
        match size {
            Size::Large => self.pl_5(),
            Size::Medium => self.pl_3(),
            _ => self.pl_2(),
        }
    }

    fn input_pr(self, size: Size) -> Self {
        match size {
            Size::Large => self.pr_5(),
            Size::Medium => self.pr_3(),
            _ => self.pr_2(),
        }
    }

    fn input_px(self, size: Size) -> Self {
        match size {
            Size::Large => self.px_5(),
            Size::Medium => self.px_3(),
            _ => self.px_2(),
        }
    }

    fn input_py(self, size: Size) -> Self {
        match size {
            Size::Large => self.py_5(),
            Size::Medium => self.py_2(),
            _ => self.py_1(),
        }
    }

    fn input_h(self, size: Size) -> Self {
        match size {
            Size::Large => self.h_11().text_size(rems(1.)),
            Size::Medium => self.h_8().text_size(rems(0.875)),
            _ => self.h(px(26.)).text_size(rems(0.8)),
        }
    }
}

pub trait AxisExt {
    fn is_horizontal(self) -> bool;
    fn is_vertical(self) -> bool;
}

impl AxisExt for Axis {
    fn is_horizontal(self) -> bool {
        self == Axis::Horizontal
    }

    fn is_vertical(self) -> bool {
        self == Axis::Vertical
    }
}

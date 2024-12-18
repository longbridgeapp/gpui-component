use std::fmt::{self, Display, Formatter};

use crate::{
    scroll::{Scrollable, ScrollbarAxis},
    theme::ActiveTheme,
};
use gpui::{
    div, px, Axis, Div, Edges, Element, ElementId, EntityId, FocusHandle, Pixels, Styled,
    WindowContext,
};
use serde::{Deserialize, Serialize};

/// Returns a `Div` as horizontal flex layout.
pub fn h_flex() -> Div {
    div().h_flex()
}

/// Returns a `Div` as vertical flex layout.
pub fn v_flex() -> Div {
    div().v_flex()
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
    /// Apply self into a horizontal flex layout.
    fn h_flex(self) -> Self {
        self.flex().flex_row().items_center()
    }

    /// Apply self into a vertical flex layout.
    fn v_flex(self) -> Self {
        self.flex().flex_col()
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
            if focus_handle.contains_focused(cx) {
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
    fn scrollable(self, view_id: EntityId, axis: ScrollbarAxis) -> Scrollable<Self>
    where
        Self: Element,
    {
        Scrollable::new(view_id, self, axis)
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

    /// Set as Popover style
    fn popover_style(self, cx: &mut WindowContext) -> Self {
        self.bg(cx.theme().popover)
            .border_1()
            .border_color(cx.theme().border)
            .shadow_lg()
            .rounded(px(cx.theme().radius))
    }
}

impl<E: Styled> StyledExt for E {}

/// A size for elements.
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum Size {
    Size(Pixels),
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
}

impl Size {
    /// Returns the height for table row.
    pub fn table_row_height(&self) -> Pixels {
        match self {
            Size::XSmall => px(26.),
            Size::Small => px(30.),
            Size::Large => px(40.),
            _ => px(32.),
        }
    }
    /// Returns the padding for a table cell.
    pub fn table_cell_padding(&self) -> Edges<Pixels> {
        match self {
            Size::XSmall => Edges {
                top: px(2.),
                bottom: px(2.),
                left: px(4.),
                right: px(4.),
            },
            Size::Small => Edges {
                top: px(3.),
                bottom: px(3.),
                left: px(6.),
                right: px(6.),
            },
            Size::Large => Edges {
                top: px(8.),
                bottom: px(8.),
                left: px(12.),
                right: px(12.),
            },
            _ => Edges {
                top: px(4.),
                bottom: px(4.),
                left: px(8.),
                right: px(8.),
            },
        }
    }
}

impl From<Pixels> for Size {
    fn from(size: Pixels) -> Self {
        Size::Size(size)
    }
}

/// A trait for defining element that can be selected.
pub trait Selectable: Sized {
    fn element_id(&self) -> &ElementId;
    /// Set the selected state of the element.
    fn selected(self, selected: bool) -> Self;
}

/// A trait for defining element that can be disabled.
pub trait Disableable {
    /// Set the disabled state of the element.
    fn disabled(self, disabled: bool) -> Self;
}

/// A trait for setting the size of an element.
/// Size::Medium is use by default.
pub trait Sizable: Sized {
    /// Set the ui::Size of this element.
    ///
    /// Also can receive a `ButtonSize` to convert to `IconSize`,
    /// Or a `Pixels` to set a custom size: `px(30.)`
    fn with_size(self, size: impl Into<Size>) -> Self;

    /// Set to Size::XSmall
    fn xsmall(self) -> Self {
        self.with_size(Size::XSmall)
    }

    /// Set to Size::Small
    fn small(self) -> Self {
        self.with_size(Size::Small)
    }

    /// Set to Size::Large
    fn large(self) -> Self {
        self.with_size(Size::Large)
    }
}

#[allow(unused)]
pub trait StyleSized<T: Styled> {
    fn input_text_size(self, size: Size) -> Self;
    fn input_size(self, size: Size) -> Self;
    fn input_pl(self, size: Size) -> Self;
    fn input_pr(self, size: Size) -> Self;
    fn input_px(self, size: Size) -> Self;
    fn input_py(self, size: Size) -> Self;
    fn input_h(self, size: Size) -> Self;
    fn list_size(self, size: Size) -> Self;
    fn list_px(self, size: Size) -> Self;
    fn list_py(self, size: Size) -> Self;
    /// Apply size with the given `Size`.
    fn size_with(self, size: Size) -> Self;
    /// Apply the table cell size (Font size, padding) with the given `Size`.
    fn table_cell_size(self, size: Size) -> Self;
}

impl<T: Styled> StyleSized<T> for T {
    fn input_text_size(self, size: Size) -> Self {
        match size {
            Size::XSmall => self.text_xs(),
            Size::Small => self.text_sm(),
            Size::Medium => self.text_base(),
            Size::Large => self.text_lg(),
            Size::Size(size) => self.text_size(size),
        }
    }

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
            Size::Large => self.h_11(),
            Size::Medium => self.h_8(),
            _ => self.h(px(26.)),
        }
        .input_text_size(size)
    }

    fn list_size(self, size: Size) -> Self {
        self.list_px(size).list_py(size).input_text_size(size)
    }

    fn list_px(self, size: Size) -> Self {
        match size {
            Size::Small => self.px_2(),
            _ => self.px_3(),
        }
    }

    fn list_py(self, size: Size) -> Self {
        match size {
            Size::Large => self.py_2(),
            Size::Medium => self.py_1(),
            Size::Small => self.py_0p5(),
            _ => self.py_1(),
        }
    }

    fn size_with(self, size: Size) -> Self {
        match size {
            Size::Large => self.size_11(),
            Size::Medium => self.size_8(),
            Size::Small => self.size_5(),
            Size::XSmall => self.size_4(),
            Size::Size(size) => self.size(size),
        }
    }

    fn table_cell_size(self, size: Size) -> Self {
        let padding = size.table_cell_padding();
        match size {
            Size::XSmall => self.text_sm(),
            Size::Small => self.text_sm(),
            _ => self,
        }
        .pl(padding.left)
        .pr(padding.right)
        .pt(padding.top)
        .pb(padding.bottom)
    }
}

pub trait AxisExt {
    fn is_horizontal(self) -> bool;
    fn is_vertical(self) -> bool;
}

impl AxisExt for Axis {
    #[inline]
    fn is_horizontal(self) -> bool {
        self == Axis::Horizontal
    }

    #[inline]
    fn is_vertical(self) -> bool {
        self == Axis::Vertical
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    Top,
    Bottom,
    Left,
    Right,
}

impl Display for Placement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Placement::Top => write!(f, "Top"),
            Placement::Bottom => write!(f, "Bottom"),
            Placement::Left => write!(f, "Left"),
            Placement::Right => write!(f, "Right"),
        }
    }
}

impl Placement {
    #[inline]
    pub fn is_horizontal(&self) -> bool {
        match self {
            Placement::Left | Placement::Right => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_vertical(&self) -> bool {
        match self {
            Placement::Top | Placement::Bottom => true,
            _ => false,
        }
    }

    #[inline]
    pub fn axis(&self) -> Axis {
        match self {
            Placement::Top | Placement::Bottom => Axis::Vertical,
            Placement::Left | Placement::Right => Axis::Horizontal,
        }
    }
}

/// A enum for defining the side of the element.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    #[inline]
    pub(crate) fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }
}

/// A trait for defining element that can be collapsed.
pub trait Collapsible {
    fn collapsed(self, collapsed: bool) -> Self;
    fn is_collapsed(&self) -> bool;
}

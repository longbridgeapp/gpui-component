use crate::{
    scroll::{Scrollable, ScrollbarAxis},
    theme::{ActiveTheme, Colorize},
};
use gpui::{
    div, px, rems, Axis, Div, Element, EntityId, Fill, FocusHandle, Pixels, Styled, WindowContext,
};

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

    /// Set the opacity of the element.
    fn opacity(mut self, opacity: f32) -> Self {
        let bg_color = self.style().background.clone();
        let border_color = self.style().border_color;
        let box_shadow = self.style().box_shadow.clone();

        let this = if let Some(bg) = bg_color {
            match bg {
                Fill::Color(color) => self.bg(color.opacity(opacity)),
            }
        } else {
            self
        };

        let this = if let Some(color) = border_color {
            this.border_color(color.opacity(opacity))
        } else {
            this
        };

        let this = if let Some(mut shadow) = box_shadow {
            for shadow in shadow.iter_mut() {
                shadow.color = shadow.color.opacity(opacity);
            }
            this.shadow(shadow)
        } else {
            this
        };

        this
    }
}

impl<E: Styled> StyledExt for E {}

/// A size for elements.
#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Size(Pixels),
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
}

impl From<Pixels> for Size {
    fn from(size: Pixels) -> Self {
        Size::Size(size)
    }
}

/// A trait for defining element that can be selected.
pub trait Selectable: Sized {
    /// Set the selected state of the element.
    fn selected(self, selected: bool) -> Self;
}

/// A trait for defining element that can be disabled.
pub trait Disableable {
    /// Set the disabled state of the element.
    fn disabled(self, disabled: bool) -> Self;
}

/// A trait for setting the size of an element.
pub trait Sizable: Sized {
    /// Set the ui::Size of this element.
    ///
    /// Also can receive a `ButtonSize` to convert to `IconSize`,
    /// Or a `Pixels` to set a custom size: `px(30.)`
    fn with_size(self, size: impl Into<Size>) -> Self;

    /// Set to Size::Small
    fn small(self) -> Self {
        self.with_size(Size::Small)
    }

    /// Set to Size::XSmall
    fn xsmall(self) -> Self {
        self.with_size(Size::XSmall)
    }

    /// Set to Size::Medium
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
}

impl<T: Styled> StyleSized<T> for T {
    fn input_text_size(self, size: Size) -> Self {
        match size {
            Size::XSmall => self.text_size(rems(0.75)),
            Size::Small => self.text_size(rems(0.8)),
            Size::Medium => self.text_size(rems(0.875)),
            Size::Large => self.text_size(rems(1.)),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placement {
    Top,
    Bottom,
    Left,
    Right,
}

impl Placement {
    pub fn is_horizontal(&self) -> bool {
        match self {
            Placement::Top | Placement::Bottom => true,
            _ => false,
        }
    }

    pub fn is_vertical(&self) -> bool {
        match self {
            Placement::Left | Placement::Right => true,
            _ => false,
        }
    }
}

use std::ops::Deref;

use gpui::{
    hsla, point, AppContext, BoxShadow, Global, Hsla, ModelContext, Pixels, SharedString,
    ViewContext, WindowAppearance, WindowContext,
};

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for AppContext {
    fn theme(&self) -> &Theme {
        Theme::get_global(self)
    }
}

impl<'a, V> ActiveTheme for ViewContext<'a, V> {
    fn theme(&self) -> &Theme {
        self.deref().theme()
    }
}

impl<'a, V> ActiveTheme for ModelContext<'a, V> {
    fn theme(&self) -> &Theme {
        self.deref().theme()
    }
}

impl<'a> ActiveTheme for WindowContext<'a> {
    fn theme(&self) -> &Theme {
        self.deref().theme()
    }
}

/// Make a [gpui::Hsla] color.
///
/// h - 0 - 360.0
/// s - 0.0 - 100.0
/// l - 0.0 - 100.0
pub fn hsl(h: f32, s: f32, l: f32) -> Hsla {
    hsla(h / 360., s / 100.0, l / 100.0, 1.0)
}

/// Make a BoxShadow like CSS
///
/// e.g:
///
/// If CSS is `box-shadow: 0 0 10px 0 rgba(0, 0, 0, 0.1);`
///
/// Then the equivalent in Rust is `box_shadow(0., 0., 10., 0., hsla(0., 0., 0., 0.1))`
pub fn box_shadow(
    x: impl Into<Pixels>,
    y: impl Into<Pixels>,
    blur: impl Into<Pixels>,
    spread: impl Into<Pixels>,
    color: Hsla,
) -> BoxShadow {
    BoxShadow {
        offset: point(x.into(), y.into()),
        blur_radius: blur.into(),
        spread_radius: spread.into(),
        color,
    }
}
pub trait Colorize {
    fn opacity(&self, opacity: f32) -> Hsla;
    fn divide(&self, divisor: f32) -> Hsla;
    fn invert(&self) -> Hsla;
    fn invert_l(&self) -> Hsla;
    fn lighten(&self, amount: f32) -> Hsla;
    fn darken(&self, amount: f32) -> Hsla;
}

impl Colorize for Hsla {
    /// Returns a new color with the given opacity.
    ///
    /// The opacity is a value between 0.0 and 1.0, where 0.0 is fully transparent and 1.0 is fully opaque.
    fn opacity(&self, opacity: f32) -> Hsla {
        Hsla {
            a: self.a * opacity,
            ..*self
        }
    }

    /// Returns a new color with each channel divided by the given divisor.
    ///
    /// The divisor in range of 0.0 .. 1.0
    fn divide(&self, divisor: f32) -> Hsla {
        Hsla {
            a: divisor,
            ..*self
        }
    }

    /// Return inverted color
    fn invert(&self) -> Hsla {
        Hsla {
            h: (self.h + 1.8) % 3.6,
            s: 1.0 - self.s,
            l: 1.0 - self.l,
            a: self.a,
        }
    }

    /// Return inverted lightness
    fn invert_l(&self) -> Hsla {
        Hsla {
            l: 1.0 - self.l,
            ..*self
        }
    }

    fn lighten(&self, amount: f32) -> Hsla {
        let l = (self.l * (1.0 + amount)).min(1.0);

        Hsla { l, ..*self }
    }

    fn darken(&self, amount: f32) -> Hsla {
        let l = (self.l * (1.0 - amount)).max(0.0);

        Hsla { l, ..*self }
    }
}

#[derive(Debug, Clone, Copy)]
struct Colors {
    pub title_bar_background: Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub card: Hsla,
    pub card_foreground: Hsla,
    pub popover: Hsla,
    pub popover_foreground: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_active: Hsla,
    pub primary_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_hover: Hsla,
    pub secondary_active: Hsla,
    pub secondary_foreground: Hsla,
    pub destructive: Hsla,
    pub destructive_hover: Hsla,
    pub destructive_active: Hsla,
    pub destructive_foreground: Hsla,
    pub muted: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub accent_foreground: Hsla,
    pub border: Hsla,
    pub input: Hsla,
    pub ring: Hsla,
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_thumb: Hsla,
    pub panel: Hsla,
    pub tab_bar: Hsla,
    pub list: Hsla,
    pub list_even: Hsla,
    pub list_active: Hsla,
    pub list_head: Hsla,
    pub link: Hsla,
}

impl Colors {
    fn light() -> Colors {
        Colors {
            title_bar_background: hsl(0.0, 0.0, 100.),
            background: hsl(0.0, 0.0, 100.),
            foreground: hsl(240.0, 10., 3.9),
            card: hsl(0.0, 0.0, 100.0),
            card_foreground: hsl(240.0, 10.0, 3.9),
            popover: hsl(0.0, 0.0, 100.0),
            popover_foreground: hsl(240.0, 10.0, 3.9),
            primary: hsl(223.0, 5.9, 10.0),
            primary_hover: hsl(223.0, 5.9, 30.0),
            primary_active: hsl(223.0, 5.9, 45.0),
            primary_foreground: hsl(223.0, 0.0, 98.0),
            secondary: hsl(240.0, 4.8, 95.9),
            secondary_hover: hsl(240.0, 4.8, 99.),
            secondary_active: hsl(240.0, 5.9, 94.0),
            secondary_foreground: hsl(240.0, 59.0, 10.0),
            destructive: hsl(0.0, 84.2, 60.2),
            destructive_hover: hsl(0.0, 84.2, 65.0),
            destructive_active: hsl(0.0, 84.2, 47.0),
            destructive_foreground: hsl(0.0, 0.0, 98.0),
            muted: hsl(240.0, 4.8, 95.9),
            muted_foreground: hsl(240.0, 3.8, 46.1),
            accent: hsl(240.0, 5.0, 96.0),
            accent_foreground: hsl(240.0, 5.9, 10.0),
            border: hsl(240.0, 5.9, 90.0),
            input: hsl(240.0, 5.9, 90.0),
            ring: hsl(240.0, 5.9, 65.0),
            selection: hsl(211.0, 97.0, 85.0),
            scrollbar: hsl(0., 0., 97.),
            scrollbar_thumb: hsl(0., 0., 49.),
            panel: hsl(0.0, 0.0, 100.0),
            tab_bar: hsl(240.0, 4.8, 95.9),
            list: hsl(0.0, 0.0, 100.),
            list_even: hsl(240.0, 5.0, 96.0),
            list_active: hsl(240.0, 7., 88.0).opacity(0.75),
            list_head: hsl(0.0, 0.0, 100.),
            link: hsl(221.0, 83.0, 53.0),
        }
    }

    fn dark() -> Colors {
        Colors {
            title_bar_background: hsl(0., 0., 12.),
            background: hsl(0.0, 0.0, 6.0),
            foreground: hsl(0., 0., 98.),
            card: hsl(299.0, 2., 9.),
            card_foreground: hsl(0.0, 0.0, 98.0),
            popover: hsl(240.0, 10.0, 3.9),
            popover_foreground: hsl(0.0, 0.0, 98.0),
            primary: hsl(223.0, 0.0, 98.0),
            primary_hover: hsl(223.0, 0.0, 85.0),
            primary_active: hsl(223.0, 0.0, 60.0),
            primary_foreground: hsl(223.0, 5.9, 10.0),
            secondary: hsl(240.0, 3.7, 15.9),
            secondary_hover: hsl(240.0, 3.7, 20.9),
            secondary_active: hsl(240.0, 3.7, 8.9),
            secondary_foreground: hsl(0.0, 0.0, 98.0),
            destructive: hsl(0.0, 62.8, 30.6),
            destructive_hover: hsl(0.0, 62.8, 35.6),
            destructive_active: hsl(0.0, 62.8, 20.6),
            destructive_foreground: hsl(0.0, 0.0, 98.0),
            muted: hsl(240.0, 3.7, 15.9),
            muted_foreground: hsl(240.0, 5.0, 64.9),
            accent: hsl(240.0, 3.7, 15.9),
            accent_foreground: hsl(0.0, 0.0, 98.0),
            border: hsl(240.0, 3.7, 15.9),
            input: hsl(240.0, 3.7, 15.9),
            ring: hsl(240.0, 4.9, 83.9),
            selection: hsl(211.0, 97.0, 22.0),
            scrollbar: hsl(240., 1., 15.),
            scrollbar_thumb: hsl(0., 0., 58.),
            panel: hsl(299.0, 2., 9.),
            tab_bar: hsl(299.0, 2., 9.),
            list: hsl(0.0, 0.0, 6.0),
            list_even: hsl(240.0, 3.7, 8.0),
            list_active: hsl(240.0, 3.7, 15.0),
            list_head: hsl(0.0, 0.0, 6.0),
            link: hsl(221.0, 83.0, 53.0),
        }
    }
}

#[derive(Debug)]
pub struct Theme {
    pub mode: ThemeMode,
    pub transparent: Hsla,
    pub title_bar_background: Hsla,
    /// Basic font size
    pub font_size: f32,
    pub font_family: SharedString,
    pub background: Hsla,
    pub foreground: Hsla,
    pub card: Hsla,
    pub card_foreground: Hsla,
    pub popover: Hsla,
    pub popover_foreground: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_active: Hsla,
    pub primary_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_hover: Hsla,
    pub secondary_active: Hsla,
    pub secondary_foreground: Hsla,
    pub destructive: Hsla,
    pub destructive_hover: Hsla,
    pub destructive_active: Hsla,
    pub destructive_foreground: Hsla,
    pub muted: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub accent_foreground: Hsla,
    pub border: Hsla,
    pub input: Hsla,
    pub ring: Hsla,
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_thumb: Hsla,
    pub panel: Hsla,
    pub drag_border: Hsla,
    pub drop_target: Hsla,
    pub radius: f32,
    pub tab_bar: Hsla,
    pub tab: Hsla,
    pub tab_active: Hsla,
    pub tab_foreground: Hsla,
    pub tab_active_foreground: Hsla,
    pub progress_bar: Hsla,
    pub slider_bar: Hsla,
    pub slider_thumb: Hsla,
    pub list: Hsla,
    pub list_even: Hsla,
    pub list_head: Hsla,
    pub list_active: Hsla,
    pub list_hover: Hsla,
    pub table: Hsla,
    pub table_even: Hsla,
    pub table_head: Hsla,
    pub table_active: Hsla,
    pub table_hover: Hsla,
    pub link: Hsla,
    pub link_hover: Hsla,
    pub link_active: Hsla,
    pub skeleton: Hsla,
}

impl Global for Theme {}

impl Theme {
    pub fn get_global(cx: &AppContext) -> &Self {
        cx.global::<Self>()
    }
}

impl From<Colors> for Theme {
    fn from(colors: Colors) -> Self {
        Theme {
            mode: ThemeMode::Dark,
            transparent: Hsla::transparent_black(),
            font_size: 14.0,
            font_family: if cfg!(target_os = "macos") {
                ".SystemUIFont".into()
            } else if cfg!(target_os = "windows") {
                "Segoe UI".into()
            } else {
                "FreeMono".into()
            },
            radius: 4.0,
            title_bar_background: colors.title_bar_background,
            background: colors.background,
            foreground: colors.foreground,
            card: colors.card,
            card_foreground: colors.card_foreground,
            popover: colors.popover,
            popover_foreground: colors.popover_foreground,
            primary: colors.primary,
            primary_hover: colors.primary_hover,
            primary_active: colors.primary_active,
            primary_foreground: colors.primary_foreground,
            secondary: colors.secondary,
            secondary_hover: colors.secondary_hover,
            secondary_active: colors.secondary_active,
            secondary_foreground: colors.secondary_foreground,
            destructive: colors.destructive,
            destructive_hover: colors.destructive_hover,
            destructive_active: colors.destructive_active,
            destructive_foreground: colors.destructive_foreground,
            muted: colors.muted,
            muted_foreground: colors.muted_foreground,
            accent: colors.accent,
            accent_foreground: colors.accent_foreground,
            border: colors.border,
            input: colors.input,
            ring: colors.ring,
            scrollbar: colors.scrollbar,
            scrollbar_thumb: colors.scrollbar_thumb,
            panel: colors.panel,
            selection: colors.selection,
            drag_border: crate::blue_500(),
            drop_target: hsl(240.0, 65., 44.0).opacity(0.15),
            tab_bar: colors.tab_bar,
            tab: gpui::transparent_black(),
            tab_active: colors.background,
            tab_foreground: colors.foreground,
            tab_active_foreground: colors.foreground,
            progress_bar: colors.primary,
            slider_bar: colors.primary,
            slider_thumb: colors.background,
            list: colors.list,
            list_even: colors.list_even,
            list_head: colors.list_head,
            list_active: colors.list_active,
            list_hover: colors.list_active.opacity(0.6),
            table_head: colors.list_head,
            table: colors.list,
            table_even: colors.list_even,
            table_active: colors.list_active,
            table_hover: colors.list_active.opacity(0.8),
            link: colors.link,
            link_hover: colors.link.lighten(0.2),
            link_active: colors.link.darken(0.2),
            skeleton: hsla(colors.primary.h, colors.primary.s, colors.primary.l, 0.1),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }
}

impl Theme {
    fn new() -> Self {
        Self::from(Colors::dark())
    }

    pub fn init(cx: &mut AppContext) {
        cx.set_global(Theme::new());
        Self::sync_system_appearance(cx)
    }

    /// Sync the theme with the system appearance
    pub fn sync_system_appearance(cx: &mut AppContext) {
        match cx.window_appearance() {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => {
                Self::change(ThemeMode::Dark, cx)
            }
            WindowAppearance::Light | WindowAppearance::VibrantLight => {
                Self::change(ThemeMode::Light, cx)
            }
        }
    }

    pub fn change(mode: ThemeMode, cx: &mut AppContext) {
        let colors = match mode {
            ThemeMode::Light => Colors::light(),
            ThemeMode::Dark => Colors::dark(),
        };

        let mut theme = Theme::from(colors);
        theme.mode = mode;

        cx.set_global(theme);
        cx.refresh();
    }
}

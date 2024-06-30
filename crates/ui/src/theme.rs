use gpui::{hsla, AppContext, Global, Hsla, WindowAppearance};

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for AppContext {
    fn theme(&self) -> &Theme {
        Theme::get_global(self)
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
    pub primary_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_foreground: Hsla,
    pub muted: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub accent_foreground: Hsla,
    pub destructive: Hsla,
    pub destructive_foreground: Hsla,
    pub border: Hsla,
    pub input: Hsla,
    pub ring: Hsla,
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_thumb: Hsla,
    pub panel: Hsla,
    pub drop_target: Hsla,
}

impl Colors {
    // .light {
    //     --title_bar_background: 0 0% 100%;
    //     --background: 0 0% 100%;
    //     --foreground: 240 10% 3.9%;
    //     --card: 0 0% 100%;
    //     --card-foreground: 240 10% 3.9%;
    //     --popover: 0 0% 100%;
    //     --popover-foreground: 240 10% 3.9%;
    //     --primary: 240 5.9% 10%;
    //     --primary-foreground: 0 0% 98%;
    //     --secondary: 240 4.8% 95.9%;
    //     --secondary-foreground: 240 5.9% 10%;
    //     --muted: 240 4.8% 95.9%;
    //     --muted-foreground: 240 3.8% 46.1%;
    //     --accent: 240 4.8% 95.9%;
    //     --accent-foreground: 240 5.9% 10%;
    //     --destructive: 0 84.2% 60.2%;
    //     --destructive-foreground: 0 0% 98%;
    //     --border: 240 5.9% 90%;
    //     --input: 240 5.9% 90%;
    //     --ring: 240 5.9% 10%;
    //     --radius: 0rem;
    //     --selection: 211 97% 85%;
    // }
    fn light() -> Colors {
        Colors {
            title_bar_background: hsl(0.0, 0.0, 100.),
            background: hsl(0.0, 0.0, 100.),
            foreground: hsl(240.0, 10., 3.9),
            card: hsl(0.0, 0.0, 100.0),
            card_foreground: hsl(240.0, 10.0, 3.9),
            popover: hsl(0.0, 0.0, 100.0),
            popover_foreground: hsl(240.0, 10.0, 3.9),
            primary: hsl(240.0, 5.9, 10.0),
            primary_foreground: hsl(0.0, 0.0, 98.0),
            secondary: hsl(240.0, 4.8, 95.9),
            secondary_foreground: hsl(240.0, 59.0, 10.0),
            muted: hsl(240.0, 4.8, 95.9),
            muted_foreground: hsl(240.0, 3.8, 46.1),
            accent: hsl(240.0, 5.0, 96.0),
            accent_foreground: hsl(240.0, 5.9, 10.0),
            destructive: hsl(0.0, 84.2, 60.2),
            destructive_foreground: hsl(0.0, 0.0, 98.0),
            border: hsl(240.0, 5.9, 90.0),
            input: hsl(240.0, 5.9, 90.0),
            ring: hsl(240.0, 5.9, 10.0),
            selection: hsl(211.0, 97.0, 85.0),
            scrollbar: Hsla::transparent_black(),
            scrollbar_thumb: hsl(240.0, 5.9, 90.0),
            panel: hsl(0.0, 0.0, 100.0),
            drop_target: hsl(240.0, 65., 80.0),
        }
    }

    //   .dark {
    //     --title_bar_background: 0 0% 12%;
    //     --background: 240 10% 3.9%;
    //     --foreground: 0 0% 98%;
    //     --card: 240 10% 3.9%;
    //     --card-foreground: 0 0% 98%;
    //     --popover: 240 10% 3.9%;
    //     --popover-foreground: 0 0% 98%;
    //     --primary: 0 0% 98%;
    //     --primary-foreground: 240 5.9% 10%;
    //     --secondary: 240 3.7% 15.9%;
    //     --secondary-foreground: 0 0% 98%;
    //     --muted: 240 3.7% 15.9%;
    //     --muted-foreground: 240 5% 64.9%;
    //     --accent: 240 3.7% 15.9%;
    //     --accent-foreground: 0 0% 98%;
    //     --destructive: 0 62.8% 30.6%;
    //     --destructive-foreground: 0 0% 98%;
    //     --border: 240 3.7% 15.9%;
    //     --input: 240 3.7% 15.9%;
    //     --ring: 240 4.9% 83.9%;
    //     --selection: 211 97% 85%;
    //   }
    fn dark() -> Colors {
        Colors {
            title_bar_background: hsl(0., 0., 12.),
            background: hsl(0.0, 0.0, 6.0),
            foreground: hsl(0., 0., 98.),
            card: hsl(299.0, 2., 9.),
            card_foreground: hsl(0.0, 0.0, 98.0),
            popover: hsl(240.0, 10.0, 3.9),
            popover_foreground: hsl(0.0, 0.0, 98.0),
            primary: hsl(0.0, 0.0, 98.0),
            primary_foreground: hsl(240.0, 5.9, 10.0),
            secondary: hsl(240.0, 3.7, 15.9),
            secondary_foreground: hsl(0.0, 0.0, 98.0),
            muted: hsl(240.0, 3.7, 15.9),
            muted_foreground: hsl(240.0, 5.0, 64.9),
            accent: hsl(240.0, 3.7, 15.9),
            accent_foreground: hsl(0.0, 0.0, 98.0),
            destructive: hsl(0.0, 62.8, 30.6),
            destructive_foreground: hsl(0.0, 0.0, 98.0),
            border: hsl(240.0, 3.7, 15.9),
            input: hsl(240.0, 3.7, 15.9),
            ring: hsl(240.0, 4.9, 83.9),
            selection: hsl(211.0, 97.0, 85.0),
            scrollbar: Hsla::transparent_black(),
            scrollbar_thumb: hsl(240.0, 3.7, 15.9),
            panel: hsl(299.0, 2., 9.),
            drop_target: hsl(240.0, 65., 29.0),
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
    pub background: Hsla,
    pub foreground: Hsla,
    pub card: Hsla,
    pub card_foreground: Hsla,
    pub popover: Hsla,
    pub popover_foreground: Hsla,
    pub primary: Hsla,
    pub primary_foreground: Hsla,
    pub secondary: Hsla,
    pub secondary_foreground: Hsla,
    pub muted: Hsla,
    pub muted_foreground: Hsla,
    pub accent: Hsla,
    pub accent_foreground: Hsla,
    pub destructive: Hsla,
    pub destructive_foreground: Hsla,
    pub border: Hsla,
    pub input: Hsla,
    pub ring: Hsla,
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_thumb: Hsla,
    pub panel: Hsla,
    pub drop_target: Hsla,
    pub radius: f32,
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
            radius: 4.0,
            title_bar_background: colors.title_bar_background,
            background: colors.background,
            foreground: colors.foreground,
            card: colors.card,
            card_foreground: colors.card_foreground,
            popover: colors.popover,
            popover_foreground: colors.popover_foreground,
            primary: colors.primary,
            primary_foreground: colors.primary_foreground,
            secondary: colors.secondary,
            secondary_foreground: colors.secondary_foreground,
            muted: colors.muted,
            muted_foreground: colors.muted_foreground,
            accent: colors.accent,
            accent_foreground: colors.accent_foreground,
            destructive: colors.destructive,
            destructive_foreground: colors.destructive_foreground,
            border: colors.border,
            input: colors.input,
            ring: colors.ring,
            scrollbar: colors.scrollbar,
            scrollbar_thumb: colors.scrollbar_thumb,
            panel: colors.panel,
            selection: colors.selection,
            drop_target: colors.drop_target,
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

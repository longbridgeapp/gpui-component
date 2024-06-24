use gpui::{AppContext, Global, Hsla, Rgba};
use serde_json::json;

fn hex(color: &str) -> Hsla {
    let color: Rgba = serde_json::from_value(json!(color)).unwrap();
    color.into()
}

fn hsl(h: f32, s: f32, l: f32) -> Hsla {
    Hsla { h, s, l, a: 1.0 }
}

pub trait Colorize {
    fn opacity(&self, opacity: f32) -> Hsla;
    fn divide(&self, divisor: f32) -> Hsla;
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
}

// @layer base {
//   :root {
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
//   }

//   .dark {
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
//   }
// }

#[derive(Debug, Clone, Copy)]
struct Colors {
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
    pub radius: f32,
}

impl Colors {
    fn light() -> Colors {
        Colors {
            background: hsl(0.0, 0.0, 1.0),
            foreground: hsl(240.0, 0.1, 0.039),
            card: hsl(0.0, 0.0, 1.0),
            card_foreground: hsl(240.0, 0.1, 0.039),
            popover: hsl(0.0, 0.0, 1.0),
            popover_foreground: hsl(240.0, 0.1, 0.039),
            primary: hsl(240.0, 0.059, 0.1),
            primary_foreground: hsl(0.0, 0.0, 0.98),
            secondary: hsl(240.0, 0.048, 0.959),
            secondary_foreground: hsl(240.0, 0.059, 0.1),
            muted: hsl(240.0, 0.048, 0.959),
            muted_foreground: hsl(240.0, 0.038, 0.461),
            accent: hsl(240.0, 0.048, 0.959),
            accent_foreground: hsl(240.0, 0.059, 0.1),
            destructive: hsl(0.0, 0.842, 0.602),
            destructive_foreground: hsl(0.0, 0.0, 0.98),
            border: hsl(240.0, 0.059, 0.9),
            input: hsl(240.0, 0.059, 0.9),
            ring: hsl(240.0, 0.059, 0.1),
            radius: 0.0,
        }
    }

    fn dark() -> Colors {
        Colors {
            background: hsl(240.0, 0.1, 0.039),
            foreground: hsl(0.0, 0.0, 0.98),
            card: hsl(240.0, 0.1, 0.039),
            card_foreground: hsl(0.0, 0.0, 0.98),
            popover: hsl(240.0, 0.1, 0.039),
            popover_foreground: hsl(0.0, 0.0, 0.98),
            primary: hsl(0.0, 0.0, 0.98),
            primary_foreground: hsl(240.0, 0.059, 0.1),
            secondary: hsl(240.0, 0.037, 0.159),
            secondary_foreground: hsl(0.0, 0.0, 0.98),
            muted: hsl(240.0, 0.037, 0.159),
            muted_foreground: hsl(240.0, 0.05, 0.649),
            accent: hsl(240.0, 0.037, 0.159),
            accent_foreground: hsl(0.0, 0.0, 0.98),
            destructive: hsl(0.0, 0.628, 0.306),
            destructive_foreground: hsl(0.0, 0.0, 0.98),
            border: hsl(240.0, 0.037, 0.159),
            input: hsl(240.0, 0.037, 0.159),
            ring: hsl(240.0, 0.049, 0.839),
            radius: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct Theme {
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
    pub radius: f32,
}

impl Global for Theme {}

impl From<Colors> for Theme {
    fn from(colors: Colors) -> Self {
        Theme {
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
            radius: colors.radius,
        }
    }
}

pub enum ThemeMode {
    Light,
    Dark,
}

impl Theme {
    fn new() -> Self {
        Self::from(Colors::light())
    }

    pub fn init(cx: &mut AppContext) {
        cx.set_global(Theme::new())
    }

    pub fn change(mode: ThemeMode, cx: &mut AppContext) {
        let colors = match mode {
            ThemeMode::Light => Colors::light(),
            ThemeMode::Dark => Colors::dark(),
        };

        cx.set_global(Self::from(colors));
        cx.refresh();
    }
}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for AppContext {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

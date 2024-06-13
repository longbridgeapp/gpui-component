use gpui::{hsla, Hsla, WindowContext};

pub fn hls(h: f32, l: f32, s: f32) -> Hsla {
    hsla(h, l / 100., s / 100., 1.)
}

/// Extension trait for `Hsla` to provide more color manipulation methods.
pub trait HlsaExt {
    fn darken(&self, amount: f32) -> Hsla;
    fn lighten(&self, amount: f32) -> Hsla;
}

impl HlsaExt for Hsla {
    /// Darken the color by a percentage.
    ///
    /// `amount` value is 0.0 - 1.0
    fn darken(&self, amount: f32) -> Hsla {
        let l = self.l - (self.l * amount);
        hsla(self.h, l, self.s, self.a)
    }

    /// Lighten the color by a percentage.
    ///
    /// `amount` value is 0.0 - 1.0
    fn lighten(&self, amount: f32) -> Hsla {
        let l = self.l + (self.l * amount);
        hsla(self.h, l, self.s, self.a)
    }
}

// .dark {
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
//     --destructive-foreground: 0 85.7% 97.3%;
//     --border: 240 3.7% 15.9%;
//     --input: 240 3.7% 15.9%;
//     --ring: 240 4.9% 83.9%;
// }

pub enum Color {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
}

impl Color {
    pub fn color(&self, _cx: &WindowContext) -> Hsla {
        match self {
            Color::Background => hls(240., 10., 3.9),
            Color::Foreground => hls(0., 0., 98.),
            Color::Card => hls(240., 10., 3.9),
            Color::CardForeground => hls(0., 0., 98.),
            Color::Popover => hls(240., 10., 3.9),
            Color::PopoverForeground => hls(0., 0., 98.),
            Color::Primary => hls(0., 0., 98.),
            Color::PrimaryForeground => hls(240., 5.9, 10.),
            Color::Secondary => hls(240., 3.7, 15.9),
            Color::SecondaryForeground => hls(0., 0., 98.),
            Color::Muted => hls(240., 3.7, 15.9),
            Color::MutedForeground => hls(240., 5., 64.9),
            Color::Accent => hls(240., 3.7, 15.9),
            Color::AccentForeground => hls(0., 0., 98.),
            Color::Destructive => hls(0., 62.8, 30.6),
            Color::DestructiveForeground => hls(0., 85.7, 97.3),
            Color::Border => hls(240., 3.7, 15.9),
            Color::Input => hls(240., 3.7, 15.9),
            Color::Ring => hls(240., 4.9, 83.9),
        }
    }
}

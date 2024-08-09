use std::collections::HashMap;

use gpui::Hsla;
use serde::{de::Error, Deserialize, Deserializer};

use crate::theme::hsl;
use anyhow::Result;

static DEFAULT_COLOR: once_cell::sync::Lazy<ShacnColors> = once_cell::sync::Lazy::new(|| {
    serde_json::from_str(include_str!("../default-colors.json"))
        .expect("failed to parse default-json")
});

type ColorScales = HashMap<usize, ShacnColor>;

mod color_scales {
    use std::collections::HashMap;

    use super::{ColorScales, ShacnColor};

    use serde::de::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ColorScales, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = HashMap::new();
        for color in Vec::<ShacnColor>::deserialize(deserializer)? {
            map.insert(color.scale, color);
        }
        Ok(map)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct ShacnColors {
    black: ShacnColor,
    white: ShacnColor,
    #[serde(with = "color_scales")]
    slate: ColorScales,
    #[serde(with = "color_scales")]
    gray: ColorScales,
    #[serde(with = "color_scales")]
    zinc: ColorScales,
    #[serde(with = "color_scales")]
    neutral: ColorScales,
    #[serde(with = "color_scales")]
    stone: ColorScales,
    #[serde(with = "color_scales")]
    red: ColorScales,
    #[serde(with = "color_scales")]
    orange: ColorScales,
    #[serde(with = "color_scales")]
    amber: ColorScales,
    #[serde(with = "color_scales")]
    yellow: ColorScales,
    #[serde(with = "color_scales")]
    lime: ColorScales,
    #[serde(with = "color_scales")]
    green: ColorScales,
    #[serde(with = "color_scales")]
    emerald: ColorScales,
    #[serde(with = "color_scales")]
    teal: ColorScales,
    #[serde(with = "color_scales")]
    cyan: ColorScales,
    #[serde(with = "color_scales")]
    sky: ColorScales,
    #[serde(with = "color_scales")]
    blue: ColorScales,
    #[serde(with = "color_scales")]
    indigo: ColorScales,
    #[serde(with = "color_scales")]
    violet: ColorScales,
    #[serde(with = "color_scales")]
    purple: ColorScales,
    #[serde(with = "color_scales")]
    fuchsia: ColorScales,
    #[serde(with = "color_scales")]
    pink: ColorScales,
    #[serde(with = "color_scales")]
    rose: ColorScales,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
struct ShacnColor {
    #[serde(default)]
    scale: usize,
    #[serde(deserialize_with = "from_hsa_channel", alias = "hslChannel")]
    hsla: Hsla,
}

/// Deserialize Hsla from a string in the format "210 40% 98%"
fn from_hsa_channel<'de, D>(deserializer: D) -> Result<Hsla, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer).unwrap();

    let mut parts = s.split_whitespace();
    if parts.clone().count() != 3 {
        return Err(D::Error::custom(
            "expected hslChannel has 3 parts, e.g: '210 40% 98%'",
        ));
    }

    fn parse_number(s: &str) -> f32 {
        s.trim_end_matches('%').parse().unwrap_or(0.0)
    }

    let (h, s, l) = (
        parse_number(parts.next().unwrap()),
        parse_number(parts.next().unwrap()),
        parse_number(parts.next().unwrap()),
    );

    Ok(hsl(h, s, l))
}

macro_rules! color_method {
    ($color:tt, $scale:tt) => {
        paste::paste! {
            #[allow(unused)]
            pub fn [<$color _ $scale>]() -> Hsla {
                if let Some(color) = DEFAULT_COLOR.$color.get(&($scale as usize)) {
                    return color.hsla;
                }

                black()
            }
        }
    };
}

macro_rules! color_methods {
    ($color:tt) => {
        color_method!($color, 50);
        color_method!($color, 100);
        color_method!($color, 200);
        color_method!($color, 300);
        color_method!($color, 400);
        color_method!($color, 500);
        color_method!($color, 600);
        color_method!($color, 700);
        color_method!($color, 800);
        color_method!($color, 900);
        color_method!($color, 950);
    };
}

pub fn black() -> Hsla {
    DEFAULT_COLOR.black.hsla
}

pub fn white() -> Hsla {
    DEFAULT_COLOR.white.hsla
}

color_methods!(slate);
color_methods!(gray);
color_methods!(zinc);
color_methods!(neutral);
color_methods!(stone);
color_methods!(red);
color_methods!(orange);
color_methods!(amber);
color_methods!(yellow);
color_methods!(lime);
color_methods!(green);
color_methods!(emerald);
color_methods!(teal);
color_methods!(cyan);
color_methods!(sky);
color_methods!(blue);
color_methods!(indigo);
color_methods!(violet);
color_methods!(purple);
color_methods!(fuchsia);
color_methods!(pink);
color_methods!(rose);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_colors() {
        assert_eq!(white(), hsl(0.0, 0.0, 100.0));
        assert_eq!(black(), hsl(0.0, 0.0, 0.0));

        assert_eq!(slate_50(), hsl(210.0, 40.0, 98.0));
        assert_eq!(slate_100(), hsl(210.0, 40.0, 96.1));
        assert_eq!(slate_900(), hsl(222.2, 47.4, 11.2));

        assert_eq!(red_50(), hsl(0.0, 85.7, 97.3));
        assert_eq!(yellow_100(), hsl(54.9, 96.7, 88.0));
        assert_eq!(green_200(), hsl(141.0, 78.9, 85.1));
        assert_eq!(cyan_300(), hsl(187.0, 92.4, 69.0));
        assert_eq!(blue_400(), hsl(213.1, 93.9, 67.8));
        assert_eq!(indigo_500(), hsl(238.7, 83.5, 66.7));
    }
}

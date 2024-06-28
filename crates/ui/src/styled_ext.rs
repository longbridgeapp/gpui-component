use crate::theme::ActiveTheme;
use gpui::{hsla, point, px, BoxShadow, Styled, WindowContext};
use smallvec::{smallvec, SmallVec};

pub enum ElevationIndex {
    Surface,
    ElevatedSurface,
    ModalSurface,
}

impl ElevationIndex {
    pub fn shadow(self) -> SmallVec<[BoxShadow; 2]> {
        match self {
            ElevationIndex::Surface => smallvec![],

            ElevationIndex::ElevatedSurface => smallvec![BoxShadow {
                color: hsla(0., 0., 0., 0.12),
                offset: point(px(0.), px(2.)),
                blur_radius: px(3.),
                spread_radius: px(0.),
            }],

            ElevationIndex::ModalSurface => smallvec![
                BoxShadow {
                    color: hsla(0., 0., 0., 0.12),
                    offset: point(px(0.), px(2.)),
                    blur_radius: px(3.),
                    spread_radius: px(0.),
                },
                BoxShadow {
                    color: hsla(0., 0., 0., 0.08),
                    offset: point(px(0.), px(3.)),
                    blur_radius: px(6.),
                    spread_radius: px(0.),
                },
                BoxShadow {
                    color: hsla(0., 0., 0., 0.04),
                    offset: point(px(0.), px(6.)),
                    blur_radius: px(12.),
                    spread_radius: px(0.),
                },
            ],

            _ => smallvec![],
        }
    }
}

fn elevated<E: Styled>(this: E, cx: &mut WindowContext, index: ElevationIndex) -> E {
    let theme = cx.theme();

    this.bg(theme.popover)
        .rounded(px(8.))
        .border_1()
        .border_color(theme.border)
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
    fn elevation_1(self, cx: &mut WindowContext) -> Self {
        elevated(self, cx, ElevationIndex::Surface)
    }

    /// Appear above most UI elements
    fn elevation_2(self, cx: &mut WindowContext) -> Self {
        elevated(self, cx, ElevationIndex::ElevatedSurface)
    }

    // Above all other UI elements and are located above the wash layer
    fn elevation_3(self, cx: &mut WindowContext) -> Self {
        elevated(self, cx, ElevationIndex::ModalSurface)
    }
}

impl<E: Styled> StyledExt for E {}

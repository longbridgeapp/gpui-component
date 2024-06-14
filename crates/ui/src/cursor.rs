#![allow(unused)]
use gpui::{
    div, fill, outline, point, px, size, AnyElement, AvailableSpace, Bounds, Hsla, IntoElement,
    ParentElement, Pixels, ShapedLine, SharedString, Styled, WindowContext,
};

/// The shape of a selection cursor.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum CursorShape {
    /// A vertical bar
    #[default]
    Bar,
    /// A block that surrounds the following character
    Block,
    /// An underline that runs along the following character
    Underscore,
    /// A box drawn around the following character
    Hollow,
}

pub struct CursorLayout {
    origin: gpui::Point<Pixels>,
    block_width: Pixels,
    line_height: Pixels,
    color: Hsla,
    shape: CursorShape,
}

#[derive(Debug)]
pub struct CursorName {
    string: SharedString,
    color: Hsla,
    is_top_row: bool,
}

impl CursorLayout {
    pub fn new(
        origin: gpui::Point<Pixels>,
        block_width: Pixels,
        line_height: Pixels,
        color: Hsla,
        shape: CursorShape,
        block_text: Option<ShapedLine>,
    ) -> CursorLayout {
        CursorLayout {
            origin,
            block_width,
            line_height,
            color,
            shape,
        }
    }

    pub fn bounding_rect(&self, origin: gpui::Point<Pixels>) -> Bounds<Pixels> {
        Bounds {
            origin: self.origin + origin,
            size: size(self.block_width, self.line_height),
        }
    }

    fn bounds(&self, origin: gpui::Point<Pixels>) -> Bounds<Pixels> {
        match self.shape {
            CursorShape::Bar => Bounds {
                origin: self.origin + origin,
                size: size(px(2.0), self.line_height),
            },
            CursorShape::Block | CursorShape::Hollow => Bounds {
                origin: self.origin + origin,
                size: size(self.block_width, self.line_height),
            },
            CursorShape::Underscore => Bounds {
                origin: self.origin
                    + origin
                    + gpui::Point::new(Pixels::ZERO, self.line_height - px(2.0)),
                size: size(self.block_width, px(2.0)),
            },
        }
    }

    pub fn layout(
        &mut self,
        origin: gpui::Point<Pixels>,
        cursor_name: Option<CursorName>,
        cx: &mut WindowContext,
    ) {
    }

    pub fn paint(&mut self, origin: gpui::Point<Pixels>, cx: &mut WindowContext) {
        let bounds = self.bounds(origin);

        //Draw background or border quad
        let cursor = if matches!(self.shape, CursorShape::Hollow) {
            outline(bounds, self.color)
        } else {
            fill(bounds, self.color)
        };

        cx.paint_quad(cursor);
    }

    pub fn shape(&self) -> CursorShape {
        self.shape
    }
}

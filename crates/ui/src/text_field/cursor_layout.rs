use gpui::{outline, px, Bounds, Hsla, Pixels, ShapedLine, Size, ViewContext};

pub struct CursorLayout {
    origin: gpui::Point<Pixels>,
    #[allow(unused)]
    block_width: Pixels,
    line_height: Pixels,
    color: Hsla,
    block_text: Option<ShapedLine>,
}

impl CursorLayout {
    pub fn new(
        origin: gpui::Point<Pixels>,
        block_width: Pixels,
        line_height: Pixels,
        color: Hsla,
        block_text: Option<ShapedLine>,
    ) -> CursorLayout {
        CursorLayout {
            origin,
            block_width,
            line_height,
            color,
            block_text,
        }
    }

    fn bounds(&self, origin: gpui::Point<Pixels>) -> Bounds<Pixels> {
        Bounds {
            origin: self.origin + origin,
            size: Size {
                width: px(2.0),
                height: self.line_height,
            },
        }
    }

    pub fn paint(&mut self, origin: gpui::Point<Pixels>, cx: &mut ViewContext<Self>) {
        let bounds = self.bounds(origin);

        let cursor = outline(bounds, self.color);

        cx.paint_quad(cursor);

        if let Some(block_text) = &self.block_text {
            block_text
                .paint(self.origin + origin, self.line_height, cx)
                .unwrap()
        }
    }
}

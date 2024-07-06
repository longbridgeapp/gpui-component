use std::sync::Arc;

use anyhow::Context;
use gpui::{
    px, Bounds, Element, Hitbox, ImageData, InteractiveElement, Interactivity, IntoElement,
    SharedString, StyleRefinement, Styled, WindowContext,
};

use image::ImageBuffer;

#[derive(Clone, Copy, Debug)]
struct SvgSize {
    width: u32,
    height: u32,
}

pub struct SvgImg {
    interactivity: Interactivity,
    size: SvgSize,
    data: Option<Arc<ImageData>>,
}

impl Clone for SvgImg {
    fn clone(&self) -> Self {
        Self {
            interactivity: Interactivity::default(),
            size: self.size,
            data: self.data.clone(),
        }
    }
}

/// An SVG image element.
pub fn svg_img(width: usize, height: usize) -> SvgImg {
    SvgImg::new(width, height)
}

impl SvgImg {
    /// Create a new svg image element.
    ///
    /// The `src_width` and `src_height` are the original width and height of the svg image.
    pub fn new(src_width: usize, src_height: usize) -> Self {
        Self {
            interactivity: Interactivity::default(),
            size: SvgSize {
                width: src_width as u32,
                height: src_height as u32,
            },
            data: None,
        }
    }

    /// Set the path of the svg image from the asset.
    pub fn path(self, path: impl Into<SharedString>, cx: &WindowContext) -> anyhow::Result<Self> {
        let svg = cx
            .asset_source()
            .load(&path.into())
            .expect("failed to load svg from asset")
            .expect("failed to load svg from asset, return none");

        self.svg(&svg, cx)
    }

    /// Set the svg image from the bytes.
    pub fn svg(mut self, svg: &[u8], cx: &WindowContext) -> anyhow::Result<Self> {
        let data = self.to_image_data(svg, cx)?;
        self.data = Some(Arc::new(data));
        Ok(self)
    }

    pub fn to_image_data(&self, bytes: &[u8], cx: &WindowContext) -> anyhow::Result<ImageData> {
        if self.size.width == 0 || self.size.height == 0 {
            return Err(usvg::Error::InvalidSize.into());
        }

        let scale = cx.scale_factor() as u32;
        let size = SvgSize {
            width: self.size.width * scale,
            height: self.size.height * scale,
        };

        let options = usvg::Options {
            ..Default::default()
        };
        let tree = usvg::Tree::from_data(&bytes, &options)?;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width, size.height)
            .ok_or(usvg::Error::InvalidSize)?;

        let transform = tree.view_box().to_transform(
            resvg::tiny_skia::Size::from_wh(size.width as f32, size.height as f32)
                .ok_or(usvg::Error::InvalidSize)?,
        );
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let mut buffer = ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.take())
            .context("invalid svg image buffer")?;

        // Convert from RGBA to BGRA.
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Ok(ImageData::new(buffer))
    }
}

impl IntoElement for SvgImg {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SvgImg {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<gpui::ElementId> {
        self.interactivity.element_id.clone()
    }

    fn request_layout(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let layout_id = self
            .interactivity
            .request_layout(global_id, cx, |style, cx| cx.request_layout(style, None));
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        self.interactivity
            .prepaint(global_id, bounds, bounds.size, cx, |_, _, hitbox, _| hitbox)
    }

    fn paint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        self.interactivity
            .paint(global_id, bounds, hitbox.as_ref(), cx, |_style, cx| {
                if let Some(data) = self.data.as_ref() {
                    // To calculate the ratio of the original image size to the container bounds size.
                    // Scale by shortest side (width or height) to get a fit image.
                    // And center the image in the container bounds.
                    let ratio = if bounds.size.width < bounds.size.height {
                        bounds.size.width.0 / self.size.width as f32
                    } else {
                        bounds.size.height.0 / self.size.height as f32
                    };

                    let ratio = ratio.min(1.0);

                    let new_size = gpui::Size {
                        width: px(self.size.width as f32 * ratio),
                        height: px(self.size.height as f32 * ratio),
                    };
                    let new_origin = gpui::Point {
                        x: bounds.origin.x + px(((bounds.size.width - new_size.width) / 2.).into()),
                        y: bounds.origin.y
                            + px(((bounds.size.height - new_size.height) / 2.).into()),
                    };

                    let img_bounds = Bounds {
                        size: new_size,
                        origin: new_origin,
                    };

                    match cx.paint_image(img_bounds, px(0.).into(), data.clone(), false) {
                        Ok(_) => {}
                        Err(err) => eprintln!("failed to paint svg image: {:?}", err),
                    }
                }
            })
    }
}

impl Styled for SvgImg {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.interactivity.base_style
    }
}

impl InteractiveElement for SvgImg {
    fn interactivity(&mut self) -> &mut Interactivity {
        &mut self.interactivity
    }
}

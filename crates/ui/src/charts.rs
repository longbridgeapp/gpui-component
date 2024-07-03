use gpui::{IntoElement, RenderOnce, SharedString, StyleRefinement, Styled, WindowContext};

use charts_rs::{BarChart, PieChart};

use crate::{svg_img, SvgImg};

#[derive(IntoElement)]
pub struct Chart {
    svg: SvgImg,
}

impl Clone for Chart {
    fn clone(&self) -> Self {
        Self {
            svg: self.svg.clone(),
        }
    }
}

pub enum ChartKind {
    Bar,
    Pie,
}

impl ChartKind {
    fn to_svg(&self, json: SharedString) -> anyhow::Result<String> {
        Ok(match self {
            Self::Bar => BarChart::from_json(&json)?.svg()?,
            Self::Pie => PieChart::from_json(&json)?.svg()?,
        })
    }
}

impl Chart {
    pub fn new(
        kind: ChartKind,
        width: usize,
        height: usize,
        json: impl Into<SharedString>,
        cx: &WindowContext,
    ) -> anyhow::Result<Self> {
        let json = kind.to_svg(json.into())?;

        Ok(Self {
            svg: svg_img(width, height).svg(json.as_bytes(), cx)?,
        })
    }
}

impl Styled for Chart {
    fn style(&mut self) -> &mut StyleRefinement {
        self.svg.style()
    }
}

impl RenderOnce for Chart {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        self.svg.into_element()
    }
}

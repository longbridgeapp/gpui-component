use gpui::{
    div, px, AnyElement, IntoElement, ParentElement as _, Render, SharedString, Styled, View,
    ViewContext, VisualContext, WindowContext,
};
use ui::theme::ActiveTheme;
use ui::{
    resizable::{h_resizable, resizable_panel, v_resizable, ResizablePanelGroup},
    v_flex,
};

pub struct ResizableStory {
    group1: View<ResizablePanelGroup>,
    group2: View<ResizablePanelGroup>,
}

impl ResizableStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    fn new(cx: &mut WindowContext) -> Self {
        fn panel_box(content: impl Into<SharedString>, cx: &WindowContext) -> AnyElement {
            div()
                .p_4()
                .border_1()
                .border_color(cx.theme().border)
                .size_full()
                .child(content.into())
                .into_any_element()
        }

        let group1 = cx.new_view(|cx| {
            v_resizable()
                .group(
                    h_resizable()
                        .size(px(150.))
                        .child(
                            resizable_panel()
                                .size(px(300.))
                                .min_size(px(120.))
                                .content(|cx| panel_box("Left 1 (Min 120px)", cx)),
                            cx,
                        )
                        .child(
                            resizable_panel()
                                .size(px(400.))
                                .min_size(px(100.))
                                .content(|cx| panel_box("Center 1", cx)),
                            cx,
                        )
                        .child(
                            resizable_panel()
                                .size(px(300.))
                                .min_size(px(100.))
                                .grow()
                                .content(|cx| panel_box("Right (Grow)", cx)),
                            cx,
                        ),
                    cx,
                )
                .child(
                    resizable_panel()
                        .size(px(150.))
                        .max_size(px(550.))
                        .min_size(px(100.))
                        .grow()
                        .content(|cx| panel_box("Center (Grow)", cx)),
                    cx,
                )
                .child(
                    resizable_panel()
                        .size(px(210.))
                        .min_size(px(100.))
                        .content(|cx| panel_box("Bottom", cx)),
                    cx,
                )
        });

        let group2 = cx.new_view(|cx| {
            h_resizable()
                .child(
                    resizable_panel()
                        .size(px(300.))
                        .min_size(px(100.))
                        .content(|cx| panel_box("Left 2", cx)),
                    cx,
                )
                .child(
                    resizable_panel()
                        .size(px(400.))
                        .max_size(px(550.))
                        .min_size(px(100.))
                        .grow()
                        .content(|cx| panel_box("Right (Grow)", cx)),
                    cx,
                )
        });
        Self { group1, group2 }
    }
}

impl Render for ResizableStory {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .gap_6()
            .child(self.group1.clone())
            .child(self.group2.clone())
    }
}

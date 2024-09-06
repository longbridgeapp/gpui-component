use gpui::{
    div, Corners, Div, Edges, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, WindowContext,
};

use crate::button::Button;

#[derive(IntoElement)]
pub struct ButtonGroup {
    pub base: Div,
    id: ElementId,
    children: Vec<Button>,
}

impl ButtonGroup {
    /// Creates a new ButtonGroup.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div(),
            children: Vec::new(),
            id: id.into(),
        }
    }

    /// Adds a button as a child to the ButtonGroup.
    pub fn child(mut self, child: Button) -> Self {
        self.children.push(child);
        self
    }
}

impl Styled for ButtonGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let children_len = self.children.len();
        // Render a div container with a flex layout to group buttons horizontally.
        self.base
            .id(self.id)
            .flex()
            .items_center()
            .children(if children_len > 1 {
                self.children
                    .into_iter()
                    .enumerate()
                    .map(|(index, button)| {
                        if index == 0 {
                            // First
                            button
                                .border_corners(Corners {
                                    top_left: true,
                                    top_right: false,
                                    bottom_left: true,
                                    bottom_right: false,
                                })
                                .border_edges(Edges {
                                    left: true,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                        } else if index == children_len - 1 {
                            // Last
                            button
                                .border_edges(Edges {
                                    left: false,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                                .border_corners(Corners {
                                    top_left: false,
                                    top_right: true,
                                    bottom_left: false,
                                    bottom_right: true,
                                })
                        } else {
                            // Middle
                            button
                                .border_corners(Corners::all(false))
                                .border_edges(Edges {
                                    left: false,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                        }
                    })
                    .collect()
            } else {
                self.children
            })
    }
}

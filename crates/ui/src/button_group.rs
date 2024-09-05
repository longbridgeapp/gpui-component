use gpui::{
    div, Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    WindowContext,
};

use crate::button::{Button, ButtonBorderSide, ButtonRounded, ButtonRoundedSide};

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
                            // First button: Rounded on the left side only
                            button
                                .rounded_side(ButtonRoundedSide::Left)
                                .border_side(ButtonBorderSide::NoRight)
                        } else if index == children_len - 1 {
                            // Last button: Rounded on the right side only
                            button.rounded_side(ButtonRoundedSide::Right)
                        } else {
                            // Middle buttons: No rounding
                            button
                                .rounded(ButtonRounded::None)
                                .border_side(ButtonBorderSide::NoRight)
                        }
                    })
                    .collect()
            } else {
                self.children
            })
    }
}

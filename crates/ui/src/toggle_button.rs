use crate::button::Button;
use gpui::{
    div, prelude::FluentBuilder as _, Corners, Div, Edges, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement as _, Styled, WindowContext,
};
use std::{cell::Cell, rc::Rc};

/// The ToggleButton controls the selected state of its child buttons.
#[derive(IntoElement)]
pub struct ToggleButton {
    base: Div,
    id: ElementId,
    children: Vec<Button>,
    exclusive: bool,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&Vec<usize>, &mut WindowContext) + 'static>>,
}

impl ToggleButton {
    /// Creates a new ToggleButton.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div(),
            id: id.into(),
            children: Vec::new(),
            exclusive: false,
            disabled: false,
            on_click: None,
        }
    }

    /// Adds a ToggleButton as a child to the ToggleButton.
    pub fn child(mut self, child: Button) -> Self {
        self.children.push(child);
        self
    }

    /// Restrict that there can only be one currently selected value for the ToggleButton.
    pub fn exclusive(mut self, val: bool) -> Self {
        self.exclusive = val;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&Vec<usize>, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Styled for ToggleButton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}
impl RenderOnce for ToggleButton {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let children_len = self.children.len();
        let mut selected: Vec<usize> = Vec::new();
        let shared_state = Rc::new(Cell::new(None)); // Shared state to store the child index

        for (child_index, child) in self.children.iter().enumerate() {
            if child.selected {
                selected.push(child_index);
            }
        }

        self.base
            .id(self.id)
            .flex()
            .items_center()
            .children(
                self.children
                    .into_iter()
                    .enumerate()
                    .map(|(child_index, child)| {
                        let shared_state_clone = Rc::clone(&shared_state);
                        if children_len == 1 {
                            child
                        } else if child_index == 0 {
                            // First
                            child
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
                        } else if child_index == children_len - 1 {
                            // Last
                            child
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
                            child
                                .border_corners(Corners::all(false))
                                .border_edges(Edges {
                                    left: false,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                        }
                        .stop_propagation(false)
                        .on_click(move |_, ctx| {
                            shared_state_clone.set(Some(child_index)); // Record child_index into shared state
                            ctx.refresh();
                        })
                    }),
            )
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                move |this, on_click| {
                    this.on_click(move |_, cx| {
                        let mut selected = selected.clone();
                        if let Some(index) = shared_state.get() {
                            if self.exclusive {
                                selected.clear(); // Clear the existing selection
                                selected.push(index); // Replace with the new selection
                            } else {
                                if let Some(pos) = selected.iter().position(|&i| i == index) {
                                    selected.remove(pos); // Toggle off if already selected
                                } else {
                                    selected.push(index); // Toggle on if not selected
                                }
                            }
                        }

                        on_click(&selected, cx);
                        cx.refresh()
                    })
                },
            )
    }
}

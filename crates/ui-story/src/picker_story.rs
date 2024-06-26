use gpui::{
    div, prelude::FluentBuilder as _, px, Div, Empty, Entity, InteractiveElement as _, IntoElement,
    ParentElement, Render, SharedString, Styled, Task, View, ViewContext, VisualContext as _,
    WindowContext,
};

use ui::{
    button::Button,
    h_flex,
    label::Label,
    picker::{Picker, PickerDelegate},
    switch::{LabelSide, Switch},
    theme::{ActiveTheme, Colorize},
    v_flex, Clickable as _, Disableable as _, StyledExt,
};

use super::story_case;

pub struct ListItemDeletegate {
    selected_index: usize,
    items: Vec<String>,
    matches: Vec<String>,
}

impl PickerDelegate for ListItemDeletegate {
    type ListItem = Div;

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _cx: &mut ViewContext<Picker<Self>>) {
        self.selected_index = index
    }

    fn render_match(
        &self,
        ix: usize,
        selected: bool,
        cx: &mut ViewContext<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        if let Some(item) = self.matches.get(ix) {
            let list_item = div()
                .py_1()
                .px_3()
                .when(!selected, |this| {
                    this.hover(|this| this.bg(cx.theme().card))
                })
                .child(item.clone())
                .text_base()
                .text_color(cx.theme().foreground)
                .when(selected, |this| this.bg(cx.theme().card.lighten(0.1)));
            Some(list_item)
        } else {
            None
        }
    }

    fn update_matches(
        &mut self,
        query: &str,
        cx: &mut ViewContext<Picker<Self>>,
    ) -> gpui::Task<()> {
        let matched_items = self
            .items
            .iter()
            .filter(|item| item.contains(query))
            .cloned()
            .collect();

        self.matches = matched_items;

        Task::ready(())
    }
}

pub struct PickerStory {
    picker: View<Picker<ListItemDeletegate>>,
    open: bool,
    selected_value: Option<String>,
}

impl PickerStory {
    pub(crate) fn new(cx: &mut WindowContext) -> Self {
        let items = [
            "Baguette (France)",
            "Baklava (Turkey)",
            "Beef Wellington (UK)",
            "Biryani (India)",
            "Borscht (Ukraine)",
            "Bratwurst (Germany)",
            "Bulgogi (Korea)",
            "Burrito (USA)",
            "Ceviche (Peru)",
            "Chicken Tikka Masala (India)",
            "Churrasco (Brazil)",
            "Couscous (North Africa)",
            "Croissant (France)",
            "Dim Sum (China)",
            "Empanada (Argentina)",
            "Fajitas (Mexico)",
            "Falafel (Middle East)",
            "Feijoada (Brazil)",
            "Fish and Chips (UK)",
            "Fondue (Switzerland)",
            "Goulash (Hungary)",
            "Haggis (Scotland)",
            "Kebab (Middle East)",
            "Kimchi (Korea)",
            "Lasagna (Italy)",
            "Maple Syrup Pancakes (Canada)",
            "Moussaka (Greece)",
            "Pad Thai (Thailand)",
            "Paella (Spain)",
            "Pancakes (USA)",
            "Pasta Carbonara (Italy)",
            "Pavlova (Australia)",
            "Peking Duck (China)",
            "Pho (Vietnam)",
            "Pierogi (Poland)",
            "Pizza (Italy)",
            "Poutine (Canada)",
            "Pretzel (Germany)",
            "Ramen (Japan)",
            "Rendang (Indonesia)",
            "Sashimi (Japan)",
            "Satay (Indonesia)",
            "Shepherd's Pie (Ireland)",
            "Sushi (Japan)",
            "Tacos (Mexico)",
            "Tandoori Chicken (India)",
            "Tortilla (Spain)",
            "Tzatziki (Greece)",
            "Wiener Schnitzel (Austria)",
        ];

        let picker = cx.new_view(|cx| {
            let items: Vec<String> = items.iter().map(|s| s.to_string()).collect();

            let mut picker = Picker::uniform_list(
                ListItemDeletegate {
                    selected_index: 0,
                    matches: items.clone(),
                    items,
                },
                cx,
            )
            .modal(true)
            .max_height(Some(px(350.0).into()));
            picker.focus(cx);
            picker.set_query("c", cx);
            picker
        });

        Self {
            picker,
            open: false,
            selected_value: None,
        }
    }
}

impl Render for PickerStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case("Picker", "Picker is a list of items that can be selected.")
            .child(v_flex().items_start().child(
                Button::new("show-picker", "Show Picker...").on_click(cx.listener(
                    |this, _, cx| {
                        this.open = !this.open;
                        cx.notify();
                    },
                )),
            ))
            .when_some(self.selected_value.clone(), |this, selected_value| {
                this.child("Selected: ").child(Label::new(selected_value))
            })
            .when(self.open, |this| {
                this.child(
                    div().absolute().size_full().top_0().left_0().child(
                        v_flex()
                            // .h(px(0.0))
                            .top_10()
                            .flex()
                            .flex_col()
                            .items_center()
                            .track_focus(&self.picker.focus_handle(cx))
                            .child(
                                h_flex()
                                    .w(px(450.))
                                    .occlude()
                                    .child(self.picker.clone())
                                    .on_mouse_down_out(cx.listener(|this, _, cx| {
                                        this.open = false;
                                        cx.notify();
                                    })),
                            ),
                    ),
                )
            })
    }
}
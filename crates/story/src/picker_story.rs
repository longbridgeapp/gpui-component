use std::{sync::Arc, time::Duration};

use fake::Fake;
use gpui::{
    deferred, div, prelude::FluentBuilder as _, px, Animation, AnimationExt as _, FocusHandle,
    FocusableView, InteractiveElement as _, IntoElement, ParentElement, Render, Styled, Task,
    Timer, View, ViewContext, VisualContext as _, WeakView, WindowContext,
};

use ui::{
    button::{Button, ButtonStyle},
    h_flex,
    list::{List, ListDelegate, ListItem},
    theme::ActiveTheme as _,
    v_flex, Clickable as _, Icon, IconName, StyledExt,
};

pub struct ListItemDeletegate {
    story: WeakView<PickerStory>,
    confirmed_index: Option<usize>,
    selected_index: Option<usize>,
    items: Vec<Arc<String>>,
    matches: Vec<Arc<String>>,
}

impl ListDelegate for ListItemDeletegate {
    type Item = ListItem;

    fn items_count(&self) -> usize {
        self.matches.len()
    }

    fn confirmed_index(&self) -> Option<usize> {
        self.confirmed_index
    }

    fn perform_search(&mut self, query: &str, cx: &mut ViewContext<List<Self>>) -> Task<()> {
        let query = query.to_string();
        cx.spawn(move |this, mut cx| async move {
            // Simulate a slow search.
            let sleep = (0.05..0.1).fake();
            Timer::after(Duration::from_secs_f64(sleep)).await;

            this.update(&mut cx, |this, cx| {
                this.delegate_mut().matches = this
                    .delegate()
                    .items
                    .iter()
                    .filter(|item| item.to_lowercase().contains(&query.to_lowercase()))
                    .cloned()
                    .collect();
                cx.notify();
            })
            .ok();
        })
    }

    fn render_item(&self, ix: usize, _: &mut ViewContext<List<Self>>) -> Option<Self::Item> {
        let confirmed = Some(ix) == self.confirmed_index;
        let selected = Some(ix) == self.selected_index;

        if let Some(item) = self.matches.get(ix) {
            let list_item = ListItem::new(("item", ix))
                .check_icon(ui::IconName::Check)
                .confirmed(confirmed)
                .selected(selected)
                .py_1()
                .px_3()
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(item.to_string()),
                )
                .suffix(|cx| {
                    Button::new("like", cx)
                        .icon(IconName::Heart)
                        .style(ButtonStyle::Ghost)
                        .size(px(18.))
                        .on_click(move |_, cx| {
                            cx.stop_propagation();
                            cx.prevent_default();

                            println!("You have clicked like.");
                        })
                });
            Some(list_item)
        } else {
            None
        }
    }

    fn render_empty(&self, cx: &mut ViewContext<List<Self>>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                Icon::new(IconName::Inbox)
                    .size(px(50.))
                    .text_color(cx.theme().muted_foreground),
            )
            .child("No matches found")
            .items_center()
            .justify_center()
            .p_3()
            .bg(cx.theme().muted)
            .text_color(cx.theme().muted_foreground)
    }

    fn cancel(&mut self, cx: &mut ViewContext<List<Self>>) {
        if let Some(story) = self.story.upgrade() {
            cx.update_view(&story, |story, cx| {
                story.open = false;
                cx.notify();
            });
        }
    }

    fn confirm(&mut self, ix: Option<usize>, cx: &mut ViewContext<List<Self>>) {
        if let Some(story) = self.story.upgrade() {
            cx.update_view(&story, |story, cx| {
                if let Some(ix) = ix {
                    self.confirmed_index = Some(ix);
                    if let Some(item) = self.matches.get(ix) {
                        story.selected_value = Some(item.clone());
                    }
                }
                story.open = false;
                cx.notify();
            });
        }
    }

    fn set_selected_index(&mut self, ix: Option<usize>, cx: &mut ViewContext<List<Self>>) {
        self.selected_index = ix;

        if let Some(_) = ix {
            cx.notify();
        }
    }
}

pub struct PickerStory {
    list: View<List<ListItemDeletegate>>,
    open: bool,
    selected_value: Option<Arc<String>>,
}

impl PickerStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let items: Vec<Arc<String>> = [
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
        ]
        .iter()
        .map(|s| Arc::new(s.to_string()))
        .collect();

        let story = cx.view().downgrade();
        let delegate = ListItemDeletegate {
            story,
            selected_index: None,
            confirmed_index: None,
            items: items.clone(),
            matches: items.clone(),
        };
        let list = cx.new_view(|cx| {
            let mut list = List::new(delegate, cx);
            list.focus(cx);
            list
        });

        Self {
            list,
            open: false,
            selected_value: None,
        }
    }
}

impl FocusableView for PickerStory {
    fn focus_handle(&self, cx: &gpui::AppContext) -> FocusHandle {
        self.list.focus_handle(cx)
    }
}

impl Render for PickerStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .gap_6()
            .child(
                v_flex().items_start().child(
                    Button::new("show-picker", cx)
                        .label("Show Picker...")
                        .icon(IconName::Search)
                        .on_click(cx.listener(|this, _, cx| {
                            this.open = !this.open;
                            this.list.focus_handle(cx).focus(cx);
                            cx.notify();
                        })),
                ),
            )
            .when_some(self.selected_value.clone(), |this, selected_value| {
                this.child(
                    h_flex().gap_1().child("You have selected:").child(
                        div()
                            .child(selected_value.to_string())
                            .text_color(gpui::red()),
                    ),
                )
            })
            .when(self.open, |this| {
                this.child(deferred(
                    div()
                        .absolute()
                        .size_full()
                        .top_0()
                        .left_0()
                        .child(
                            v_flex().flex().flex_col().items_center().child(
                                v_flex()
                                    .occlude()
                                    .w(px(450.))
                                    .h(px(350.))
                                    .elevation_3(cx)
                                    .child(self.list.clone())
                                    .on_mouse_down_out(cx.listener(|this, _, cx| {
                                        this.open = false;
                                        cx.notify();
                                    })),
                            ),
                        )
                        .with_animation(
                            "slide-down",
                            Animation::new(Duration::from_secs_f64(0.15)),
                            move |this, delta| {
                                let y = px(-10.) + delta * px(10.);
                                this.top(y).opacity((1.0 * delta + 0.3).min(1.0))
                            },
                        ),
                ))
            })
    }
}

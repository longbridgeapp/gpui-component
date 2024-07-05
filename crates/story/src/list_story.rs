use core::time;

use fake::Fake;
use gpui::{
    actions, div, px, ElementId, FocusHandle, FocusableView, InteractiveElement, IntoElement,
    ParentElement, Render, RenderOnce, Styled, Timer, View, ViewContext, VisualContext,
    WindowContext,
};

use ui::{
    h_flex,
    label::Label,
    list::ListItem,
    picker::{Picker, PickerDelegate},
    theme::{hsl, ActiveTheme, Colorize as _},
    v_flex,
};

actions!(list_story, [SelectedCompany]);

#[derive(Clone)]
struct Company {
    name: String,
    industry: String,
    last_done: f64,
    prev_close: f64,
    // description: String,
}

impl Company {
    fn random_update(&mut self) {
        self.last_done = self.prev_close * (1.0 + (-0.2..0.2).fake::<f64>());
    }

    fn change_percent(&self) -> f64 {
        (self.last_done - self.prev_close) / self.prev_close
    }
}

#[derive(IntoElement)]
struct CompanyListItem {
    base: ListItem,
    ix: usize,
    company: Company,
    selected: bool,
}

impl CompanyListItem {
    pub fn new(id: impl Into<ElementId>, company: Company, ix: usize, selected: bool) -> Self {
        CompanyListItem {
            company,
            ix,
            base: ListItem::new(id),
            selected,
        }
    }
}

impl RenderOnce for CompanyListItem {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let text_color = if self.selected {
            cx.theme().accent_foreground
        } else {
            cx.theme().foreground
        };

        let trend_color = match self.company.change_percent() {
            change if change > 0.0 => hsl(0.0, 79.0, 53.0),
            change if change < 0.0 => hsl(100.0, 79.0, 53.0),
            _ => cx.theme().foreground,
        };

        let bg_color = if self.selected {
            cx.theme().accent
        } else {
            if self.ix % 2 == 0 {
                cx.theme().background
            } else {
                cx.theme().accent.opacity(0.3)
            }
        };

        self.base
            .px_3()
            .py_1()
            .overflow_x_hidden()
            .bg(bg_color)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .text_color(text_color)
                    .child(
                        v_flex()
                            .gap_2()
                            .max_w(px(500.))
                            .overflow_x_hidden()
                            .flex_nowrap()
                            .child(Label::new(self.company.name.clone()).whitespace_nowrap())
                            .child(
                                div().text_sm().overflow_x_hidden().child(
                                    Label::new(self.company.industry.clone())
                                        .whitespace_nowrap()
                                        .text_color(text_color.opacity(0.5)),
                                ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .justify_end()
                            .child(
                                div()
                                    .w(px(65.))
                                    .text_color(text_color)
                                    .child(format!("{:.2}", self.company.last_done)),
                            )
                            .child(
                                h_flex().w(px(65.)).justify_end().child(
                                    div()
                                        .rounded_md()
                                        .whitespace_nowrap()
                                        .text_size(px(12.))
                                        .px_1()
                                        .text_color(trend_color)
                                        .child(format!("{:.2}%", self.company.change_percent())),
                                ),
                            ),
                    ),
            )
    }
}

struct CompanyListDelegate {
    companies: Vec<Company>,
    selected_index: usize,
}

impl PickerDelegate for CompanyListDelegate {
    type ListItem = CompanyListItem;

    fn match_count(&self) -> usize {
        self.companies.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, ix: usize, cx: &mut ViewContext<ui::picker::Picker<Self>>) {
        self.selected_index = ix;
        cx.dispatch_action(Box::new(SelectedCompany));
    }

    fn render_item(
        &self,
        ix: usize,
        selected: bool,
        _cx: &mut ViewContext<ui::picker::Picker<Self>>,
    ) -> Option<Self::ListItem> {
        if let Some(company) = self.companies.get(ix) {
            return Some(CompanyListItem::new(ix, company.clone(), ix, selected));
        }

        None
    }
}

impl CompanyListDelegate {
    fn selected_company(&self) -> Option<Company> {
        self.companies.get(self.selected_index).cloned()
    }
}

pub struct ListStory {
    focus_handle: FocusHandle,
    company_list: View<Picker<CompanyListDelegate>>,
    selected_company: Option<Company>,
}

impl ListStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let companies = (0..1_000)
            .map(|_| random_company())
            .collect::<Vec<Company>>();

        let company_list = cx.new_view(|cx| {
            Picker::uniform_list(
                CompanyListDelegate {
                    companies,
                    selected_index: 0,
                },
                cx,
            )
            // .max_height(Some(px(350.0).into()))
            .no_query()
        });

        // Spawn a background to random refresh the list
        cx.spawn(move |this, mut cx| async move {
            loop {
                Timer::after(time::Duration::from_secs_f64(0.5)).await;
                this.update(&mut cx, |this, cx| {
                    this.company_list.update(cx, |picker, _| {
                        picker
                            .delegate_mut()
                            .companies
                            .iter_mut()
                            .for_each(|company| {
                                company.random_update();
                            });
                    });
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self {
            focus_handle: cx.focus_handle(),
            company_list,
            selected_company: None,
        }
    }

    fn selected_company(&mut self, _: &SelectedCompany, cx: &mut ViewContext<Self>) {
        let picker = self.company_list.read(cx);
        if let Some(company) = picker.delegate().selected_company() {
            self.selected_company = Some(company);
        }
    }
}

fn random_company() -> Company {
    let last_done = (0.0..999.0).fake::<f64>();
    let prev_close = last_done * (-0.1..0.1).fake::<f64>();
    Company {
        name: fake::faker::company::en::CompanyName().fake(),
        industry: fake::faker::company::en::Industry().fake(),
        // description: fake::faker::lorem::en::Paragraph(3..5).fake(),
        last_done,
        prev_close,
    }
}

impl FocusableView for ListStory {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ListStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::selected_company))
            .size_full()
            .gap_4()
            .mb_4()
            .child(
                v_flex()
                    .h_full()
                    .w_full()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .child(self.company_list.clone()),
            )
        // .child(
        //     div()
        //         .invisible()
        //         .flex_1()
        //         .size_full()
        //         .border_1()
        //         .border_color(cx.theme().border)
        //         .py_1()
        //         .px_4()
        //         .rounded_md()
        //         .when_some(self.selected_company.clone(), |this, company| {
        //             this.child(
        //                 div()
        //                     .flex_1()
        //                     .gap_2()
        //                     .child(
        //                         h_flex()
        //                             .items_start()
        //                             .justify_between()
        //                             .child(div().text_3xl().mb_6().child(company.name.clone()))
        //                             .child(format!("{:.2}", company.last_done)),
        //                     )
        //                     .child(company.industry.clone())
        //                     .child(company.description.clone()),
        //             )
        //         }),
        // )
    }
}

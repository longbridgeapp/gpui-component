use std::{fmt::format, rc::Rc};

use fake::Fake;
use gpui::{
    actions, div, impl_actions, prelude::FluentBuilder as _, px, Action, ElementId,
    InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, Styled, View, ViewContext,
    VisualContext, WindowContext,
};

use ui::{
    h_flex,
    label::Label,
    list::ListItem,
    picker::{Picker, PickerDelegate},
    theme::{hsl, ActiveTheme, Colorize as _},
    v_flex, Icon, IconName,
};

use super::story_case;

actions!(list_story, [SelectedCompany]);

struct Company {
    name: String,
    industry: String,
    last_done: f64,
    prev_close: f64,
    description: String,
}

impl Company {
    fn random_update(&mut self) {
        self.last_done = self.prev_close * (-0.1..0.1).fake::<f64>();
    }

    fn change_percent(&self) -> f64 {
        (self.last_done - self.prev_close) / self.prev_close
    }
}

#[derive(IntoElement)]
struct CompanyListItem {
    base: ListItem,
    copmpany: Rc<Company>,
    selected: bool,
}

impl CompanyListItem {
    pub fn new(id: impl Into<ElementId>, copmpany: Rc<Company>, selected: bool) -> Self {
        CompanyListItem {
            copmpany,
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

        let trend_color = match self.copmpany.change_percent() {
            change if change > 0.0 => hsl(0.0, 79.0, 53.0),
            change if change < 0.0 => hsl(100.0, 79.0, 53.0),
            _ => cx.theme().foreground,
        };

        self.base
            .px_3()
            .py_1()
            .rounded_lg()
            .overflow_x_hidden()
            .when(self.selected, |this| this.bg(cx.theme().accent))
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
                            .child(Label::new(self.copmpany.name.clone()))
                            .child(
                                div().text_sm().overflow_x_hidden().child(
                                    Label::new(self.copmpany.industry.clone())
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
                                    .child(format!("{:.2}", self.copmpany.last_done)),
                            )
                            .child(
                                div().w(px(65.)).child(
                                    div()
                                        .flex_none()
                                        .text_color(cx.theme().primary_foreground)
                                        .rounded_md()
                                        .text_size(px(12.))
                                        .px_1()
                                        .bg(trend_color)
                                        .child(format!("{:.2}%", self.copmpany.change_percent())),
                                ),
                            ),
                    ),
            )
    }
}

struct CompanyListDelegate {
    companies: Vec<Rc<Company>>,
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
            return Some(CompanyListItem::new(ix, company.clone(), selected));
        }

        None
    }
}

impl CompanyListDelegate {
    fn selected_company(&self) -> Option<Rc<Company>> {
        self.companies.get(self.selected_index).cloned()
    }
}

pub struct ListStory {
    company_list: View<Picker<CompanyListDelegate>>,
    selected_company: Option<Rc<Company>>,
}

fn random_company() -> Company {
    let last_done = (0.0..999.0).fake::<f64>();
    let prev_close = last_done * (-0.1..0.1).fake::<f64>();
    Company {
        name: fake::faker::company::en::CompanyName().fake(),
        industry: fake::faker::company::en::Industry().fake(),
        description: fake::faker::lorem::en::Paragraph(3..5).fake(),
        last_done,
        prev_close,
    }
}

impl ListStory {
    pub(crate) fn new(cx: &mut ViewContext<Self>) -> Self {
        let companies = (0..10_000)
            .into_iter()
            .map(|_| Rc::new(random_company()))
            .collect::<Vec<Rc<Company>>>();

        let company_list = cx.new_view(|cx| {
            Picker::uniform_list(
                CompanyListDelegate {
                    companies,
                    selected_index: 0,
                },
                cx,
            )
            .no_query()
        });

        Self {
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

impl Render for ListStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case("List", "A list for container 10K ListItems.").child(
            h_flex()
                .size_full()
                .on_action(cx.listener(Self::selected_company))
                .gap_4()
                .mb_4()
                .child(
                    div()
                        .h_full()
                        .border_1()
                        .border_color(cx.theme().border)
                        .p_1()
                        .rounded_md()
                        .w(px(400.))
                        .child(self.company_list.clone()),
                )
                .child(
                    div()
                        .flex_1()
                        .size_full()
                        .border_1()
                        .border_color(cx.theme().border)
                        .py_1()
                        .px_4()
                        .rounded_md()
                        .when_some(self.selected_company.clone(), |this, company| {
                            this.child(
                                div()
                                    .flex_1()
                                    .gap_2()
                                    .child(div().text_3xl().mb_6().child(company.name.clone()))
                                    .child(company.industry.clone())
                                    .child(company.description.clone()),
                            )
                        }),
                ),
        )
    }
}

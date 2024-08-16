use fake::Fake;
use gpui::{
    div, img, IntoElement, ParentElement, Pixels, Render, SharedString, Styled, View, ViewContext,
    VisualContext as _, WindowContext,
};
use ui::{
    checkbox::Checkbox,
    h_flex,
    label::Label,
    table::{ColSort, Table, TableDelegate, TableEvent},
    theme::ActiveTheme as _,
    v_flex, Icon, IconName, Selectable,
};

struct Customer {
    id: usize,
    login: String,
    first_name: String,
    last_name: String,
    company: String,
    city: String,
    country: String,
    email: String,
    phone: String,
    gender: usize,
    age: usize,
    verified: bool,
    confirmed: bool,
}

impl Customer {
    fn render_avatar(&self, _: &mut WindowContext) -> impl IntoElement {
        let image_id = self.id % 70 + 1;
        let avatar_url = format!("https://i.pravatar.cc/40?image={}", image_id);
        img(avatar_url).size_5().rounded_full()
    }
}

fn randome_customers(size: usize) -> Vec<Customer> {
    (0..size)
        .map(|id| Customer {
            id,
            login: fake::faker::internet::en::Username().fake::<String>(),
            first_name: fake::faker::name::en::FirstName().fake::<String>(),
            last_name: fake::faker::name::en::LastName().fake::<String>(),
            company: fake::faker::company::en::CompanyName().fake::<String>(),
            city: fake::faker::address::en::CityName().fake::<String>(),
            country: fake::faker::address::en::CountryName().fake::<String>(),
            email: fake::faker::internet::en::FreeEmail().fake::<String>(),
            phone: fake::faker::phone_number::en::PhoneNumber().fake::<String>(),
            gender: (0..=1).fake(),
            age: (18..80).fake(),
            verified: (0..=1).fake::<u8>() == 1,
            confirmed: (0..=1).fake::<u8>() == 1,
        })
        .collect()
}

struct Column {
    id: SharedString,
    name: SharedString,
    sort: Option<ColSort>,
}

impl Column {
    fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        sort: Option<ColSort>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sort,
        }
    }
}

struct CustomerTableDelegate {
    customers: Vec<Customer>,
    columns: Vec<Column>,
    loop_selection: bool,
    col_resize: bool,
    col_order: bool,
    col_sort: bool,
    col_selection: bool,
}

impl CustomerTableDelegate {
    fn new(size: usize) -> Self {
        Self {
            customers: randome_customers(size),
            columns: vec![
                Column::new("id", "ID", Some(ColSort::Ascending)),
                Column::new("login", "Login", Some(ColSort::Default)),
                Column::new("first_name", "First Name", Some(ColSort::Default)),
                Column::new("last_name", "Last Name", Some(ColSort::Default)),
                Column::new("company", "Company", Some(ColSort::Default)),
                Column::new("city", "City", Some(ColSort::Default)),
                Column::new("country", "Country", Some(ColSort::Default)),
                Column::new("email", "Email", Some(ColSort::Default)),
                Column::new("phone", "Phone", None),
                Column::new("gender", "Gender", None),
                Column::new("age", "Age", Some(ColSort::Default)),
                Column::new("verified", "Verified", None),
                Column::new("confirmed", "Confirmed", None),
                Column::new("twitter", "Twitter", None),
            ],
            loop_selection: true,
            col_resize: true,
            col_order: true,
            col_sort: true,
            col_selection: true,
        }
    }
}

impl TableDelegate for CustomerTableDelegate {
    fn cols_count(&self) -> usize {
        self.columns.len()
    }

    fn rows_count(&self) -> usize {
        self.customers.len()
    }

    fn col_name(&self, col_ix: usize) -> SharedString {
        if let Some(col) = self.columns.get(col_ix) {
            col.name.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize) -> Option<Pixels> {
        if let Some(col) = self.columns.get(col_ix) {
            Some(
                match col.id.as_ref() {
                    "id" => 100.0,
                    "login" => 220.0,
                    "first_name" => 150.0,
                    "last_name" => 150.0,
                    "company" => 300.0,
                    "city" => 200.0,
                    "country" => 200.0,
                    "email" => 350.0,
                    "phone" => 240.0,
                    "gender" => 80.0,
                    "age" => 90.0,
                    "verified" => 90.0,
                    "confirmed" => 90.0,
                    "twitter" => 90.0,
                    _ => 200.0,
                }
                .into(),
            )
        } else {
            None
        }
    }

    fn can_resize_col(&self, col_ix: usize) -> bool {
        return self.col_resize && col_ix > 1;
    }

    fn can_select_col(&self, _: usize) -> bool {
        return self.col_selection;
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        cx: &mut ViewContext<Table<Self>>,
    ) -> impl IntoElement {
        let customer = self.customers.get(row_ix).unwrap();

        let col = self.columns.get(col_ix).unwrap();
        match col.id.as_ref() {
            "id" => customer.id.to_string().into_any_element(),
            "login" => h_flex()
                .items_center()
                .gap_2()
                .child(customer.render_avatar(cx))
                .child(customer.login.clone())
                .into_any_element(),
            "first_name" => customer.first_name.clone().into_any_element(),
            "last_name" => customer.last_name.clone().into_any_element(),
            "company" => h_flex()
                .items_center()
                .justify_between()
                .gap_1()
                .child(customer.company.clone())
                .child(IconName::Info)
                .into_any_element(),
            "city" => customer.city.clone().into_any_element(),
            "country" => customer.country.clone().into_any_element(),
            "email" => customer.email.clone().into_any_element(),
            "phone" => customer.phone.clone().into_any_element(),
            "gender" => match customer.gender {
                0 => "Male",
                1 => "Famale",
                _ => "",
            }
            .into_any_element(),
            "age" => customer.age.to_string().into_any_element(),
            "verified" => match customer.verified {
                true => "Yes".to_string().into_any_element(),
                false => "No".to_string().into_any_element(),
            },
            "confirmed" => match customer.confirmed {
                true => Icon::new(IconName::Check).size_4().into_any_element(),
                false => div().into_any_element(),
            },
            _ => Label::new("--")
                .text_color(cx.theme().muted_foreground)
                .into_any_element(),
        }
    }

    fn can_loop_select(&self) -> bool {
        self.loop_selection
    }

    fn can_move_col(&self, _: usize) -> bool {
        self.col_order
    }

    fn move_col(&mut self, col_ix: usize, to_ix: usize) {
        let col = self.columns.remove(col_ix);
        self.columns.insert(to_ix, col);
    }

    fn col_sort(&self, col_ix: usize) -> Option<ColSort> {
        if !self.col_sort {
            return None;
        }

        self.columns.get(col_ix).and_then(|c| c.sort)
    }

    fn perform_sort(&mut self, col_ix: usize, sort: ColSort, _: &mut ViewContext<Table<Self>>) {
        if !self.col_sort {
            return;
        }

        if let Some(col) = self.columns.get_mut(col_ix) {
            col.sort = Some(sort);
            let asc = matches!(sort, ColSort::Ascending);

            match col.id.as_ref() {
                "id" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.id.cmp(&b.id)
                    } else {
                        b.id.cmp(&a.id)
                    }
                }),
                "login" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.login.cmp(&b.login)
                    } else {
                        b.login.cmp(&a.login)
                    }
                }),
                "first_name" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.first_name.cmp(&b.first_name)
                    } else {
                        b.first_name.cmp(&a.first_name)
                    }
                }),
                "last_name" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.last_name.cmp(&b.last_name)
                    } else {
                        b.last_name.cmp(&a.last_name)
                    }
                }),
                "company" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.company.cmp(&b.company)
                    } else {
                        b.company.cmp(&a.company)
                    }
                }),
                "city" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.city.cmp(&b.city)
                    } else {
                        b.city.cmp(&a.city)
                    }
                }),
                "country" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.country.cmp(&b.country)
                    } else {
                        b.country.cmp(&a.country)
                    }
                }),
                "email" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.email.cmp(&b.email)
                    } else {
                        b.email.cmp(&a.email)
                    }
                }),
                "age" => self.customers.sort_by(|a, b| {
                    if asc {
                        a.age.cmp(&b.age)
                    } else {
                        b.age.cmp(&a.age)
                    }
                }),
                _ => {}
            }

            for col in self.columns.iter_mut() {
                if let Some(ColSort::Ascending) = col.sort {
                    col.sort = Some(ColSort::Default);
                }
            }
        }
    }
}

pub struct TableStory {
    table: View<Table<CustomerTableDelegate>>,
}

impl TableStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let delegate = CustomerTableDelegate::new(5000);
        let table = cx.new_view(|cx| Table::new(delegate, cx));

        cx.subscribe(&table, Self::on_table_event).detach();

        Self { table }
    }

    fn toggle_loop_selection(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().loop_selection = *checked;
            cx.notify();
        });
    }

    fn toggle_col_resize(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().col_resize = *checked;
            cx.notify();
        });
    }

    fn toggle_col_order(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().col_order = *checked;
            cx.notify();
        });
    }

    fn toggle_col_sort(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().col_sort = *checked;
            cx.notify();
        });
    }

    fn toggle_col_selection(&mut self, checked: &bool, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().col_selection = *checked;
            cx.notify();
        });
    }

    fn on_table_event(
        &mut self,
        _: View<Table<CustomerTableDelegate>>,
        event: &TableEvent,
        _cx: &mut ViewContext<Self>,
    ) {
        match event {
            TableEvent::ColWidthsChanged(col_widths) => {
                println!("Col widths changed: {:?}", col_widths)
            }
            TableEvent::SelectCol(ix) => println!("Select col: {}", ix),
            TableEvent::SelectRow(ix) => println!("Select row: {}", ix),
        }
    }
}

impl Render for TableStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        let delegate = self.table.read(cx).delegate();

        v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Checkbox::new("loop-selection")
                            .label("Loop Selection")
                            .selected(delegate.loop_selection)
                            .on_click(cx.listener(Self::toggle_loop_selection)),
                    )
                    .child(
                        Checkbox::new("col-resize")
                            .label("Column Resize")
                            .selected(delegate.col_resize)
                            .on_click(cx.listener(Self::toggle_col_resize)),
                    )
                    .child(
                        Checkbox::new("col-order")
                            .label("Column Order")
                            .selected(delegate.col_order)
                            .on_click(cx.listener(Self::toggle_col_order)),
                    )
                    .child(
                        Checkbox::new("col-sort")
                            .label("Column Sort")
                            .selected(delegate.col_sort)
                            .on_click(cx.listener(Self::toggle_col_sort)),
                    )
                    .child(
                        Checkbox::new("col-selection")
                            .label("Column Selection")
                            .selected(delegate.col_selection)
                            .on_click(cx.listener(Self::toggle_col_selection)),
                    ),
            )
            .child(self.table.clone())
    }
}

use fake::Fake;
use gpui::{
    ParentElement, Pixels, Render, SharedString, Styled, View, ViewContext, VisualContext as _,
    WindowContext,
};
use ui::{
    checkbox::Checkbox,
    h_flex,
    table::{Table, TableDelegate, TableEvent},
    v_flex, Selectable, Selection,
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
            gender: (0..1).fake(),
            age: (18..80).fake(),
            verified: (0..1).fake::<u8>() == 1,
            confirmed: (0..1).fake::<u8>() == 1,
        })
        .collect()
}
struct CustomerTableDelegate {
    customers: Vec<Customer>,
    col_names: Vec<(SharedString, SharedString)>,
    loop_selection: bool,
    col_resize: bool,
    col_order: bool,
}

impl CustomerTableDelegate {
    fn new(size: usize) -> Self {
        Self {
            customers: randome_customers(size),
            col_names: vec![
                ("id".into(), "ID".into()),
                ("login".into(), "Login".into()),
                ("first_name".into(), "First Name".into()),
                ("last_name".into(), "Last Name".into()),
                ("company".into(), "Company".into()),
                ("city".into(), "City".into()),
                ("country".into(), "Country".into()),
                ("email".into(), "Email".into()),
                ("phone".into(), "Phone".into()),
                ("gender".into(), "Gender".into()),
                ("age".into(), "Age".into()),
                ("verified".into(), "Verified".into()),
                ("confirmed".into(), "Confirmed".into()),
                ("twitter".into(), "Twitter".into()),
            ],
            loop_selection: true,
            col_resize: true,
            col_order: true,
        }
    }
}

impl TableDelegate for CustomerTableDelegate {
    fn cols_count(&self) -> usize {
        self.col_names.len()
    }

    fn rows_count(&self) -> usize {
        self.customers.len()
    }

    fn col_name(&self, col_ix: usize) -> SharedString {
        if let Some(col) = self.col_names.get(col_ix) {
            col.1.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize) -> Option<Pixels> {
        if let Some(col) = self.col_names.get(col_ix) {
            Some(
                match col.0.as_ref() {
                    "id" => 50.0,
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

    fn render_td(&self, row_ix: usize, col_ix: usize) -> impl gpui::IntoElement {
        let customer = self.customers.get(row_ix).unwrap();

        let col = self.col_names.get(col_ix).unwrap();
        let text = match col.0.as_ref() {
            "id" => customer.id.to_string(),
            "login" => customer.login.clone(),
            "first_name" => customer.first_name.clone(),
            "last_name" => customer.last_name.clone(),
            "company" => customer.company.clone(),
            "city" => customer.city.clone(),
            "country" => customer.country.clone(),
            "email" => customer.email.clone(),
            "phone" => customer.phone.clone(),
            "gender" => customer.gender.to_string(),
            "age" => customer.age.to_string(),
            "verified" => customer.verified.to_string(),
            "confirmed" => customer.confirmed.to_string(),
            "twitter" => "twitter".to_string(),
            _ => "--".to_string(),
        };

        SharedString::from(text)
    }

    fn can_loop_select(&self) -> bool {
        self.loop_selection
    }

    fn can_move_col(&self, _: usize) -> bool {
        self.col_order
    }

    fn move_col(&mut self, col_ix: usize, to_ix: usize) {
        let col = self.col_names.remove(col_ix);
        self.col_names.insert(to_ix, col);
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
        let delegate = CustomerTableDelegate::new(2000);
        let table = cx.new_view(|cx| Table::new(delegate, cx));

        cx.subscribe(&table, Self::on_table_event).detach();

        Self { table }
    }

    fn toggle_loop_selection(&mut self, s: &Selection, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().loop_selection = s.is_selected();
            cx.notify();
        });
    }

    fn toggle_col_resize(&mut self, s: &Selection, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().col_resize = s.is_selected();
            cx.notify();
        });
    }

    fn toggle_col_order(&mut self, s: &Selection, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().col_order = s.is_selected();
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
                    ),
            )
            .child(self.table.clone())
    }
}

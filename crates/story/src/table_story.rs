
use fake::Fake;
use gpui::{
    ParentElement, Render, SharedString, Styled, View, ViewContext,
    VisualContext as _, WindowContext,
};
use ui::{
    checkbox::Checkbox,
    h_flex,
    table::{Table, TableDelegate},
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
impl Customer {
    fn col_names() -> Vec<SharedString> {
        vec![
            "ID".into(),
            "Login".into(),
            "First Name".into(),
            "Last Name".into(),
            "Company".into(),
            "City".into(),
            "Country".into(),
            "Email".into(),
            "Phone".into(),
            "Gender".into(),
            "Age".into(),
            "Verified".into(),
            "Confirmed".into(),
            "Twitter".into(),
        ]
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
            gender: (0..1).fake(),
            age: (18..80).fake(),
            verified: (0..1).fake::<u8>() == 1,
            confirmed: (0..1).fake::<u8>() == 1,
        })
        .collect()
}
struct CustomerTableDelegate {
    customers: Vec<Customer>,
    loop_selection: bool,
}

impl CustomerTableDelegate {
    fn new(size: usize) -> Self {
        Self {
            customers: randome_customers(size),
            loop_selection: true,
        }
    }
}

impl TableDelegate for CustomerTableDelegate {
    fn cols_count(&self) -> usize {
        Customer::col_names().len()
    }

    fn rows_count(&self) -> usize {
        self.customers.len()
    }

    fn column_name(&self, col_ix: usize) -> SharedString {
        if let Some(name) = Customer::col_names().get(col_ix) {
            name.clone()
        } else {
            "--".into()
        }
    }

    fn col_width(&self, col_ix: usize) -> Option<f32> {
        match col_ix {
            0 => Some(50.0),
            1 => Some(220.0),
            2 => Some(150.0),
            3 => Some(150.0),
            4 => Some(300.0),
            5 => Some(200.0),
            6 => Some(200.0),
            7 => Some(350.0),
            8 => Some(240.0),
            9 => Some(80.0),
            10 => Some(90.0),
            11 => Some(90.0),
            12 => Some(90.0),
            13 => Some(90.0),
            _ => None,
        }
    }

    fn render_td(&self, row_ix: usize, col_ix: usize) -> impl gpui::IntoElement {
        let customer = self.customers.get(row_ix).unwrap();
        let text = match col_ix {
            0 => customer.id.to_string(),
            1 => customer.login.clone(),
            2 => customer.first_name.clone(),
            3 => customer.last_name.clone(),
            4 => customer.company.clone(),
            5 => customer.city.clone(),
            6 => customer.country.clone(),
            7 => customer.email.clone(),
            8 => customer.phone.clone(),
            9 => customer.gender.to_string(),
            10 => customer.age.to_string(),
            11 => customer.verified.to_string(),
            12 => customer.confirmed.to_string(),
            _ => "--".to_string(),
        };

        SharedString::from(text)
    }

    fn can_loop_select(&self) -> bool {
        self.loop_selection
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
        Self { table }
    }

    fn toggle_loop_selection(&mut self, s: &Selection, cx: &mut ViewContext<Self>) {
        let table = self.table.clone();
        table.update(cx, |table, cx| {
            table.delegate_mut().loop_selection = s.is_selected();
            cx.notify();
        });
    }
}

impl Render for TableStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        let delegate = self.table.read(cx).delegate();

        v_flex()
            .size_full()
            .gap_2()
            .child(
                h_flex().items_center().child(
                    Checkbox::new("loop-selection")
                        .label("Loop Selection")
                        .selected(delegate.loop_selection)
                        .on_click(cx.listener(Self::toggle_loop_selection)),
                ),
            )
            .child(self.table.clone())
    }
}

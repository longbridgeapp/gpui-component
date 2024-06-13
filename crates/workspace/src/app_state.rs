use std::sync::{Arc, Weak};

use gpui::{AppContext, Global};

pub struct AppState {}

struct GlobalAppState(Weak<AppState>);

impl Global for GlobalAppState {}

impl AppState {
    pub fn set_global(app_state: Weak<AppState>, cx: &mut AppContext) {
        cx.set_global(GlobalAppState(app_state));
    }
}

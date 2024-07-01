use std::sync::Weak;

use gpui::{AppContext, Global};

pub struct AppState {}

struct GlobalAppState();

impl Global for GlobalAppState {}

impl AppState {
    pub fn set_global(_app_state: Weak<AppState>, cx: &mut AppContext) {
        cx.set_global(GlobalAppState());
    }
}

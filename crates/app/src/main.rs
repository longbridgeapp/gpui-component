use std::sync::Arc;

use anyhow::Result;
use assets::Assets;
use gpui::{App, AppContext};
use workspace::AppState;

mod assets;

fn init(app_state: Arc<AppState>, cx: &mut AppContext) -> Result<()> {
    workspace::init(app_state.clone(), cx);

    Ok(())
}

fn main() {
    let app_state = Arc::new(AppState {});

    let app = App::new().with_assets(Assets);

    app.run(move |cx| {
        AppState::set_global(Arc::downgrade(&app_state), cx);

        if let Err(e) = init(app_state.clone(), cx) {
            log::error!("{}", e);
            return;
        }

        workspace::open_new(app_state.clone(), cx, |_workspace, _cx| {
            // do something
        })
        .detach();
    });
}

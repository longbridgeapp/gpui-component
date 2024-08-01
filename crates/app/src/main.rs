use std::sync::Arc;

use anyhow::Result;
use app_state::AppState;
use assets::Assets;
use gpui::{actions, App, AppContext, KeyBinding, Menu, MenuItem};
use ui::input::{Copy, Cut, Paste, Redo, Undo};

mod app_state;
mod assets;
mod story_workspace;

actions!(main_menu, [Quit]);

fn init(app_state: Arc<AppState>, cx: &mut AppContext) -> Result<()> {
    story_workspace::init(app_state.clone(), cx);

    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

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

        cx.on_action(quit);

        cx.set_menus(vec![
            Menu {
                name: "GPUI App".into(),
                items: vec![MenuItem::action("Quit", Quit)],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::os_action("Undo", Undo, gpui::OsAction::Undo),
                    MenuItem::os_action("Redo", Redo, gpui::OsAction::Redo),
                    MenuItem::separator(),
                    MenuItem::os_action("Cut", Cut, gpui::OsAction::Cut),
                    MenuItem::os_action("Copy", Copy, gpui::OsAction::Copy),
                    MenuItem::os_action("Paste", Paste, gpui::OsAction::Paste),
                ],
            },
        ]);
        cx.activate(true);

        story_workspace::open_new(app_state.clone(), cx, |_workspace, _cx| {
            // do something
        })
        .detach();
    });
}

fn quit(_: &Quit, cx: &mut AppContext) {
    cx.quit();
}

use std::path::PathBuf;

use gpui::{
    App, AppContext as _, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;
use gpui_component_assets::Assets;
use lootbox_gui::app::{
    AddCredential, AppView, BackToListFromEnvVars, BackToListFromReadView, CancelNewFile,
    CancelRemove, ConfirmNewFile, ConfirmRemove, CopyEnvLine, CopyKey, CopyValue, ExportEnv,
    QuitApp, RemoveCredential, SelectNext, SelectPrev, ShowCredential, ToggleEnvVisibility,
    ToggleReadViewVisibility, UpdateCredential,
};
use lootbox_gui::screens;

fn main() {
    let file_path = parse_file_path_arg();

    let application = Application::new().with_assets(Assets);

    application.run(move |cx: &mut App| {
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new("y", ConfirmNewFile, Some(screens::new_file_confirm::CONTEXT)),
            KeyBinding::new(
                "enter",
                ConfirmNewFile,
                Some(screens::new_file_confirm::CONTEXT),
            ),
            KeyBinding::new("n", CancelNewFile, Some(screens::new_file_confirm::CONTEXT)),
            KeyBinding::new(
                "escape",
                CancelNewFile,
                Some(screens::new_file_confirm::CONTEXT),
            ),
            KeyBinding::new("q", CancelNewFile, Some(screens::new_file_confirm::CONTEXT)),
            KeyBinding::new("q", QuitApp, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("escape", QuitApp, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("up", SelectPrev, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("down", SelectNext, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("a", AddCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("u", UpdateCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("r", RemoveCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("s", ShowCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("e", ExportEnv, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("enter", ConfirmRemove, Some(screens::remove_confirm::CONTEXT)),
            KeyBinding::new("escape", CancelRemove, Some(screens::remove_confirm::CONTEXT)),
            KeyBinding::new(
                "tab",
                ToggleReadViewVisibility,
                Some(screens::read_view::CONTEXT),
            ),
            KeyBinding::new("k", CopyKey, Some(screens::read_view::CONTEXT)),
            KeyBinding::new("v", CopyValue, Some(screens::read_view::CONTEXT)),
            KeyBinding::new(
                "escape",
                BackToListFromReadView,
                Some(screens::read_view::CONTEXT),
            ),
            KeyBinding::new("tab", ToggleEnvVisibility, Some(screens::env_vars::CONTEXT)),
            KeyBinding::new("c", CopyEnvLine, Some(screens::env_vars::CONTEXT)),
            KeyBinding::new(
                "escape",
                BackToListFromEnvVars,
                Some(screens::env_vars::CONTEXT),
            ),
        ]);

        let bounds = Bounds::centered(None, size(px(900.0), px(640.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                let view = cx.new(|cx| AppView::new(file_path.clone(), window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open lootbox-gui window");

        cx.activate(true);
    });
}

fn parse_file_path_arg() -> PathBuf {
    match std::env::args().nth(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Usage: lootbox-gui <vault-file>");
            std::process::exit(1);
        }
    }
}

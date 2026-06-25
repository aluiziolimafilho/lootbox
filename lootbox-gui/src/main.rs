use std::path::PathBuf;

use gpui::{
    App, AppContext as _, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;
use gpui_component_assets::Assets;
use lootbox_gui::app::{
    AddCredential, AppView, CancelNewFile, ConfirmNewFile, ConfirmRemove, CopyEnvLine, CopyKey,
    CopyValue, DeselectCredential, ExportCsv, ExportEnv, ImportCsv, QuitApp, RemoveCredential,
    SelectNext, SelectPrev, ToggleValueVisibility, UpdateCredential,
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
            KeyBinding::new("up", SelectPrev, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("down", SelectNext, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new(
                "escape",
                DeselectCredential,
                Some(screens::credential_list::CONTEXT),
            ),
            KeyBinding::new("a", AddCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("u", UpdateCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("r", RemoveCredential, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("e", ExportEnv, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("x", ExportCsv, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("i", ImportCsv, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new(
                "enter",
                ConfirmRemove,
                Some(screens::credential_list::CONTEXT),
            ),
            KeyBinding::new(
                "tab",
                ToggleValueVisibility,
                Some(screens::credential_list::CONTEXT),
            ),
            KeyBinding::new("k", CopyKey, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("v", CopyValue, Some(screens::credential_list::CONTEXT)),
            KeyBinding::new("c", CopyEnvLine, Some(screens::credential_list::CONTEXT)),
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

fn parse_file_path_arg() -> Option<PathBuf> {
    std::env::args().nth(1).map(PathBuf::from)
}

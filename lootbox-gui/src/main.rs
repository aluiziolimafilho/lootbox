mod app;
mod mask;
mod screens;

use std::path::PathBuf;

use app::{AppView, CancelNewFile, ConfirmNewFile, QuitApp};
use gpui::{
    App, AppContext as _, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;
use gpui_component_assets::Assets;

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

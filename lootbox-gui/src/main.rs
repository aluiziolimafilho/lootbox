use std::path::PathBuf;

use gpui::{
    App, AppContext as _, Application, Bounds, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;
use gpui_component_assets::Assets;
use lootbox_gui::app::AppView;

fn main() {
    let file_path = parse_file_path_arg();

    let application = Application::new().with_assets(Assets);

    application.run(move |cx: &mut App| {
        gpui_component::init(cx);

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

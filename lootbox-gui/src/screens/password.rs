use gpui::{Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::input::{Input, InputState};

use crate::app::AppView;

pub const CONTEXT: &str = "password";

pub fn render(
    input: Entity<InputState>,
    error: Option<String>,
    is_new: bool,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let title = if is_new {
        "LootBox \u{2014} New Vault"
    } else {
        "LootBox"
    };

    div()
        .key_context(CONTEXT)
        .on_action(cx.listener(AppView::submit_password))
        .on_action(cx.listener(AppView::handle_password_escape))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .child(title)
        .child(
            div()
                .w(gpui::px(320.0))
                .child(Input::new(&input)),
        )
        .children(error.map(|message| div().text_color(gpui::red()).child(message)))
}

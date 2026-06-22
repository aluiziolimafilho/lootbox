use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled,
    Window, div,
};
use gpui_component::button::Button;
use gpui_component::clipboard::Clipboard;

use crate::app::{AppView, BackToListFromEnvVars};
use crate::{clipboard, mask};

pub const CONTEXT: &str = "env_vars";

pub fn render(
    id: usize,
    env_name: String,
    value: String,
    value_visible: bool,
    clipboard_status: Option<String>,
    error: Option<String>,
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let back_button = Button::new("back-to-list")
        .outline()
        .label("Back (Esc)")
        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
            view.back_to_list_from_env_vars(&BackToListFromEnvVars, window, cx)
        }));

    let mut container = div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .on_action(cx.listener(AppView::toggle_env_visibility))
        .on_action(cx.listener(AppView::copy_env_line))
        .on_action(cx.listener(AppView::back_to_list_from_env_vars))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(format!("Exporting credential [{id}]"));

    if let Some(reason) = error {
        container = container
            .child(div().text_color(gpui::red()).child(reason))
            .child(back_button);
        return container;
    }

    let displayed_value = if value_visible { value.clone() } else { mask::MASK.to_string() };
    let export_line = format!("export {env_name}={displayed_value}");
    let copy_line = format!("export {env_name}={}", clipboard::shell_escape(&value));

    container
        .child(export_line)
        .child("(Tab to toggle, C to re-copy)")
        .child(
            div()
                .flex()
                .gap_2()
                .child(Clipboard::new("copy-env-line").value(copy_line))
                .child("Copy export line"),
        )
        .children(clipboard_status.map(|status| div().child(status)))
        .child(back_button)
}

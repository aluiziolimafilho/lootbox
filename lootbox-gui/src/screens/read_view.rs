use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled,
    Window, div,
};
use gpui_component::button::Button;
use gpui_component::clipboard::Clipboard;
use lootbox::Credential;

use crate::app::{AppView, BackToListFromReadView};
use crate::mask;

pub const CONTEXT: &str = "read_view";

pub fn render(
    id: usize,
    credential: Credential,
    value_visible: bool,
    clipboard_status: Option<String>,
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let displayed_value = if value_visible {
        credential.value.clone()
    } else {
        mask::MASK.to_string()
    };

    div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .on_action(cx.listener(AppView::toggle_read_view_visibility))
        .on_action(cx.listener(AppView::copy_read_view_key))
        .on_action(cx.listener(AppView::copy_read_view_value))
        .on_action(cx.listener(AppView::back_to_list_from_read_view))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(format!("[{id}] Key: {}", credential.key))
        .child(
            div()
                .flex()
                .gap_2()
                .items_center()
                .child(format!("Value: {displayed_value}"))
                .child("(Tab to toggle)"),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(Clipboard::new("copy-key").value(credential.key.clone()))
                .child("Copy key (K)")
                .child(Clipboard::new("copy-value").value(credential.value.clone()))
                .child("Copy value (V)"),
        )
        .children(clipboard_status.map(|status| div().child(status)))
        .child(
            Button::new("back-to-list")
                .outline()
                .label("Back (Esc)")
                .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                    view.back_to_list_from_read_view(&BackToListFromReadView, window, cx)
                })),
        )
}

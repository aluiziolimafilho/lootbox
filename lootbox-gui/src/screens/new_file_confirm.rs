use std::path::Path;

use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled,
    Window, div,
};
use gpui_component::button::{Button, ButtonVariants};

use crate::app::{AppView, CancelNewFile, ConfirmNewFile};

pub const CONTEXT: &str = "new_file_confirm";

pub fn render(
    file_path: &Path,
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .on_action(cx.listener(AppView::confirm_new_file))
        .on_action(cx.listener(AppView::cancel_new_file))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .child(format!("No vault found at {}", file_path.display()))
        .child("Create a new encrypted vault here?")
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("confirm-new-file")
                        .primary()
                        .label("Create Vault (Y)")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.confirm_new_file(&ConfirmNewFile, window, cx)
                        })),
                )
                .child(
                    Button::new("cancel-new-file")
                        .outline()
                        .label("Cancel (Esc)")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.cancel_new_file(&CancelNewFile, window, cx)
                        })),
                ),
        )
}

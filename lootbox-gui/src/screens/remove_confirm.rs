use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled,
    Window, div,
};
use gpui_component::button::{Button, ButtonVariants};

use crate::app::{AppView, CancelRemove, ConfirmRemove};

pub const CONTEXT: &str = "remove_confirm";

pub fn render(
    id: usize,
    key: String,
    error: Option<String>,
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .on_action(cx.listener(AppView::confirm_remove))
        .on_action(cx.listener(AppView::cancel_remove))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .child(format!("Remove [{}] {}?", id, key))
        .children(error.map(|message| div().text_color(gpui::red()).child(message)))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("confirm-remove")
                        .danger()
                        .label("Remove (Enter)")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.confirm_remove(&ConfirmRemove, window, cx)
                        })),
                )
                .child(
                    Button::new("cancel-remove")
                        .outline()
                        .label("Cancel (Esc)")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.cancel_remove(&CancelRemove, window, cx)
                        })),
                ),
        )
}

use std::path::Path;

use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled,
    Window, div,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::group_box::{GroupBox, GroupBoxVariants as _};

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
        .child(
            div().w(gpui::px(380.0)).child(
                GroupBox::new()
                    .title("No vault found")
                    .outline()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(format!("{}", file_path.display()))
                            .child("Create a new encrypted vault here?")
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(
                                        Button::new("confirm-new-file")
                                            .primary()
                                            .icon(gpui_component::IconName::Plus)
                                            .label("Create Vault")
                                            .on_click(cx.listener(
                                                |view, _: &ClickEvent, window, cx| {
                                                    view.confirm_new_file(
                                                        &ConfirmNewFile,
                                                        window,
                                                        cx,
                                                    )
                                                },
                                            )),
                                    )
                                    .child(
                                        Button::new("cancel-new-file")
                                            .outline()
                                            .label("Cancel")
                                            .on_click(cx.listener(
                                                |view, _: &ClickEvent, window, cx| {
                                                    view.cancel_new_file(
                                                        &CancelNewFile,
                                                        window,
                                                        cx,
                                                    )
                                                },
                                            )),
                                    ),
                            ),
                    ),
            ),
        )
}

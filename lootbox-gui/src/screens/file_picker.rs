use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::group_box::{GroupBox, GroupBoxVariants as _};

use crate::app::AppView;

pub const CONTEXT: &str = "file_picker";

pub fn render(
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .child(
            div().w(gpui::px(380.0)).child(
                GroupBox::new()
                    .title("LootBox")
                    .outline()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child("Open an existing vault or create a new one to get started.")
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        Button::new("open-existing-vault")
                                            .primary()
                                            .w_full()
                                            .icon(gpui_component::IconName::FolderOpen)
                                            .label("Open Existing Vault…")
                                            .on_click(cx.listener(
                                                AppView::open_existing_vault_dialog,
                                            )),
                                    )
                                    .child(
                                        Button::new("create-new-vault")
                                            .outline()
                                            .w_full()
                                            .icon(gpui_component::IconName::Plus)
                                            .label("Create New Vault…")
                                            .on_click(
                                                cx.listener(AppView::create_new_vault_dialog),
                                            ),
                                    ),
                            ),
                    ),
            ),
        )
}

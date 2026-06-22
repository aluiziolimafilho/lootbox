use gpui::prelude::FluentBuilder;
use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
};
use gpui_component::Disableable;
use gpui_component::button::{Button, ButtonVariants as _};
use lootbox::Credential;

use crate::app::{
    AddCredential, AppView, ExportCsv, ExportEnv, ImportCsv, QuitApp, RemoveCredential,
    ShowCredential, UpdateCredential,
};
use crate::mask;

pub const CONTEXT: &str = "credential_list";

pub fn render(
    credentials: &[Credential],
    selected: usize,
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let has_credentials = !credentials.is_empty();

    div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .on_action(cx.listener(AppView::quit_app))
        .on_action(cx.listener(AppView::select_prev))
        .on_action(cx.listener(AppView::select_next))
        .on_action(cx.listener(AppView::open_add_form))
        .on_action(cx.listener(AppView::open_update_form))
        .on_action(cx.listener(AppView::open_remove_confirm))
        .on_action(cx.listener(AppView::open_read_view))
        .on_action(cx.listener(AppView::open_env_vars))
        .on_action(cx.listener(AppView::open_export_csv))
        .on_action(cx.listener(AppView::open_import_csv))
        .size_full()
        .flex()
        .flex_col()
        .gap_2()
        .p_4()
        .child("Credentials")
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .children(credentials.iter().enumerate().map(|(i, credential)| {
                    let is_selected = i == selected;
                    div()
                        .id(("credential-row", i))
                        .flex()
                        .gap_4()
                        .px_2()
                        .py_1()
                        .when(is_selected, |row| row.bg(gpui::blue().opacity(0.2)))
                        .child(format!("[{}] {}", i + 1, credential.key))
                        .child(mask::MASK)
                })),
        )
        .when(credentials.is_empty(), |this| {
            this.child("No credentials yet.")
        })
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("add")
                        .primary()
                        .label("Add (A)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_add_form(&AddCredential, window, cx)
                        })),
                )
                .child(
                    Button::new("update")
                        .outline()
                        .disabled(!has_credentials)
                        .label("Update (U)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_update_form(&UpdateCredential, window, cx)
                        })),
                )
                .child(
                    Button::new("remove")
                        .danger()
                        .disabled(!has_credentials)
                        .label("Remove (R)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_remove_confirm(&RemoveCredential, window, cx)
                        })),
                )
                .child(
                    Button::new("show")
                        .info()
                        .disabled(!has_credentials)
                        .label("Show (S)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_read_view(&ShowCredential, window, cx)
                        })),
                )
                .child(
                    Button::new("env")
                        .info()
                        .disabled(!has_credentials)
                        .label("Env (E)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_env_vars(&ExportEnv, window, cx)
                        })),
                )
                .child(
                    Button::new("export-csv")
                        .warning()
                        .label("Export CSV (X)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_export_csv(&ExportCsv, window, cx)
                        })),
                )
                .child(
                    Button::new("import-csv")
                        .warning()
                        .label("Import CSV (I)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_import_csv(&ImportCsv, window, cx)
                        })),
                )
                .child(
                    Button::new("quit")
                        .outline()
                        .label("Quit (Q)")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.quit_app(&QuitApp, window, cx)
                        })),
                ),
        )
}

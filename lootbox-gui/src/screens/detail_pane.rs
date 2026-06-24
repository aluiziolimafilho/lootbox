use gpui::{
    ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled, Window,
    div,
};
use gpui_component::alert::Alert;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::clipboard::Clipboard;
use gpui_component::description_list::DescriptionList;
use gpui_component::group_box::{GroupBox, GroupBoxVariants as _};
use gpui_component::input::{Input, InputState};
use lootbox::Credential;

use crate::app::{
    AppView, CancelRemove, ConfirmRemove, CopyEnvLine, CopyKey, CopyValue, DetailPane, EditMode,
    ExportEnv, RemoveCredential, UpdateCredential,
};
use crate::{clipboard, mask};

/// Dispatches on the active `DetailPane` variant. Replaces the former standalone
/// `read_view.rs`/`credential_form.rs`/`remove_confirm.rs`/`env_vars.rs` screens -- those are
/// now sub-states of the right-hand detail panel rather than full-window takeovers.
pub fn render(
    detail: &DetailPane,
    window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div().flex_1().h_full().p_4().child(match detail {
        DetailPane::Empty => render_empty().into_any_element(),
        DetailPane::Read {
            id,
            credential,
            value_visible,
            clipboard_status,
        } => render_read(
            *id,
            credential.clone(),
            *value_visible,
            clipboard_status.clone(),
            window,
            cx,
        )
        .into_any_element(),
        DetailPane::Form {
            mode,
            key_input,
            value_input,
            value_visible,
            error,
        } => render_form(
            mode,
            key_input.clone(),
            value_input.clone(),
            *value_visible,
            error.clone(),
            window,
            cx,
        )
        .into_any_element(),
        DetailPane::EnvVars {
            id,
            env_name,
            value,
            value_visible,
            clipboard_status,
            error,
        } => render_env_vars(
            *id,
            env_name.clone(),
            value.clone(),
            *value_visible,
            clipboard_status.clone(),
            error.clone(),
            window,
            cx,
        )
        .into_any_element(),
        DetailPane::RemoveConfirm { id, key, error } => {
            render_remove_confirm(*id, key.clone(), error.clone(), window, cx).into_any_element()
        }
    })
}

fn render_empty() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child("Select a credential to view it here.")
}

fn render_read(
    id: usize,
    credential: Credential,
    value_visible: bool,
    clipboard_status: Option<String>,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let displayed_value = if value_visible {
        credential.value.clone()
    } else {
        mask::MASK.to_string()
    };

    // `Clipboard`'s own click handling already copies to the OS clipboard; `on_copied` routes
    // the resulting feedback through the same AppView methods the K/V keyboard shortcuts use,
    // so mouse and keyboard copying show identical status/toast feedback.
    let view = cx.entity();
    let key_clipboard = {
        let view = view.clone();
        Clipboard::new("copy-key")
            .value(credential.key.clone())
            .on_copied(move |_, window, cx| {
                view.update(cx, |view, cx| view.copy_read_view_key(&CopyKey, window, cx));
            })
    };
    let value_clipboard = {
        let view = view.clone();
        Clipboard::new("copy-value")
            .value(credential.value.clone())
            .on_copied(move |_, window, cx| {
                view.update(cx, |view, cx| view.copy_read_view_value(&CopyValue, window, cx));
            })
    };

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(format!("Credential #{id}"))
        .child(
            DescriptionList::new()
                .bordered(true)
                .item("Key", credential.key.clone(), 1)
                .item("Value", displayed_value, 1),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(key_clipboard)
                .child("Copy key")
                .child(value_clipboard)
                .child("Copy value")
                .child("(Tab to reveal/hide value)"),
        )
        .children(clipboard_status.map(|status| div().child(status)))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("update")
                        .outline()
                        .icon(gpui_component::IconName::Settings)
                        .label("Update")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.open_update_form(&UpdateCredential, window, cx)
                        })),
                )
                .child(
                    Button::new("remove")
                        .danger()
                        .icon(gpui_component::IconName::Delete)
                        .label("Remove")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.open_remove_confirm(&RemoveCredential, window, cx)
                        })),
                )
                .child(
                    Button::new("env")
                        .info()
                        .icon(gpui_component::IconName::ExternalLink)
                        .label("Export as env var")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.open_env_vars(&ExportEnv, window, cx)
                        })),
                ),
        )
}

fn render_form(
    mode: &EditMode,
    key_input: Entity<InputState>,
    value_input: Entity<InputState>,
    _value_visible: bool,
    error: Option<String>,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let title = match mode {
        EditMode::Add => "Add Credential",
        EditMode::Update { .. } => "Update Credential",
    };

    div()
        .on_action(cx.listener(AppView::cancel_credential_form))
        .flex()
        .flex_col()
        .gap_3()
        .child(
            GroupBox::new()
                .title(title)
                .outline()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .on_action(cx.listener(AppView::advance_from_key_field))
                                .child("Key")
                                .child(Input::new(&key_input)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .on_action(cx.listener(AppView::submit_credential_form))
                                .on_action(cx.listener(
                                    AppView::toggle_value_visibility_from_value_field,
                                ))
                                .on_action(cx.listener(AppView::move_focus_to_key_field))
                                .child("Value")
                                .child(Input::new(&value_input).mask_toggle())
                                .child("(Tab also toggles reveal)"),
                        ),
                ),
        )
        .children(error.map(|message| Alert::error("credential-form-error", message).banner()))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("save-credential")
                        .primary()
                        .label("Save")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.submit_credential_form(
                                &gpui_component::input::Enter { secondary: false },
                                window,
                                cx,
                            )
                        })),
                )
                .child(
                    Button::new("cancel-credential-form")
                        .outline()
                        .label("Cancel")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.cancel_credential_form(
                                &gpui_component::input::Escape,
                                window,
                                cx,
                            )
                        })),
                ),
        )
}

fn render_remove_confirm(
    id: usize,
    key: String,
    error: Option<String>,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(format!("Remove credential #{id} (\"{key}\")?"))
        .children(error.map(|message| Alert::error("remove-confirm-error", message).banner()))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("confirm-remove")
                        .danger()
                        .label("Remove")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.confirm_remove(&ConfirmRemove, window, cx)
                        })),
                )
                .child(
                    Button::new("cancel-remove")
                        .outline()
                        .label("Cancel")
                        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                            view.cancel_remove(&CancelRemove, window, cx)
                        })),
                ),
        )
}

fn render_env_vars(
    id: usize,
    env_name: String,
    value: String,
    value_visible: bool,
    clipboard_status: Option<String>,
    error: Option<String>,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let back_button = Button::new("back-from-env-vars")
        .outline()
        .label("Back")
        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
            view.back_to_list_from_env_vars(window, cx)
        }));

    let container = div()
        .flex()
        .flex_col()
        .gap_3()
        .child(format!("Exporting credential #{id}"));

    if let Some(reason) = error {
        return container
            .child(Alert::error("env-vars-error", reason).banner())
            .child(back_button)
            .into_any_element();
    }

    let displayed_value = if value_visible {
        value.clone()
    } else {
        mask::MASK.to_string()
    };
    let export_line = format!("export {env_name}={displayed_value}");
    let copy_line = format!("export {env_name}={}", clipboard::shell_escape(&value));

    let view = cx.entity();
    let copy_clipboard = Clipboard::new("copy-env-line")
        .value(copy_line)
        .on_copied(move |_, window, cx| {
            view.update(cx, |view, cx| view.copy_env_line(&CopyEnvLine, window, cx));
        });

    container
        .child(
            div()
                .p_2()
                .rounded_md()
                .border_1()
                .child(export_line),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(copy_clipboard)
                .child("Copy export line")
                .child("(Tab to reveal/hide, C to re-copy)"),
        )
        .children(clipboard_status.map(|status| div().child(status)))
        .child(back_button)
        .into_any_element()
}

use gpui::{
    ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled, Window,
    div,
};
use gpui_component::alert::Alert;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::clipboard::Clipboard;
use gpui_component::description_list::DescriptionList;
use gpui_component::StyledExt as _;
use gpui_component::group_box::{GroupBox, GroupBoxVariants as _};
use gpui_component::input::{Input, InputState};
use lootbox::Credential;

use crate::app::{
    AppView, CancelRemove, ConfirmRemove, CopyEnvLine, CopyKey, CopyUrl, CopyValue, DetailPane,
    EditMode, ExportEnv, RemoveCredential, ToggleValueVisibility, UpdateCredential,
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
        DetailPane::About => render_about().into_any_element(),
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
            name_input,
            key_input,
            value_input,
            url_input,
            description_input,
            value_visible,
            error,
        } => render_form(
            mode,
            name_input.clone(),
            key_input.clone(),
            value_input.clone(),
            url_input.clone(),
            description_input.clone(),
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

fn render_about() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_xl().child("LootBox"))
        .child(
            DescriptionList::new()
                .bordered(true)
                .item("Version", lootbox::VERSION, 1)
                .item("Build", lootbox::GIT_HASH, 1),
        )
}

fn field_row(label: &str, value: impl Into<gpui::SharedString>) -> gpui::Div {
    use gpui::px;
    div()
        .flex()
        .gap_2()
        .items_start()
        .child(
            div()
                .w(px(110.0))
                .flex_shrink_0()
                .text_color(gpui::rgb(0x6b7280))
                .child(label.to_string()),
        )
        .child(div().flex_1().truncate().child(value.into()))
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

    let view = cx.entity();

    // Key row — label + value + copy button
    let key_clipboard = {
        let view = view.clone();
        Clipboard::new("copy-key")
            .value(credential.key.clone())
            .on_copied(move |_, window, cx| {
                view.update(cx, |view, cx| view.copy_read_view_key(&CopyKey, window, cx));
            })
    };
    let key_row = field_row("Key", credential.key.clone())
        .child(key_clipboard);

    // Value row — label + masked value + eye toggle + copy button
    let value_clipboard = {
        let view = view.clone();
        Clipboard::new("copy-value")
            .value(credential.value.clone())
            .on_copied(move |_, window, cx| {
                view.update(cx, |view, cx| view.copy_read_view_value(&CopyValue, window, cx));
            })
    };
    let toggle_icon = if value_visible {
        gpui_component::IconName::EyeOff
    } else {
        gpui_component::IconName::Eye
    };
    let eye_button = Button::new("toggle-value-visibility")
        .ghost()
        .icon(toggle_icon)
        .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
            view.toggle_value_visibility(&ToggleValueVisibility, window, cx)
        }));
    let value_row = field_row("Value", displayed_value)
        .child(eye_button)
        .child(value_clipboard);

    let mut container = div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_xl().font_bold().truncate().child(format!("#{id}  {}", credential.display_name())))
        .child(key_row)
        .child(value_row);

    // URL row — only if present
    if let Some(url) = credential.url.clone() {
        let view = view.clone();
        let url_clipboard = Clipboard::new("copy-url")
            .value(url.clone())
            .on_copied(move |_, window, cx| {
                view.update(cx, |view, cx| view.copy_url(&CopyUrl, window, cx));
            });
        container = container.child(field_row("URL", url).child(url_clipboard));
    }

    // Description row — only if present, no copy button
    if let Some(desc) = credential.description.clone() {
        container = container.child(field_row("Description", desc));
    }

    container
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
    name_input: Entity<InputState>,
    key_input: Entity<InputState>,
    value_input: Entity<InputState>,
    url_input: Entity<InputState>,
    description_input: Entity<InputState>,
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
                                .on_action(cx.listener(AppView::advance_from_name_field))
                                .child("Name")
                                .child(Input::new(&name_input)),
                        )
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
                                .on_action(cx.listener(AppView::advance_from_value_field))
                                .child("Value")
                                .child(Input::new(&value_input).mask_toggle()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .on_action(cx.listener(AppView::advance_from_url_field))
                                .child("URL (optional)")
                                .child(Input::new(&url_input)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .on_action(cx.listener(AppView::submit_credential_form))
                                .child("Description (optional)")
                                .child(Input::new(&description_input)),
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

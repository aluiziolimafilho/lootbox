use std::path::PathBuf;

use gpui::{
    AppContext as _, ClipboardItem, Context, Entity, FocusHandle, IntoElement, Render, Window,
    actions,
};
use gpui_component::input::InputState;
use lootbox::Credential;

use crate::screens;

actions!(new_file_confirm, [ConfirmNewFile, CancelNewFile]);
actions!(
    credential_list,
    [
        QuitApp,
        SelectPrev,
        SelectNext,
        AddCredential,
        UpdateCredential,
        RemoveCredential,
        ShowCredential,
        ExportEnv
    ]
);
actions!(remove_confirm, [ConfirmRemove, CancelRemove]);
actions!(
    read_view,
    [
        ToggleReadViewVisibility,
        CopyKey,
        CopyValue,
        BackToListFromReadView
    ]
);
actions!(
    env_vars,
    [ToggleEnvVisibility, CopyEnvLine, BackToListFromEnvVars]
);

pub enum EditMode {
    Add,
    Update { id: usize },
}

pub enum AppScreen {
    NewFileConfirm,
    Password {
        input: Entity<InputState>,
        error: Option<String>,
        is_new: bool,
    },
    CredentialList {
        credentials: Vec<Credential>,
        selected: usize,
    },
    CredentialForm {
        mode: EditMode,
        key_input: Entity<InputState>,
        value_input: Entity<InputState>,
        value_visible: bool,
        error: Option<String>,
    },
    RemoveConfirm {
        id: usize,
        key: String,
        error: Option<String>,
    },
    ReadView {
        id: usize,
        credential: Credential,
        value_visible: bool,
        clipboard_status: Option<String>,
    },
    EnvVars {
        id: usize,
        env_name: String,
        value: String,
        value_visible: bool,
        clipboard_status: Option<String>,
        error: Option<String>,
    },
}

pub struct AppView {
    pub file_path: PathBuf,
    pub password: String,
    pub screen: AppScreen,
    pub focus_handle: FocusHandle,
}

impl AppView {
    pub fn new(file_path: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let screen = if file_path.exists() {
            Self::build_password_screen(false, window, cx)
        } else {
            focus_handle.focus(window);
            AppScreen::NewFileConfirm
        };

        Self {
            file_path,
            password: String::new(),
            screen,
            focus_handle,
        }
    }

    fn build_password_screen(
        is_new: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AppScreen {
        let input = cx.new(|cx| InputState::new(window, cx).masked(true));
        input.update(cx, |state, cx| state.focus(window, cx));
        AppScreen::Password {
            input,
            error: None,
            is_new,
        }
    }

    pub fn confirm_new_file(
        &mut self,
        _: &ConfirmNewFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.screen = Self::build_password_screen(true, window, cx);
        cx.notify();
    }

    pub fn back_to_new_file_confirm(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window);
        self.screen = AppScreen::NewFileConfirm;
        cx.notify();
    }

    pub fn cancel_new_file(
        &mut self,
        _: &CancelNewFile,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.quit();
    }

    pub fn quit_app(&mut self, _: &QuitApp, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    /// Handles the `Escape` keystroke while a Password-screen `Input` is focused. `InputState`
    /// only consumes Escape for its own internal states (context menu, IME, clear-on-escape);
    /// otherwise it calls `cx.propagate()`, which is what lets this ancestor handler fire.
    pub fn handle_password_escape(
        &mut self,
        _: &gpui_component::input::Escape,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Password { is_new, .. } = &self.screen else {
            return;
        };
        if *is_new {
            self.back_to_new_file_confirm(window, cx);
        } else {
            cx.quit();
        }
    }

    /// Handles the `Enter` keystroke while a Password-screen `Input` is focused.
    /// `InputState::enter` propagates for single-line inputs (it has no multi-line newline to
    /// insert), which is what lets this ancestor handler fire with a real `&mut Window`.
    pub fn submit_password(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Password { input, is_new, .. } = &self.screen else {
            return;
        };
        let is_new = *is_new;
        let value = input.read(cx).value().to_string();

        if is_new {
            match lootbox::validate_password(&value) {
                Ok(()) => {
                    self.password = value;
                    self.go_to_credential_list(vec![], window, cx);
                }
                Err(err) => self.set_password_error(err.to_string(), cx),
            }
        } else {
            match lootbox::list_credentials(&self.file_path, &value) {
                Ok(credentials) => {
                    self.password = value;
                    self.go_to_credential_list(credentials, window, cx);
                }
                Err(err) => self.set_password_error(err.to_string(), cx),
            }
        }
    }

    fn set_password_error(&mut self, message: String, cx: &mut Context<Self>) {
        if let AppScreen::Password { error, .. } = &mut self.screen {
            *error = Some(message);
        }
        cx.notify();
    }

    fn go_to_credential_list(
        &mut self,
        credentials: Vec<Credential>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window);
        self.screen = AppScreen::CredentialList {
            credentials,
            selected: 0,
        };
        cx.notify();
    }

    /// Reloads credentials from disk and returns to the list, clamping `desired_selected` to
    /// the new (possibly shorter, after a remove) bounds -- mirrors the TUI's behavior of
    /// re-fetching after every CRUD operation rather than mutating an in-memory copy.
    fn refresh_credential_list(
        &mut self,
        desired_selected: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let credentials = lootbox::list_credentials(&self.file_path, &self.password)
            .unwrap_or_default();
        let selected = if credentials.is_empty() {
            0
        } else {
            desired_selected.min(credentials.len() - 1)
        };
        self.focus_handle.focus(window);
        self.screen = AppScreen::CredentialList {
            credentials,
            selected,
        };
        cx.notify();
    }

    pub fn select_prev(&mut self, _: &SelectPrev, _window: &mut Window, cx: &mut Context<Self>) {
        if let AppScreen::CredentialList { selected, .. } = &mut self.screen {
            *selected = selected.saturating_sub(1);
        }
        cx.notify();
    }

    pub fn select_next(&mut self, _: &SelectNext, _window: &mut Window, cx: &mut Context<Self>) {
        if let AppScreen::CredentialList {
            credentials,
            selected,
        } = &mut self.screen
        {
            if !credentials.is_empty() {
                *selected = (*selected + 1).min(credentials.len() - 1);
            }
        }
        cx.notify();
    }

    fn build_credential_form_screen(
        mode: EditMode,
        key_value: Option<(String, String)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AppScreen {
        let initial_key = key_value.as_ref().map(|(key, _)| key.clone());
        let initial_value = key_value.as_ref().map(|(_, value)| value.clone());
        let key_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Key");
            if let Some(key) = initial_key {
                state = state.default_value(key);
            }
            state
        });
        let value_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).masked(true).placeholder("Value");
            if let Some(value) = initial_value {
                state = state.default_value(value);
            }
            state
        });
        key_input.update(cx, |state, cx| state.focus(window, cx));
        AppScreen::CredentialForm {
            mode,
            key_input,
            value_input,
            value_visible: false,
            error: None,
        }
    }

    pub fn open_add_form(
        &mut self,
        _: &AddCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.screen = Self::build_credential_form_screen(EditMode::Add, None, window, cx);
        cx.notify();
    }

    pub fn open_update_form(
        &mut self,
        _: &UpdateCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialList {
            credentials,
            selected,
        } = &self.screen
        else {
            return;
        };
        let Some(credential) = credentials.get(*selected) else {
            return;
        };
        let id = *selected + 1;
        let key_value = (credential.key.clone(), credential.value.clone());
        self.screen = Self::build_credential_form_screen(
            EditMode::Update { id },
            Some(key_value),
            window,
            cx,
        );
        cx.notify();
    }

    pub fn open_remove_confirm(
        &mut self,
        _: &RemoveCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialList {
            credentials,
            selected,
        } = &self.screen
        else {
            return;
        };
        let Some(credential) = credentials.get(*selected) else {
            return;
        };
        self.focus_handle.focus(window);
        self.screen = AppScreen::RemoveConfirm {
            id: *selected + 1,
            key: credential.key.clone(),
            error: None,
        };
        cx.notify();
    }

    /// `Enter` on the Key field moves focus to the Value field (mirrors the TUI's `handle_add`).
    pub fn advance_from_key_field(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialForm { value_input, .. } = &self.screen else {
            return;
        };
        value_input.update(cx, |state, cx| state.focus(window, cx));
    }

    /// `Tab` on the Value field toggles visibility instead of moving focus -- this reproduces
    /// the TUI's deliberate quirk. `IndentInline` is bound to `tab` inside gpui-component's
    /// "Input" key context for every Input, but only *handled* when the input is multi-line;
    /// for our single-line fields it has no handler and therefore propagates here unconsumed.
    pub fn toggle_value_visibility_from_value_field(
        &mut self,
        _: &gpui_component::input::IndentInline,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialForm {
            value_input,
            value_visible,
            ..
        } = &mut self.screen
        else {
            return;
        };
        *value_visible = !*value_visible;
        let now_visible = *value_visible;
        value_input.update(cx, |state, cx| state.set_masked(!now_visible, window, cx));
        cx.notify();
    }

    /// `Shift-Tab` on the Value field always moves focus back to the Key field (never toggles
    /// visibility), matching the TUI's asymmetric Tab/BackTab handling in `handle_add`.
    pub fn move_focus_to_key_field(
        &mut self,
        _: &gpui_component::input::OutdentInline,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialForm { key_input, .. } = &self.screen else {
            return;
        };
        key_input.update(cx, |state, cx| state.focus(window, cx));
    }

    /// `Escape` from either field cancels the form and returns to the list, matching the TUI.
    pub fn cancel_credential_form(
        &mut self,
        _: &gpui_component::input::Escape,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialForm { mode, .. } = &self.screen else {
            return;
        };
        let selected = match mode {
            EditMode::Add => 0,
            EditMode::Update { id } => id.saturating_sub(1),
        };
        self.refresh_credential_list(selected, window, cx);
    }

    /// `Enter` on the Value field submits the form -- always sends `Some(current_box_contents)`
    /// for both fields, never `None`. Unlike the CLI's optional prompts (where a blank Enter
    /// means "leave unchanged"), the GUI's fields are always pre-filled, so a cleared field is
    /// a deliberate edit and should hit the normal non-empty validation error, not silently
    /// keep the old value.
    pub fn submit_credential_form(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialForm {
            mode,
            key_input,
            value_input,
            ..
        } = &self.screen
        else {
            return;
        };
        let key = key_input.read(cx).value().to_string();
        let value = value_input.read(cx).value().to_string();

        let result = match mode {
            EditMode::Add => {
                lootbox::save_credential(&self.file_path, &self.password, &key, &value)
            }
            EditMode::Update { id } => lootbox::update_credential(
                &self.file_path,
                &self.password,
                *id,
                Some(key.as_str()),
                Some(value.as_str()),
            ),
        };

        match result {
            Ok(()) => {
                let selected = match mode {
                    EditMode::Add => usize::MAX, // clamps to the new last row
                    EditMode::Update { id } => id.saturating_sub(1),
                };
                self.refresh_credential_list(selected, window, cx);
            }
            Err(err) => {
                if let AppScreen::CredentialForm { error, .. } = &mut self.screen {
                    *error = Some(err.to_string());
                }
                cx.notify();
            }
        }
    }

    pub fn confirm_remove(
        &mut self,
        _: &ConfirmRemove,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::RemoveConfirm { id, .. } = &self.screen else {
            return;
        };
        let id = *id;
        match lootbox::remove_credential(&self.file_path, &self.password, id) {
            Ok(()) => self.refresh_credential_list(id.saturating_sub(1), window, cx),
            Err(err) => {
                if let AppScreen::RemoveConfirm { error, .. } = &mut self.screen {
                    *error = Some(err.to_string());
                }
                cx.notify();
            }
        }
    }

    pub fn cancel_remove(
        &mut self,
        _: &CancelRemove,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::RemoveConfirm { id, .. } = &self.screen else {
            return;
        };
        self.refresh_credential_list(id.saturating_sub(1), window, cx);
    }

    pub fn open_read_view(
        &mut self,
        _: &ShowCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CredentialList {
            credentials,
            selected,
        } = &self.screen
        else {
            return;
        };
        let Some(credential) = credentials.get(*selected) else {
            return;
        };
        self.focus_handle.focus(window);
        self.screen = AppScreen::ReadView {
            id: *selected + 1,
            credential: credential.clone(),
            value_visible: false,
            clipboard_status: None,
        };
        cx.notify();
    }

    pub fn toggle_read_view_visibility(
        &mut self,
        _: &ToggleReadViewVisibility,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppScreen::ReadView { value_visible, .. } = &mut self.screen {
            *value_visible = !*value_visible;
        }
        cx.notify();
    }

    pub fn copy_read_view_key(
        &mut self,
        _: &CopyKey,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::ReadView {
            credential,
            clipboard_status,
            ..
        } = &mut self.screen
        else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(credential.key.clone()));
        *clipboard_status = Some("Key copied!".to_string());
        cx.notify();
    }

    pub fn copy_read_view_value(
        &mut self,
        _: &CopyValue,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::ReadView {
            credential,
            clipboard_status,
            ..
        } = &mut self.screen
        else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(credential.value.clone()));
        *clipboard_status = Some("Value copied!".to_string());
        cx.notify();
    }

    pub fn back_to_list_from_read_view(
        &mut self,
        _: &BackToListFromReadView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::ReadView { id, .. } = &self.screen else {
            return;
        };
        self.refresh_credential_list(id.saturating_sub(1), window, cx);
    }

    pub fn open_env_vars(&mut self, _: &ExportEnv, window: &mut Window, cx: &mut Context<Self>) {
        let AppScreen::CredentialList {
            credentials,
            selected,
        } = &self.screen
        else {
            return;
        };
        if credentials.get(*selected).is_none() {
            return;
        }
        let id = *selected + 1;

        let screen = match lootbox::generate_env_vars(&self.file_path, &self.password, id) {
            Ok(mut result) => {
                if let Some(entry) = result.created.pop() {
                    AppScreen::EnvVars {
                        id,
                        env_name: entry.env_name,
                        value: entry.value,
                        value_visible: false,
                        clipboard_status: None,
                        error: None,
                    }
                } else if let Some(invalid) = result.invalid.pop() {
                    AppScreen::EnvVars {
                        id,
                        env_name: String::new(),
                        value: String::new(),
                        value_visible: false,
                        clipboard_status: None,
                        error: Some(invalid.reason),
                    }
                } else {
                    return;
                }
            }
            Err(err) => AppScreen::EnvVars {
                id,
                env_name: String::new(),
                value: String::new(),
                value_visible: false,
                clipboard_status: None,
                error: Some(err.to_string()),
            },
        };

        self.focus_handle.focus(window);
        self.screen = screen;
        cx.notify();
    }

    pub fn toggle_env_visibility(
        &mut self,
        _: &ToggleEnvVisibility,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppScreen::EnvVars { value_visible, .. } = &mut self.screen {
            *value_visible = !*value_visible;
        }
        cx.notify();
    }

    /// Copies the full `export KEY='value'` line (shell-escaped, matching the CLI's `env`
    /// command), not just the bare value -- this is what the TUI's `C` re-copy does too.
    pub fn copy_env_line(
        &mut self,
        _: &CopyEnvLine,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::EnvVars {
            env_name,
            value,
            clipboard_status,
            ..
        } = &mut self.screen
        else {
            return;
        };
        let line = format!("export {}={}", env_name, crate::clipboard::shell_escape(value));
        cx.write_to_clipboard(ClipboardItem::new_string(line));
        *clipboard_status = Some("Copied to clipboard!".to_string());
        cx.notify();
    }

    pub fn back_to_list_from_env_vars(
        &mut self,
        _: &BackToListFromEnvVars,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::EnvVars { id, .. } = &self.screen else {
            return;
        };
        self.refresh_credential_list(id.saturating_sub(1), window, cx);
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.screen {
            AppScreen::NewFileConfirm => screens::new_file_confirm::render(
                &self.file_path,
                self.focus_handle.clone(),
                window,
                cx,
            )
            .into_any_element(),
            AppScreen::Password {
                input,
                error,
                is_new,
            } => screens::password::render(input.clone(), error.clone(), *is_new, window, cx)
                .into_any_element(),
            AppScreen::CredentialList {
                credentials,
                selected,
            } => screens::credential_list::render(
                credentials,
                *selected,
                self.focus_handle.clone(),
                window,
                cx,
            )
            .into_any_element(),
            AppScreen::CredentialForm {
                mode,
                key_input,
                value_input,
                value_visible,
                error,
            } => screens::credential_form::render(
                mode,
                key_input.clone(),
                value_input.clone(),
                *value_visible,
                error.clone(),
                window,
                cx,
            )
            .into_any_element(),
            AppScreen::RemoveConfirm { id, key, error } => screens::remove_confirm::render(
                *id,
                key.clone(),
                error.clone(),
                self.focus_handle.clone(),
                window,
                cx,
            )
            .into_any_element(),
            AppScreen::ReadView {
                id,
                credential,
                value_visible,
                clipboard_status,
            } => screens::read_view::render(
                *id,
                credential.clone(),
                *value_visible,
                clipboard_status.clone(),
                self.focus_handle.clone(),
                window,
                cx,
            )
            .into_any_element(),
            AppScreen::EnvVars {
                id,
                env_name,
                value,
                value_visible,
                clipboard_status,
                error,
            } => screens::env_vars::render(
                *id,
                env_name.clone(),
                value.clone(),
                *value_visible,
                clipboard_status.clone(),
                error.clone(),
                self.focus_handle.clone(),
                window,
                cx,
            )
            .into_any_element(),
        }
    }
}

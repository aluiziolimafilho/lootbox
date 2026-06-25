use std::path::{Path, PathBuf};

use gpui::{
    AppContext as _, ClickEvent, ClipboardItem, Context, Entity, FocusHandle,
    InteractiveElement, IntoElement, ParentElement, PathPromptOptions, Render, Styled, Window,
    actions, div,
};
use gpui_component::input::InputState;
use gpui_component::notification::Notification;
use gpui_component::{Root, WindowExt as _};
use lootbox::Credential;

use crate::screens;

actions!(new_file_confirm, [ConfirmNewFile, CancelNewFile]);
actions!(
    credential_list,
    [
        QuitApp,
        SelectPrev,
        SelectNext,
        DeselectCredential,
        AddCredential,
        UpdateCredential,
        RemoveCredential,
        ExportEnv,
        ExportCsv,
        ImportCsv,
        OpenAbout
    ]
);
actions!(remove_confirm, [ConfirmRemove, CancelRemove]);
actions!(read_view, [ToggleValueVisibility, CopyKey, CopyValue, CopyUrl]);
actions!(env_vars, [CopyEnvLine]);

pub enum EditMode {
    Add,
    Update { id: usize },
}

pub enum AppScreen {
    FilePicker,
    NewFileConfirm,
    Password {
        input: Entity<InputState>,
        error: Option<String>,
        is_new: bool,
    },
    Unlocked {
        credentials: Vec<Credential>,
        selected: Option<usize>,
        detail: DetailPane,
    },
    CsvForm {
        mode: CsvMode,
        path_input: Entity<InputState>,
        status: Option<String>,
        error: Option<String>,
    },
}

pub enum DetailPane {
    Empty,
    About,
    Read {
        id: usize,
        credential: Credential,
        value_visible: bool,
        clipboard_status: Option<String>,
    },
    Form {
        mode: EditMode,
        name_input: Entity<InputState>,
        key_input: Entity<InputState>,
        value_input: Entity<InputState>,
        url_input: Entity<InputState>,
        description_input: Entity<InputState>,
        value_visible: bool,
        error: Option<String>,
    },
    EnvVars {
        id: usize,
        env_name: String,
        value: String,
        value_visible: bool,
        clipboard_status: Option<String>,
        error: Option<String>,
    },
    RemoveConfirm {
        id: usize,
        key: String,
        error: Option<String>,
    },
}

/// Row selection is disabled while editing/confirming, so the list and detail pane never
/// disagree about which credential is "current" mid-edit.
fn is_detail_locked(detail: &DetailPane) -> bool {
    matches!(detail, DetailPane::Form { .. } | DetailPane::RemoveConfirm { .. })
}

#[derive(Clone, Copy)]
pub enum CsvMode {
    Export,
    Import,
}

pub struct AppView {
    pub file_path: Option<PathBuf>,
    pub password: String,
    pub screen: AppScreen,
    pub focus_handle: FocusHandle,
}

impl AppView {
    pub fn new(file_path: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let screen = match &file_path {
            Some(path) if path.exists() => Self::build_password_screen(false, window, cx),
            Some(_) => {
                focus_handle.focus(window);
                AppScreen::NewFileConfirm
            }
            None => {
                focus_handle.focus(window);
                AppScreen::FilePicker
            }
        };

        Self {
            file_path,
            password: String::new(),
            screen,
            focus_handle,
        }
    }

    /// By the time any code reaches `Password`, `Unlocked`, or `CsvForm`, a file path has
    /// always already been chosen -- either via the CLI arg or via `FilePicker`'s two
    /// transition methods, the only ways to leave `FilePicker`/`NewFileConfirm`.
    fn file_path(&self) -> &PathBuf {
        self.file_path
            .as_ref()
            .expect("file_path must be set before reaching Password/Unlocked/CsvForm")
    }

    /// The OS "Open" dialog only lists files that already exist, so no extra confirmation step
    /// is needed (unlike a CLI-arg path, which might not exist yet and goes through
    /// `NewFileConfirm`).
    pub fn open_existing_vault_at(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.file_path = Some(path);
        self.screen = Self::build_password_screen(false, window, cx);
        cx.notify();
    }

    /// Skips `NewFileConfirm` deliberately: the Save dialog itself was the user's explicit
    /// "create a new vault" action, so a second "Create a new encrypted vault here?"
    /// confirmation would be redundant.
    pub fn create_new_vault_at(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.file_path = Some(path);
        self.screen = Self::build_password_screen(true, window, cx);
        cx.notify();
    }

    /// Raises the native OS "Open" file dialog. Any non-selection outcome (user cancelled, the
    /// dialog channel was dropped, or a platform error) collapses to "stay on `FilePicker`" --
    /// cancelling a native dialog is not an error state.
    pub fn open_existing_vault_dialog(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = rx.await else {
                return;
            };
            let Some(path) = paths.pop() else {
                return;
            };
            cx.update(|window, cx| {
                this.update(cx, |this, cx| this.open_existing_vault_at(path, window, cx))
                    .ok();
            })
            .ok();
        })
        .detach();
    }

    /// Raises the native OS "Save As"-style dialog for choosing where to create a new vault.
    /// `Path::new("")` lets the platform pick its own default starting directory.
    pub fn create_new_vault_dialog(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let rx = cx.prompt_for_new_path(Path::new(""), None);
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(path))) = rx.await else {
                return;
            };
            cx.update(|window, cx| {
                this.update(cx, |this, cx| this.create_new_vault_at(path, window, cx))
                    .ok();
            })
            .ok();
        })
        .detach();
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

    pub fn back_to_new_file_confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub fn open_about(&mut self, _: &OpenAbout, _window: &mut Window, cx: &mut Context<Self>) {
        if let AppScreen::Unlocked { detail, .. } = &mut self.screen {
            *detail = DetailPane::About;
        }
        cx.notify();
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
                    self.go_to_unlocked(vec![], window, cx);
                }
                Err(err) => self.set_password_error(err.to_string(), cx),
            }
        } else {
            match lootbox::list_credentials(self.file_path(), &value) {
                Ok(credentials) => {
                    self.password = value;
                    self.go_to_unlocked(credentials, window, cx);
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

    fn initial_detail(credentials: &[Credential]) -> (Option<usize>, DetailPane) {
        match credentials.first() {
            Some(credential) => (
                Some(0),
                DetailPane::Read {
                    id: 1,
                    credential: credential.clone(),
                    value_visible: false,
                    clipboard_status: None,
                },
            ),
            None => (None, DetailPane::Empty),
        }
    }

    fn go_to_unlocked(
        &mut self,
        credentials: Vec<Credential>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window);
        let (selected, detail) = Self::initial_detail(&credentials);
        self.screen = AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        };
        cx.notify();
    }

    /// Reloads credentials from disk and shows `desired_selected` (clamped to the new, possibly
    /// shorter, bounds after a remove) in the detail pane -- mirrors the TUI's behavior of
    /// re-fetching after every CRUD operation rather than mutating an in-memory copy.
    fn refresh_unlocked(
        &mut self,
        desired_selected: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let credentials =
            lootbox::list_credentials(self.file_path(), &self.password).unwrap_or_default();
        self.focus_handle.focus(window);

        if credentials.is_empty() {
            self.screen = AppScreen::Unlocked {
                credentials,
                selected: None,
                detail: DetailPane::Empty,
            };
        } else {
            let idx = desired_selected.unwrap_or(0).min(credentials.len() - 1);
            let detail = DetailPane::Read {
                id: idx + 1,
                credential: credentials[idx].clone(),
                value_visible: false,
                clipboard_status: None,
            };
            self.screen = AppScreen::Unlocked {
                credentials,
                selected: Some(idx),
                detail,
            };
        }
        cx.notify();
    }

    /// Reverts the detail pane to `Read` of the currently selected row (or `Empty`), discarding
    /// whatever Form/EnvVars/RemoveConfirm content was showing. Used by every "cancel"/"back"
    /// action, since none of them change which row is selected.
    fn return_to_read(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let AppScreen::Unlocked { selected, .. } = &self.screen else {
            return;
        };
        let selected = *selected;
        self.refresh_unlocked(selected, window, cx);
    }

    pub fn select_credential(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &mut self.screen
        else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let Some(credential) = credentials.get(index) else {
            return;
        };
        *selected = Some(index);
        *detail = DetailPane::Read {
            id: index + 1,
            credential: credential.clone(),
            value_visible: false,
            clipboard_status: None,
        };
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub fn select_prev(&mut self, _: &SelectPrev, window: &mut Window, cx: &mut Context<Self>) {
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &self.screen
        else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let Some(current) = *selected else {
            return;
        };
        self.select_credential(current.saturating_sub(1), window, cx);
    }

    pub fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &self.screen
        else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let Some(current) = *selected else {
            return;
        };
        if credentials.is_empty() {
            return;
        }
        let next = (current + 1).min(credentials.len() - 1);
        self.select_credential(next, window, cx);
    }

    /// `Escape` on the list/detail screen deselects the current row, clearing the detail pane.
    /// Quitting from the main screen is reserved for the explicit Quit action/button (Q) --
    /// Esc silently exiting the whole app from the hub screen was a TUI-only convenience that
    /// doesn't fit a mouse-first app.
    /// `Escape` on the Unlocked screen when no Input is focused. Only resolves to this action
    /// at all when no Input is focused (Form's Value/Key fields claim a more specific keymap
    /// context, so this never fires while editing -- see `cancel_credential_form`). Branches on
    /// the current detail pane: RemoveConfirm/EnvVars revert to Read (same as their Cancel
    /// button); Read with something selected clears the selection.
    pub fn deselect_credential(
        &mut self,
        _: &DeselectCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &self.screen
        else {
            return;
        };
        let should_return_to_read =
            matches!(detail, DetailPane::RemoveConfirm { .. } | DetailPane::EnvVars { .. });
        let should_deselect = matches!(detail, DetailPane::Read { .. }) && selected.is_some();

        if should_return_to_read {
            self.return_to_read(window, cx);
        } else if should_deselect {
            if let AppScreen::Unlocked {
                selected, detail, ..
            } = &mut self.screen
            {
                *selected = None;
                *detail = DetailPane::Empty;
            }
            self.focus_handle.focus(window);
            cx.notify();
        }
    }

    fn build_form_detail(
        mode: EditMode,
        existing: Option<&Credential>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> DetailPane {
        let name_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Name");
            if let Some(c) = existing {
                state = state.default_value(c.name.clone());
            }
            state
        });
        let key_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Key");
            if let Some(c) = existing {
                state = state.default_value(c.key.clone());
            }
            state
        });
        let value_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).masked(true).placeholder("Value");
            if let Some(c) = existing {
                state = state.default_value(c.value.clone());
            }
            state
        });
        let url_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("URL (optional)");
            if let Some(c) = existing {
                state = state.default_value(c.url.clone().unwrap_or_default());
            }
            state
        });
        let description_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Description (optional)");
            if let Some(c) = existing {
                state = state.default_value(c.description.clone().unwrap_or_default());
            }
            state
        });
        name_input.update(cx, |state, cx| state.focus(window, cx));
        DetailPane::Form {
            mode,
            name_input,
            key_input,
            value_input,
            url_input,
            description_input,
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
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let form = Self::build_form_detail(EditMode::Add, None, window, cx);
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        *detail = form;
        cx.notify();
    }


    pub fn open_update_form(
        &mut self,
        _: &UpdateCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &self.screen
        else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let Some(idx) = *selected else {
            return;
        };
        let Some(credential) = credentials.get(idx) else {
            return;
        };
        let id = idx + 1;
        let form = Self::build_form_detail(EditMode::Update { id }, Some(credential), window, cx);
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        *detail = form;
        cx.notify();
    }

    pub fn open_remove_confirm(
        &mut self,
        _: &RemoveCredential,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &self.screen
        else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let Some(idx) = *selected else {
            return;
        };
        let Some(credential) = credentials.get(idx) else {
            return;
        };
        let id = idx + 1;
        let key = credential.key.clone();
        self.focus_handle.focus(window);
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        *detail = DetailPane::RemoveConfirm {
            id,
            key,
            error: None,
        };
        cx.notify();
    }

    /// `Enter` on the Name field moves focus to the Key field.
    pub fn advance_from_name_field(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        let DetailPane::Form { key_input, .. } = detail else {
            return;
        };
        key_input.update(cx, |state, cx| state.focus(window, cx));
    }

    /// `Enter` on the Key field moves focus to the Value field (mirrors the TUI's `handle_add`).
    pub fn advance_from_key_field(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        let DetailPane::Form { value_input, .. } = detail else {
            return;
        };
        value_input.update(cx, |state, cx| state.focus(window, cx));
    }

    /// `Enter` on the URL field moves focus to the Description field.
    pub fn advance_from_url_field(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        let DetailPane::Form { description_input, .. } = detail else {
            return;
        };
        description_input.update(cx, |state, cx| state.focus(window, cx));
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
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        let DetailPane::Form {
            value_input,
            value_visible,
            ..
        } = detail
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
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        let DetailPane::Form { key_input, .. } = detail else {
            return;
        };
        key_input.update(cx, |state, cx| state.focus(window, cx));
    }

    /// `Escape` from either field cancels the form and returns to the Read pane.
    pub fn cancel_credential_form(
        &mut self,
        _: &gpui_component::input::Escape,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.return_to_read(window, cx);
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
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        let DetailPane::Form {
            mode,
            name_input,
            key_input,
            value_input,
            url_input,
            description_input,
            ..
        } = detail
        else {
            return;
        };
        let update_id = match mode {
            EditMode::Add => None,
            EditMode::Update { id } => Some(*id),
        };
        let name = name_input.read(cx).value().to_string();
        let key = key_input.read(cx).value().to_string();
        let value = value_input.read(cx).value().to_string();
        let url = url_input.read(cx).value().to_string();
        let description = description_input.read(cx).value().to_string();

        fn opt(s: &str) -> Option<&str> {
            if s.is_empty() { None } else { Some(s) }
        }

        let result = match update_id {
            None => lootbox::save_credential(
                self.file_path(),
                &self.password,
                &key,
                &value,
                opt(&name),
                opt(&description),
                opt(&url),
            ),
            Some(id) => lootbox::update_credential(
                self.file_path(),
                &self.password,
                id,
                Some(key.as_str()),
                Some(value.as_str()),
                Some(name.as_str()),
                Some(description.as_str()),
                Some(url.as_str()),
            ),
        };

        match result {
            Ok(()) => {
                let desired = update_id
                    .map(|id| id.saturating_sub(1))
                    .unwrap_or(usize::MAX); // clamps to the new last row when adding
                self.refresh_unlocked(Some(desired), window, cx);
            }
            Err(err) => {
                if let AppScreen::Unlocked { detail, .. } = &mut self.screen
                    && let DetailPane::Form { error, .. } = detail
                {
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
        let AppScreen::Unlocked { detail, .. } = &self.screen else {
            return;
        };
        let DetailPane::RemoveConfirm { id, .. } = detail else {
            return;
        };
        let id = *id;
        match lootbox::remove_credential(self.file_path(), &self.password, id) {
            Ok(()) => self.refresh_unlocked(Some(id.saturating_sub(1)), window, cx),
            Err(err) => {
                if let AppScreen::Unlocked { detail, .. } = &mut self.screen
                    && let DetailPane::RemoveConfirm { error, .. } = detail
                {
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
        self.return_to_read(window, cx);
    }

    /// `Tab` toggles whichever masked value is currently showing -- Read's credential value or
    /// EnvVars' export value. A single action (rather than one per detail kind) avoids binding
    /// "tab" twice in the same key context, since only one of the two is ever active at once.
    pub fn toggle_value_visibility(
        &mut self,
        _: &ToggleValueVisibility,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let AppScreen::Unlocked { detail, .. } = &mut self.screen {
            match detail {
                DetailPane::Read { value_visible, .. } => *value_visible = !*value_visible,
                DetailPane::EnvVars { value_visible, .. } => *value_visible = !*value_visible,
                _ => return,
            }
        }
        cx.notify();
    }

    pub fn copy_read_view_key(
        &mut self,
        _: &CopyKey,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        let DetailPane::Read {
            credential,
            clipboard_status,
            ..
        } = detail
        else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(credential.key.clone()));
        *clipboard_status = Some("Key copied!".to_string());
        // `push_notification` updates the Root entity, which is already being updated for the
        // duration of this very call (we're invoked from inside an action dispatch on a view
        // that's a child of Root); deferring avoids a "cannot update Root while it is already
        // being updated" panic from that reentrant access.
        window.defer(cx, |window, cx| {
            window.push_notification(Notification::success("Key copied!").autohide(true), cx);
        });
        cx.notify();
    }

    pub fn copy_read_view_value(
        &mut self,
        _: &CopyValue,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        let DetailPane::Read {
            credential,
            clipboard_status,
            ..
        } = detail
        else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(credential.value.clone()));
        *clipboard_status = Some("Value copied!".to_string());
        window.defer(cx, |window, cx| {
            window.push_notification(Notification::success("Value copied!").autohide(true), cx);
        });
        cx.notify();
    }

    pub fn copy_url(
        &mut self,
        _: &CopyUrl,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        let DetailPane::Read {
            credential,
            clipboard_status,
            ..
        } = detail
        else {
            return;
        };
        let Some(url) = credential.url.clone() else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(url));
        *clipboard_status = Some("URL copied!".to_string());
        window.defer(cx, |window, cx| {
            window.push_notification(Notification::success("URL copied!").autohide(true), cx);
        });
        cx.notify();
    }

    pub fn open_env_vars(&mut self, _: &ExportEnv, window: &mut Window, cx: &mut Context<Self>) {
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &self.screen
        else {
            return;
        };
        if is_detail_locked(detail) {
            return;
        }
        let Some(idx) = *selected else {
            return;
        };
        let id = idx + 1;

        let new_detail = match lootbox::generate_env_vars(self.file_path(), &self.password, id) {
            Ok(mut result) => {
                if let Some(entry) = result.created.pop() {
                    DetailPane::EnvVars {
                        id,
                        env_name: entry.env_name,
                        value: entry.value,
                        value_visible: false,
                        clipboard_status: None,
                        error: None,
                    }
                } else if let Some(invalid) = result.invalid.pop() {
                    DetailPane::EnvVars {
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
            Err(err) => DetailPane::EnvVars {
                id,
                env_name: String::new(),
                value: String::new(),
                value_visible: false,
                clipboard_status: None,
                error: Some(err.to_string()),
            },
        };

        self.focus_handle.focus(window);
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        *detail = new_detail;
        cx.notify();
    }

    /// Copies the full `export KEY='value'` line (shell-escaped, matching the CLI's `env`
    /// command), not just the bare value -- this is what the TUI's `C` re-copy does too.
    pub fn copy_env_line(
        &mut self,
        _: &CopyEnvLine,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::Unlocked { detail, .. } = &mut self.screen else {
            return;
        };
        let DetailPane::EnvVars {
            env_name,
            value,
            clipboard_status,
            ..
        } = detail
        else {
            return;
        };
        let line = format!("export {}={}", env_name, crate::clipboard::shell_escape(value));
        cx.write_to_clipboard(ClipboardItem::new_string(line));
        *clipboard_status = Some("Copied to clipboard!".to_string());
        window.defer(cx, |window, cx| {
            window.push_notification(Notification::success("Copied to clipboard!").autohide(true), cx);
        });
        cx.notify();
    }

    pub fn back_to_list_from_env_vars(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.return_to_read(window, cx);
    }

    fn build_csv_form_screen(mode: CsvMode, window: &mut Window, cx: &mut Context<Self>) -> AppScreen {
        let path_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Enter CSV file path..."));
        path_input.update(cx, |state, cx| state.focus(window, cx));
        AppScreen::CsvForm {
            mode,
            path_input,
            status: None,
            error: None,
        }
    }

    pub fn open_export_csv(&mut self, _: &ExportCsv, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Self::build_csv_form_screen(CsvMode::Export, window, cx);
        cx.notify();
    }

    pub fn open_import_csv(&mut self, _: &ImportCsv, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Self::build_csv_form_screen(CsvMode::Import, window, cx);
        cx.notify();
    }

    /// `Enter` on the path field. Mirrors the TUI's `handle_export_csv_form`/
    /// `handle_import_csv_form`: once `status` is set (a previous submit already succeeded),
    /// a further Enter just returns to the list instead of re-submitting.
    pub fn submit_csv_form(
        &mut self,
        _: &gpui_component::input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let AppScreen::CsvForm {
            mode,
            path_input,
            status,
            ..
        } = &self.screen
        else {
            return;
        };
        if status.is_some() {
            self.refresh_unlocked(None, window, cx);
            return;
        }

        let is_export = matches!(mode, CsvMode::Export);
        let csv_path = PathBuf::from(path_input.read(cx).value().trim());

        let result = if is_export {
            lootbox::export_credentials_to_csv(self.file_path(), &self.password, &csv_path)
                .map(|()| format!("Exported to {}", csv_path.display()))
        } else {
            lootbox::import_credentials_from_csv(self.file_path(), &self.password, &csv_path)
                .map(|count| format!("Imported {count} credential(s)."))
        };

        match result {
            Ok(message) => {
                if let AppScreen::CsvForm { status, .. } = &mut self.screen {
                    *status = Some(message);
                }
            }
            Err(err) => {
                if let AppScreen::CsvForm { error, .. } = &mut self.screen {
                    *error = Some(err.to_string());
                }
            }
        }
        cx.notify();
    }

    pub fn cancel_csv_form(
        &mut self,
        _: &gpui_component::input::Escape,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.refresh_unlocked(None, window, cx);
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match &self.screen {
            AppScreen::FilePicker => {
                screens::file_picker::render(self.focus_handle.clone(), window, cx)
                    .into_any_element()
            }
            AppScreen::NewFileConfirm => screens::new_file_confirm::render(
                self.file_path(),
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
            AppScreen::Unlocked {
                credentials,
                selected,
                detail,
            } => div()
                .key_context(screens::credential_list::CONTEXT)
                .track_focus(&self.focus_handle)
                .on_action(cx.listener(AppView::quit_app))
                .on_action(cx.listener(AppView::select_prev))
                .on_action(cx.listener(AppView::select_next))
                .on_action(cx.listener(AppView::deselect_credential))
                .on_action(cx.listener(AppView::open_add_form))
                .on_action(cx.listener(AppView::open_update_form))
                .on_action(cx.listener(AppView::open_remove_confirm))
                .on_action(cx.listener(AppView::open_env_vars))
                .on_action(cx.listener(AppView::open_export_csv))
                .on_action(cx.listener(AppView::open_import_csv))
                .on_action(cx.listener(AppView::toggle_value_visibility))
                .on_action(cx.listener(AppView::copy_read_view_key))
                .on_action(cx.listener(AppView::copy_read_view_value))
                .on_action(cx.listener(AppView::copy_url))
                .on_action(cx.listener(AppView::copy_env_line))
                .on_action(cx.listener(AppView::confirm_remove))
                .on_action(cx.listener(AppView::open_about))
                .flex()
                .flex_row()
                .size_full()
                .child(screens::credential_list::render(
                    credentials,
                    *selected,
                    is_detail_locked(detail),
                    window,
                    cx,
                ))
                .child(screens::detail_pane::render(detail, window, cx))
                .into_any_element(),
            AppScreen::CsvForm {
                mode,
                path_input,
                status,
                error,
            } => screens::csv_form::render(
                *mode,
                path_input.clone(),
                status.clone(),
                error.clone(),
                window,
                cx,
            )
            .into_any_element(),
        };

        div()
            .relative()
            .size_full()
            .child(content)
            .children(Root::render_notification_layer(window, cx))
    }
}

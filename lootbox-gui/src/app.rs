use std::path::PathBuf;

use gpui::{AppContext as _, Context, Entity, FocusHandle, IntoElement, Render, Window, actions};
use gpui_component::input::InputState;
use lootbox::Credential;

use crate::screens;

actions!(new_file_confirm, [ConfirmNewFile, CancelNewFile]);
actions!(credential_list, [QuitApp]);

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
        }
    }
}

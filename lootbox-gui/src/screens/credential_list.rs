use gpui::prelude::FluentBuilder;
use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
};
use gpui_component::button::Button;
use lootbox::Credential;

use crate::app::{AppView, QuitApp};
use crate::mask;

pub const CONTEXT: &str = "credential_list";

pub fn render(
    credentials: &[Credential],
    selected: usize,
    focus_handle: FocusHandle,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div()
        .key_context(CONTEXT)
        .track_focus(&focus_handle)
        .on_action(cx.listener(AppView::quit_app))
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
            Button::new("quit")
                .outline()
                .label("Quit (Q)")
                .on_click(cx.listener(|view, _, window, cx| view.quit_app(&QuitApp, window, cx))),
        )
}

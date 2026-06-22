use gpui::{
    ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled, Window,
    div,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};

use crate::app::{AppView, EditMode};

pub fn render(
    mode: &EditMode,
    key_input: Entity<InputState>,
    value_input: Entity<InputState>,
    value_visible: bool,
    error: Option<String>,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let title = match mode {
        EditMode::Add => "Add Credential",
        EditMode::Update { .. } => "Update Credential",
    };
    let visibility_hint = if value_visible {
        "Tab: hide value"
    } else {
        "Tab: reveal value"
    };

    div()
        .on_action(cx.listener(AppView::cancel_credential_form))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .child(title)
        .child(
            div()
                .w(gpui::px(360.0))
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
                        .on_action(cx.listener(AppView::toggle_value_visibility_from_value_field))
                        .on_action(cx.listener(AppView::move_focus_to_key_field))
                        .child("Value")
                        .child(Input::new(&value_input))
                        .child(visibility_hint),
                )
                .children(
                    error.map(|message| div().text_color(gpui::red()).child(message)),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            Button::new("save-credential")
                                .primary()
                                .label("Save (Enter)")
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
                                .label("Cancel (Esc)")
                                .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                                    view.cancel_credential_form(
                                        &gpui_component::input::Escape,
                                        window,
                                        cx,
                                    )
                                })),
                        ),
                ),
        )
}

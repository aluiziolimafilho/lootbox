use gpui::{
    ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement, Styled, Window,
    div,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};

use crate::app::{AppView, CsvMode};

pub fn render(
    mode: CsvMode,
    path_input: Entity<InputState>,
    status: Option<String>,
    error: Option<String>,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    let title = match mode {
        CsvMode::Export => "Export CSV",
        CsvMode::Import => "Import CSV",
    };
    let submit_label = if status.is_some() {
        "Back to list (Enter)"
    } else {
        "Submit (Enter)"
    };

    div()
        .on_action(cx.listener(AppView::cancel_csv_form))
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .child(title)
        .child(
            div()
                .w(gpui::px(420.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .on_action(cx.listener(AppView::submit_csv_form))
                        .child("CSV file path")
                        .child(Input::new(&path_input)),
                )
                .children(status.map(|message| div().child(message)))
                .children(error.map(|message| div().text_color(gpui::red()).child(message)))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            Button::new("submit-csv-form")
                                .primary()
                                .label(submit_label)
                                .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                                    view.submit_csv_form(
                                        &gpui_component::input::Enter { secondary: false },
                                        window,
                                        cx,
                                    )
                                })),
                        )
                        .child(
                            Button::new("cancel-csv-form")
                                .outline()
                                .label("Cancel (Esc)")
                                .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                                    view.cancel_csv_form(
                                        &gpui_component::input::Escape,
                                        window,
                                        cx,
                                    )
                                })),
                        ),
                ),
        )
}

use gpui::prelude::FluentBuilder;
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div, px,
};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::list::ListItem;
use gpui_component::Sizable as _;
use lootbox::Credential;

use crate::app::{AddCredential, AppView, ExportCsv, ImportCsv, OpenAbout, QuitApp};
use crate::mask;

pub const CONTEXT: &str = "credential_list";

/// Renders only the left panel (toolbar + rows). Action wiring (`on_action`/`key_context`/
/// `track_focus`) lives on the shared wrapper in `AppView::render`'s `Unlocked` arm, since
/// keyboard shortcuts here (Update/Remove/Env) act on state that's also rendered in the detail
/// pane -- there's no single screen module that "owns" those actions.
pub fn render(
    credentials: &[Credential],
    selected: Option<usize>,
    detail_locked: bool,
    _window: &mut Window,
    cx: &mut Context<AppView>,
) -> impl IntoElement {
    div()
        .w(px(280.0))
        .h_full()
        .flex()
        .flex_col()
        .border_r_1()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .p_2()
                .child(
                    Button::new("add")
                        .primary()
                        .icon(gpui_component::IconName::Plus)
                        .label("Add Credential")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_add_form(&AddCredential, window, cx)
                        })),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("export-csv")
                                .outline()
                                .small()
                                .icon(gpui_component::IconName::File)
                                .label("Export CSV")
                                .on_click(cx.listener(|view, _, window, cx| {
                                    view.open_export_csv(&ExportCsv, window, cx)
                                })),
                        )
                        .child(
                            Button::new("import-csv")
                                .outline()
                                .small()
                                .icon(gpui_component::IconName::FolderOpen)
                                .label("Import CSV")
                                .on_click(cx.listener(|view, _, window, cx| {
                                    view.open_import_csv(&ImportCsv, window, cx)
                                })),
                        ),
                ),
        )
        .child(
            div()
                .id("credential-rows")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(credentials.iter().enumerate().map(|(i, credential)| {
                    let is_selected = selected == Some(i);
                    ListItem::new(("credential-row", i))
                        .selected(is_selected)
                        .disabled(detail_locked)
                        .on_click(cx.listener(move |view, _, window, cx| {
                            view.select_credential(i, window, cx);
                        }))
                        .child(credential.key.clone())
                        .child(mask::MASK)
                })),
        )
        .when(credentials.is_empty(), |this| {
            this.child(div().p_2().child("No credentials yet."))
        })
        .child(
            div().p_2().border_t_1().flex().gap_1()
                .child(
                    Button::new("about")
                        .outline()
                        .small()
                        .icon(gpui_component::IconName::Info)
                        .label("About")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_about(&OpenAbout, window, cx)
                        })),
                )
                .child(
                    Button::new("quit")
                        .outline()
                        .small()
                        .icon(gpui_component::IconName::Close)
                        .label("Quit")
                        .on_click(cx.listener(|view, _, window, cx| view.quit_app(&QuitApp, window, cx))),
                ),
        )
}

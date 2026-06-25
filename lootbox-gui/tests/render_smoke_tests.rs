use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{AppContext as _, Entity, Render, TestAppContext, WindowHandle, WindowOptions};
use gpui_component::Root;
use lootbox_gui::app::{AddCredential, AppScreen, AppView, ConfirmNewFile, ExportCsv, ExportEnv, RemoveCredential};

/// Triggers `AppView::render` without going through `WindowHandle<Root>::update`, which would
/// hold an exclusive lock on the Root entity for the whole call (it hands back `&mut Root`,
/// even though we never use it). `AppView::render` reads Root itself (for the notification
/// layer), so rendering from inside an already-active Root update panics with "cannot read
/// Root while it is already being updated" -- a conflict that doesn't exist in the real paint
/// pipeline, where `&mut Window` comes from the platform/frame loop, not from locking Root.
/// The lower-level `cx.update_window` only hands back a type-erased `AnyView`, so it never
/// locks Root at all.
fn render_view(window: WindowHandle<Root>, view: &Entity<AppView>, cx: &mut TestAppContext) {
    cx.update_window(window.into(), |_, window, cx| {
        view.update(cx, |view, cx| {
            view.render(window, cx);
        });
    })
    .unwrap();
}

fn scratch_vault_path() -> PathBuf {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("vault.enc");
    std::mem::forget(dir);
    path
}

/// See the matching helper in `app_state_tests.rs` for why the window's root view must be a
/// `gpui_component::Root` wrapping `AppView`, not `AppView` directly.
fn open_test_window(
    cx: &mut TestAppContext,
    file_path: PathBuf,
) -> (WindowHandle<Root>, Entity<AppView>) {
    let captured: Rc<RefCell<Option<Entity<AppView>>>> = Rc::new(RefCell::new(None));
    let captured_for_closure = captured.clone();

    let window = cx.update(|cx| {
        gpui_component::init(cx);
        cx.open_window(WindowOptions::default(), move |window, cx| {
            let view = cx.new(|cx| AppView::new(Some(file_path.clone()), window, cx));
            *captured_for_closure.borrow_mut() = Some(view.clone());
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap()
    });

    let view = captured
        .borrow_mut()
        .take()
        .expect("AppView entity captured during window construction");
    (window, view)
}

/// Mirrors `open_test_window`, but for the no-CLI-arg launch path -- `AppView::new(None, ...)`.
fn open_test_window_no_path(cx: &mut TestAppContext) -> (WindowHandle<Root>, Entity<AppView>) {
    let captured: Rc<RefCell<Option<Entity<AppView>>>> = Rc::new(RefCell::new(None));
    let captured_for_closure = captured.clone();

    let window = cx.update(|cx| {
        gpui_component::init(cx);
        cx.open_window(WindowOptions::default(), move |window, cx| {
            let view = cx.new(|cx| AppView::new(None, window, cx));
            *captured_for_closure.borrow_mut() = Some(view.clone());
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap()
    });

    let view = captured
        .borrow_mut()
        .take()
        .expect("AppView entity captured during window construction");
    (window, view)
}

#[gpui::test]
fn render_file_picker_does_not_panic(cx: &mut TestAppContext) {
    let (window, view) = open_test_window_no_path(cx);

    render_view(window, &view, cx);
}

#[gpui::test]
fn render_new_file_confirm_does_not_panic(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    render_view(window, &view, cx);
}

#[gpui::test]
fn render_password_screen_does_not_panic(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_new_file(&ConfirmNewFile, window, cx);
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

/// A brand new vault has zero credentials, so the right panel renders `DetailPane::Empty` --
/// a distinct render path from `Read` that didn't exist before the split-pane redesign.
#[gpui::test]
fn render_empty_unlocked_does_not_panic(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_new_file(&ConfirmNewFile, window, cx);
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("a-valid-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

#[gpui::test]
fn render_credential_form_does_not_panic(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api_key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("correct-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
                view.open_add_form(&AddCredential, window, cx);
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

#[gpui::test]
fn render_remove_confirm_does_not_panic(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api_key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("correct-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
                view.open_remove_confirm(&RemoveCredential, window, cx);
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

/// Unlocking with at least one credential auto-selects and shows it, so this also covers the
/// `DetailPane::Read` render path without a separate "open read view" step.
#[gpui::test]
fn render_unlocked_with_read_detail_does_not_panic(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api_key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("correct-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

#[gpui::test]
fn render_env_vars_does_not_panic(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api_key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("correct-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
                view.open_env_vars(&ExportEnv, window, cx);
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

#[gpui::test]
fn render_env_vars_invalid_key_does_not_panic(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api@key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("correct-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
                view.open_env_vars(&ExportEnv, window, cx);
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

#[gpui::test]
fn render_csv_form_does_not_panic(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api_key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("correct-password", window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
                view.open_export_csv(&ExportCsv, window, cx);
            });
        })
        .unwrap();
    render_view(window, &view, cx);
}

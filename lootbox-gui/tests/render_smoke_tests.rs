use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{AppContext as _, Entity, Render, TestAppContext, WindowHandle, WindowOptions};
use gpui_component::Root;
use lootbox_gui::app::{
    AddCredential, AppScreen, AppView, ConfirmNewFile, ExportCsv, ExportEnv, RemoveCredential,
    ShowCredential,
};

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
            let view = cx.new(|cx| AppView::new(file_path.clone(), window, cx));
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
fn render_new_file_confirm_does_not_panic(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.render(window, cx);
            });
        })
        .unwrap();
}

#[gpui::test]
fn render_password_screen_does_not_panic(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_new_file(&ConfirmNewFile, window, cx);
                view.render(window, cx);
            });
        })
        .unwrap();
}

#[gpui::test]
fn render_empty_credential_list_does_not_panic(cx: &mut TestAppContext) {
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
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
                view.render(window, cx);
            });
        })
        .unwrap();
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
                view.render(window, cx);
            });
        })
        .unwrap();
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
                view.render(window, cx);
            });
        })
        .unwrap();
}

#[gpui::test]
fn render_read_view_does_not_panic(cx: &mut TestAppContext) {
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
                view.open_read_view(&ShowCredential, window, cx);
                view.render(window, cx);
            });
        })
        .unwrap();
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
                view.render(window, cx);
            });
        })
        .unwrap();
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
                view.render(window, cx);
            });
        })
        .unwrap();
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
                view.render(window, cx);
            });
        })
        .unwrap();
}

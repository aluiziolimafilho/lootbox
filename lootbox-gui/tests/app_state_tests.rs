use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{AppContext as _, Entity, TestAppContext, WindowHandle, WindowOptions};
use gpui_component::Root;
use lootbox_gui::app::{AppScreen, AppView, CancelNewFile, ConfirmNewFile};

fn scratch_vault_path() -> PathBuf {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("vault.enc");
    // Leak the tempdir so it outlives the test; files are cleaned up by the OS tmp
    // reaper, and these tests never write large amounts of data.
    std::mem::forget(dir);
    path
}

/// `gpui_component`'s widgets (Input, dialog layer, ...) expect the window's root view to be
/// a `gpui_component::Root` wrapping the real view (matching production's `main.rs` setup),
/// and they read theme state registered by `gpui_component::init` -- skipping either panics.
/// Returns both the `Root`-wrapping window handle (needed for layout/paint) and a direct
/// `Entity<AppView>` handle so tests can drive `AppView`'s own methods, which take `&mut
/// Window` the same way production action handlers do.
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
fn new_file_confirm_confirm_transitions_to_password_is_new(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    view.update(cx, |view, _| {
        assert!(matches!(view.screen, AppScreen::NewFileConfirm));
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_new_file(&ConfirmNewFile, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        assert!(matches!(
            view.screen,
            AppScreen::Password { is_new: true, .. }
        ));
    });
}

#[gpui::test]
fn new_file_confirm_cancel_quits_app(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    // Cancel calls cx.quit(); just confirm it doesn't panic when invoked.
    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.cancel_new_file(&CancelNewFile, window, cx);
            });
        })
        .unwrap();
}

#[gpui::test]
fn password_new_vault_rejects_short_password_and_sets_error(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_new_file(&ConfirmNewFile, window, cx);
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| state.set_value("short", window, cx));
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
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Password { error, .. } = &view.screen else {
            panic!("expected to remain on Password screen after a validation error");
        };
        assert!(error.is_some(), "expected a password validation error");
    });
}

#[gpui::test]
fn password_new_vault_valid_password_transitions_to_empty_list(cx: &mut TestAppContext) {
    let file_path = scratch_vault_path();
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_new_file(&ConfirmNewFile, window, cx);
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("a-valid-password", window, cx)
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
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialList { credentials, .. } = &view.screen else {
            panic!("expected to land on an empty CredentialList screen");
        };
        assert!(credentials.is_empty());
        assert_eq!(view.password, "a-valid-password");
    });
}

#[gpui::test]
fn password_unlock_wrong_password_sets_error(cx: &mut TestAppContext) {
    // Save a real credential first via lootbox-core, so the file exists on unlock.
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, "correct-password", "api_key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_test_window(cx, file_path);

    view.update(cx, |view, _| {
        assert!(matches!(
            view.screen,
            AppScreen::Password { is_new: false, .. }
        ));
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value("totally-wrong-password", window, cx)
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
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Password { error, .. } = &view.screen else {
            panic!("expected to remain on Password screen after wrong password");
        };
        assert!(error.is_some());
    });
}

#[gpui::test]
fn password_unlock_correct_password_loads_existing_credentials(cx: &mut TestAppContext) {
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
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialList { credentials, .. } = &view.screen else {
            panic!("expected to land on CredentialList screen");
        };
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].key, "api_key");
        assert_eq!(view.password, "correct-password");
    });
}

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{AppContext as _, Entity, TestAppContext, WindowHandle, WindowOptions};
use gpui_component::Root;
use lootbox_gui::app::{
    AddCredential, AppScreen, AppView, CancelNewFile, CancelRemove, ConfirmNewFile, ConfirmRemove,
    EditMode, RemoveCredential, SelectNext, SelectPrev, UpdateCredential,
};

const VAULT_PASSWORD: &str = "correct-password";

fn scratch_vault_path() -> PathBuf {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("vault.enc");
    // Leak the tempdir so it outlives the test; files are cleaned up by the OS tmp
    // reaper, and these tests never write large amounts of data.
    std::mem::forget(dir);
    path
}

/// Seeds a vault with `count` credentials (`key1`..`keyN`, `value1`..`valueN`) and returns its
/// path, for tests that need a non-empty `CredentialList` to act on.
fn seed_vault(count: usize) -> PathBuf {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("vault.enc");
    std::mem::forget(dir);
    for i in 1..=count {
        lootbox::save_credential(
            &path,
            VAULT_PASSWORD,
            &format!("key{i}"),
            &format!("value{i}"),
        )
        .expect("seed credential");
    }
    path
}

/// Drives `open_test_window` through to an unlocked, populated `CredentialList` screen.
fn open_unlocked_window(
    cx: &mut TestAppContext,
    file_path: PathBuf,
) -> (WindowHandle<Root>, Entity<AppView>) {
    let (window, view) = open_test_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Password { input, .. } = &view.screen else {
                    panic!("expected Password screen");
                };
                input.update(cx, |state, cx| {
                    state.set_value(VAULT_PASSWORD, window, cx)
                });
                view.submit_password(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    (window, view)
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

#[gpui::test]
fn select_next_and_prev_clamp_to_bounds(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(3));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.select_prev(&SelectPrev, window, cx); // already at 0, stays clamped
            });
        })
        .unwrap();
    view.update(cx, |view, _| {
        let AppScreen::CredentialList { selected, .. } = &view.screen else {
            panic!("expected CredentialList");
        };
        assert_eq!(*selected, 0);
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.select_next(&SelectNext, window, cx);
                view.select_next(&SelectNext, window, cx);
                view.select_next(&SelectNext, window, cx); // one past the end, should clamp
            });
        })
        .unwrap();
    view.update(cx, |view, _| {
        let AppScreen::CredentialList { selected, .. } = &view.screen else {
            panic!("expected CredentialList");
        };
        assert_eq!(*selected, 2);
    });
}

#[gpui::test]
fn add_credential_success_appends_to_list_and_returns_to_list(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_add_form(&AddCredential, window, cx);
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::CredentialForm {
                    key_input,
                    value_input,
                    ..
                } = &view.screen
                else {
                    panic!("expected CredentialForm");
                };
                key_input.update(cx, |state, cx| state.set_value("key2", window, cx));
                value_input.update(cx, |state, cx| state.set_value("value2", window, cx));
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_credential_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialList {
            credentials,
            selected,
        } = &view.screen
        else {
            panic!("expected to return to CredentialList after a successful add");
        };
        assert_eq!(credentials.len(), 2);
        assert_eq!(credentials[1].key, "key2");
        assert_eq!(*selected, 1, "selection should land on the newly added row");
    });
}

#[gpui::test]
fn add_credential_empty_key_is_rejected_with_inline_error(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_add_form(&AddCredential, window, cx);
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::CredentialForm { value_input, .. } = &view.screen else {
                    panic!("expected CredentialForm");
                };
                // Leave the key field empty; only fill in a value.
                value_input.update(cx, |state, cx| state.set_value("some-value", window, cx));
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_credential_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialForm { error, .. } = &view.screen else {
            panic!("expected to remain on CredentialForm after a validation error");
        };
        assert!(error.is_some());
    });
}

#[gpui::test]
fn update_credential_changes_key_and_value(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.select_next(&SelectNext, window, cx); // select row index 1 ("key2")
                view.open_update_form(&UpdateCredential, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, cx| {
        let AppScreen::CredentialForm { mode, key_input, value_input, .. } = &view.screen else {
            panic!("expected CredentialForm");
        };
        assert!(matches!(mode, EditMode::Update { id: 2 }));
        assert_eq!(key_input.read(cx).value().to_string(), "key2");
        assert_eq!(value_input.read(cx).value().to_string(), "value2");
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::CredentialForm {
                    key_input,
                    value_input,
                    ..
                } = &view.screen
                else {
                    panic!("expected CredentialForm");
                };
                key_input.update(cx, |state, cx| state.set_value("key2-renamed", window, cx));
                value_input.update(cx, |state, cx| {
                    state.set_value("value2-updated", window, cx)
                });
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_credential_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialList { credentials, .. } = &view.screen else {
            panic!("expected to return to CredentialList after a successful update");
        };
        assert_eq!(credentials.len(), 2);
        assert_eq!(credentials[1].key, "key2-renamed");
        assert_eq!(credentials[1].value, "value2-updated");
    });
}

/// Exercises the deliberate product decision documented on `AppView::submit_credential_form`:
/// clearing a pre-filled field and submitting must hit the normal non-empty validation error,
/// not silently keep the old value (unlike the CLI's blank-means-skip prompts).
#[gpui::test]
fn update_credential_cleared_value_is_rejected(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_update_form(&UpdateCredential, window, cx);
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::CredentialForm { value_input, .. } = &view.screen else {
                    panic!("expected CredentialForm");
                };
                value_input.update(cx, |state, cx| state.set_value("", window, cx));
            });
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_credential_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, cx| {
        let AppScreen::CredentialForm {
            error, value_input, ..
        } = &view.screen
        else {
            panic!("expected to remain on CredentialForm after a validation error");
        };
        assert!(error.is_some(), "expected a non-empty-value validation error");
        assert_eq!(
            value_input.read(cx).value().to_string(),
            "",
            "cleared field must not silently revert to the old value"
        );
    });
}

#[gpui::test]
fn remove_credential_confirm_removes_and_shifts_ids_down(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(3));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_remove_confirm(&RemoveCredential, window, cx); // removes "key1"
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::RemoveConfirm { id, key, .. } = &view.screen else {
            panic!("expected RemoveConfirm");
        };
        assert_eq!(*id, 1);
        assert_eq!(key, "key1");
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.confirm_remove(&ConfirmRemove, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialList { credentials, .. } = &view.screen else {
            panic!("expected to return to CredentialList after a successful remove");
        };
        assert_eq!(credentials.len(), 2);
        assert_eq!(credentials[0].key, "key2", "key2 should have shifted to id 1");
        assert_eq!(credentials[1].key, "key3");
    });
}

#[gpui::test]
fn remove_credential_cancel_returns_to_list_unchanged(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_remove_confirm(&RemoveCredential, window, cx);
                view.cancel_remove(&CancelRemove, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CredentialList { credentials, .. } = &view.screen else {
            panic!("expected to return to CredentialList");
        };
        assert_eq!(credentials.len(), 2, "cancel must not remove anything");
    });
}

/// Mirrors the TUI's `if cred_count > 0` guard: Update/Remove must no-op on an empty list.
#[gpui::test]
fn list_actions_noop_when_credentials_empty(cx: &mut TestAppContext) {
    // A brand new vault has zero credentials and isn't written to disk until the first
    // `save_credential` call, so this drives the "new vault" path rather than `seed_vault`.
    let (window, view) = open_test_window(cx, scratch_vault_path());

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
                view.open_update_form(&UpdateCredential, window, cx);
                view.open_remove_confirm(&RemoveCredential, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        assert!(
            matches!(view.screen, AppScreen::CredentialList { .. }),
            "Update/Remove on an empty list must stay on CredentialList, not panic or transition"
        );
    });
}

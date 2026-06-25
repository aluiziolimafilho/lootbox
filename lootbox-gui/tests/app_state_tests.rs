use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{AppContext as _, Entity, TestAppContext, WindowHandle, WindowOptions};
use gpui_component::Root;
use lootbox_gui::app::{
    AddCredential, AppScreen, AppView, CancelNewFile, ConfirmNewFile, ConfirmRemove,
    CopyEnvLine, CopyKey, CopyValue, DeselectCredential, DetailPane, EditMode, ExportCsv,
    ExportEnv, ImportCsv, RemoveCredential, SelectNext, SelectPrev, ToggleValueVisibility,
    UpdateCredential,
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
/// path, for tests that need a non-empty unlocked vault to act on.
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

/// Drives `open_test_window` through to an unlocked `AppScreen::Unlocked` screen.
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
fn password_new_vault_valid_password_transitions_to_empty_unlocked(cx: &mut TestAppContext) {
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
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &view.screen
        else {
            panic!("expected to land on Unlocked");
        };
        assert!(credentials.is_empty());
        assert_eq!(*selected, None);
        assert!(matches!(detail, DetailPane::Empty));
        assert_eq!(view.password, "a-valid-password");
    });
}

#[gpui::test]
fn password_unlock_wrong_password_sets_error(cx: &mut TestAppContext) {
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
fn password_unlock_correct_password_loads_existing_credentials_and_shows_first(
    cx: &mut TestAppContext,
) {
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
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &view.screen
        else {
            panic!("expected to land on Unlocked");
        };
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].key, "api_key");
        assert_eq!(*selected, Some(0));
        let DetailPane::Read { id, credential, .. } = detail else {
            panic!("expected the first credential to auto-load into the Read detail pane");
        };
        assert_eq!(*id, 1);
        assert_eq!(credential.key, "api_key");
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
        let AppScreen::Unlocked { selected, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        assert_eq!(*selected, Some(0));
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
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &view.screen
        else {
            panic!("expected Unlocked");
        };
        assert_eq!(*selected, Some(2));
        let DetailPane::Read { id, .. } = detail else {
            panic!("selecting should switch the detail pane to Read");
        };
        assert_eq!(*id, 3);
    });
}

#[gpui::test]
fn select_credential_is_a_noop_while_form_is_open(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(3));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_add_form(&AddCredential, window, cx);
                // Selection changes must be ignored while the Add form is open.
                view.select_credential(2, window, cx);
                view.select_next(&SelectNext, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &view.screen
        else {
            panic!("expected Unlocked");
        };
        assert_eq!(*selected, Some(0), "selection must not change while editing");
        assert!(matches!(detail, DetailPane::Form { .. }), "form must stay open");
    });
}

#[gpui::test]
fn deselect_credential_clears_selection_from_read(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.deselect_credential(&DeselectCredential, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &view.screen
        else {
            panic!("expected Unlocked");
        };
        assert_eq!(*selected, None);
        assert!(matches!(detail, DetailPane::Empty));
    });
}

#[gpui::test]
fn add_credential_success_appends_to_list_and_selects_new_row(cx: &mut TestAppContext) {
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
                let AppScreen::Unlocked { detail, .. } = &view.screen else {
                    panic!("expected Unlocked");
                };
                let DetailPane::Form {
                    key_input,
                    value_input,
                    ..
                } = detail
                else {
                    panic!("expected Form detail pane");
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
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &view.screen
        else {
            panic!("expected to return to Unlocked after a successful add");
        };
        assert_eq!(credentials.len(), 2);
        assert_eq!(credentials[1].key, "key2");
        assert_eq!(*selected, Some(1), "selection should land on the newly added row");
        let DetailPane::Read { credential, .. } = detail else {
            panic!("expected Read detail pane after add");
        };
        assert_eq!(credential.key, "key2");
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
                let AppScreen::Unlocked { detail, .. } = &view.screen else {
                    panic!("expected Unlocked");
                };
                let DetailPane::Form { value_input, .. } = detail else {
                    panic!("expected Form detail pane");
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
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Form { error, .. } = detail else {
            panic!("expected to remain on Form after a validation error");
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
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Form {
            mode,
            key_input,
            value_input,
            ..
        } = detail
        else {
            panic!("expected Form detail pane");
        };
        assert!(matches!(mode, EditMode::Update { id: 2 }));
        assert_eq!(key_input.read(cx).value().to_string(), "key2");
        assert_eq!(value_input.read(cx).value().to_string(), "value2");
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                let AppScreen::Unlocked { detail, .. } = &view.screen else {
                    panic!("expected Unlocked");
                };
                let DetailPane::Form {
                    key_input,
                    value_input,
                    ..
                } = detail
                else {
                    panic!("expected Form detail pane");
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
        let AppScreen::Unlocked { credentials, .. } = &view.screen else {
            panic!("expected to return to Unlocked after a successful update");
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
                let AppScreen::Unlocked { detail, .. } = &view.screen else {
                    panic!("expected Unlocked");
                };
                let DetailPane::Form { value_input, .. } = detail else {
                    panic!("expected Form detail pane");
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
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Form {
            error, value_input, ..
        } = detail
        else {
            panic!("expected to remain on Form after a validation error");
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
fn cancel_add_form_reverts_to_previous_selection(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.select_next(&SelectNext, window, cx); // select row index 1 ("key2")
                view.open_add_form(&AddCredential, window, cx);
                view.cancel_credential_form(&gpui_component::input::Escape, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked {
            credentials,
            selected,
            detail,
        } = &view.screen
        else {
            panic!("expected Unlocked");
        };
        assert_eq!(credentials.len(), 2, "cancel must not create anything");
        assert_eq!(*selected, Some(1), "selection should be unchanged after cancel");
        let DetailPane::Read { credential, .. } = detail else {
            panic!("expected to revert to Read of the previously selected row");
        };
        assert_eq!(credential.key, "key2");
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
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::RemoveConfirm { id, key, .. } = detail else {
            panic!("expected RemoveConfirm detail pane");
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
        let AppScreen::Unlocked { credentials, .. } = &view.screen else {
            panic!("expected to return to Unlocked after a successful remove");
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
                view.cancel_remove(&lootbox_gui::app::CancelRemove, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { credentials, .. } = &view.screen else {
            panic!("expected to return to Unlocked");
        };
        assert_eq!(credentials.len(), 2, "cancel must not remove anything");
    });
}

#[gpui::test]
fn deselect_escape_returns_from_remove_confirm_to_read(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_remove_confirm(&RemoveCredential, window, cx);
                view.deselect_credential(&DeselectCredential, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked {
            credentials, detail, ..
        } = &view.screen
        else {
            panic!("expected Unlocked");
        };
        assert_eq!(credentials.len(), 2, "Escape on RemoveConfirm must not remove anything");
        assert!(matches!(detail, DetailPane::Read { .. }));
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
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        assert!(
            matches!(detail, DetailPane::Empty),
            "Update/Remove on an empty list must stay Empty, not panic or transition"
        );
    });
}

#[gpui::test]
fn select_credential_toggle_value_visibility(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Read {
            id,
            credential,
            value_visible,
            ..
        } = detail
        else {
            panic!("expected Read detail pane");
        };
        assert_eq!(*id, 1);
        assert_eq!(credential.key, "key1");
        assert!(!value_visible, "value should start masked");
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.toggle_value_visibility(&ToggleValueVisibility, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Read { value_visible, .. } = detail else {
            panic!("expected Read detail pane");
        };
        assert!(*value_visible, "Tab should reveal the value");
    });
}

#[gpui::test]
fn copy_key_and_value_set_clipboard_status(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.copy_read_view_key(&CopyKey, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Read {
            clipboard_status, ..
        } = detail
        else {
            panic!("expected Read detail pane");
        };
        assert_eq!(clipboard_status.as_deref(), Some("Key copied!"));
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.copy_read_view_value(&CopyValue, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::Read {
            clipboard_status, ..
        } = detail
        else {
            panic!("expected Read detail pane");
        };
        assert_eq!(clipboard_status.as_deref(), Some("Value copied!"));
    });
}

#[gpui::test]
fn env_vars_valid_key_shows_created_entry(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_env_vars(&ExportEnv, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::EnvVars {
            env_name,
            value,
            error,
            ..
        } = detail
        else {
            panic!("expected EnvVars detail pane");
        };
        assert_eq!(env_name, "KEY1");
        assert_eq!(value, "value1");
        assert!(error.is_none());
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.copy_env_line(&CopyEnvLine, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::EnvVars {
            clipboard_status, ..
        } = detail
        else {
            panic!("expected EnvVars detail pane");
        };
        assert_eq!(clipboard_status.as_deref(), Some("Copied to clipboard!"));
    });
}

#[gpui::test]
fn env_vars_invalid_key_shows_invalid_reason(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file_path = dir.path().join("vault.enc");
    lootbox::save_credential(&file_path, VAULT_PASSWORD, "api@key", "secret-value")
        .expect("seed vault");

    let (window, view) = open_unlocked_window(cx, file_path);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_env_vars(&ExportEnv, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { detail, .. } = &view.screen else {
            panic!("expected Unlocked");
        };
        let DetailPane::EnvVars { error, .. } = detail else {
            panic!("expected EnvVars detail pane");
        };
        assert!(
            error
                .as_deref()
                .is_some_and(|reason| reason.contains("invalid character")),
            "expected an invalid-character reason, got {error:?}"
        );
    });
}

#[gpui::test]
fn back_to_read_from_env_vars_preserves_selection(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.select_next(&SelectNext, window, cx); // select row index 1 ("key2")
                view.open_env_vars(&ExportEnv, window, cx);
                view.toggle_value_visibility(&ToggleValueVisibility, window, cx);
                view.back_to_list_from_env_vars(window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked {
            selected, detail, ..
        } = &view.screen
        else {
            panic!("expected Unlocked");
        };
        assert_eq!(*selected, Some(1), "back should return to the same row");
        assert!(matches!(detail, DetailPane::Read { .. }));
    });
}

#[gpui::test]
fn export_csv_writes_file_and_status_message_matches(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(2));

    let export_dir = tempfile::tempdir().expect("create temp dir");
    let csv_path = export_dir.path().join("export.csv");
    let csv_path_str = csv_path.to_str().unwrap().to_string();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_export_csv(&ExportCsv, window, cx);
                let AppScreen::CsvForm { path_input, .. } = &view.screen else {
                    panic!("expected CsvForm");
                };
                path_input.update(cx, |state, cx| {
                    state.set_value(csv_path_str.clone(), window, cx)
                });
                view.submit_csv_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CsvForm { status, .. } = &view.screen else {
            panic!("expected to remain on CsvForm to show the success status");
        };
        assert_eq!(
            status.as_deref(),
            Some(format!("Exported to {}", csv_path.display()).as_str())
        );
    });

    let contents = std::fs::read_to_string(&csv_path).expect("export.csv should exist");
    assert!(contents.starts_with("key,value"));
    assert!(contents.contains("key1,value1"));
    assert!(contents.contains("key2,value2"));

    // A second Enter (status already set) returns to the list instead of re-submitting.
    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_csv_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        assert!(matches!(view.screen, AppScreen::Unlocked { .. }));
    });
}

#[gpui::test]
fn import_csv_appends_credentials_and_reports_correct_count(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    let import_dir = tempfile::tempdir().expect("create temp dir");
    let csv_path = import_dir.path().join("import.csv");
    std::fs::write(&csv_path, "key,value\nimported1,val1\nimported2,val2\n")
        .expect("write import.csv");
    let csv_path_str = csv_path.to_str().unwrap().to_string();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_import_csv(&ImportCsv, window, cx);
                let AppScreen::CsvForm { path_input, .. } = &view.screen else {
                    panic!("expected CsvForm");
                };
                path_input.update(cx, |state, cx| {
                    state.set_value(csv_path_str.clone(), window, cx)
                });
                view.submit_csv_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CsvForm { status, .. } = &view.screen else {
            panic!("expected to remain on CsvForm to show the success status");
        };
        assert_eq!(status.as_deref(), Some("Imported 2 credential(s)."));
    });

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.submit_csv_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { credentials, .. } = &view.screen else {
            panic!("expected to return to Unlocked");
        };
        assert_eq!(credentials.len(), 3, "1 seeded + 2 imported");
    });
}

#[gpui::test]
fn csv_form_cancel_returns_to_list_unchanged(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_export_csv(&ExportCsv, window, cx);
                view.cancel_csv_form(&gpui_component::input::Escape, window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::Unlocked { credentials, .. } = &view.screen else {
            panic!("expected to return to Unlocked");
        };
        assert_eq!(credentials.len(), 1, "cancel must not change anything");
    });
}

#[gpui::test]
fn export_csv_invalid_path_sets_error(cx: &mut TestAppContext) {
    let (window, view) = open_unlocked_window(cx, seed_vault(1));

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_export_csv(&ExportCsv, window, cx);
                let AppScreen::CsvForm { path_input, .. } = &view.screen else {
                    panic!("expected CsvForm");
                };
                // A directory that doesn't exist, with a nonexistent parent -- write must fail.
                path_input.update(cx, |state, cx| {
                    state.set_value("/this/path/does/not/exist/export.csv", window, cx)
                });
                view.submit_csv_form(
                    &gpui_component::input::Enter { secondary: false },
                    window,
                    cx,
                );
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        let AppScreen::CsvForm { error, status, .. } = &view.screen else {
            panic!("expected to remain on CsvForm after a write error");
        };
        assert!(error.is_some());
        assert!(status.is_none());
    });
}

#[gpui::test]
fn file_picker_shown_when_no_path_given(cx: &mut TestAppContext) {
    let (_window, view) = open_test_window_no_path(cx);

    view.update(cx, |view, _| {
        assert!(matches!(view.screen, AppScreen::FilePicker));
        assert!(view.file_path.is_none());
    });
}

#[gpui::test]
fn open_existing_vault_at_transitions_to_password_not_new(cx: &mut TestAppContext) {
    let (window, view) = open_test_window_no_path(cx);
    let seeded_path = seed_vault(1);

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.open_existing_vault_at(seeded_path.clone(), window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        assert_eq!(view.file_path, Some(seeded_path));
        assert!(matches!(
            view.screen,
            AppScreen::Password { is_new: false, .. }
        ));
    });
}

#[gpui::test]
fn create_new_vault_at_transitions_to_password_is_new(cx: &mut TestAppContext) {
    let (window, view) = open_test_window_no_path(cx);
    let fresh_path = scratch_vault_path();

    window
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.create_new_vault_at(fresh_path.clone(), window, cx);
            });
        })
        .unwrap();

    view.update(cx, |view, _| {
        assert_eq!(view.file_path, Some(fresh_path));
        assert!(matches!(
            view.screen,
            AppScreen::Password { is_new: true, .. }
        ));
        assert!(
            !matches!(view.screen, AppScreen::NewFileConfirm),
            "creating via the dialog must skip the extra NewFileConfirm step"
        );
    });
}

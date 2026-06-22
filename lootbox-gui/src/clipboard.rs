/// Wraps a value in single quotes and escapes any single quotes within it, producing a
/// POSIX-safe shell string for use in `export KEY='value'`. Mirrors `lootbox-core`'s CLI
/// `shell_escape` helper, but that one is private to `main.rs` -- this is purely clipboard/
/// display formatting, not storage logic, so it's kept GUI-side rather than shared.
pub fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

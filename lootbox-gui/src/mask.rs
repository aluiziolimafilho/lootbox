/// Fixed-width mask for read-only display of secret values, matching the CLI `list`
/// command's documented convention (see CLAUDE.md): exactly 10 asterisks regardless of
/// actual length, so a passive viewer can never infer a secret's length.
///
/// This is distinct from `InputState::masked(true)`'s own per-character password-field
/// masking used while actively typing (Password screen, Value field) -- that conventional
/// password-input behavior reveals length to the typist, which isn't the leak this guards
/// against, so don't apply this constant there.
pub const MASK: &str = "**********";

use crate::storage::{
    generate_env_vars, list_credentials, read_credential, remove_credential, save_credential,
    update_credential, Credential,
};
use crate::validation::validate_password;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::{
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

// ─────────────────────────────────────────────────────────────── state types ─

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Key,
    Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateStep {
    EditFields,
}

enum AppState {
    NewFileConfirm,
    PasswordInput {
        input: String,
        error: Option<String>,
        is_new: bool,
    },
    CredentialList {
        credentials: Vec<Credential>,
        selected: usize,
    },
    AddForm {
        key: String,
        value: String,
        focus: Field,
        value_visible: bool,
        error: Option<String>,
    },
    UpdateForm {
        step: UpdateStep,
        id: String,
        key: String,
        value: String,
        focus: Field,
        value_visible: bool,
        error: Option<String>,
    },
    RemoveConfirm {
        id: usize,
        key: String,
        error: Option<String>,
    },
    ReadView {
        id: usize,
        credential: Credential,
        value_visible: bool,
    },
    EnvVars {
        env_name: String,
        value: String,
        value_visible: bool,
        clipboard_status: Option<String>,
        error: Option<String>,
    },
}

struct App {
    file_path: PathBuf,
    password: String,
    state: AppState,
}

impl App {
    fn new(file_path: PathBuf) -> Self {
        let state = if file_path.exists() {
            AppState::PasswordInput { input: String::new(), error: None, is_new: false }
        } else {
            AppState::NewFileConfirm
        };
        Self { file_path, password: String::new(), state }
    }

    fn reload_credentials(&self) -> Result<Vec<Credential>> {
        list_credentials(&self.file_path, &self.password)
    }

    fn credential_list_state(&self) -> AppState {
        AppState::CredentialList {
            credentials: self.reload_credentials().unwrap_or_default(),
            selected: 0,
        }
    }
}

// ──────────────────────────────────────────────────────────────── public API ─

pub fn run(file_path: PathBuf) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, file_path);

    restore_terminal(&mut terminal)?;
    result
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────── event loop ─

fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, file_path: PathBuf) -> Result<()> {
    let mut app = App::new(file_path);

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if handle_key(key.code, key.modifiers, &mut app)? {
                    break;
                }
            }
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────── key dispatcher ─

fn handle_key(code: KeyCode, modifiers: KeyModifiers, app: &mut App) -> Result<bool> {
    match &app.state {
        AppState::NewFileConfirm => handle_new_file_confirm(code, app),
        AppState::PasswordInput { .. } => handle_password(code, app),
        AppState::CredentialList { .. } => handle_list(code, app),
        AppState::AddForm { .. } => handle_add(code, modifiers, app),
        AppState::UpdateForm { .. } => handle_update(code, modifiers, app),
        AppState::RemoveConfirm { .. } => handle_remove(code, app),
        AppState::ReadView { .. } => handle_read_view(code, app),
        AppState::EnvVars { .. } => handle_env_vars(code, app),
    }
}

fn handle_new_file_confirm(code: KeyCode, app: &mut App) -> Result<bool> {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.state = AppState::PasswordInput {
                input: String::new(),
                error: None,
                is_new: true,
            };
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

fn handle_password(code: KeyCode, app: &mut App) -> Result<bool> {
    let AppState::PasswordInput { ref mut input, ref mut error, is_new } = app.state else {
        return Ok(false);
    };
    let is_new = is_new;

    match code {
        KeyCode::Esc => {
            if is_new {
                app.state = AppState::NewFileConfirm;
            } else {
                return Ok(true);
            }
        }
        KeyCode::Char('q') if !is_new => return Ok(true),
        KeyCode::Backspace => {
            if let AppState::PasswordInput { ref mut input, ref mut error, .. } = app.state {
                input.pop();
                *error = None;
            }
        }
        KeyCode::Enter => {
            let password = input.clone();
            if is_new {
                match validate_password(&password) {
                    Ok(()) => {
                        app.password = password;
                        app.state = AppState::CredentialList { credentials: vec![], selected: 0 };
                    }
                    Err(e) => {
                        *error = Some(e.to_string());
                        input.clear();
                    }
                }
            } else {
                match list_credentials(&app.file_path, &password) {
                    Ok(credentials) => {
                        app.password = password;
                        app.state = AppState::CredentialList { credentials, selected: 0 };
                    }
                    Err(e) => {
                        *error = Some(e.to_string());
                        input.clear();
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            if let AppState::PasswordInput { ref mut input, ref mut error, .. } = app.state {
                input.push(c);
                *error = None;
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_list(code: KeyCode, app: &mut App) -> Result<bool> {
    // Read current selection before mutating state
    let (cred_count, selected) =
        if let AppState::CredentialList { ref credentials, selected } = app.state {
            (credentials.len(), selected)
        } else {
            (0, 0)
        };

    match code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),

        KeyCode::Up => {
            if let AppState::CredentialList { ref mut selected, .. } = app.state {
                if *selected > 0 { *selected -= 1; }
            }
        }
        KeyCode::Down => {
            if let AppState::CredentialList { ref mut selected, .. } = app.state {
                if *selected + 1 < cred_count { *selected += 1; }
            }
        }

        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.state = AppState::AddForm {
                key: String::new(),
                value: String::new(),
                focus: Field::Key,
                value_visible: false,
                error: None,
            };
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            if cred_count > 0 {
                let pos = selected + 1;
                if let Ok(cred) = read_credential(&app.file_path, &app.password, pos) {
                    app.state = AppState::UpdateForm {
                        step: UpdateStep::EditFields,
                        id: format!("{pos}"),
                        key: cred.key,
                        value: cred.value,
                        focus: Field::Key,
                        value_visible: false,
                        error: None,
                    };
                }
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if cred_count > 0 {
                let key = if let AppState::CredentialList { ref credentials, selected } = app.state {
                    credentials[selected].key.clone()
                } else {
                    String::new()
                };
                app.state = AppState::RemoveConfirm {
                    id: selected + 1,
                    key,
                    error: None,
                };
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if cred_count > 0 {
                let pos = selected + 1;
                if let Ok(cred) = read_credential(&app.file_path, &app.password, pos) {
                    app.state = AppState::ReadView {
                        id: pos,
                        credential: cred,
                        value_visible: false,
                    };
                }
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if cred_count > 0 {
                let pos = selected + 1;
                match generate_env_vars(&app.file_path, &app.password, pos) {
                    Ok(result) => {
                        if let Some(entry) = result.created.first() {
                            let escaped = entry.value.replace('\'', "'\\''");
                            let export_line = format!("export {}='{}'", entry.env_name, escaped);
                            let clipboard_status = match arboard::Clipboard::new()
                                .and_then(|mut c| c.set_text(export_line))
                            {
                                Ok(()) => Some("Copied to clipboard!".to_string()),
                                Err(_) => Some("Clipboard unavailable".to_string()),
                            };
                            app.state = AppState::EnvVars {
                                env_name: entry.env_name.clone(),
                                value: entry.value.clone(),
                                value_visible: false,
                                clipboard_status,
                                error: None,
                            };
                        } else if let Some(inv) = result.invalid.first() {
                            app.state = AppState::EnvVars {
                                env_name: String::new(),
                                value: String::new(),
                                value_visible: false,
                                clipboard_status: None,
                                error: Some(inv.reason.clone()),
                            };
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_add(code: KeyCode, modifiers: KeyModifiers, app: &mut App) -> Result<bool> {
    let AppState::AddForm {
        ref mut key,
        ref mut value,
        ref mut focus,
        ref mut value_visible,
        ref mut error,
    } = app.state
    else {
        return Ok(false);
    };

    match code {
        KeyCode::Esc => {
            app.state = app.credential_list_state();
        }
        KeyCode::Tab => {
            if *focus == Field::Key {
                *focus = Field::Value;
            } else {
                *value_visible = !*value_visible;
            }
            *error = None;
        }
        KeyCode::BackTab => {
            if *focus == Field::Value {
                *focus = Field::Key;
            }
            *error = None;
        }
        KeyCode::Backspace => {
            if *focus == Field::Key {
                key.pop();
            } else {
                value.pop();
            }
            *error = None;
        }
        KeyCode::Enter => {
            if *focus == Field::Key && !modifiers.contains(KeyModifiers::SHIFT) {
                *focus = Field::Value;
                return Ok(false);
            }
            let k = key.clone();
            let v = value.clone();
            match save_credential(&app.file_path, &app.password, &k, &v) {
                Ok(()) => {
                    app.state = app.credential_list_state();
                }
                Err(e) => {
                    *error = Some(e.to_string());
                }
            }
        }
        KeyCode::Char(c) => {
            if *focus == Field::Key {
                key.push(c);
            } else {
                value.push(c);
            }
            *error = None;
        }
        _ => {}
    }
    Ok(false)
}

fn handle_update(code: KeyCode, modifiers: KeyModifiers, app: &mut App) -> Result<bool> {
    let AppState::UpdateForm {
        ref mut step,
        ref mut id,
        ref mut key,
        ref mut value,
        ref mut focus,
        ref mut value_visible,
        ref mut error,
    } = app.state
    else {
        return Ok(false);
    };

    match step {
        UpdateStep::EditFields => match code {
            KeyCode::Esc => {
                app.state = app.credential_list_state();
            }
            KeyCode::Tab => {
                if *focus == Field::Key {
                    *focus = Field::Value;
                } else {
                    *value_visible = !*value_visible;
                }
                *error = None;
            }
            KeyCode::BackTab => {
                if *focus == Field::Value {
                    *focus = Field::Key;
                }
                *error = None;
            }
            KeyCode::Backspace => {
                if *focus == Field::Key {
                    key.pop();
                } else {
                    value.pop();
                }
                *error = None;
            }
            KeyCode::Enter => {
                if *focus == Field::Key && !modifiers.contains(KeyModifiers::SHIFT) {
                    *focus = Field::Value;
                    return Ok(false);
                }
                let pos: usize = id.parse().unwrap_or(0);
                let new_key = Some(key.as_str());
                let new_value = Some(value.as_str());
                match update_credential(&app.file_path, &app.password, pos, new_key, new_value) {
                    Ok(()) => {
                        app.state = app.credential_list_state();
                    }
                    Err(e) => {
                        *error = Some(e.to_string());
                    }
                }
            }
            KeyCode::Char(c) => {
                if *focus == Field::Key {
                    key.push(c);
                } else {
                    value.push(c);
                }
                *error = None;
            }
            _ => {}
        },
    }
    Ok(false)
}

fn handle_remove(code: KeyCode, app: &mut App) -> Result<bool> {
    let AppState::RemoveConfirm { id, ref mut error, .. } = app.state else {
        return Ok(false);
    };

    match code {
        KeyCode::Esc => {
            app.state = app.credential_list_state();
        }
        KeyCode::Enter => {
            match remove_credential(&app.file_path, &app.password, id) {
                Ok(()) => {
                    app.state = app.credential_list_state();
                }
                Err(e) => {
                    *error = Some(e.to_string());
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_read_view(code: KeyCode, app: &mut App) -> Result<bool> {
    let AppState::ReadView { ref mut value_visible, .. } = app.state else {
        return Ok(false);
    };

    match code {
        KeyCode::Tab => {
            *value_visible = !*value_visible;
        }
        KeyCode::Esc => {
            app.state = app.credential_list_state();
        }
        _ => {}
    }
    Ok(false)
}

fn handle_env_vars(code: KeyCode, app: &mut App) -> Result<bool> {
    let AppState::EnvVars {
        ref env_name,
        ref value,
        ref mut value_visible,
        ref mut clipboard_status,
        ..
    } = app.state
    else {
        return Ok(false);
    };

    match code {
        KeyCode::Tab => {
            *value_visible = !*value_visible;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if !env_name.is_empty() {
                let escaped = value.replace('\'', "'\\''");
                let export_line = format!("export {}='{}'", env_name, escaped);
                *clipboard_status = match arboard::Clipboard::new()
                    .and_then(|mut c| c.set_text(export_line))
                {
                    Ok(()) => Some("Copied to clipboard!".to_string()),
                    Err(_) => Some("Clipboard unavailable".to_string()),
                };
            }
        }
        _ => {
            app.state = app.credential_list_state();
        }
    }
    Ok(false)
}

// ─────────────────────────────────────────────────────────────────── drawing ─

fn draw(f: &mut Frame, app: &App) {
    match &app.state {
        AppState::NewFileConfirm => draw_new_file_confirm(f, &app.file_path),
        AppState::PasswordInput { input, error, is_new } => {
            draw_password(f, &app.file_path, input, error, *is_new);
        }
        AppState::CredentialList { credentials, selected } => {
            draw_list(f, &app.file_path, credentials, *selected);
        }
        AppState::AddForm { key, value, focus, value_visible, error } => {
            draw_list_bg(f);
            draw_add_form(f, key, value, *focus, *value_visible, error);
        }
        AppState::UpdateForm { step, id, key, value, focus, value_visible, error } => {
            draw_list_bg(f);
            draw_update_form(f, *step, id, key, value, *focus, *value_visible, error);
        }
        AppState::RemoveConfirm { id, key, error } => {
            draw_list_bg(f);
            draw_remove_form(f, *id, key, error);
        }
        AppState::ReadView { id, credential, value_visible } => {
            draw_read_view(f, *id, credential, *value_visible);
        }
        AppState::EnvVars { env_name, value, value_visible, clipboard_status, error } => {
            draw_env_vars(f, env_name, value, *value_visible, clipboard_status, error);
        }
    }
}

fn draw_new_file_confirm(f: &mut Frame, file_path: &PathBuf) {
    let area = f.area();

    let block = Block::default()
        .title(" LootBox ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);

    let inner = centered_rect(60, 40, area);
    let file_name = file_path.display().to_string();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&file_name, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  File does not exist. Create a new vault?",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [Y]es / Enter → create   [N]o / Esc → quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);
}

fn draw_password(f: &mut Frame, file_path: &PathBuf, input: &str, error: &Option<String>, is_new: bool) {
    let area = f.area();

    let title = if is_new { " LootBox — New Vault " } else { " LootBox " };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = centered_rect(60, 40, area);

    let masked = "●".repeat(input.len());
    let file_name = file_path.display().to_string();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&file_name, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Password: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}_", masked),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(err) = error {
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(""));
    }

    let hint = if is_new {
        "  Enter → set password   Esc → back"
    } else {
        "  Enter → unlock   Esc → quit"
    };
    lines.push(Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))));

    let para = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(para, inner);
}

fn draw_list(f: &mut Frame, file_path: &PathBuf, credentials: &[Credential], selected: usize) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let title = format!(" LootBox — {} ", file_path.display());
    let block = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let header = Row::new(vec!["ID", "Key", "Value"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = credentials
        .iter()
        .enumerate()
        .map(|(i, c)| {
            Row::new(vec![
                format!("{}", i + 1),
                c.key.clone(),
                "**********".to_string(),
            ])
            .style(Style::default().fg(Color::White))
        })
        .collect();

    let widths = [Constraint::Length(4), Constraint::Min(20), Constraint::Length(12)];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(2)
        .row_highlight_style(
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(Some(selected));
    f.render_stateful_widget(table, chunks[0], &mut state);

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(" [A]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("dd  ", Style::default().fg(Color::White)),
        Span::styled("[U]", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled("pdate  ", Style::default().fg(Color::White)),
        Span::styled("[R]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("emove  ", Style::default().fg(Color::White)),
        Span::styled("[S]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("how  ", Style::default().fg(Color::White)),
        Span::styled("[E]", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("nv  ", Style::default().fg(Color::White)),
        Span::styled("[Q]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Span::styled("uit  ", Style::default().fg(Color::White)),
        Span::styled("↑↓ scroll", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(bar, chunks[1]);
}

fn draw_list_bg(f: &mut Frame) {
    let area = f.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(block, area);
}

fn mask(s: &str) -> String {
    "●".repeat(s.chars().count())
}

fn draw_add_form(
    f: &mut Frame,
    key: &str,
    value: &str,
    focus: Field,
    value_visible: bool,
    error: &Option<String>,
) {
    let area = centered_rect(60, 55, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Add Credential ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let key_style = if focus == Field::Key {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let val_style = if focus == Field::Value {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let displayed_value = if value_visible {
        format!("{}_", value)
    } else {
        format!("{}_", mask(value))
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Key:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}_", key), key_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Value: ", Style::default().fg(Color::DarkGray)),
            Span::styled(displayed_value, val_style),
        ]),
        Line::from(""),
    ];

    if let Some(err) = error {
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(""));
    }

    let hint = if focus == Field::Key {
        "  Tab → go to value   Enter → save   Esc → cancel"
    } else {
        "  Tab → show/hide value   BackTab → go to key   Enter → save   Esc → cancel"
    };
    lines.push(Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_update_form(
    f: &mut Frame,
    step: UpdateStep,
    id: &str,
    key: &str,
    value: &str,
    focus: Field,
    value_visible: bool,
    error: &Option<String>,
) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Update Credential ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![Line::from("")];

    match step {
        UpdateStep::EditFields => {
            let key_style = if focus == Field::Key {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let val_style = if focus == Field::Value {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let displayed_value = if value_visible {
                format!("{}_", value)
            } else {
                format!("{}_", mask(value))
            };

            lines.push(Line::from(Span::styled(
                format!("  Editing ID: {id}"),
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Key:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}_", key), key_style),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Value: ", Style::default().fg(Color::DarkGray)),
                Span::styled(displayed_value, val_style),
            ]));
            lines.push(Line::from(""));
            if let Some(err) = error {
                lines.push(Line::from(Span::styled(
                    format!("  {err}"),
                    Style::default().fg(Color::Red),
                )));
                lines.push(Line::from(""));
            }
            let hint = if focus == Field::Key {
                "  Tab → go to value   Enter → save   Esc → cancel"
            } else {
                "  Tab → show/hide value   BackTab → go to key   Enter → save   Esc → cancel"
            };
            lines.push(Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))));
        }
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_remove_form(f: &mut Frame, id: usize, key: &str, error: &Option<String>) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Remove Credential ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Removing: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("[{id}] {key}"),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(err) = error {
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "  Enter → confirm   Esc → cancel",
        Style::default().fg(Color::DarkGray),
    )));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_read_view(f: &mut Frame, id: usize, credential: &Credential, value_visible: bool) {
    let area = f.area();

    let block = Block::default()
        .title(" Show Credential ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let displayed_value = if value_visible {
        credential.value.clone()
    } else {
        mask(&credential.value)
    };

    let toggle_label = if value_visible { "hide" } else { "show" };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ID:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", id),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Key:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(&credential.key, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Value: ", Style::default().fg(Color::DarkGray)),
            Span::styled(displayed_value, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Tab → {toggle_label} value   Esc → back"),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_env_vars(
    f: &mut Frame,
    env_name: &str,
    value: &str,
    value_visible: bool,
    clipboard_status: &Option<String>,
    error: &Option<String>,
) {
    let area = f.area();

    let block = Block::default()
        .title(" Environment Variable ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![Line::from("")];

    if let Some(err) = error {
        lines.push(Line::from(Span::styled(
            format!("  Error: {}", err),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Press any key to go back",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let displayed_value = if value_visible {
            value.replace('\'', "'\\''")
        } else {
            mask(value)
        };
        let export_line = format!("  export {}='{}'", env_name, displayed_value);

        lines.push(Line::from(Span::styled(
            export_line,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if let Some(status) = clipboard_status {
            let status_style = if status.contains("unavailable") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(Span::styled(
                format!("  # {}", status),
                status_style,
            )));
            lines.push(Line::from(""));
        }

        let toggle_label = if value_visible { "hide value" } else { "show value" };
        lines.push(Line::from(Span::styled(
            format!("  Tab → {}   C → copy again   Esc → back", toggle_label),
            Style::default().fg(Color::DarkGray),
        )));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

// ──────────────────────────────────────────────────────────────────── helpers ─

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

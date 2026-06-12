use crate::storage::{
    generate_env_vars, list_credentials, remove_credential, save_credential, update_credential,
    Credential,
};
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
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
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
    SelectId,
    EditFields,
}

enum AppState {
    PasswordInput {
        input: String,
        error: Option<String>,
    },
    CredentialList {
        credentials: Vec<Credential>,
    },
    AddForm {
        key: String,
        value: String,
        focus: Field,
        error: Option<String>,
    },
    UpdateForm {
        step: UpdateStep,
        id: String,
        key: String,
        value: String,
        focus: Field,
        error: Option<String>,
    },
    RemoveConfirm {
        id: String,
        error: Option<String>,
    },
    EnvVars {
        output: String,
    },
}

struct App {
    file_path: PathBuf,
    password: String,
    state: AppState,
}

impl App {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            password: String::new(),
            state: AppState::PasswordInput {
                input: String::new(),
                error: None,
            },
        }
    }

    fn reload_credentials(&self) -> Result<Vec<Credential>> {
        list_credentials(&self.file_path, &self.password)
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
        AppState::PasswordInput { .. } => handle_password(code, app),
        AppState::CredentialList { .. } => handle_list(code, app),
        AppState::AddForm { .. } => handle_add(code, modifiers, app),
        AppState::UpdateForm { .. } => handle_update(code, modifiers, app),
        AppState::RemoveConfirm { .. } => handle_remove(code, app),
        AppState::EnvVars { .. } => {
            app.state = AppState::CredentialList {
                credentials: app.reload_credentials().unwrap_or_default(),
            };
            Ok(false)
        }
    }
}

fn handle_password(code: KeyCode, app: &mut App) -> Result<bool> {
    let AppState::PasswordInput { ref mut input, ref mut error } = app.state else {
        return Ok(false);
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
        KeyCode::Backspace => {
            input.pop();
            *error = None;
        }
        KeyCode::Enter => {
            let password = input.clone();
            match list_credentials(&app.file_path, &password) {
                Ok(credentials) => {
                    app.password = password;
                    app.state = AppState::CredentialList { credentials };
                }
                Err(e) => {
                    *error = Some(e.to_string());
                    input.clear();
                }
            }
        }
        KeyCode::Char(c) => {
            input.push(c);
            *error = None;
        }
        _ => {}
    }
    Ok(false)
}

fn handle_list(code: KeyCode, app: &mut App) -> Result<bool> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.state = AppState::AddForm {
                key: String::new(),
                value: String::new(),
                focus: Field::Key,
                error: None,
            };
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            app.state = AppState::UpdateForm {
                step: UpdateStep::SelectId,
                id: String::new(),
                key: String::new(),
                value: String::new(),
                focus: Field::Key,
                error: None,
            };
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.state = AppState::RemoveConfirm {
                id: String::new(),
                error: None,
            };
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            match generate_env_vars(&app.file_path, &app.password) {
                Ok(result) => {
                    let mut output = String::new();
                    for entry in &result.created {
                        let escaped = entry.value.replace('\'', "'\\''");
                        output.push_str(&format!("export {}='{}'\n", entry.env_name, escaped));
                    }
                    if !result.invalid.is_empty() {
                        output.push_str("\n# Skipped:\n");
                        for entry in &result.invalid {
                            output.push_str(&format!(
                                "#   {} - {}\n",
                                entry.original_key, entry.reason
                            ));
                        }
                    }
                    if output.is_empty() {
                        output = "(no valid environment variables)".to_string();
                    }
                    app.state = AppState::EnvVars { output };
                }
                Err(e) => {
                    // show error inline by reloading list with a transient message not stored
                    // (simplest: stay on list screen, nothing changes)
                    let _ = e;
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_add(code: KeyCode, modifiers: KeyModifiers, app: &mut App) -> Result<bool> {
    let AppState::AddForm { ref mut key, ref mut value, ref mut focus, ref mut error } =
        app.state
    else {
        return Ok(false);
    };

    match code {
        KeyCode::Esc => {
            let credentials = list_credentials(&app.file_path, &app.password).unwrap_or_default();
            app.state = AppState::CredentialList { credentials };
        }
        KeyCode::Tab => {
            *focus = if *focus == Field::Key { Field::Value } else { Field::Key };
            *error = None;
        }
        KeyCode::BackTab => {
            *focus = if *focus == Field::Key { Field::Value } else { Field::Key };
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
                    let credentials =
                        list_credentials(&app.file_path, &app.password).unwrap_or_default();
                    app.state = AppState::CredentialList { credentials };
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
        ref mut error,
    } = app.state
    else {
        return Ok(false);
    };

    match step {
        UpdateStep::SelectId => match code {
            KeyCode::Esc => {
                let credentials =
                    list_credentials(&app.file_path, &app.password).unwrap_or_default();
                app.state = AppState::CredentialList { credentials };
            }
            KeyCode::Backspace => {
                id.pop();
                *error = None;
            }
            KeyCode::Enter => match id.parse::<usize>() {
                Ok(_) => {
                    *step = UpdateStep::EditFields;
                    *error = None;
                }
                Err(_) => {
                    *error = Some("ID must be a positive integer".to_string());
                }
            },
            KeyCode::Char(c) if c.is_ascii_digit() => {
                id.push(c);
                *error = None;
            }
            _ => {}
        },
        UpdateStep::EditFields => match code {
            KeyCode::Esc => {
                let credentials =
                    list_credentials(&app.file_path, &app.password).unwrap_or_default();
                app.state = AppState::CredentialList { credentials };
            }
            KeyCode::Tab => {
                *focus = if *focus == Field::Key { Field::Value } else { Field::Key };
                *error = None;
            }
            KeyCode::BackTab => {
                *focus = if *focus == Field::Key { Field::Value } else { Field::Key };
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
                let new_key = if key.is_empty() { None } else { Some(key.as_str()) };
                let new_value = if value.is_empty() { None } else { Some(value.as_str()) };
                match update_credential(&app.file_path, &app.password, pos, new_key, new_value) {
                    Ok(()) => {
                        let credentials =
                            list_credentials(&app.file_path, &app.password).unwrap_or_default();
                        app.state = AppState::CredentialList { credentials };
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
    let AppState::RemoveConfirm { ref mut id, ref mut error } = app.state else {
        return Ok(false);
    };

    match code {
        KeyCode::Esc => {
            let credentials = list_credentials(&app.file_path, &app.password).unwrap_or_default();
            app.state = AppState::CredentialList { credentials };
        }
        KeyCode::Backspace => {
            id.pop();
            *error = None;
        }
        KeyCode::Enter => match id.parse::<usize>() {
            Ok(pos) => match remove_credential(&app.file_path, &app.password, pos) {
                Ok(()) => {
                    let credentials =
                        list_credentials(&app.file_path, &app.password).unwrap_or_default();
                    app.state = AppState::CredentialList { credentials };
                }
                Err(e) => {
                    *error = Some(e.to_string());
                }
            },
            Err(_) => {
                *error = Some("ID must be a positive integer".to_string());
            }
        },
        KeyCode::Char(c) if c.is_ascii_digit() => {
            id.push(c);
            *error = None;
        }
        _ => {}
    }
    Ok(false)
}

// ─────────────────────────────────────────────────────────────────── drawing ─

fn draw(f: &mut Frame, app: &App) {
    match &app.state {
        AppState::PasswordInput { input, error } => draw_password(f, &app.file_path, input, error),
        AppState::CredentialList { credentials } => {
            draw_list(f, &app.file_path, credentials);
        }
        AppState::AddForm { key, value, focus, error } => {
            draw_list_bg(f);
            draw_add_form(f, key, value, *focus, error);
        }
        AppState::UpdateForm { step, id, key, value, focus, error } => {
            draw_list_bg(f);
            draw_update_form(f, *step, id, key, value, *focus, error);
        }
        AppState::RemoveConfirm { id, error } => {
            draw_list_bg(f);
            draw_remove_form(f, id, error);
        }
        AppState::EnvVars { output } => draw_env_vars(f, output),
    }
}

fn draw_password(f: &mut Frame, file_path: &PathBuf, input: &str, error: &Option<String>) {
    let area = f.area();

    let block = Block::default()
        .title(" LootBox ")
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

    lines.push(Line::from(Span::styled(
        "  Enter → unlock   Esc → quit",
        Style::default().fg(Color::DarkGray),
    )));

    let para = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(para, inner);
}

fn draw_list(f: &mut Frame, file_path: &PathBuf, credentials: &[Credential]) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    // Main block
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
        .column_spacing(2);

    f.render_widget(table, chunks[0]);

    // Bottom bar
    let bar = Paragraph::new(Line::from(vec![
        Span::styled(" [A]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("dd  ", Style::default().fg(Color::White)),
        Span::styled("[U]", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled("pdate  ", Style::default().fg(Color::White)),
        Span::styled("[R]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled("emove  ", Style::default().fg(Color::White)),
        Span::styled("[E]", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("nv  ", Style::default().fg(Color::White)),
        Span::styled("[Q]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Span::styled("uit", Style::default().fg(Color::White)),
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

fn draw_add_form(
    f: &mut Frame,
    key: &str,
    value: &str,
    focus: Field,
    error: &Option<String>,
) {
    let area = centered_rect(60, 50, f.area());
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

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Key:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}_", key), key_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Value: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}_", value), val_style),
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
        "  Tab → switch field   Enter → save   Esc → cancel",
        Style::default().fg(Color::DarkGray),
    )));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_update_form(
    f: &mut Frame,
    step: UpdateStep,
    id: &str,
    key: &str,
    value: &str,
    focus: Field,
    error: &Option<String>,
) {
    let area = centered_rect(60, 55, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Update Credential ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![Line::from("")];

    match step {
        UpdateStep::SelectId => {
            lines.push(Line::from(vec![
                Span::styled("  Select ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}_", id),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
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
        }
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

            lines.push(Line::from(Span::styled(
                format!("  Editing ID: {id}"),
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  New Key:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}_", key), key_style),
            ]));
            lines.push(Line::from(Span::styled(
                "             (blank = keep current)",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  New Value: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}_", value), val_style),
            ]));
            lines.push(Line::from(Span::styled(
                "             (blank = keep current)",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            if let Some(err) = error {
                lines.push(Line::from(Span::styled(
                    format!("  {err}"),
                    Style::default().fg(Color::Red),
                )));
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "  Tab → switch field   Enter → save   Esc → cancel",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_remove_form(f: &mut Frame, id: &str, error: &Option<String>) {
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
            Span::styled("  Select ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}_", id),
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
        "  Enter → remove   Esc → cancel",
        Style::default().fg(Color::DarkGray),
    )));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_env_vars(f: &mut Frame, output: &str) {
    let area = f.area();

    let block = Block::default()
        .title(" Environment Variables — press any key to go back ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = output
        .lines()
        .map(|l| {
            if l.starts_with('#') {
                Line::from(Span::styled(l, Style::default().fg(Color::DarkGray)))
            } else {
                Line::from(Span::styled(l, Style::default().fg(Color::Green)))
            }
        })
        .collect();

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

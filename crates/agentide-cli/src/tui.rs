//! Console renderer and keyboard surface over the shared projection.

use std::io;
use std::time::Duration;

use agentide_core::{Engine, Snapshot};
use agentide_substrate::SubstratePort;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use serde_json::json;

/// Runs an interactive TUI. Every state-changing key invokes a semantic surface intent.
pub fn run(engine: &Engine<SubstratePort>, session_id: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut output = io::stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    let result = loop_run(&mut terminal, engine, session_id);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn loop_run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    engine: &Engine<SubstratePort>,
    session_id: &str,
) -> Result<()> {
    let mut mode = Mode::Normal;
    let mut notice = String::new();
    let mut observation = String::new();
    loop {
        let snapshot = engine.snapshot(session_id)?;
        terminal.draw(|frame| draw(frame, &snapshot, &mode, &notice, &observation))?;
        if !event::poll(Duration::from_millis(500))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match &mut mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('o') => mode = Mode::OpenFile(String::new()),
                KeyCode::Char('d') => {
                    notice = invoke(engine, session_id, "diff_show", json!({}));
                    match engine.call(session_id, "code_changes", json!({})) {
                        Ok(value) => observation = render_observation(&value),
                        Err(error) => notice = error.to_string(),
                    }
                }
                KeyCode::Tab => {
                    if let Some(next) = next_pane(&snapshot) {
                        notice = invoke(engine, session_id, "pane_focus", json!({"pane_id": next}));
                    }
                }
                KeyCode::Char('x') => {
                    if let Some(id) = &snapshot.workbench.focused_pane {
                        notice = invoke(engine, session_id, "pane_close", json!({"pane_id": id}));
                    }
                }
                _ => {}
            },
            Mode::OpenFile(path) => match key.code {
                KeyCode::Esc => mode = Mode::Normal,
                KeyCode::Backspace => {
                    path.pop();
                }
                KeyCode::Char(character) => path.push(character),
                KeyCode::Enter => {
                    let requested = std::mem::take(path);
                    notice = invoke(engine, session_id, "file_open", json!({"path": &requested}));
                    match engine.call(session_id, "code_read", json!({"path": requested})) {
                        Ok(value) => observation = render_observation(&value),
                        Err(error) => notice = error.to_string(),
                    }
                    mode = Mode::Normal;
                }
                _ => {}
            },
        }
    }
}

fn invoke(
    engine: &Engine<SubstratePort>,
    session_id: &str,
    intent: &str,
    input: serde_json::Value,
) -> String {
    match engine.call(session_id, intent, input) {
        Ok(_) => format!("{intent} completed"),
        Err(error) => error.to_string(),
    }
}

fn next_pane(snapshot: &Snapshot) -> Option<String> {
    let panes = &snapshot.workbench.panes;
    if panes.is_empty() {
        return None;
    }
    let current = snapshot
        .workbench
        .focused_pane
        .as_deref()
        .and_then(|id| panes.iter().position(|pane| pane.id == id))
        .unwrap_or(0);
    Some(panes[(current + 1) % panes.len()].id.clone())
}

fn render_observation(value: &serde_json::Value) -> String {
    let rendered = value
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map_or_else(
            || {
                serde_json::to_string_pretty(value)
                    .unwrap_or_else(|_| "unrenderable observation".into())
            },
            ToOwned::to_owned,
        );
    rendered.chars().take(100_000).collect()
}

fn draw(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &Snapshot,
    mode: &Mode,
    notice: &str,
    observation: &str,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(frame.area());
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),
            Constraint::Min(30),
            Constraint::Length(34),
        ])
        .split(rows[1]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " AgentIDE ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " {} · {} · event {}",
                snapshot.objective, snapshot.status, snapshot.cursor
            )),
        ]))
        .block(Block::default().borders(Borders::ALL)),
        rows[0],
    );

    let files = snapshot
        .workbench
        .open_files
        .iter()
        .map(|path| ListItem::new(path.as_str()));
    frame.render_widget(
        List::new(files).block(Block::default().title(" Open files ").borders(Borders::ALL)),
        body[0],
    );

    let focused = snapshot
        .workbench
        .focused_pane
        .as_deref()
        .and_then(|id| snapshot.workbench.panes.iter().find(|pane| pane.id == id));
    let detail = focused.map_or_else(
        || "No pane focused. Press o to open a file or d to show changes.".into(),
        |pane| {
            format!(
                "{}\n\nkind: {}\npath: {}\ncursor: {}:{}\n\n{}",
                pane.title,
                pane.kind,
                pane.path.as_deref().unwrap_or("—"),
                pane.line
                    .map_or_else(|| "—".into(), |line| line.to_string()),
                pane.column
                    .map_or_else(|| "—".into(), |column| column.to_string()),
                observation,
            )
        },
    );
    frame.render_widget(
        Paragraph::new(detail)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(" Focus ").borders(Borders::ALL)),
        body[1],
    );

    let context_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(body[2]);
    let panes = snapshot.workbench.panes.iter().map(|pane| {
        let prefix = if snapshot.workbench.focused_pane.as_deref() == Some(&pane.id) {
            "●"
        } else {
            "○"
        };
        ListItem::new(format!("{prefix} {}  {}", pane.kind, pane.title))
    });
    frame.render_widget(
        List::new(panes).block(
            Block::default()
                .title(" Virtual panes ")
                .borders(Borders::ALL),
        ),
        context_rows[0],
    );
    frame.render_widget(
        Paragraph::new(format!(
            "approvals  {}\nprocesses  {}\nagents     {}\nevidence   {}",
            snapshot.pending_approvals.len(),
            snapshot.processes.len(),
            snapshot.agents.len(),
            snapshot.evidence.len(),
        ))
        .block(
            Block::default()
                .title(" Session context ")
                .borders(Borders::ALL),
        ),
        context_rows[1],
    );

    let footer = match mode {
        Mode::Normal => {
            format!("o open file · d diff · Tab focus · x close pane · q quit\n{notice}")
        }
        Mode::OpenFile(path) => {
            format!("Open workspace-relative file: {path}_\nEnter confirm · Esc cancel")
        }
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().title(" Commands ").borders(Borders::ALL)),
        rows[2],
    );
}

enum Mode {
    Normal,
    OpenFile(String),
}

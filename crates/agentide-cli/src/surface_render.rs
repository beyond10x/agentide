//! Pure Ratatui projection of a validated AgentIDE surface profile.

use agentide_contracts::{SurfaceProfile, SurfaceTheme};
use agentide_core::{Pane, Snapshot};
use agentide_harness::ApprovalRequest;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap};

use crate::surface_ui::{
    InputMode, MainView, Region, SurfaceState, action_disabled, file_candidates, palette_actions,
};

pub(crate) struct RenderState<'a> {
    pub(crate) snapshot: &'a Snapshot,
    pub(crate) profile: &'a SurfaceProfile,
    pub(crate) surface: &'a SurfaceState,
    pub(crate) model: &'a str,
    pub(crate) harness_status: &'a str,
    pub(crate) transcript: &'a str,
    pub(crate) activity: &'a [String],
    pub(crate) observation: &'a str,
    pub(crate) notice: &'a str,
    pub(crate) turn: u64,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) reasoning: bool,
    pub(crate) busy: bool,
    pub(crate) approval: Option<&'a ApprovalRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorMode {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    background: Color,
    panel: Color,
    raised: Color,
    line: Color,
    muted: Color,
    text: Color,
    accent: Color,
    warning: Color,
    danger: Color,
    success: Color,
}

impl Theme {
    fn from_profile(profile: &SurfaceTheme, mode: ColorMode) -> Self {
        let color = |role: &str| match mode {
            ColorMode::TrueColor => profile
                .truecolor
                .get(role)
                .and_then(|value| rgb(value))
                .unwrap_or(Color::Reset),
            ColorMode::Ansi256 => profile
                .color_256
                .get(role)
                .copied()
                .map_or(Color::Reset, Color::Indexed),
            ColorMode::Ansi16 => profile
                .color_16
                .get(role)
                .map_or(Color::Reset, |value| ansi(value)),
            ColorMode::Mono => Color::Reset,
        };
        Self {
            background: color("background"),
            panel: color("panel"),
            raised: color("raised"),
            line: color("line"),
            muted: color("muted"),
            text: color("text"),
            accent: color("accent"),
            warning: color("warning"),
            danger: color("danger"),
            success: color("success"),
        }
    }
}

pub(crate) fn detect_color_mode() -> ColorMode {
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorMode::Mono;
    }
    let color_term = std::env::var("COLORTERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if color_term.contains("truecolor") || color_term.contains("24bit") {
        return ColorMode::TrueColor;
    }
    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if term.contains("256color") {
        ColorMode::Ansi256
    } else {
        ColorMode::Ansi16
    }
}

pub(crate) fn draw(frame: &mut Frame<'_>, state: &RenderState<'_>) {
    draw_with_mode(frame, state, detect_color_mode());
}

fn draw_with_mode(frame: &mut Frame<'_>, state: &RenderState<'_>, color_mode: ColorMode) {
    let theme = Theme::from_profile(&state.profile.theme, color_mode);
    let ascii = color_mode == ColorMode::Mono
        || std::env::var("TERM").is_ok_and(|term| term.eq_ignore_ascii_case("dumb"));
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.background).fg(theme.text)),
        frame.area(),
    );
    if frame.area().height < 20 {
        draw_too_small(frame, state, theme);
        if let Some(approval) = state.approval {
            draw_compact_approval(frame, approval, theme);
        }
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(frame.area());
    draw_header(frame, rows[0], state, theme, ascii);
    draw_tabs(frame, rows[1], state, theme, ascii);

    match state.surface.viewport.as_str() {
        "wide" => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(26),
                    Constraint::Length(1),
                    Constraint::Min(72),
                    Constraint::Length(1),
                    Constraint::Length(34),
                ])
                .split(rows[2]);
            draw_explorer(frame, columns[0], state, theme, ascii);
            separator(frame, columns[1], theme);
            draw_canvas(frame, columns[2], state, theme);
            separator(frame, columns[3], theme);
            draw_right_rail(frame, columns[4], state, theme, ascii);
        }
        "standard" => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(26),
                    Constraint::Length(1),
                    Constraint::Min(60),
                ])
                .split(rows[2]);
            draw_explorer(frame, columns[0], state, theme, ascii);
            separator(frame, columns[1], theme);
            draw_canvas(frame, columns[2], state, theme);
        }
        _ => draw_canvas(frame, rows[2], state, theme),
    }
    draw_composer(frame, rows[3], state, theme);
    draw_status(frame, rows[4], state, theme, ascii);
    draw_region_overlay(frame, rows[2], state, theme, ascii);
    draw_mode_overlay(frame, state, theme);
    if let Some(approval) = state.approval {
        draw_approval(frame, approval, state, theme);
    }
}

fn draw_header(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let brand = glyph(state.profile, "brand", ascii);
    let busy = if state.busy {
        format!(" {} busy", glyph(state.profile, "busy", ascii))
    } else {
        String::new()
    };
    let right = format!(
        "  {} · event {} · {}{} ",
        state.model, state.snapshot.cursor, state.harness_status, busy
    );
    let available = usize::from(area.width).saturating_sub(right.chars().count() + 13);
    let objective: String = state.snapshot.objective.chars().take(available).collect();
    let line = Line::from(vec![
        Span::styled(
            format!(" {brand} AgentIDE "),
            Style::default()
                .fg(theme.background)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {objective}"), Style::default().fg(theme.text)),
        Span::styled(right, Style::default().fg(theme.muted)),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Left)
            .style(Style::default().bg(theme.panel))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.line)),
            ),
        area,
    );
}

fn draw_tabs(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let mut spans = vec![Span::styled(
        match state.surface.view {
            MainView::Agent => " AGENT ",
            MainView::Workbench => " WORKBENCH ",
        },
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )];
    for pane in &state.snapshot.workbench.panes {
        let focused = state.snapshot.workbench.focused_pane.as_deref() == Some(&pane.id);
        let marker = if focused { "▔" } else { " " };
        spans.push(Span::styled(
            format!(
                " {marker}{} {} ",
                glyph(state.profile, &pane.kind, ascii),
                pane.title
            ),
            if focused {
                Style::default()
                    .fg(theme.text)
                    .bg(theme.raised)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted).bg(theme.panel)
            },
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::default().bg(theme.panel))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.line)),
            ),
        area,
    );
}

fn draw_explorer(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(area);
    let active = state.surface.focus == Region::Explorer;
    let files: Vec<_> = state
        .snapshot
        .workbench
        .open_files
        .iter()
        .skip(usize::from(state.surface.scroll(Region::Explorer).vertical))
        .map(|path| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", glyph(state.profile, "editor", ascii)),
                    Style::default().fg(theme.warning),
                ),
                Span::styled(path.as_str(), Style::default().fg(theme.text)),
            ]))
        })
        .collect();
    let files = if files.is_empty() {
        vec![ListItem::new(Span::styled(
            "No open files\nCtrl+P to open",
            Style::default().fg(theme.muted),
        ))]
    } else {
        files
    };
    frame.render_widget(
        List::new(files).block(section_block(" OPEN FILES ", active, theme)),
        regions[0],
    );
    let panes: Vec<_> = state
        .snapshot
        .workbench
        .panes
        .iter()
        .skip(usize::from(state.surface.scroll(Region::Explorer).vertical))
        .map(|pane| {
            let focused = state.snapshot.workbench.focused_pane.as_deref() == Some(&pane.id);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(
                        "{} ",
                        glyph(
                            state.profile,
                            if focused { "active" } else { "inactive" },
                            ascii
                        )
                    ),
                    Style::default().fg(if focused { theme.accent } else { theme.muted }),
                ),
                Span::styled(
                    format!("{}  {}", pane.kind, pane.title),
                    Style::default().fg(if focused { theme.text } else { theme.muted }),
                ),
            ]))
        })
        .collect();
    frame.render_widget(
        List::new(panes).block(section_block(" VIRTUAL PANES ", active, theme)),
        regions[1],
    );
}

fn draw_canvas(frame: &mut Frame<'_>, area: Rect, state: &RenderState<'_>, theme: Theme) {
    let active = state.surface.focus == Region::Canvas;
    match state.surface.view {
        MainView::Agent => {
            let lines = transcript_lines(state.transcript, theme);
            frame.render_widget(
                Paragraph::new(lines)
                    .scroll((state.surface.scroll(Region::Canvas).vertical, 0))
                    .wrap(Wrap { trim: false })
                    .block(section_block(" AGENT TRANSCRIPT ", active, theme)),
                area,
            );
        }
        MainView::Workbench => {
            let pane = focused_pane(state.snapshot);
            let title = pane.map_or_else(
                || " WORKBENCH ".into(),
                |pane| {
                    let location = pane.line.map_or_else(String::new, |line| {
                        format!(" · {line}:{}", pane.column.unwrap_or(1))
                    });
                    format!(
                        " {} · {}{location} ",
                        pane.kind.to_ascii_uppercase(),
                        pane.title
                    )
                },
            );
            let scroll = state.surface.scroll(Region::Canvas);
            if state.observation.is_empty() {
                frame.render_widget(
                    Paragraph::new(vec![
                        Line::from(""),
                        Line::styled(
                            "No observation loaded",
                            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                        ),
                        Line::styled(
                            "Ctrl+P opens a file · D shows workspace changes",
                            Style::default().fg(theme.muted),
                        ),
                    ])
                    .alignment(Alignment::Center)
                    .block(section_block(&title, active, theme)),
                    area,
                );
            } else if pane.is_some_and(|pane| pane.kind == "diff") {
                frame.render_widget(
                    Paragraph::new(diff_lines(state.observation, theme))
                        .scroll((scroll.vertical, scroll.horizontal))
                        .block(section_block(&title, active, theme)),
                    area,
                );
            } else {
                frame.render_widget(
                    Paragraph::new(editor_lines(state.observation, pane, theme))
                        .scroll((scroll.vertical, scroll.horizontal))
                        .block(section_block(&title, active, theme)),
                    area,
                );
            }
        }
    }
}

fn draw_right_rail(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);
    draw_activity(frame, regions[0], state, theme, ascii);
    draw_context(frame, regions[1], state, theme);
}

fn draw_activity(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let rows = state
        .activity
        .iter()
        .rev()
        .take(40)
        .rev()
        .skip(usize::from(state.surface.scroll(Region::Activity).vertical))
        .map(|line| {
            let (glyph_name, color) = if line.starts_with('!') || line.starts_with('×') {
                ("failure", theme.danger)
            } else if line.starts_with('?') {
                ("approvals", theme.warning)
            } else if line.starts_with('✓') {
                ("success", theme.success)
            } else {
                ("inactive", theme.muted)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", glyph(state.profile, glyph_name, ascii)),
                    Style::default().fg(color),
                ),
                Span::styled(strip_activity_marker(line), Style::default().fg(theme.text)),
            ]))
        });
    frame.render_widget(
        List::new(rows).block(section_block(
            " HARNESS ACTIVITY ",
            state.surface.focus == Region::Activity,
            theme,
        )),
        area,
    );
}

fn draw_context(frame: &mut Frame<'_>, area: Rect, state: &RenderState<'_>, theme: Theme) {
    let reasoning = if state.reasoning { "streaming" } else { "idle" };
    let lines = vec![
        metric("turn", state.turn.to_string(), theme),
        metric(
            "tokens",
            format!("{} in / {} out", state.input_tokens, state.output_tokens),
            theme,
        ),
        metric(
            "approvals",
            state.snapshot.pending_approvals.len().to_string(),
            theme,
        ),
        metric(
            "processes",
            state.snapshot.processes.len().to_string(),
            theme,
        ),
        metric("agents", state.snapshot.agents.len().to_string(), theme),
        metric("evidence", state.snapshot.evidence.len().to_string(), theme),
        metric("reasoning", reasoning.into(), theme),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((state.surface.scroll(Region::Context).vertical, 0))
            .block(section_block(
                " SESSION CONTEXT ",
                state.surface.focus == Region::Context,
                theme,
            )),
        area,
    );
}

fn draw_composer(frame: &mut Frame<'_>, area: Rect, state: &RenderState<'_>, theme: Theme) {
    let (label, text, color) = match &state.surface.mode {
        InputMode::Prompt(input) => (" PROMPT AGENT ", format!("> {input}_"), theme.accent),
        InputMode::QuickOpen { query, .. } => {
            (" QUICK OPEN · PATH ", format!("> {query}_"), theme.accent)
        }
        InputMode::Palette { query, .. } => {
            (" COMMAND PALETTE ", format!("> {query}_"), theme.accent)
        }
        InputMode::Normal | InputMode::Help => (" STATUS ", state.notice.into(), theme.text),
    };
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(color).bg(theme.panel))
            .block(
                Block::default()
                    .title(Span::styled(label, Style::default().fg(theme.muted)))
                    .borders(Borders::TOP)
                    .border_style(
                        Style::default().fg(if state.surface.focus == Region::Composer {
                            theme.accent
                        } else {
                            theme.line
                        }),
                    )
                    .padding(Padding::horizontal(1)),
            ),
        area,
    );
}

fn draw_status(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let keys = if state.approval.is_some() {
        "↑↓←→ inspect   Y approve exact plan   N/Esc deny"
    } else {
        "Ctrl+K commands   Ctrl+P quick open   Tab region   [ ] pane   I prompt   ? help"
    };
    let boundary = format!(
        " {} Substrate boundary ",
        glyph(state.profile, "active", ascii)
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                boundary,
                Style::default().fg(theme.background).bg(theme.accent),
            ),
            Span::styled(
                format!(" {keys}"),
                Style::default().fg(theme.muted).bg(theme.raised),
            ),
        ]))
        .style(Style::default().bg(theme.raised)),
        area,
    );
}

fn draw_region_overlay(
    frame: &mut Frame<'_>,
    body: Rect,
    state: &RenderState<'_>,
    theme: Theme,
    ascii: bool,
) {
    let Some(region) = state.surface.overlay else {
        return;
    };
    let width = if region == Region::Explorer { 34 } else { 44 };
    let area = if region == Region::Explorer {
        Rect::new(body.x, body.y, width.min(body.width), body.height)
    } else {
        Rect::new(
            body.right().saturating_sub(width.min(body.width)),
            body.y,
            width.min(body.width),
            body.height,
        )
    };
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .style(Style::default().bg(theme.panel))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent)),
        area,
    );
    let inner = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 1,
    });
    match region {
        Region::Explorer => draw_explorer(frame, inner, state, theme, ascii),
        Region::Activity => draw_activity(frame, inner, state, theme, ascii),
        Region::Context => draw_context(frame, inner, state, theme),
        Region::Canvas | Region::Composer => {}
    }
}

fn draw_mode_overlay(frame: &mut Frame<'_>, state: &RenderState<'_>, theme: Theme) {
    match &state.surface.mode {
        InputMode::Palette { query, selected } => {
            let actions = palette_actions(state.profile, query);
            let rows = actions.iter().enumerate().map(|(index, action)| {
                let disabled = action_disabled(Some(action), state.busy, state.snapshot);
                let marker = if index == *selected { ">" } else { " " };
                let style = if index == *selected {
                    Style::default().fg(theme.background).bg(theme.accent)
                } else if disabled.is_some() {
                    Style::default().fg(theme.muted)
                } else {
                    Style::default().fg(theme.text)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {marker} {:28}", action.label), style),
                    Span::styled(
                        disabled.unwrap_or(action.id.as_str()),
                        Style::default().fg(theme.muted),
                    ),
                ]))
            });
            modal(
                frame,
                " COMMAND PALETTE · type to filter ",
                List::new(rows),
                68,
                18,
                theme,
            );
        }
        InputMode::QuickOpen { query, selected } => {
            let files = file_candidates(state.snapshot, query);
            let rows = if files.is_empty() {
                vec![ListItem::new(Line::styled(
                    " Enter accepts the typed workspace-relative path ",
                    Style::default().fg(theme.muted),
                ))]
            } else {
                files
                    .iter()
                    .enumerate()
                    .map(|(index, path)| {
                        ListItem::new(Line::styled(
                            format!(" {} {path}", if index == *selected { ">" } else { " " }),
                            if index == *selected {
                                Style::default().fg(theme.background).bg(theme.accent)
                            } else {
                                Style::default().fg(theme.text)
                            },
                        ))
                    })
                    .collect()
            };
            modal(frame, " QUICK OPEN ", List::new(rows), 68, 16, theme);
        }
        InputMode::Help => {
            let rows = state
                .profile
                .mode("normal")
                .into_iter()
                .flat_map(|mode| &mode.bindings)
                .filter_map(|binding| {
                    state.profile.action(&binding.action).map(|action| {
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!(" {:12}", binding.key),
                                Style::default().fg(theme.accent),
                            ),
                            Span::styled(action.label.as_str(), Style::default().fg(theme.text)),
                        ]))
                    })
                });
            modal(
                frame,
                " KEYBOARD HELP · Esc closes ",
                List::new(rows),
                58,
                22,
                theme,
            );
        }
        InputMode::Normal | InputMode::Prompt(_) => {}
    }
}

fn draw_approval(
    frame: &mut Frame<'_>,
    approval: &ApprovalRequest,
    state: &RenderState<'_>,
    theme: Theme,
) {
    let area = centered(frame.area(), 88, 30);
    frame.render_widget(Clear, area);
    let effects = approval
        .envelope
        .effects
        .iter()
        .map(|effect| format!("{effect:?}").to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(", ");
    let access = approval
        .envelope
        .access
        .iter()
        .map(|access| format!("{access:?}").to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(", ");
    let subjects = approval
        .subjects
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let arguments = serde_json::to_string_pretty(&approval.arguments)
        .unwrap_or_else(|_| "<arguments could not be rendered>".into());
    let detail = vec![
        Line::from(vec![
            Span::styled("INTENT      ", Style::default().fg(theme.muted)),
            Span::styled(
                &approval.intent,
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(format!("COMMAND     {}", approval.command)),
        Line::from(format!("EFFECTS     {effects}")),
        Line::from(format!("RISK        {:?}", approval.envelope.risk).to_ascii_lowercase()),
        Line::from(format!(
            "ACCESS      {}",
            if access.is_empty() { "none" } else { &access }
        )),
        Line::from(format!("IDEMPOTENCY {:?}", approval.envelope.idempotency).to_ascii_lowercase()),
        Line::from(format!(
            "SUBJECTS    {}",
            if subjects.is_empty() {
                "none"
            } else {
                &subjects
            }
        )),
        Line::from(""),
        Line::from(format!("driver      {}", approval.plan.driver)),
        Line::from(format!("operation   {}", approval.plan.operation)),
        Line::from(format!("plan        {}", approval.plan.digest)),
        Line::from(format!("input       {}", approval.plan.input_sha256)),
        Line::from(format!("binding     {}", approval.plan.binding_sha256)),
        Line::from(""),
        Line::styled("EXACT SEMANTIC ARGUMENTS", Style::default().fg(theme.muted)),
        Line::from(arguments),
    ];
    let scroll = state.surface.approval_scroll();
    frame.render_widget(
        Paragraph::new(detail)
            .scroll((scroll.vertical, scroll.horizontal))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme.text).bg(theme.panel))
            .block(
                Block::default()
                    .title(Span::styled(
                        " APPROVAL REQUIRED · exact plan ",
                        Style::default()
                            .fg(theme.warning)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .title_bottom(Line::from(vec![
                        Span::styled(
                            " Y approve ",
                            Style::default()
                                .fg(theme.background)
                                .bg(theme.warning)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("  "),
                        Span::styled(
                            " N / Esc deny ",
                            Style::default()
                                .fg(theme.text)
                                .bg(theme.danger)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.warning))
                    .padding(Padding::uniform(1)),
            ),
        area,
    );
}

fn draw_too_small(frame: &mut Frame<'_>, state: &RenderState<'_>, theme: Theme) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "AgentIDE",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("Terminal too small for the workbench."),
            Line::styled(
                format!(
                    "{}×{} · minimum height 20",
                    frame.area().width,
                    frame.area().height
                ),
                Style::default().fg(theme.muted),
            ),
            Line::from("Press q to quit. Resize to continue."),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().padding(Padding::uniform(1))),
        frame.area(),
    );
    if state.busy {
        frame.render_widget(
            Paragraph::new("Harness operation remains active")
                .style(Style::default().fg(theme.warning)),
            Rect::new(
                frame.area().x,
                frame.area().bottom().saturating_sub(1),
                frame.area().width,
                1,
            ),
        );
    }
}

fn draw_compact_approval(frame: &mut Frame<'_>, approval: &ApprovalRequest, theme: Theme) {
    let area = centered(frame.area(), 70, 9);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "APPROVAL REQUIRED",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(
                format!("{} · {:?}", approval.intent, approval.envelope.risk).to_ascii_lowercase(),
            ),
            Line::from(format!("plan {}", short(&approval.plan.digest))),
            Line::from("Y approve exact plan · N/Esc deny"),
        ])
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.warning)),
        ),
        area,
    );
}

fn modal<W: ratatui::widgets::Widget>(
    frame: &mut Frame<'_>,
    title: &str,
    widget: W,
    width: u16,
    height: u16,
    theme: Theme,
) {
    let area = centered(frame.area(), width, height);
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.panel))
        .padding(Padding::uniform(1));
    frame.render_widget(widget, block.inner(area));
    frame.render_widget(block, area);
}

fn section_block(title: &str, active: bool, theme: Theme) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(if active { theme.accent } else { theme.muted })
                .add_modifier(if active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(if active { theme.accent } else { theme.line }))
        .style(Style::default().bg(theme.background))
        .padding(Padding::horizontal(1))
}

fn separator(frame: &mut Frame<'_>, area: Rect, theme: Theme) {
    frame.render_widget(
        Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.line)),
        area,
    );
}

fn transcript_lines(text: &str, theme: Theme) -> Vec<Line<'_>> {
    text.lines()
        .map(|line| {
            if let Some(body) = line.strip_prefix("You › ") {
                Line::from(vec![
                    Span::styled(
                        "YOU    ",
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(body, Style::default().fg(theme.text)),
                ])
            } else if let Some(body) = line.strip_prefix("Agent › ") {
                Line::from(vec![
                    Span::styled(
                        "AGENT  ",
                        Style::default()
                            .fg(theme.warning)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(body, Style::default().fg(theme.text)),
                ])
            } else if line.starts_with('!') {
                Line::styled(line, Style::default().fg(theme.danger))
            } else {
                Line::styled(line, Style::default().fg(theme.text))
            }
        })
        .collect()
}

fn editor_lines<'a>(text: &'a str, pane: Option<&Pane>, theme: Theme) -> Vec<Line<'a>> {
    let count = text.lines().count().max(1);
    let digits = count.to_string().len();
    let current = pane.and_then(|pane| pane.line).unwrap_or(0);
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            let number = index + 1;
            let active = u64::try_from(number).is_ok_and(|number| number == current);
            Line::from(vec![
                Span::styled(
                    format!("{number:>digits$} "),
                    Style::default().fg(if active { theme.accent } else { theme.muted }),
                ),
                Span::styled(
                    line,
                    Style::default().fg(theme.text).bg(if active {
                        theme.raised
                    } else {
                        theme.background
                    }),
                ),
            ])
        })
        .collect()
}

fn diff_lines(text: &str, theme: Theme) -> Vec<Line<'_>> {
    text.lines()
        .map(|line| {
            let style = if line.starts_with("+++") || line.starts_with("---") {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if line.starts_with("@@") {
                Style::default().fg(theme.warning)
            } else if line.starts_with('+') {
                Style::default().fg(theme.success)
            } else if line.starts_with('-') {
                Style::default().fg(theme.danger)
            } else {
                Style::default().fg(theme.text)
            };
            Line::styled(line, style)
        })
        .collect()
}

fn metric(label: &str, value: String, theme: Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<11}"), Style::default().fg(theme.muted)),
        Span::styled(value, Style::default().fg(theme.text)),
    ])
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2)).max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn focused_pane(snapshot: &Snapshot) -> Option<&Pane> {
    snapshot
        .workbench
        .focused_pane
        .as_deref()
        .and_then(|id| snapshot.workbench.panes.iter().find(|pane| pane.id == id))
}

fn strip_activity_marker(line: &str) -> &str {
    line.strip_prefix(|character: char| matches!(character, '!' | '?' | '✓' | '×' | '→'))
        .map_or(line, str::trim_start)
}

fn glyph<'a>(profile: &'a SurfaceProfile, name: &str, ascii: bool) -> &'a str {
    profile.theme.glyphs.get(name).map_or(
        "□",
        |glyph| {
            if ascii { &glyph.ascii } else { &glyph.unicode }
        },
    )
}

fn rgb(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#')?;
    Some(Color::Rgb(
        u8::from_str_radix(value.get(0..2)?, 16).ok()?,
        u8::from_str_radix(value.get(2..4)?, 16).ok()?,
        u8::from_str_radix(value.get(4..6)?, 16).ok()?,
    ))
}

fn ansi(value: &str) -> Color {
    match value {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::Gray,
        "bright_black" => Color::DarkGray,
        "bright_red" => Color::LightRed,
        "bright_green" => Color::LightGreen,
        "bright_yellow" => Color::LightYellow,
        "bright_blue" => Color::LightBlue,
        "bright_magenta" => Color::LightMagenta,
        "bright_cyan" => Color::LightCyan,
        "bright_white" => Color::White,
        _ => Color::Reset,
    }
}

fn short(value: &str) -> &str {
    value.get(..12).unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentide_core::{Pane, Workbench};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn snapshot(kind: &str) -> Snapshot {
        Snapshot {
            format: "agentide.snapshot/1".into(),
            session_id: "session".into(),
            objective: "Make the workbench legible".into(),
            status: "active".into(),
            cursor: 7,
            workbench: Workbench {
                panes: vec![Pane {
                    id: "pane-1".into(),
                    kind: kind.into(),
                    title: "src/main.rs".into(),
                    path: Some("src/main.rs".into()),
                    line: Some(2),
                    column: Some(4),
                }],
                focused_pane: Some("pane-1".into()),
                open_files: vec!["src/main.rs".into()],
            },
            pending_approvals: Vec::new(),
            processes: Vec::new(),
            agents: Vec::new(),
            evidence: Vec::new(),
            last_result: None,
        }
    }

    fn rendered(width: u16, height: u16, kind: &str, observation: &str) -> String {
        let profile = SurfaceProfile::embedded().expect("profile");
        let snapshot = snapshot(kind);
        let mut surface = SurfaceState::default();
        surface.view = MainView::Workbench;
        surface.reduce(
            crate::surface_ui::UiEvent::Resize {
                columns: width,
                rows: height,
            },
            &profile,
            &snapshot,
            false,
            false,
        );
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
        terminal
            .draw(|frame| {
                draw_with_mode(
                    frame,
                    &RenderState {
                        snapshot: &snapshot,
                        profile: &profile,
                        surface: &surface,
                        model: "test-model",
                        harness_status: "ready",
                        transcript: "You › inspect\nAgent › done",
                        activity: &["✓ tool completed".into()],
                        observation,
                        notice: "ready",
                        turn: 1,
                        input_tokens: 10,
                        output_tokens: 3,
                        reasoning: false,
                        busy: false,
                        approval: None,
                    },
                    ColorMode::Mono,
                );
            })
            .expect("draw");
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn adaptive_sizes_render_without_clipping_or_panics() {
        for (width, height, expected) in [
            (80, 24, "Ctrl+K commands"),
            (120, 32, "OPEN FILES"),
            (180, 50, "HARNESS ACTIVITY"),
        ] {
            let output = rendered(width, height, "editor", "fn main() {}\nlet answer = 42;");
            assert!(output.contains("AgentIDE"));
            assert!(
                output.contains(expected),
                "missing {expected} at {width}x{height}"
            );
        }
    }

    #[test]
    fn editor_and_diff_keep_non_color_semantics() {
        let editor = rendered(120, 32, "editor", "one\ntwo\nthree");
        assert!(editor.contains("1 one"));
        assert!(editor.contains("2 two"));
        let diff = rendered(120, 32, "diff", "@@ -1 +1 @@\n-old\n+new");
        assert!(diff.contains("@@ -1 +1 @@"));
        assert!(diff.contains("-old"));
        assert!(diff.contains("+new"));
    }

    #[test]
    fn too_small_view_remains_safe_and_exitable() {
        let output = rendered(60, 12, "editor", "text");
        assert!(output.contains("Terminal too small"));
        assert!(output.contains("Press q to quit"));
    }

    #[test]
    fn every_color_mode_has_a_complete_theme() {
        let profile = SurfaceProfile::embedded().expect("profile");
        for mode in [
            ColorMode::TrueColor,
            ColorMode::Ansi256,
            ColorMode::Ansi16,
            ColorMode::Mono,
        ] {
            let theme = Theme::from_profile(&profile.theme, mode);
            if mode == ColorMode::Mono {
                assert_eq!(theme.accent, Color::Reset);
            } else {
                assert_ne!(theme.accent, Color::Reset);
            }
        }
    }
}

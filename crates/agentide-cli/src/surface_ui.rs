//! Deterministic interaction reducer for the console workbench.

use std::collections::BTreeMap;

use agentide_contracts::{ActionAvailability, SurfaceAction, SurfaceProfile};
use agentide_core::Snapshot;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MainView {
    Agent,
    Workbench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Region {
    Explorer,
    Canvas,
    Activity,
    Context,
    Composer,
}

impl Region {
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::Explorer => "explorer",
            Self::Canvas => "canvas",
            Self::Activity => "activity",
            Self::Context => "context",
            Self::Composer => "composer",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "explorer" => Some(Self::Explorer),
            "canvas" => Some(Self::Canvas),
            "activity" => Some(Self::Activity),
            "context" => Some(Self::Context),
            "composer" => Some(Self::Composer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    Prompt(String),
    QuickOpen { query: String, selected: usize },
    Palette { query: String, selected: usize },
    Help,
}

impl InputMode {
    pub(crate) const fn id(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Prompt(_) => "prompt",
            Self::QuickOpen { .. } => "quick_open",
            Self::Palette { .. } => "palette",
            Self::Help => "help",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Scroll {
    pub(crate) vertical: u16,
    pub(crate) horizontal: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfaceState {
    pub(crate) mode: InputMode,
    pub(crate) view: MainView,
    pub(crate) focus: Region,
    pub(crate) overlay: Option<Region>,
    pub(crate) viewport: String,
    scroll: BTreeMap<Region, Scroll>,
    approval_scroll: Scroll,
}

impl Default for SurfaceState {
    fn default() -> Self {
        Self {
            mode: InputMode::Normal,
            view: MainView::Agent,
            focus: Region::Canvas,
            overlay: None,
            viewport: "compact".into(),
            scroll: BTreeMap::new(),
            approval_scroll: Scroll::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UiEffect {
    Prompt(String),
    Intent { name: String, input: Value },
    Refresh,
    Approval(bool),
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UiEvent {
    Key {
        chord: String,
        character: Option<char>,
    },
    Resize {
        columns: u16,
        rows: u16,
    },
}

impl SurfaceState {
    pub(crate) fn scroll(&self, region: Region) -> Scroll {
        self.scroll.get(&region).copied().unwrap_or_default()
    }

    pub(crate) const fn approval_scroll(&self) -> Scroll {
        self.approval_scroll
    }

    pub(crate) fn reset_approval_scroll(&mut self) {
        self.approval_scroll = Scroll::default();
    }

    pub(crate) fn reduce(
        &mut self,
        event: UiEvent,
        profile: &SurfaceProfile,
        snapshot: &Snapshot,
        busy: bool,
        approval: bool,
    ) -> Vec<UiEffect> {
        if let UiEvent::Resize { columns, rows } = event {
            self.resize(profile, columns, rows);
            return Vec::new();
        }
        let UiEvent::Key { chord, character } = event else {
            unreachable!();
        };
        let mode = if approval { "approval" } else { self.mode.id() };
        if let Some(action) = binding(profile, mode, &chord) {
            return self.apply_action(action, profile, snapshot, busy, approval);
        }
        if approval {
            return Vec::new();
        }
        match &mut self.mode {
            InputMode::Prompt(input) => {
                edit(input, &chord, character);
            }
            InputMode::QuickOpen { query, selected } | InputMode::Palette { query, selected } => {
                if edit(query, &chord, character) {
                    *selected = 0;
                }
            }
            InputMode::Normal | InputMode::Help => {}
        }
        Vec::new()
    }

    fn apply_action(
        &mut self,
        action_id: &str,
        profile: &SurfaceProfile,
        snapshot: &Snapshot,
        busy: bool,
        approval: bool,
    ) -> Vec<UiEffect> {
        if approval {
            return match action_id {
                "approve" => vec![UiEffect::Approval(true)],
                "deny" => vec![UiEffect::Approval(false)],
                "scroll_up" => {
                    self.approval_scroll.vertical = self.approval_scroll.vertical.saturating_sub(3);
                    Vec::new()
                }
                "scroll_down" => {
                    self.approval_scroll.vertical = self.approval_scroll.vertical.saturating_add(3);
                    Vec::new()
                }
                "scroll_left" => {
                    self.approval_scroll.horizontal =
                        self.approval_scroll.horizontal.saturating_sub(4);
                    Vec::new()
                }
                "scroll_right" => {
                    self.approval_scroll.horizontal =
                        self.approval_scroll.horizontal.saturating_add(4);
                    Vec::new()
                }
                _ => Vec::new(),
            };
        }

        if action_disabled(profile.action(action_id), busy, snapshot).is_some() {
            return Vec::new();
        }
        match action_id {
            "command_palette" => {
                self.mode = InputMode::Palette {
                    query: String::new(),
                    selected: 0,
                };
                Vec::new()
            }
            "quick_open" => {
                self.mode = InputMode::QuickOpen {
                    query: String::new(),
                    selected: 0,
                };
                Vec::new()
            }
            "prompt" => {
                self.mode = InputMode::Prompt(String::new());
                self.focus = Region::Composer;
                Vec::new()
            }
            "show_agent" => {
                self.view = MainView::Agent;
                self.focus = Region::Canvas;
                self.overlay = None;
                Vec::new()
            }
            "show_workbench" => {
                self.view = MainView::Workbench;
                self.focus = Region::Canvas;
                self.overlay = None;
                Vec::new()
            }
            "show_diff" => {
                self.view = MainView::Workbench;
                self.focus = Region::Canvas;
                self.overlay = None;
                vec![
                    UiEffect::Intent {
                        name: "diff_show".into(),
                        input: json!({}),
                    },
                    UiEffect::Intent {
                        name: "code_changes".into(),
                        input: json!({}),
                    },
                ]
            }
            "close_pane" => snapshot
                .workbench
                .focused_pane
                .as_ref()
                .map(|pane_id| {
                    vec![UiEffect::Intent {
                        name: "pane_close".into(),
                        input: json!({"pane_id": pane_id}),
                    }]
                })
                .unwrap_or_default(),
            "previous_pane" => pane_at_offset(snapshot, -1)
                .as_deref()
                .map(focus_effect)
                .into_iter()
                .collect(),
            "next_pane" => pane_at_offset(snapshot, 1)
                .as_deref()
                .map(focus_effect)
                .into_iter()
                .collect(),
            "refresh" => vec![UiEffect::Refresh],
            "focus_next" => {
                self.move_focus(profile, 1);
                Vec::new()
            }
            "focus_previous" => {
                self.move_focus(profile, -1);
                Vec::new()
            }
            "show_explorer" => {
                self.show_region(profile, Region::Explorer);
                Vec::new()
            }
            "show_activity" => {
                self.show_region(profile, Region::Activity);
                Vec::new()
            }
            "show_context" => {
                self.show_region(profile, Region::Context);
                Vec::new()
            }
            "help" => {
                self.mode = InputMode::Help;
                Vec::new()
            }
            "close_overlay" => {
                self.overlay = None;
                self.focus = Region::Canvas;
                Vec::new()
            }
            "quit" => vec![UiEffect::Quit],
            "cancel_input" => {
                self.mode = InputMode::Normal;
                self.focus = Region::Canvas;
                Vec::new()
            }
            "submit_input" => self.submit(snapshot),
            "palette_previous" => {
                self.move_selection(profile, snapshot, busy, -1);
                Vec::new()
            }
            "palette_next" => {
                self.move_selection(profile, snapshot, busy, 1);
                Vec::new()
            }
            "palette_run" => self.run_palette(profile, snapshot, busy),
            "scroll_up" => {
                let scroll = self.scroll_mut();
                scroll.vertical = scroll.vertical.saturating_sub(3);
                Vec::new()
            }
            "scroll_down" => {
                let scroll = self.scroll_mut();
                scroll.vertical = scroll.vertical.saturating_add(3);
                Vec::new()
            }
            "scroll_left" => {
                let scroll = self.scroll_mut();
                scroll.horizontal = scroll.horizontal.saturating_sub(4);
                Vec::new()
            }
            "scroll_right" => {
                let scroll = self.scroll_mut();
                scroll.horizontal = scroll.horizontal.saturating_add(4);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn resize(&mut self, profile: &SurfaceProfile, columns: u16, rows: u16) {
        let viewport = profile.viewport(columns, rows);
        self.viewport.clone_from(&viewport.id);
        if self.overlay.is_none()
            && !viewport
                .visible_regions
                .iter()
                .any(|id| id == self.focus.id())
        {
            self.focus = Region::from_id(&viewport.default_focus).unwrap_or(Region::Canvas);
        }
        if viewport.visible_regions.iter().any(|id| {
            self.overlay
                .is_some_and(|overlay| id.as_str() == overlay.id())
        }) {
            self.overlay = None;
        }
    }

    fn show_region(&mut self, profile: &SurfaceProfile, region: Region) {
        self.focus = region;
        let visible = profile
            .viewports
            .iter()
            .find(|viewport| viewport.id == self.viewport)
            .is_some_and(|viewport| viewport.visible_regions.iter().any(|id| id == region.id()));
        self.overlay = (!visible).then_some(region);
        self.mode = InputMode::Normal;
    }

    fn move_focus(&mut self, profile: &SurfaceProfile, offset: isize) {
        self.overlay = None;
        let Some(viewport) = profile
            .viewports
            .iter()
            .find(|viewport| viewport.id == self.viewport)
        else {
            self.focus = Region::Canvas;
            return;
        };
        let focusable: Vec<_> = viewport
            .visible_regions
            .iter()
            .filter(|id| profile.region(id).is_some_and(|region| region.focusable))
            .filter_map(|id| Region::from_id(id))
            .collect();
        if focusable.is_empty() {
            self.focus = Region::Canvas;
            return;
        }
        let current = focusable
            .iter()
            .position(|region| *region == self.focus)
            .unwrap_or(0);
        self.focus = focusable[wrap_index(current, focusable.len(), offset)];
    }

    fn submit(&mut self, snapshot: &Snapshot) -> Vec<UiEffect> {
        let mode = std::mem::replace(&mut self.mode, InputMode::Normal);
        self.focus = Region::Canvas;
        match mode {
            InputMode::Prompt(prompt) if !prompt.trim().is_empty() => {
                vec![UiEffect::Prompt(prompt.trim().into())]
            }
            InputMode::QuickOpen { query, selected } => {
                let candidates = file_candidates(snapshot, &query);
                let path = if query.trim().is_empty() {
                    candidates.get(selected).copied().unwrap_or_default()
                } else {
                    query.trim()
                };
                if path.is_empty() {
                    Vec::new()
                } else {
                    self.view = MainView::Workbench;
                    vec![
                        UiEffect::Intent {
                            name: "file_open".into(),
                            input: json!({"path": path}),
                        },
                        UiEffect::Intent {
                            name: "code_read".into(),
                            input: json!({"path": path}),
                        },
                    ]
                }
            }
            _ => Vec::new(),
        }
    }

    fn run_palette(
        &mut self,
        profile: &SurfaceProfile,
        snapshot: &Snapshot,
        busy: bool,
    ) -> Vec<UiEffect> {
        let InputMode::Palette { query, selected } = &self.mode else {
            return Vec::new();
        };
        let selected = palette_actions(profile, query).get(*selected).copied();
        if selected.is_some_and(|action| action_disabled(Some(action), busy, snapshot).is_some()) {
            return Vec::new();
        }
        self.mode = InputMode::Normal;
        selected.map_or_else(Vec::new, |action| {
            self.apply_action(&action.id, profile, snapshot, busy, false)
        })
    }

    fn move_selection(
        &mut self,
        profile: &SurfaceProfile,
        snapshot: &Snapshot,
        _busy: bool,
        offset: isize,
    ) {
        match &mut self.mode {
            InputMode::Palette { query, selected } => {
                let count = palette_actions(profile, query).len();
                *selected = wrap_index(*selected, count, offset);
            }
            InputMode::QuickOpen { query, selected } => {
                let count = file_candidates(snapshot, query).len();
                *selected = wrap_index(*selected, count, offset);
            }
            InputMode::Normal | InputMode::Prompt(_) | InputMode::Help => {}
        }
    }

    fn scroll_mut(&mut self) -> &mut Scroll {
        self.scroll.entry(self.focus).or_default()
    }
}

pub(crate) fn palette_actions<'a>(
    profile: &'a SurfaceProfile,
    query: &str,
) -> Vec<&'a SurfaceAction> {
    let query = query.trim().to_ascii_lowercase();
    profile
        .actions
        .iter()
        .filter(|action| action.palette)
        .filter(|action| {
            query.is_empty()
                || action.id.contains(&query)
                || action.label.to_ascii_lowercase().contains(&query)
        })
        .collect()
}

pub(crate) fn file_candidates<'a>(snapshot: &'a Snapshot, query: &str) -> Vec<&'a str> {
    let query = query.trim().to_ascii_lowercase();
    snapshot
        .workbench
        .open_files
        .iter()
        .map(String::as_str)
        .filter(|path| query.is_empty() || path.to_ascii_lowercase().contains(&query))
        .collect()
}

pub(crate) fn action_disabled(
    action: Option<&SurfaceAction>,
    busy: bool,
    snapshot: &Snapshot,
) -> Option<&'static str> {
    match action.map(|action| action.availability) {
        None => Some("unknown action"),
        Some(ActionAvailability::Idle | ActionAvailability::Pane) if busy => {
            Some("Harness is busy")
        }
        Some(ActionAvailability::Pane) if snapshot.workbench.focused_pane.is_none() => {
            Some("No focused pane")
        }
        Some(ActionAvailability::Always | ActionAvailability::Idle | ActionAvailability::Pane) => {
            None
        }
    }
}

fn binding<'a>(profile: &'a SurfaceProfile, mode: &str, chord: &str) -> Option<&'a str> {
    profile
        .mode(mode)?
        .bindings
        .iter()
        .find(|binding| binding.key == chord)
        .map(|binding| binding.action.as_str())
}

fn edit(input: &mut String, chord: &str, character: Option<char>) -> bool {
    if chord == "backspace" {
        return input.pop().is_some();
    }
    if let Some(character) = character {
        input.push(character);
        return true;
    }
    false
}

fn pane_at_offset(snapshot: &Snapshot, offset: isize) -> Option<String> {
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
    Some(panes[wrap_index(current, panes.len(), offset)].id.clone())
}

fn focus_effect(pane_id: &str) -> UiEffect {
    UiEffect::Intent {
        name: "pane_focus".into(),
        input: json!({"pane_id": pane_id}),
    }
}

fn wrap_index(current: usize, count: usize, offset: isize) -> usize {
    if count == 0 {
        return 0;
    }
    let steps = offset.unsigned_abs() % count;
    if offset.is_negative() {
        (current + count - steps) % count
    } else {
        (current + steps) % count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentide_core::Workbench;

    fn snapshot() -> Snapshot {
        Snapshot {
            format: "agentide.snapshot/1".into(),
            session_id: "session".into(),
            objective: "test".into(),
            status: "active".into(),
            cursor: 0,
            workbench: Workbench::default(),
            pending_approvals: Vec::new(),
            processes: Vec::new(),
            agents: Vec::new(),
            evidence: Vec::new(),
            last_result: None,
        }
    }

    fn key(chord: &str, character: Option<char>) -> UiEvent {
        UiEvent::Key {
            chord: chord.into(),
            character,
        }
    }

    #[test]
    fn resize_uses_profile_breakpoints_and_repairs_focus() {
        let profile = SurfaceProfile::embedded().expect("profile");
        let mut state = SurfaceState {
            focus: Region::Activity,
            ..SurfaceState::default()
        };
        state.reduce(
            UiEvent::Resize {
                columns: 80,
                rows: 24,
            },
            &profile,
            &snapshot(),
            false,
            false,
        );
        assert_eq!(state.viewport, "compact");
        assert_eq!(state.focus, Region::Canvas);
        state.reduce(
            UiEvent::Resize {
                columns: 180,
                rows: 50,
            },
            &profile,
            &snapshot(),
            false,
            false,
        );
        state.show_region(&profile, Region::Activity);
        assert_eq!(state.focus, Region::Activity);
    }

    #[test]
    fn palette_filters_and_dispatches_semantic_actions() {
        let profile = SurfaceProfile::embedded().expect("profile");
        let mut state = SurfaceState::default();
        state.reduce(key("ctrl+k", None), &profile, &snapshot(), false, false);
        for character in "changes".chars() {
            state.reduce(
                key(&character.to_string(), Some(character)),
                &profile,
                &snapshot(),
                false,
                false,
            );
        }
        let effects = state.reduce(key("enter", None), &profile, &snapshot(), false, false);
        assert_eq!(effects.len(), 2);
        assert!(matches!(
            &effects[0],
            UiEffect::Intent { name, .. } if name == "diff_show"
        ));
    }

    #[test]
    fn approval_mode_ignores_text_and_only_emits_exact_decisions() {
        let profile = SurfaceProfile::embedded().expect("profile");
        let mut state = SurfaceState::default();
        assert!(
            state
                .reduce(key("q", Some('q')), &profile, &snapshot(), false, true)
                .is_empty()
        );
        assert_eq!(
            state.reduce(key("y", Some('y')), &profile, &snapshot(), false, true),
            vec![UiEffect::Approval(true)]
        );
        assert_eq!(
            state.reduce(key("escape", None), &profile, &snapshot(), false, true),
            vec![UiEffect::Approval(false)]
        );
    }

    #[test]
    fn region_scroll_is_independent() {
        let profile = SurfaceProfile::embedded().expect("profile");
        let mut state = SurfaceState::default();
        state.reduce(key("down", None), &profile, &snapshot(), false, false);
        assert_eq!(state.scroll(Region::Canvas).vertical, 3);
        state.focus = Region::Explorer;
        state.reduce(key("down", None), &profile, &snapshot(), false, false);
        assert_eq!(state.scroll(Region::Explorer).vertical, 3);
        assert_eq!(state.scroll(Region::Canvas).vertical, 3);
    }
}

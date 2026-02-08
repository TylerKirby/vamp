use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use ratatui::layout::Rect;

use crate::state::{ActivityEntry, Agent, BeadsState, FilesState, MetricsState, SidebarState, TreeState};

pub const AGENT_TYPES: [&str; 3] = ["claude", "codex", "cursor"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    AddAgent { selected: usize },
    ConfirmKill { agent_id: String, agent_name: String },
    RenameAgent { agent_id: String, input: String },
    BeadDetail { issue_index: usize, scroll: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Players,
    Charts,
    Beads,
    Setlist,
    Files,
    Metrics,
}

impl Tab {
    pub const ALL: [Tab; 6] = [Tab::Players, Tab::Charts, Tab::Beads, Tab::Setlist, Tab::Files, Tab::Metrics];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Players => "players",
            Tab::Charts => "charts",
            Tab::Beads => "beads",
            Tab::Setlist => "setlist",
            Tab::Files => "files",
            Tab::Metrics => "metrics",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Tab::Players => "plyr",
            Tab::Charts => "chrt",
            Tab::Beads => "bead",
            Tab::Setlist => "set",
            Tab::Files => "file",
            Tab::Metrics => "metr",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Players => 0,
            Tab::Charts => 1,
            Tab::Beads => 2,
            Tab::Setlist => 3,
            Tab::Files => 4,
            Tab::Metrics => 5,
        }
    }

    pub fn from_index(i: usize) -> Tab {
        match i {
            0 => Tab::Players,
            1 => Tab::Charts,
            2 => Tab::Beads,
            3 => Tab::Setlist,
            4 => Tab::Files,
            5 => Tab::Metrics,
            _ => Tab::Players,
        }
    }

    pub fn next(&self) -> Tab {
        Tab::from_index((self.index() + 1) % 6)
    }

    pub fn prev(&self) -> Tab {
        Tab::from_index((self.index() + 5) % 6)
    }
}

pub struct App {
    pub active_tab: Tab,
    pub state_file: Option<PathBuf>,
    pub state: SidebarState,
    pub files_state: FilesState,
    pub beads_state: BeadsState,
    pub activity_log: Vec<ActivityEntry>,
    pub tree_state: TreeState,
    pub metrics_state: MetricsState,
    pub collapsed_dirs: std::collections::HashSet<String>,
    pub selected_index: usize,
    pub beads_filter_active: bool,
    pub beads_status_filter: Option<String>,
    pub beads_type_filter: Option<String>,
    pub dialog: Option<Dialog>,
    pub content_area: Option<Rect>,
    pub status_message: Option<(String, Instant)>,
    // Mtimes for change detection
    files_mtime: Option<SystemTime>,
    beads_mtime: Option<SystemTime>,
    activity_mtime: Option<SystemTime>,
    tree_mtime: Option<SystemTime>,
    metrics_mtime: Option<SystemTime>,
}

impl App {
    pub fn new(state_file: Option<PathBuf>) -> Self {
        Self {
            active_tab: Tab::Players,
            state_file,
            state: SidebarState::default(),
            files_state: FilesState::default(),
            beads_state: BeadsState::default(),
            activity_log: Vec::new(),
            tree_state: TreeState::default(),
            metrics_state: MetricsState::default(),
            collapsed_dirs: std::collections::HashSet::new(),
            selected_index: 0,
            beads_filter_active: true,
            beads_status_filter: None,
            beads_type_filter: None,
            dialog: None,
            content_area: None,
            status_message: None,
            files_mtime: None,
            beads_mtime: None,
            activity_mtime: None,
            tree_mtime: None,
            metrics_mtime: None,
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        if self.active_tab == Tab::Players {
            self.snap_to_first_agent();
        } else {
            self.selected_index = 0;
        }
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = self.active_tab.prev();
        if self.active_tab == Tab::Players {
            self.snap_to_first_agent();
        } else {
            self.selected_index = 0;
        }
    }

    pub fn select_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        if tab == Tab::Players {
            self.snap_to_first_agent();
        } else {
            self.selected_index = 0;
        }
    }

    pub fn select_next(&mut self) {
        if self.active_tab == Tab::Players {
            let flat = build_flat_list(&self.state.agents);
            let current_flat = self.selected_index + 1;
            for i in (current_flat + 1)..flat.len() {
                if matches!(flat[i], FlatRow::Agent(_)) {
                    self.selected_index = i.saturating_sub(1);
                    return;
                }
            }
        } else {
            let max = self.max_selectable_index();
            if self.selected_index < max {
                self.selected_index += 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.active_tab == Tab::Players {
            let flat = build_flat_list(&self.state.agents);
            let current_flat = self.selected_index + 1;
            for i in (0..current_flat).rev() {
                if matches!(flat[i], FlatRow::Agent(_)) {
                    self.selected_index = i.saturating_sub(1);
                    return;
                }
            }
        } else {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    /// Max valid selected_index for the current tab's data
    fn max_selectable_index(&self) -> usize {
        match self.active_tab {
            Tab::Charts => self.files_state.files.len().saturating_sub(1),
            Tab::Beads => self.filtered_beads_count().saturating_sub(1),
            Tab::Files => self.visible_file_count().saturating_sub(1),
            _ => usize::MAX, // Players handled separately; Setlist/Metrics have no selection
        }
    }

    fn filtered_beads_count(&self) -> usize {
        self.beads_state.issues.iter()
            .filter(|i| {
                if self.beads_filter_active && (i.status == "closed" || i.status == "done") { return false; }
                if let Some(ref sf) = self.beads_status_filter { if i.status != *sf { return false; } }
                if let Some(ref tf) = self.beads_type_filter { if i.issue_type != *tf { return false; } }
                true
            })
            .count()
    }

    fn visible_file_count(&self) -> usize {
        self.tree_state.entries.iter()
            .filter(|e| {
                let parts: Vec<&str> = e.path.split('/').collect();
                for i in 1..parts.len() {
                    let ancestor = parts[..i].join("/");
                    if self.collapsed_dirs.contains(&ancestor) {
                        return false;
                    }
                }
                true
            })
            .count()
    }

    /// Clamp selected_index to valid range after data changes
    pub fn clamp_selection(&mut self) {
        if self.active_tab == Tab::Players {
            let flat = build_flat_list(&self.state.agents);
            let current_flat = self.selected_index + 1;
            if current_flat >= flat.len() || !matches!(flat.get(current_flat), Some(FlatRow::Agent(_))) {
                self.snap_to_first_agent();
            }
        } else {
            let max = self.max_selectable_index();
            if max != usize::MAX && self.selected_index > max {
                self.selected_index = max;
            }
        }
    }

    /// Snap selection to the first agent row (used when switching to Players tab)
    pub fn snap_to_first_agent(&mut self) {
        let flat = build_flat_list(&self.state.agents);
        for (i, row) in flat.iter().enumerate() {
            if matches!(row, FlatRow::Agent(_)) {
                self.selected_index = i.saturating_sub(1);
                return;
            }
        }
        self.selected_index = 0;
    }

    pub fn toggle_beads_filter(&mut self) {
        self.beads_filter_active = !self.beads_filter_active;
    }

    pub fn cycle_beads_status_filter(&mut self) {
        const STATUSES: [&str; 4] = ["open", "in_progress", "blocked", "closed"];
        self.beads_status_filter = match &self.beads_status_filter {
            None => Some(STATUSES[0].to_string()),
            Some(s) => {
                if let Some(idx) = STATUSES.iter().position(|&x| x == s) {
                    let next = idx + 1;
                    if next >= STATUSES.len() { None } else { Some(STATUSES[next].to_string()) }
                } else {
                    None
                }
            }
        };
        self.selected_index = 0;
    }

    pub fn cycle_beads_type_filter(&mut self) {
        const TYPES: [&str; 3] = ["task", "bug", "feature"];
        self.beads_type_filter = match &self.beads_type_filter {
            None => Some(TYPES[0].to_string()),
            Some(s) => {
                if let Some(idx) = TYPES.iter().position(|&x| x == s) {
                    let next = idx + 1;
                    if next >= TYPES.len() { None } else { Some(TYPES[next].to_string()) }
                } else {
                    None
                }
            }
        };
        self.selected_index = 0;
    }

    /// Set a temporary status message (auto-expires after timeout_secs)
    pub fn set_status(&mut self, msg: String, timeout_secs: u64) {
        self.status_message = Some((msg, Instant::now() + std::time::Duration::from_secs(timeout_secs)));
    }

    /// Reload main state file
    pub fn reload_state(&mut self) {
        if let Some(ref path) = self.state_file {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&contents) {
                    self.state = state;
                    self.clamp_selection();
                }
            }
        }
    }

    /// Reload auxiliary data files (files.json, beads.json, activity.log)
    pub fn reload_aux_data(&mut self) {
        if let Some(ref state_path) = self.state_file {
            let vamp_dir = state_path.parent().unwrap_or(state_path);

            // files.json
            let files_path = vamp_dir.join("files.json");
            if let Some(new_mtime) = check_mtime(&files_path, &self.files_mtime) {
                self.files_mtime = Some(new_mtime);
                if let Ok(contents) = std::fs::read_to_string(&files_path) {
                    if let Ok(state) = serde_json::from_str(&contents) {
                        self.files_state = state;
                    }
                }
            }

            // beads.json
            let beads_path = vamp_dir.join("beads.json");
            if let Some(new_mtime) = check_mtime(&beads_path, &self.beads_mtime) {
                self.beads_mtime = Some(new_mtime);
                if let Ok(contents) = std::fs::read_to_string(&beads_path) {
                    if let Ok(state) = serde_json::from_str(&contents) {
                        self.beads_state = state;
                    }
                }
            }

            // activity.log (JSONL format, one entry per line)
            let activity_path = vamp_dir.join("activity.log");
            if let Some(new_mtime) = check_mtime(&activity_path, &self.activity_mtime) {
                self.activity_mtime = Some(new_mtime);
                if let Ok(contents) = std::fs::read_to_string(&activity_path) {
                    self.activity_log = contents
                        .lines()
                        .filter_map(|line| serde_json::from_str(line).ok())
                        .collect();
                }
            }

            // tree.json
            let tree_path = vamp_dir.join("tree.json");
            if let Some(new_mtime) = check_mtime(&tree_path, &self.tree_mtime) {
                self.tree_mtime = Some(new_mtime);
                if let Ok(contents) = std::fs::read_to_string(&tree_path) {
                    if let Ok(state) = serde_json::from_str(&contents) {
                        self.tree_state = state;
                    }
                }
            }

            // metrics.json
            let metrics_path = vamp_dir.join("metrics.json");
            if let Some(new_mtime) = check_mtime(&metrics_path, &self.metrics_mtime) {
                self.metrics_mtime = Some(new_mtime);
                if let Ok(contents) = std::fs::read_to_string(&metrics_path) {
                    if let Ok(state) = serde_json::from_str(&contents) {
                        self.metrics_state = state;
                    }
                }
            }

            self.clamp_selection();
        }
    }
}

/// Compute scroll offset so `selected` stays visible within `visible_height`.
/// Returns the number of items to skip from the start of the list.
pub fn scroll_offset(selected: usize, visible_height: usize) -> usize {
    if visible_height == 0 {
        return 0;
    }
    if selected >= visible_height {
        selected - visible_height + 1
    } else {
        0
    }
}

/// Rows in the flat agent list (matching players.rs rendering layout)
pub enum FlatRow {
    Header(String),
    Agent(usize), // index into state.agents
    Spacer,
}

/// Build a flat list of rows matching how players.rs renders agents.
/// Used for mapping selected_index and mouse clicks to actual agents.
pub fn build_flat_list(agents: &[Agent]) -> Vec<FlatRow> {
    let mut rows = Vec::new();
    let type_order = ["claude", "codex", "cursor"];
    let mut grouped: Vec<(&str, Vec<usize>)> = Vec::new();

    for t in &type_order {
        let indices: Vec<usize> = agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.agent_type == *t)
            .map(|(i, _)| i)
            .collect();
        if !indices.is_empty() {
            grouped.push((t, indices));
        }
    }

    // Collect agents with unknown types
    let known: Vec<&str> = type_order.to_vec();
    let other_indices: Vec<usize> = agents
        .iter()
        .enumerate()
        .filter(|(_, a)| !known.contains(&a.agent_type.as_str()))
        .map(|(i, _)| i)
        .collect();
    if !other_indices.is_empty() {
        grouped.push(("other", other_indices));
    }

    for (agent_type, indices) in grouped {
        rows.push(FlatRow::Header(agent_type.to_string()));
        for idx in indices {
            rows.push(FlatRow::Agent(idx));
        }
        rows.push(FlatRow::Spacer);
    }

    rows
}

/// Check if file mtime has changed
fn check_mtime(path: &PathBuf, last: &Option<SystemTime>) -> Option<SystemTime> {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            if last.as_ref() != Some(&mtime) {
                return Some(mtime);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Agent, Tokens};

    fn make_agent(id: &str, agent_type: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: id.to_string(),
            agent_type: agent_type.to_string(),
            status: "running".to_string(),
            model: String::new(),
            branch: String::new(),
            worktree: String::new(),
            tmux_window: String::new(),
            task: String::new(),
            pid: 0,
            started_at: String::new(),
            tokens: Tokens::default(),
            cost_usd: 0.0,
        }
    }

    // --- Tab tests ---

    #[test]
    fn tab_next_cycles() {
        assert_eq!(Tab::Players.next(), Tab::Charts);
        assert_eq!(Tab::Charts.next(), Tab::Beads);
        assert_eq!(Tab::Beads.next(), Tab::Setlist);
        assert_eq!(Tab::Setlist.next(), Tab::Files);
        assert_eq!(Tab::Files.next(), Tab::Metrics);
        assert_eq!(Tab::Metrics.next(), Tab::Players);
    }

    #[test]
    fn tab_prev_cycles() {
        assert_eq!(Tab::Players.prev(), Tab::Metrics);
        assert_eq!(Tab::Metrics.prev(), Tab::Files);
        assert_eq!(Tab::Files.prev(), Tab::Setlist);
        assert_eq!(Tab::Setlist.prev(), Tab::Beads);
        assert_eq!(Tab::Beads.prev(), Tab::Charts);
        assert_eq!(Tab::Charts.prev(), Tab::Players);
    }

    #[test]
    fn tab_from_index() {
        assert_eq!(Tab::from_index(0), Tab::Players);
        assert_eq!(Tab::from_index(1), Tab::Charts);
        assert_eq!(Tab::from_index(2), Tab::Beads);
        assert_eq!(Tab::from_index(3), Tab::Setlist);
        assert_eq!(Tab::from_index(4), Tab::Files);
        assert_eq!(Tab::from_index(5), Tab::Metrics);
        // Out of bounds wraps to Players
        assert_eq!(Tab::from_index(99), Tab::Players);
    }

    #[test]
    fn tab_index_roundtrip() {
        for tab in Tab::ALL {
            assert_eq!(Tab::from_index(tab.index()), tab);
        }
    }

    #[test]
    fn tab_labels() {
        assert_eq!(Tab::Players.label(), "players");
        assert_eq!(Tab::Charts.label(), "charts");
        assert_eq!(Tab::Beads.label(), "beads");
        assert_eq!(Tab::Setlist.label(), "setlist");
        assert_eq!(Tab::Files.label(), "files");
        assert_eq!(Tab::Metrics.label(), "metrics");
    }

    // --- build_flat_list tests ---

    #[test]
    fn flat_list_empty() {
        let flat = build_flat_list(&[]);
        assert!(flat.is_empty());
    }

    #[test]
    fn flat_list_single_type() {
        let agents = vec![make_agent("a1", "claude"), make_agent("a2", "claude")];
        let flat = build_flat_list(&agents);
        // Should be: Header("claude"), Agent(0), Agent(1), Spacer
        assert_eq!(flat.len(), 4);
        assert!(matches!(&flat[0], FlatRow::Header(t) if t == "claude"));
        assert!(matches!(flat[1], FlatRow::Agent(0)));
        assert!(matches!(flat[2], FlatRow::Agent(1)));
        assert!(matches!(flat[3], FlatRow::Spacer));
    }

    #[test]
    fn flat_list_multiple_types() {
        let agents = vec![
            make_agent("a1", "claude"),
            make_agent("a2", "codex"),
            make_agent("a3", "cursor"),
        ];
        let flat = build_flat_list(&agents);
        // 3 groups × (Header + Agent + Spacer) = 9
        assert_eq!(flat.len(), 9);
        assert!(matches!(&flat[0], FlatRow::Header(t) if t == "claude"));
        assert!(matches!(&flat[3], FlatRow::Header(t) if t == "codex"));
        assert!(matches!(&flat[6], FlatRow::Header(t) if t == "cursor"));
    }

    #[test]
    fn flat_list_unknown_types() {
        let agents = vec![
            make_agent("a1", "claude"),
            make_agent("a2", "gemini"),
        ];
        let flat = build_flat_list(&agents);
        // claude group + "other" group
        let headers: Vec<&String> = flat.iter().filter_map(|r| match r {
            FlatRow::Header(t) => Some(t),
            _ => None,
        }).collect();
        assert_eq!(headers, vec!["claude", "other"]);
    }

    #[test]
    fn flat_list_preserves_agent_indices() {
        let agents = vec![
            make_agent("a1", "codex"),
            make_agent("a2", "claude"),
        ];
        let flat = build_flat_list(&agents);
        // claude comes first in type_order, so agent index 1 (a2) appears before index 0 (a1)
        let agent_indices: Vec<usize> = flat.iter().filter_map(|r| match r {
            FlatRow::Agent(i) => Some(*i),
            _ => None,
        }).collect();
        assert_eq!(agent_indices, vec![1, 0]);
    }

    // --- Filter cycling tests ---

    #[test]
    fn cycle_beads_status_filter() {
        let mut app = App::new(None);
        assert_eq!(app.beads_status_filter, None);

        app.cycle_beads_status_filter();
        assert_eq!(app.beads_status_filter.as_deref(), Some("open"));

        app.cycle_beads_status_filter();
        assert_eq!(app.beads_status_filter.as_deref(), Some("in_progress"));

        app.cycle_beads_status_filter();
        assert_eq!(app.beads_status_filter.as_deref(), Some("blocked"));

        app.cycle_beads_status_filter();
        assert_eq!(app.beads_status_filter.as_deref(), Some("closed"));

        // Wraps back to None
        app.cycle_beads_status_filter();
        assert_eq!(app.beads_status_filter, None);
    }

    #[test]
    fn cycle_beads_type_filter() {
        let mut app = App::new(None);
        assert_eq!(app.beads_type_filter, None);

        app.cycle_beads_type_filter();
        assert_eq!(app.beads_type_filter.as_deref(), Some("task"));

        app.cycle_beads_type_filter();
        assert_eq!(app.beads_type_filter.as_deref(), Some("bug"));

        app.cycle_beads_type_filter();
        assert_eq!(app.beads_type_filter.as_deref(), Some("feature"));

        app.cycle_beads_type_filter();
        assert_eq!(app.beads_type_filter, None);
    }

    #[test]
    fn cycle_filter_resets_selected_index() {
        let mut app = App::new(None);
        app.selected_index = 5;
        app.cycle_beads_status_filter();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn cycle_filter_unknown_value_resets_to_none() {
        let mut app = App::new(None);
        app.beads_status_filter = Some("unknown_status".to_string());
        app.cycle_beads_status_filter();
        assert_eq!(app.beads_status_filter, None);
    }

    // --- App state tests ---

    #[test]
    fn set_status_stores_message() {
        let mut app = App::new(None);
        app.set_status("test message".to_string(), 5);
        assert!(app.status_message.is_some());
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert_eq!(msg, "test message");
    }

    #[test]
    fn toggle_beads_filter() {
        let mut app = App::new(None);
        assert!(app.beads_filter_active);
        app.toggle_beads_filter();
        assert!(!app.beads_filter_active);
        app.toggle_beads_filter();
        assert!(app.beads_filter_active);
    }

    #[test]
    fn new_app_defaults() {
        let app = App::new(None);
        assert_eq!(app.active_tab, Tab::Players);
        assert_eq!(app.selected_index, 0);
        assert!(app.beads_filter_active);
        assert!(app.beads_status_filter.is_none());
        assert!(app.beads_type_filter.is_none());
        assert!(app.dialog.is_none());
        assert!(app.status_message.is_none());
    }

    #[test]
    fn next_tab_cycles_through_all() {
        let mut app = App::new(None);
        assert_eq!(app.active_tab, Tab::Players);
        app.next_tab();
        assert_eq!(app.active_tab, Tab::Charts);
        app.next_tab();
        assert_eq!(app.active_tab, Tab::Beads);
        app.next_tab();
        assert_eq!(app.active_tab, Tab::Setlist);
        app.next_tab();
        assert_eq!(app.active_tab, Tab::Files);
        app.next_tab();
        assert_eq!(app.active_tab, Tab::Metrics);
        app.next_tab();
        assert_eq!(app.active_tab, Tab::Players);
    }

    #[test]
    fn prev_tab_cycles_backward() {
        let mut app = App::new(None);
        app.prev_tab();
        assert_eq!(app.active_tab, Tab::Metrics);
        app.prev_tab();
        assert_eq!(app.active_tab, Tab::Files);
        app.prev_tab();
        assert_eq!(app.active_tab, Tab::Setlist);
        app.prev_tab();
        assert_eq!(app.active_tab, Tab::Beads);
    }

    #[test]
    fn select_tab_resets_index() {
        let mut app = App::new(None);
        app.selected_index = 10;
        app.select_tab(Tab::Charts);
        assert_eq!(app.active_tab, Tab::Charts);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn select_next_non_players_tab() {
        use crate::state::{FilesState, FileEntry};
        let mut app = App::new(None);
        app.select_tab(Tab::Charts);
        // Populate with files so there's something to select
        app.files_state = FilesState {
            files: vec![
                FileEntry { path: "a.rs".into(), agents: Default::default(), conflict: false },
                FileEntry { path: "b.rs".into(), agents: Default::default(), conflict: false },
                FileEntry { path: "c.rs".into(), agents: Default::default(), conflict: false },
            ],
            ..FilesState::default()
        };
        app.selected_index = 0;
        app.select_next();
        assert_eq!(app.selected_index, 1);
        // Should clamp at max index (2)
        app.selected_index = 2;
        app.select_next();
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn select_next_empty_tab_stays_at_zero() {
        let mut app = App::new(None);
        app.select_tab(Tab::Charts);
        app.selected_index = 0;
        app.select_next();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn select_prev_non_players_tab() {
        let mut app = App::new(None);
        app.select_tab(Tab::Charts);
        app.selected_index = 3;
        app.select_prev();
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn select_prev_at_zero_stays() {
        let mut app = App::new(None);
        app.select_tab(Tab::Charts);
        app.selected_index = 0;
        app.select_prev();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn select_next_players_with_agents() {
        let mut app = App::new(None);
        app.state.agents = vec![
            make_agent("a1", "claude"),
            make_agent("a2", "claude"),
        ];
        // Snap to first agent
        app.snap_to_first_agent();
        let initial = app.selected_index;
        app.select_next();
        // Should move to next agent
        assert!(app.selected_index > initial);
    }

    #[test]
    fn snap_to_first_agent_empty() {
        let mut app = App::new(None);
        app.selected_index = 5;
        app.snap_to_first_agent();
        assert_eq!(app.selected_index, 0);
    }
}

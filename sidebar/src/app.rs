use std::path::PathBuf;
use std::time::SystemTime;

use crate::state::{ActivityEntry, BeadsState, FilesState, SidebarState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Players,
    Charts,
    Beads,
    Setlist,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Players, Tab::Charts, Tab::Beads, Tab::Setlist];

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Players => "players",
            Tab::Charts => "charts",
            Tab::Beads => "beads",
            Tab::Setlist => "setlist",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Players => 0,
            Tab::Charts => 1,
            Tab::Beads => 2,
            Tab::Setlist => 3,
        }
    }

    pub fn from_index(i: usize) -> Tab {
        match i {
            0 => Tab::Players,
            1 => Tab::Charts,
            2 => Tab::Beads,
            3 => Tab::Setlist,
            _ => Tab::Players,
        }
    }

    pub fn next(&self) -> Tab {
        Tab::from_index((self.index() + 1) % 4)
    }

    pub fn prev(&self) -> Tab {
        Tab::from_index((self.index() + 3) % 4)
    }
}

pub struct App {
    pub active_tab: Tab,
    pub running: bool,
    pub state_file: Option<PathBuf>,
    pub state: SidebarState,
    pub files_state: FilesState,
    pub beads_state: BeadsState,
    pub activity_log: Vec<ActivityEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub beads_filter_active: bool,
    // Mtimes for change detection
    files_mtime: Option<SystemTime>,
    beads_mtime: Option<SystemTime>,
    activity_mtime: Option<SystemTime>,
}

impl App {
    pub fn new(state_file: Option<PathBuf>) -> Self {
        Self {
            active_tab: Tab::Players,
            running: true,
            state_file,
            state: SidebarState::default(),
            files_state: FilesState::default(),
            beads_state: BeadsState::default(),
            activity_log: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            beads_filter_active: true,
            files_mtime: None,
            beads_mtime: None,
            activity_mtime: None,
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = self.active_tab.prev();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn select_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn select_next(&mut self) {
        self.selected_index = self.selected_index.saturating_add(1);
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn toggle_beads_filter(&mut self) {
        self.beads_filter_active = !self.beads_filter_active;
    }

    /// Reload main state file
    pub fn reload_state(&mut self) {
        if let Some(ref path) = self.state_file {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&contents) {
                    self.state = state;
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
        }
    }
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

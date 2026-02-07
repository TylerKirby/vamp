use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SidebarState {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub agents: Vec<Agent>,
    #[serde(default)]
    pub totals: Totals,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub agent_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub worktree: String,
    #[serde(default)]
    pub tmux_window: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub pid: u32,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub tokens: Tokens,
    #[serde(default)]
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tokens {
    #[serde(default)]
    pub input: u64,
    #[serde(default)]
    pub output: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Totals {
    #[serde(default)]
    pub tokens: u64,
    #[serde(default)]
    pub cost_usd: f64,
}

// Charts tab data — file changes across agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesState {
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub locks: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    #[serde(default)]
    pub agents: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub conflict: bool,
}

// Beads tab data — issues from bd list
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeadsState {
    #[serde(default)]
    pub issues: Vec<BeadIssue>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadIssue {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub priority: u8,
    #[serde(default, rename = "type")]
    pub issue_type: String,
    #[serde(default)]
    pub assignee: String,
}

// Setlist tab data — activity log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub timestamp: String,
    pub agent: String,
    pub event: String,
    #[serde(default)]
    pub detail: String,
}

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
    pub focused_agent: String,
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

// Charts tab data — git branches, worktrees, and file changes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesState {
    #[serde(default)]
    pub branches: Vec<BranchInfo>,
    #[serde(default)]
    pub remotes: Vec<String>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub worktrees: Vec<WorktreeStatus>,
    #[serde(default)]
    pub locks: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    #[serde(default)]
    pub is_current: bool,
    #[serde(default)]
    pub tracking: String,
    #[serde(default)]
    pub ahead: u32,
    #[serde(default)]
    pub behind: u32,
    #[serde(default)]
    pub last_commit: String,
    #[serde(default)]
    pub is_agent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeStatus {
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub agent_type: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub commits_ahead: u32,
    #[serde(default)]
    pub dirty: bool,
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
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub dependency_count: u32,
    #[serde(default)]
    pub dependent_count: u32,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub closed_at: String,
    #[serde(default)]
    pub close_reason: String,
}

impl BeadIssue {
    /// An issue is effectively blocked if it has open dependencies,
    /// regardless of its status field (beads keeps status as "open").
    pub fn is_blocked(&self) -> bool {
        self.dependency_count > 0 && self.status != "closed" && self.status != "done"
    }

    /// The effective status, accounting for dependency-based blocking.
    pub fn effective_status(&self) -> &str {
        if self.is_blocked() {
            "blocked"
        } else {
            &self.status
        }
    }
}

// Files tab data — project file tree
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TreeState {
    #[serde(default)]
    pub entries: Vec<TreeEntry>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub is_dir: bool,
}

// Metrics tab data — system and Claude usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsState {
    #[serde(default)]
    pub system: SystemMetrics,
    #[serde(default)]
    pub claude: ClaudeMetrics,
    #[serde(default)]
    pub agents: Vec<AgentProcess>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    #[serde(default)]
    pub load_avg: [f64; 3],
    #[serde(default)]
    pub memory_pct: f64,
    #[serde(default)]
    pub memory_total_mb: u64,
    #[serde(default)]
    pub memory_used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeMetrics {
    #[serde(default)]
    pub today_messages: u64,
    #[serde(default)]
    pub today_sessions: u64,
    #[serde(default)]
    pub today_tool_calls: u64,
    #[serde(default)]
    pub total_messages: u64,
    #[serde(default)]
    pub total_sessions: u64,
    #[serde(default)]
    pub models: Vec<ModelUsage>,
    #[serde(default)]
    pub hour_counts: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProcess {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub process_status: String,
    #[serde(default, rename = "type")]
    pub agent_type: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub pid: u32,
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

/// Build demo fixture data for --demo mode
pub fn demo_state() -> (SidebarState, FilesState, BeadsState, Vec<ActivityEntry>, TreeState, MetricsState) {
    let state = SidebarState {
        version: "2.0.0".to_string(),
        project: Project {
            name: "my-project".to_string(),
            path: "/home/user/my-project".to_string(),
            session: "vamp-my-project".to_string(),
        },
        agents: vec![
            Agent {
                id: "agent-001".to_string(),
                name: "miles".to_string(),
                agent_type: "claude".to_string(),
                status: "running".to_string(),
                model: "opus".to_string(),
                branch: "agent/miles".to_string(),
                worktree: ".vamp-agents/miles".to_string(),
                tmux_window: "vamp-my-project:1".to_string(),
                task: "Implementing auth module".to_string(),
                pid: 12345,
                started_at: "2025-01-15T10:30:00Z".to_string(),
                tokens: Tokens { input: 45000, output: 12000 },
                cost_usd: 1.25,
            },
            Agent {
                id: "agent-002".to_string(),
                name: "coltrane".to_string(),
                agent_type: "codex".to_string(),
                status: "waiting".to_string(),
                model: "".to_string(),
                branch: "agent/coltrane".to_string(),
                worktree: ".vamp-agents/coltrane".to_string(),
                tmux_window: "vamp-my-project:2".to_string(),
                task: "Writing tests for API".to_string(),
                pid: 12346,
                started_at: "2025-01-15T10:45:00Z".to_string(),
                tokens: Tokens { input: 20000, output: 8000 },
                cost_usd: 0.60,
            },
            Agent {
                id: "agent-003".to_string(),
                name: "monk".to_string(),
                agent_type: "cursor".to_string(),
                status: "paused".to_string(),
                model: "".to_string(),
                branch: "agent/monk".to_string(),
                worktree: ".vamp-agents/monk".to_string(),
                tmux_window: "vamp-my-project:3".to_string(),
                task: "Refactoring database layer".to_string(),
                pid: 12347,
                started_at: "2025-01-15T11:00:00Z".to_string(),
                tokens: Tokens { input: 10000, output: 3000 },
                cost_usd: 0.30,
            },
        ],
        focused_agent: "agent-001".to_string(),
        totals: Totals { tokens: 98000, cost_usd: 2.15 },
        updated_at: "2025-01-15T11:05:00Z".to_string(),
    };

    let mut agents_map1 = std::collections::HashMap::new();
    agents_map1.insert("agent-001".to_string(), "M".to_string());
    agents_map1.insert("agent-003".to_string(), "M".to_string());

    let mut agents_map2 = std::collections::HashMap::new();
    agents_map2.insert("agent-001".to_string(), "A".to_string());

    let mut agents_map3 = std::collections::HashMap::new();
    agents_map3.insert("agent-002".to_string(), "M".to_string());

    let files_state = FilesState {
        branches: vec![
            BranchInfo {
                name: "main".to_string(),
                is_current: true,
                tracking: "origin/main".to_string(),
                ahead: 0,
                behind: 0,
                last_commit: "feat: vamp 2.0".to_string(),
                is_agent: false,
            },
            BranchInfo {
                name: "agent/miles".to_string(),
                is_current: false,
                tracking: String::new(),
                ahead: 3,
                behind: 1,
                last_commit: "wip: auth module".to_string(),
                is_agent: true,
            },
            BranchInfo {
                name: "agent/coltrane".to_string(),
                is_current: false,
                tracking: String::new(),
                ahead: 1,
                behind: 0,
                last_commit: "test: api routes".to_string(),
                is_agent: true,
            },
            BranchInfo {
                name: "feat/dark-mode".to_string(),
                is_current: false,
                tracking: "origin/feat/dark-mode".to_string(),
                ahead: 7,
                behind: 0,
                last_commit: "feat: dark mode toggle".to_string(),
                is_agent: false,
            },
        ],
        remotes: vec![
            "origin/main".to_string(),
            "origin/feat/dark-mode".to_string(),
            "origin/fix/login-bug".to_string(),
        ],
        files: vec![
            FileEntry {
                path: "src/auth.rs".to_string(),
                agents: agents_map1,
                conflict: true,
            },
            FileEntry {
                path: "src/api/routes.rs".to_string(),
                agents: agents_map2,
                conflict: false,
            },
            FileEntry {
                path: "tests/api_test.rs".to_string(),
                agents: agents_map3,
                conflict: false,
            },
        ],
        worktrees: vec![
            WorktreeStatus {
                agent_id: "agent-001".to_string(),
                agent_type: "claude".to_string(),
                branch: "agent/miles".to_string(),
                commits_ahead: 3,
                dirty: true,
            },
            WorktreeStatus {
                agent_id: "agent-002".to_string(),
                agent_type: "codex".to_string(),
                branch: "agent/coltrane".to_string(),
                commits_ahead: 1,
                dirty: false,
            },
        ],
        locks: std::collections::HashMap::new(),
        updated_at: "2025-01-15T11:05:00Z".to_string(),
    };

    let beads_state = BeadsState {
        issues: vec![
            BeadIssue {
                id: "beads-001".to_string(),
                title: "Add user authentication".to_string(),
                status: "in_progress".to_string(),
                priority: 1,
                issue_type: "feature".to_string(),
                assignee: "miles".to_string(),
                description: "Implement OAuth2 login flow".to_string(),
                created_at: "2025-01-14T09:00:00Z".to_string(),
                updated_at: "2025-01-15T10:30:00Z".to_string(),
                dependency_count: 0,
                dependent_count: 2,
                labels: vec!["auth".to_string()],
                closed_at: String::new(),
                close_reason: String::new(),
            },
            BeadIssue {
                id: "beads-002".to_string(),
                title: "Fix API rate limiting".to_string(),
                status: "open".to_string(),
                priority: 2,
                issue_type: "bug".to_string(),
                assignee: String::new(),
                description: "Rate limiter is not resetting correctly".to_string(),
                created_at: "2025-01-15T08:00:00Z".to_string(),
                updated_at: "2025-01-15T08:00:00Z".to_string(),
                dependency_count: 0,
                dependent_count: 0,
                labels: vec!["api".to_string(), "bug".to_string()],
                closed_at: String::new(),
                close_reason: String::new(),
            },
            BeadIssue {
                id: "beads-003".to_string(),
                title: "Write integration tests".to_string(),
                status: "blocked".to_string(),
                priority: 2,
                issue_type: "task".to_string(),
                assignee: "coltrane".to_string(),
                description: "Blocked on auth feature".to_string(),
                created_at: "2025-01-14T10:00:00Z".to_string(),
                updated_at: "2025-01-15T09:00:00Z".to_string(),
                dependency_count: 1,
                dependent_count: 0,
                labels: vec!["testing".to_string()],
                closed_at: String::new(),
                close_reason: String::new(),
            },
        ],
        updated_at: "2025-01-15T11:00:00Z".to_string(),
    };

    let activity = vec![
        ActivityEntry {
            timestamp: "2025-01-15T10:30:00Z".to_string(),
            agent: "miles".to_string(),
            event: "agent_create".to_string(),
            detail: "claude".to_string(),
        },
        ActivityEntry {
            timestamp: "2025-01-15T10:45:00Z".to_string(),
            agent: "coltrane".to_string(),
            event: "agent_create".to_string(),
            detail: "codex".to_string(),
        },
        ActivityEntry {
            timestamp: "2025-01-15T11:00:00Z".to_string(),
            agent: "monk".to_string(),
            event: "agent_pause".to_string(),
            detail: String::new(),
        },
    ];

    let tree_state = TreeState {
        entries: vec![
            TreeEntry { path: "src".to_string(), status: String::new(), is_dir: true },
            TreeEntry { path: "src/api".to_string(), status: String::new(), is_dir: true },
            TreeEntry { path: "src/api/routes.rs".to_string(), status: "A".to_string(), is_dir: false },
            TreeEntry { path: "src/api/handlers.rs".to_string(), status: String::new(), is_dir: false },
            TreeEntry { path: "src/auth.rs".to_string(), status: "M".to_string(), is_dir: false },
            TreeEntry { path: "src/main.rs".to_string(), status: String::new(), is_dir: false },
            TreeEntry { path: "src/db.rs".to_string(), status: "M".to_string(), is_dir: false },
            TreeEntry { path: "tests".to_string(), status: String::new(), is_dir: true },
            TreeEntry { path: "tests/api_test.rs".to_string(), status: "A".to_string(), is_dir: false },
            TreeEntry { path: "tests/auth_test.rs".to_string(), status: String::new(), is_dir: false },
            TreeEntry { path: "Cargo.toml".to_string(), status: "M".to_string(), is_dir: false },
            TreeEntry { path: "README.md".to_string(), status: String::new(), is_dir: false },
            TreeEntry { path: ".gitignore".to_string(), status: String::new(), is_dir: false },
        ],
        updated_at: "2025-01-15T11:05:00Z".to_string(),
    };

    let metrics_state = MetricsState {
        system: SystemMetrics {
            load_avg: [1.24, 0.98, 0.87],
            memory_pct: 67.0,
            memory_total_mb: 16384,
            memory_used_mb: 10977,
        },
        claude: ClaudeMetrics {
            today_messages: 1234,
            today_sessions: 4,
            today_tool_calls: 456,
            total_messages: 18500,
            total_sessions: 87,
            models: vec![
                ModelUsage {
                    model: "opus-4-6".to_string(),
                    input_tokens: 187000,
                    output_tokens: 78000,
                    cache_read_tokens: 42000,
                },
                ModelUsage {
                    model: "sonnet-4-5".to_string(),
                    input_tokens: 1500000,
                    output_tokens: 949000,
                    cache_read_tokens: 310000,
                },
            ],
            hour_counts: vec![0, 0, 0, 0, 0, 0, 12, 45, 52, 67, 89, 78, 65, 58, 61, 72, 48, 35, 8, 0, 0, 0, 0, 0],
        },
        agents: vec![
            AgentProcess {
                id: "agent-001".to_string(), name: "miles".to_string(),
                status: "running".to_string(), process_status: "alive".to_string(),
                agent_type: "claude".to_string(), branch: "agent/miles".to_string(),
                task: "Implementing auth module".to_string(), pid: 12345,
            },
            AgentProcess {
                id: "agent-002".to_string(), name: "coltrane".to_string(),
                status: "waiting".to_string(), process_status: "alive".to_string(),
                agent_type: "codex".to_string(), branch: "agent/coltrane".to_string(),
                task: "Writing tests for API".to_string(), pid: 12346,
            },
            AgentProcess {
                id: "agent-003".to_string(), name: "monk".to_string(),
                status: "paused".to_string(), process_status: "dead".to_string(),
                agent_type: "cursor".to_string(), branch: "agent/monk".to_string(),
                task: String::new(), pid: 12347,
            },
        ],
        updated_at: "2025-01-15T11:05:00Z".to_string(),
    };

    (state, files_state, beads_state, activity, tree_state, metrics_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_state_roundtrip() {
        let state = SidebarState {
            version: "2.0.0".to_string(),
            project: Project {
                name: "test".to_string(),
                path: "/tmp/test".to_string(),
                session: "vamp-test".to_string(),
            },
            agents: vec![Agent {
                id: "a1".to_string(),
                name: "miles".to_string(),
                agent_type: "claude".to_string(),
                status: "running".to_string(),
                model: "opus".to_string(),
                branch: "agent/miles".to_string(),
                worktree: ".vamp-agents/miles".to_string(),
                tmux_window: "vamp-test:1".to_string(),
                task: "coding".to_string(),
                pid: 1234,
                started_at: "2025-01-01T00:00:00Z".to_string(),
                tokens: Tokens { input: 100, output: 50 },
                cost_usd: 0.5,
            }],
            focused_agent: "a1".to_string(),
            totals: Totals { tokens: 150, cost_usd: 0.5 },
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&state).unwrap();
        let parsed: SidebarState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "2.0.0");
        assert_eq!(parsed.agents.len(), 1);
        assert_eq!(parsed.agents[0].agent_type, "claude");
        assert_eq!(parsed.agents[0].tokens.input, 100);
        assert_eq!(parsed.totals.cost_usd, 0.5);
    }

    #[test]
    fn agent_type_renamed_from_type_field() {
        let json = r#"{"id":"a1","type":"codex","name":"test"}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.agent_type, "codex");
    }

    #[test]
    fn agent_type_serializes_as_type() {
        let agent = Agent {
            id: "a1".to_string(),
            name: "test".to_string(),
            agent_type: "codex".to_string(),
            status: String::new(),
            model: String::new(),
            branch: String::new(),
            worktree: String::new(),
            tmux_window: String::new(),
            task: String::new(),
            pid: 0,
            started_at: String::new(),
            tokens: Tokens::default(),
            cost_usd: 0.0,
        };
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains(r#""type":"codex""#));
        assert!(!json.contains("agent_type"));
    }

    #[test]
    fn lenient_parsing_empty_json() {
        let state: SidebarState = serde_json::from_str("{}").unwrap();
        assert_eq!(state.version, "");
        assert!(state.agents.is_empty());
        assert_eq!(state.totals.tokens, 0);
    }

    #[test]
    fn lenient_parsing_missing_fields() {
        let json = r#"{"version":"1.0","agents":[{"id":"x"}]}"#;
        let state: SidebarState = serde_json::from_str(json).unwrap();
        assert_eq!(state.version, "1.0");
        assert_eq!(state.agents[0].id, "x");
        assert_eq!(state.agents[0].agent_type, "");
        assert_eq!(state.agents[0].pid, 0);
        assert_eq!(state.agents[0].cost_usd, 0.0);
    }

    #[test]
    fn real_world_json_format() {
        let json = r#"{
            "version": "2.0.0",
            "project": {"name": "vamp", "path": "/home/user/vamp", "session": "vamp-vamp"},
            "agents": [
                {
                    "id": "agent-001",
                    "name": "miles",
                    "type": "claude",
                    "status": "running",
                    "model": "opus",
                    "branch": "agent/miles",
                    "worktree": ".vamp-agents/miles",
                    "tmux_window": "vamp-vamp:1",
                    "task": "implementing feature",
                    "pid": 54321,
                    "started_at": "2025-01-15T10:00:00Z",
                    "tokens": {"input": 50000, "output": 15000},
                    "cost_usd": 1.50
                }
            ],
            "focused_agent": "agent-001",
            "totals": {"tokens": 65000, "cost_usd": 1.50},
            "updated_at": "2025-01-15T10:05:00Z"
        }"#;
        let state: SidebarState = serde_json::from_str(json).unwrap();
        assert_eq!(state.agents.len(), 1);
        assert_eq!(state.agents[0].agent_type, "claude");
        assert_eq!(state.agents[0].pid, 54321);
        assert_eq!(state.agents[0].tokens.input, 50000);
        assert_eq!(state.project.name, "vamp");
    }

    #[test]
    fn files_state_roundtrip() {
        let mut agents = std::collections::HashMap::new();
        agents.insert("a1".to_string(), "M".to_string());
        let fs = FilesState {
            branches: vec![BranchInfo {
                name: "main".to_string(),
                is_current: true,
                tracking: "origin/main".to_string(),
                ahead: 0,
                behind: 0,
                last_commit: "initial".to_string(),
                is_agent: false,
            }],
            remotes: vec!["origin/main".to_string()],
            files: vec![FileEntry {
                path: "src/main.rs".to_string(),
                agents,
                conflict: true,
            }],
            worktrees: vec![WorktreeStatus {
                agent_id: "a1".to_string(),
                agent_type: "claude".to_string(),
                branch: "agent/miles".to_string(),
                commits_ahead: 2,
                dirty: true,
            }],
            locks: std::collections::HashMap::new(),
            updated_at: "now".to_string(),
        };
        let json = serde_json::to_string(&fs).unwrap();
        let parsed: FilesState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.files.len(), 1);
        assert!(parsed.files[0].conflict);
        assert_eq!(parsed.worktrees[0].commits_ahead, 2);
    }

    #[test]
    fn beads_state_roundtrip() {
        let bs = BeadsState {
            issues: vec![BeadIssue {
                id: "beads-001".to_string(),
                title: "Fix bug".to_string(),
                status: "open".to_string(),
                priority: 1,
                issue_type: "bug".to_string(),
                assignee: "dev".to_string(),
                description: "Something broken".to_string(),
                created_at: "now".to_string(),
                updated_at: "now".to_string(),
                dependency_count: 0,
                dependent_count: 0,
                labels: vec!["urgent".to_string()],
                closed_at: String::new(),
                close_reason: String::new(),
            }],
            updated_at: "now".to_string(),
        };
        let json = serde_json::to_string(&bs).unwrap();
        let parsed: BeadsState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(parsed.issues[0].issue_type, "bug");
        assert_eq!(parsed.issues[0].priority, 1);
    }

    #[test]
    fn bead_issue_type_renamed_from_type_field() {
        let json = r#"{"id":"b1","type":"feature","title":"Add X"}"#;
        let issue: BeadIssue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.issue_type, "feature");
    }

    #[test]
    fn activity_entry_roundtrip() {
        let entry = ActivityEntry {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            agent: "miles".to_string(),
            event: "agent_create".to_string(),
            detail: "claude".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ActivityEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.agent, "miles");
        assert_eq!(parsed.event, "agent_create");
    }

    #[test]
    fn activity_entry_optional_detail() {
        let json = r#"{"timestamp":"now","agent":"x","event":"pause"}"#;
        let entry: ActivityEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.detail, "");
    }

    #[test]
    fn empty_agents_array() {
        let json = r#"{"agents":[]}"#;
        let state: SidebarState = serde_json::from_str(json).unwrap();
        assert!(state.agents.is_empty());
    }

    #[test]
    fn demo_state_produces_valid_data() {
        let (state, files, beads, activity, tree, metrics) = demo_state();
        assert_eq!(state.agents.len(), 3);
        assert_eq!(state.agents[0].agent_type, "claude");
        assert_eq!(state.agents[1].agent_type, "codex");
        assert_eq!(state.agents[2].agent_type, "cursor");
        assert!(!files.files.is_empty());
        assert!(files.files[0].conflict);
        assert_eq!(beads.issues.len(), 3);
        assert_eq!(activity.len(), 3);
        assert!(!tree.entries.is_empty());
        assert!(tree.entries.iter().any(|e| e.is_dir));
        assert!(!metrics.claude.models.is_empty());
        assert_eq!(metrics.agents.len(), 3);

        // Verify demo state roundtrips through JSON
        let json = serde_json::to_string(&state).unwrap();
        let _: SidebarState = serde_json::from_str(&json).unwrap();
    }
}

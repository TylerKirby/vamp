use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Command {
    pub fn create(agent_type: &str, name: &str) -> Self {
        Self {
            action: "create".to_string(),
            id: None,
            agent_type: Some(agent_type.to_string()),
            name: Some(name.to_string()),
        }
    }

    pub fn kill(id: &str) -> Self {
        Self {
            action: "kill".to_string(),
            id: Some(id.to_string()),
            agent_type: None,
            name: None,
        }
    }

    pub fn focus(id: &str) -> Self {
        Self {
            action: "focus".to_string(),
            id: Some(id.to_string()),
            agent_type: None,
            name: None,
        }
    }

    pub fn pause(id: &str) -> Self {
        Self {
            action: "pause".to_string(),
            id: Some(id.to_string()),
            agent_type: None,
            name: None,
        }
    }

    pub fn restart(id: &str) -> Self {
        Self {
            action: "restart".to_string(),
            id: Some(id.to_string()),
            agent_type: None,
            name: None,
        }
    }

    pub fn rename(id: &str, new_name: &str) -> Self {
        Self {
            action: "rename".to_string(),
            id: Some(id.to_string()),
            agent_type: None,
            name: Some(new_name.to_string()),
        }
    }

    pub fn merge(id: &str) -> Self {
        Self {
            action: "merge".to_string(),
            id: Some(id.to_string()),
            agent_type: None,
            name: None,
        }
    }
}

/// Append a command to the commands file using atomic write
pub fn send_command(commands_file: &Path, cmd: Command) -> std::io::Result<()> {
    // Read existing commands
    let mut commands: Vec<Command> = if commands_file.exists() {
        let contents = std::fs::read_to_string(commands_file)?;
        serde_json::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Warning: failed to parse {}: {}", commands_file.display(), e);
            Vec::new()
        })
    } else {
        Vec::new()
    };

    commands.push(cmd);

    // Atomic write: temp file + rename
    let tmp = commands_file.with_extension("tmp");
    let json = serde_json::to_string_pretty(&commands)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, commands_file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_command_json() {
        let cmd = Command::create("claude", "miles");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "create");
        assert_eq!(v["type"], "claude");
        assert_eq!(v["name"], "miles");
        assert!(v.get("id").is_none());
    }

    #[test]
    fn kill_command_json() {
        let cmd = Command::kill("agent-001");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "kill");
        assert_eq!(v["id"], "agent-001");
        assert!(v.get("type").is_none());
        assert!(v.get("name").is_none());
    }

    #[test]
    fn focus_command_json() {
        let cmd = Command::focus("agent-002");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "focus");
        assert_eq!(v["id"], "agent-002");
    }

    #[test]
    fn pause_command_json() {
        let cmd = Command::pause("agent-003");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "pause");
        assert_eq!(v["id"], "agent-003");
    }

    #[test]
    fn restart_command_json() {
        let cmd = Command::restart("agent-001");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "restart");
        assert_eq!(v["id"], "agent-001");
    }

    #[test]
    fn rename_command_json() {
        let cmd = Command::rename("agent-001", "dizzy");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "rename");
        assert_eq!(v["id"], "agent-001");
        assert_eq!(v["name"], "dizzy");
        assert!(v.get("type").is_none());
    }

    #[test]
    fn merge_command_json() {
        let cmd = Command::merge("--all");
        let json = serde_json::to_string(&cmd).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["action"], "merge");
        assert_eq!(v["id"], "--all");
    }

    #[test]
    fn send_command_creates_file() {
        let dir = std::env::temp_dir().join(format!("vamp-test-cmd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmd_file = dir.join("commands.json");

        let result = send_command(&cmd_file, Command::create("claude", "test"));
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&cmd_file).unwrap();
        let cmds: Vec<Command> = serde_json::from_str(&contents).unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].action, "create");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn send_command_appends_to_existing() {
        let dir = std::env::temp_dir().join(format!("vamp-test-append-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmd_file = dir.join("commands.json");

        send_command(&cmd_file, Command::create("claude", "first")).unwrap();
        send_command(&cmd_file, Command::kill("agent-001")).unwrap();

        let contents = std::fs::read_to_string(&cmd_file).unwrap();
        let cmds: Vec<Command> = serde_json::from_str(&contents).unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].action, "create");
        assert_eq!(cmds[1].action, "kill");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn send_command_handles_missing_file() {
        let dir = std::env::temp_dir().join(format!("vamp-test-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmd_file = dir.join("nonexistent.json");

        let result = send_command(&cmd_file, Command::focus("a1"));
        assert!(result.is_ok());

        let cmds: Vec<Command> = serde_json::from_str(
            &std::fs::read_to_string(&cmd_file).unwrap()
        ).unwrap();
        assert_eq!(cmds.len(), 1);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn send_command_handles_malformed_file() {
        let dir = std::env::temp_dir().join(format!("vamp-test-malformed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmd_file = dir.join("commands.json");

        // Write garbage to the file
        std::fs::write(&cmd_file, "not valid json!!!").unwrap();

        let result = send_command(&cmd_file, Command::kill("a1"));
        assert!(result.is_ok());

        let cmds: Vec<Command> = serde_json::from_str(
            &std::fs::read_to_string(&cmd_file).unwrap()
        ).unwrap();
        // Should have recovered — malformed content discarded, new command written
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].action, "kill");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn send_command_cleans_up_temp_file() {
        let dir = std::env::temp_dir().join(format!("vamp-test-tmp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmd_file = dir.join("commands.json");
        let tmp_file = dir.join("commands.tmp");

        send_command(&cmd_file, Command::create("codex", "test")).unwrap();

        // Temp file should not exist after successful rename
        assert!(!tmp_file.exists());
        assert!(cmd_file.exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn skip_serializing_none_fields() {
        let cmd = Command::kill("a1");
        let json = serde_json::to_string(&cmd).unwrap();
        // None fields should be omitted
        assert!(!json.contains("type"));
        assert!(!json.contains("name"));
    }

    #[test]
    fn command_roundtrip() {
        let original = Command::create("cursor", "monk");
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.action, "create");
        assert_eq!(parsed.agent_type.as_deref(), Some("cursor"));
        assert_eq!(parsed.name.as_deref(), Some("monk"));
        assert!(parsed.id.is_none());
    }
}

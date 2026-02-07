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
}

/// Append a command to the commands file using atomic write
pub fn send_command(commands_file: &Path, cmd: Command) -> std::io::Result<()> {
    // Read existing commands
    let mut commands: Vec<Command> = if commands_file.exists() {
        let contents = std::fs::read_to_string(commands_file)?;
        serde_json::from_str(&contents).unwrap_or_default()
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

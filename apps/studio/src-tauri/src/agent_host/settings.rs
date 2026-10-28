use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    Autonomous,
    AskRisky,
    AskAll,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Budget {
    pub max_tokens: i64,
    pub max_cost_usd: f64,
    pub max_turns: i64,
    pub usage: BudgetUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSettings {
    pub permission_mode: PermissionMode,
    pub model_roles: HashMap<String, String>,
    pub budgets: Budget,
}

impl Default for AgentSettings {
    fn default() -> Self {
        let mut model_roles = HashMap::new();
        model_roles.insert("primary".to_string(), "gemini-2.5-pro".to_string());
        model_roles.insert("fast".to_string(), "gemini-2.5-flash".to_string());

        Self {
            permission_mode: PermissionMode::AskRisky,
            model_roles,
            budgets: Budget {
                max_tokens: 100_000,
                max_cost_usd: 1.0,
                max_turns: 25,
                usage: BudgetUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                    estimated_cost_usd: 0.0,
                },
            },
        }
    }
}

/// Resolves user settings path: %APPDATA%/Forge/agent_settings.json
pub fn get_user_settings_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("Forge").join("agent_settings.json")
    } else {
        PathBuf::from(".forge_user_settings.json")
    }
}

/// Resolves project settings path: <project_path>/.forge/settings.json
pub fn get_project_settings_path(project_path: &str) -> PathBuf {
    Path::new(project_path).join(".forge").join("settings.json")
}

/// Reads effective AgentSettings for a project:
/// Default -> Overridden by user settings -> Overridden by project settings.
pub fn read_settings(project_path: Option<&str>) -> Result<AgentSettings, String> {
    let mut settings = AgentSettings::default();

    // 1. Try reading user settings
    let user_path = get_user_settings_path();
    if user_path.exists() {
        if let Ok(content) = fs::read_to_string(&user_path) {
            if let Ok(val) = serde_json::from_str::<AgentSettings>(&content) {
                settings = val;
            }
        }
    }

    // 2. Try reading project settings (.forge/settings.json)
    if let Some(proj) = project_path {
        let proj_path = get_project_settings_path(proj);
        if proj_path.exists() {
            if let Ok(content) = fs::read_to_string(&proj_path) {
                if let Ok(val) = serde_json::from_str::<AgentSettings>(&content) {
                    settings = val;
                }
            }
        }
    }

    Ok(settings)
}

/// Writes AgentSettings:
/// If `project_path` is provided, writes to `<project_path>/.forge/settings.json`.
/// Otherwise, writes to user global settings.
pub fn write_settings(
    settings: &AgentSettings,
    project_path: Option<&str>,
) -> Result<(), String> {
    let target_path = match project_path {
        Some(proj) => {
            let dir = Path::new(proj).join(".forge");
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create .forge directory: {e}"))?;
            dir.join("settings.json")
        }
        None => {
            let path = get_user_settings_path();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create user config directory: {e}"))?;
            }
            path
        }
    };

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;

    fs::write(&target_path, content)
        .map_err(|e| format!("Failed to write settings to {}: {e}", target_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_serialization() {
        let default_s = AgentSettings::default();
        let json = serde_json::to_string(&default_s).unwrap();
        let parsed: AgentSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(default_s, parsed);
        assert_eq!(parsed.permission_mode, PermissionMode::AskRisky);
        assert_eq!(parsed.budgets.max_turns, 25);
    }
}

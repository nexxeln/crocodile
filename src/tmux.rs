use crate::error::{CrocError, Result};
use std::process::Command;
use tracing::{debug, info};

pub struct TmuxSession {
    name: String,
}

impl TmuxSession {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn spawn(&self, command: &str) -> Result<()> {
        debug!(session = %self.name, command = %command, "Spawning tmux session");

        let output = Command::new("tmux")
            .args(["new-session", "-d", "-s", &self.name, command])
            .output()
            .map_err(|e| CrocError::Tmux {
                message: format!("Failed to spawn session '{}': {}", self.name, e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CrocError::Tmux {
                message: format!("tmux new-session failed: {}", stderr),
            });
        }

        info!(session = %self.name, "Spawned tmux session");
        Ok(())
    }

    pub fn exists(&self) -> Result<bool> {
        let status = Command::new("tmux")
            .args(["has-session", "-t", &self.name])
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| CrocError::Tmux {
                message: format!("Failed to check session '{}': {}", self.name, e),
            })?;

        Ok(status.success())
    }

    pub fn send_keys(&self, keys: &str) -> Result<()> {
        debug!(session = %self.name, keys = %keys, "Sending keys to tmux session");

        let output = Command::new("tmux")
            .args(["send-keys", "-t", &self.name, keys, "Enter"])
            .output()
            .map_err(|e| CrocError::Tmux {
                message: format!("Failed to send keys to '{}': {}", self.name, e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CrocError::Tmux {
                message: format!("tmux send-keys failed: {}", stderr),
            });
        }

        Ok(())
    }

    pub fn capture_pane(&self) -> Result<String> {
        let output = Command::new("tmux")
            .args(["capture-pane", "-t", &self.name, "-p"])
            .output()
            .map_err(|e| CrocError::Tmux {
                message: format!("Failed to capture pane '{}': {}", self.name, e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CrocError::Tmux {
                message: format!("tmux capture-pane failed: {}", stderr),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn kill(&self) -> Result<()> {
        debug!(session = %self.name, "Killing tmux session");

        let output = Command::new("tmux")
            .args(["kill-session", "-t", &self.name])
            .output()
            .map_err(|e| CrocError::Tmux {
                message: format!("Failed to kill session '{}': {}", self.name, e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CrocError::Tmux {
                message: format!("tmux kill-session failed: {}", stderr),
            });
        }

        info!(session = %self.name, "Killed tmux session");
        Ok(())
    }

    pub fn attach(&self) -> Result<()> {
        info!(session = %self.name, "Attaching to tmux session");

        let status = Command::new("tmux")
            .args(["attach-session", "-t", &self.name])
            .status()
            .map_err(|e| CrocError::Tmux {
                message: format!("Failed to attach to '{}': {}", self.name, e),
            })?;

        if !status.success() {
            return Err(CrocError::Tmux {
                message: format!("tmux attach-session failed for '{}'", self.name),
            });
        }

        Ok(())
    }
}

pub fn list_sessions() -> Result<Vec<String>> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .map_err(|e| CrocError::Tmux {
            message: format!("Failed to list sessions: {}", e),
        })?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(String::from).collect())
}

pub fn find_croc_sessions() -> Result<Vec<String>> {
    let sessions = list_sessions()?;
    Ok(sessions
        .into_iter()
        .filter(|s| s.starts_with("croc-"))
        .collect())
}

pub fn foreman_session_name(plan_id: &str) -> String {
    let id = plan_id.strip_prefix("plan-").unwrap_or(plan_id);
    format!("croc-foreman-{}", id)
}

pub fn worker_session_name(plan_id: &str, task_id: &str) -> String {
    let plan = plan_id.strip_prefix("plan-").unwrap_or(plan_id);
    let task = task_id
        .strip_prefix("task-")
        .and_then(|t| t.split('.').next_back())
        .unwrap_or(task_id);
    format!("croc-worker-{}-{}", plan, task)
}

pub fn reviewer_session_name(plan_id: &str) -> String {
    let id = plan_id.strip_prefix("plan-").unwrap_or(plan_id);
    format!("croc-reviewer-{}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreman_session_name_strips_prefix() {
        assert_eq!(foreman_session_name("plan-abc123"), "croc-foreman-abc123");
    }

    #[test]
    fn foreman_session_name_handles_no_prefix() {
        assert_eq!(foreman_session_name("abc123"), "croc-foreman-abc123");
    }

    #[test]
    fn worker_session_name_extracts_last_segment_after_dot() {
        assert_eq!(
            worker_session_name("plan-abc123", "task-abc123.1"),
            "croc-worker-abc123-1"
        );
    }

    #[test]
    fn worker_session_name_extracts_last_segment_from_nested_task_id() {
        assert_eq!(
            worker_session_name("plan-abc123", "task-abc123.1.2"),
            "croc-worker-abc123-2"
        );
    }

    #[test]
    fn worker_session_name_handles_no_prefixes() {
        assert_eq!(
            worker_session_name("abc123", "def456"),
            "croc-worker-abc123-def456"
        );
    }

    #[test]
    fn reviewer_session_name_strips_prefix() {
        assert_eq!(reviewer_session_name("plan-abc123"), "croc-reviewer-abc123");
    }
}

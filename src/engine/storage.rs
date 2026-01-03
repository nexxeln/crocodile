use crate::config::Config;
use crate::error::{CrocError, Result};
use crate::schemas::{ContextItem, Event, Plan, Review, Task};
use fs2::FileExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use tracing::debug;

pub struct Storage {
    config: Config,
}

impl Storage {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn append_jsonl_locked<T: Serialize>(&self, path: &Path, record: &T) -> Result<()> {
        debug!(path = %path.display(), "Appending to JSONL with lock");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| CrocError::Storage {
                message: format!("Failed to open {}: {}", path.display(), e),
            })?;

        file.lock_exclusive().map_err(|e| CrocError::Storage {
            message: format!("Failed to lock {}: {}", path.display(), e),
        })?;

        let json = serde_json::to_string(record)?;
        writeln!(file, "{}", json).map_err(|e| CrocError::Storage {
            message: format!("Failed to write to {}: {}", path.display(), e),
        })?;

        file.unlock().map_err(|e| CrocError::Storage {
            message: format!("Failed to unlock {}: {}", path.display(), e),
        })?;

        Ok(())
    }

    pub fn read_jsonl<T: DeserializeOwned>(&self, path: &Path) -> Result<Vec<T>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path).map_err(|e| CrocError::Storage {
            message: format!("Failed to open {}: {}", path.display(), e),
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| CrocError::Storage {
                message: format!("Failed to read line from {}: {}", path.display(), e),
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let record: T = serde_json::from_str(&line)?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn create_empty_file(&self, path: &Path) -> Result<()> {
        File::create(path).map_err(|e| CrocError::Storage {
            message: format!("Failed to create {}: {}", path.display(), e),
        })?;
        Ok(())
    }

    pub fn append_plan(&self, plan: &Plan) -> Result<()> {
        self.append_jsonl_locked(&self.config.plans_file(), plan)
    }

    pub fn append_task(&self, task: &Task) -> Result<()> {
        self.append_jsonl_locked(&self.config.tasks_file(), task)
    }

    pub fn append_context(&self, context: &ContextItem) -> Result<()> {
        self.append_jsonl_locked(&self.config.context_file(), context)
    }

    pub fn append_event(&self, event: &Event) -> Result<()> {
        self.append_jsonl_locked(&self.config.events_file(), event)
    }

    pub fn append_review(&self, review: &Review) -> Result<()> {
        self.append_jsonl_locked(&self.config.reviews_file(), review)
    }

    pub fn read_plans(&self) -> Result<Vec<Plan>> {
        self.read_jsonl(&self.config.plans_file())
    }

    pub fn read_tasks(&self) -> Result<Vec<Task>> {
        self.read_jsonl(&self.config.tasks_file())
    }

    pub fn read_context(&self) -> Result<Vec<ContextItem>> {
        self.read_jsonl(&self.config.context_file())
    }

    pub fn read_events(&self) -> Result<Vec<Event>> {
        self.read_jsonl(&self.config.events_file())
    }

    pub fn read_reviews(&self) -> Result<Vec<Review>> {
        self.read_jsonl(&self.config.reviews_file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(dir: &TempDir) -> Config {
        Config {
            croc_dir: dir.path().to_path_buf(),
        }
    }

    #[test]
    fn jsonl_round_trip_preserves_plan_data() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(test_config(&dir));
        let path = dir.path().join("test.jsonl");

        let plan = Plan::new(
            "plan-test123".to_string(),
            "Test Plan".to_string(),
            "A test plan".to_string(),
        );

        storage.append_jsonl_locked(&path, &plan).unwrap();
        let plans: Vec<Plan> = storage.read_jsonl(&path).unwrap();

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].id, "plan-test123");
        assert_eq!(plans[0].title, "Test Plan");
    }

    #[test]
    fn jsonl_appends_multiple_records() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(test_config(&dir));
        let path = dir.path().join("test.jsonl");

        for i in 0..3 {
            let plan = Plan::new(
                format!("plan-{}", i),
                format!("Plan {}", i),
                "desc".to_string(),
            );
            storage.append_jsonl_locked(&path, &plan).unwrap();
        }

        let plans: Vec<Plan> = storage.read_jsonl(&path).unwrap();
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].id, "plan-0");
        assert_eq!(plans[2].id, "plan-2");
    }

    #[test]
    fn read_jsonl_returns_empty_vec_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(test_config(&dir));
        let path = dir.path().join("nonexistent.jsonl");

        let plans: Vec<Plan> = storage.read_jsonl(&path).unwrap();
        assert!(plans.is_empty());
    }

    #[test]
    fn read_jsonl_skips_empty_lines() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(test_config(&dir));
        let path = dir.path().join("test.jsonl");

        let plan = Plan::new("plan-1".to_string(), "Plan".to_string(), "desc".to_string());
        storage.append_jsonl_locked(&path, &plan).unwrap();

        std::fs::write(
            &path,
            format!(
                "{}\n\n{}\n",
                serde_json::to_string(&plan).unwrap(),
                serde_json::to_string(&plan).unwrap()
            ),
        )
        .unwrap();

        let plans: Vec<Plan> = storage.read_jsonl(&path).unwrap();
        assert_eq!(plans.len(), 2);
    }
}

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub croc_dir: PathBuf,
}

impl Config {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            croc_dir: project_root.join(".croc"),
        }
    }

    pub fn from_current_dir() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        Ok(Self::new(cwd))
    }

    pub fn plans_file(&self) -> PathBuf {
        self.croc_dir.join("plans.jsonl")
    }

    pub fn tasks_file(&self) -> PathBuf {
        self.croc_dir.join("tasks.jsonl")
    }

    pub fn context_file(&self) -> PathBuf {
        self.croc_dir.join("context.jsonl")
    }

    pub fn events_file(&self) -> PathBuf {
        self.croc_dir.join("events.jsonl")
    }

    pub fn reviews_file(&self) -> PathBuf {
        self.croc_dir.join("reviews.jsonl")
    }

    pub fn checkpoints_dir(&self) -> PathBuf {
        self.croc_dir.join("checkpoints")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.croc_dir.join("logs")
    }

    pub fn gitignore_file(&self) -> PathBuf {
        self.croc_dir.join(".gitignore")
    }

    pub fn is_initialized(&self) -> bool {
        self.croc_dir.exists()
    }
}

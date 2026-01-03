use crate::cli::InitArgs;
use crate::config::Config;
use crate::engine::Storage;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

#[derive(Serialize)]
struct InitEvent {
    id: String,
    r#type: &'static str,
    timestamp: String,
}

pub async fn exec(args: InitArgs) -> Result<()> {
    let project_root = args.path.unwrap_or(std::env::current_dir()?);

    let config = Config::new(project_root.clone());

    if config.is_initialized() {
        println!("Already initialized at {}", config.croc_dir.display());
        return Ok(());
    }

    debug!(path = %config.croc_dir.display(), "Creating .croc directory");
    fs::create_dir_all(&config.croc_dir).context("Failed to create .croc directory")?;

    fs::create_dir_all(config.checkpoints_dir())
        .context("Failed to create checkpoints directory")?;

    fs::create_dir_all(config.logs_dir()).context("Failed to create logs directory")?;

    let storage = Storage::new(config.clone());

    create_empty_jsonl(&storage, &config.plans_file())?;
    create_empty_jsonl(&storage, &config.tasks_file())?;
    create_empty_jsonl(&storage, &config.context_file())?;
    create_empty_jsonl(&storage, &config.events_file())?;
    create_empty_jsonl(&storage, &config.reviews_file())?;

    write_gitignore(&config.gitignore_file())?;

    let init_event = InitEvent {
        id: format!("evt-{}", Utc::now().timestamp_millis()),
        r#type: "initialized",
        timestamp: Utc::now().to_rfc3339(),
    };
    storage.append_jsonl_locked(&config.events_file(), &init_event)?;

    check_git_repo(&project_root);

    info!(path = %config.croc_dir.display(), "Initialized");
    println!("Initialized crocodile at {}", config.croc_dir.display());

    Ok(())
}

fn create_empty_jsonl(storage: &Storage, path: &Path) -> Result<()> {
    debug!(file = %path.display(), "Creating JSONL file");
    storage
        .create_empty_file(path)
        .context(format!("Failed to create {}", path.display()))?;
    Ok(())
}

fn write_gitignore(path: &Path) -> Result<()> {
    debug!(file = %path.display(), "Writing .gitignore");
    fs::write(path, "cache.db\nworktrees/\nlogs/\n").context("Failed to write .gitignore")?;
    Ok(())
}

fn check_git_repo(path: &Path) {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        tracing::warn!(
            path = %path.display(),
            "Not a git repository. Some features may not work."
        );
    }
}

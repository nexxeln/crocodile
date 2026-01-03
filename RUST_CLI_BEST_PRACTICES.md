# Rust CLI Best Practices & Architecture Guide

A comprehensive guide for building production-quality Rust CLI applications with modern patterns, focusing on the Crocodile project's multi-command architecture (`croc init`, `croc plan`, `croc prime`).

---

## Table of Contents

1. [CLI Parsing with `clap`](#cli-parsing-with-clap)
2. [Project Structure](#project-structure)
3. [Error Handling](#error-handling)
4. [Environment Variables with `dotenvy`](#environment-variables-with-dotenvy)
5. [Structured Logging with `tracing`](#structured-logging-with-tracing)
6. [Progress Reporting with `indicatif`](#progress-reporting-with-indicatif)
7. [Command Registration & Routing](#command-registration--routing)
8. [Complete Example: Crocodile CLI](#complete-example-crocodile-cli)

---

## CLI Parsing with `clap`

### Modern Pattern: Derive API (Recommended)

Use `clap`'s **derive API** (not builder pattern) for 2024+ Rust CLIs. It's cleaner, more maintainable, and the modern standard.

### Key Principles

1. **Top-level `App` struct** (not enum) - allows adding global options later without refactoring
2. **Subcommands in an enum** - use `#[clap(subcommand)]` to delegate to `Command` enum
3. **Flatten reusable option groups** - use `#[clap(flatten)]` to compose options
4. **Global options marked explicitly** - use `#[clap(global = true)]` for options available everywhere

### Recommended Structure

```rust
use clap::{Parser, Subcommand, Args};

/// Crocodile: AI agent orchestrator
#[derive(Debug, Parser)]
#[clap(name = "croc", version, about)]
pub struct App {
	/// Global options
	#[clap(flatten)]
	pub global: GlobalOpts,

	/// Subcommand to execute
	#[clap(subcommand)]
	pub command: Command,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
	/// Verbosity level (can be specified multiple times: -v, -vv, -vvv)
	#[clap(short, long, global = true, action = clap::ArgAction::Count)]
	pub verbose: u8,

	/// Color output
	#[clap(long, global = true, default_value = "auto")]
	pub color: ColorMode,

	/// Configuration file path
	#[clap(short, long, global = true)]
	pub config: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ColorMode {
	Always,
	Auto,
	Never,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	/// Initialize a new Crocodile project
	Init(InitArgs),

	/// Plan the next phase of work
	Plan(PlanArgs),

	/// Prime the engine with context
	Prime(PrimeArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
	/// Project name
	#[clap(value_name = "NAME")]
	pub name: String,

	/// Project directory (defaults to current directory)
	#[clap(short, long)]
	pub dir: Option<std::path::PathBuf>,

	/// Template to use
	#[clap(short, long, default_value = "default")]
	pub template: String,
}

#[derive(Debug, Args)]
pub struct PlanArgs {
	/// Path to project
	#[clap(value_name = "PATH")]
	pub project: std::path::PathBuf,

	/// Force re-planning even if plan exists
	#[clap(short, long)]
	pub force: bool,
}

#[derive(Debug, Args)]
pub struct PrimeArgs {
	/// Path to project
	#[clap(value_name = "PATH")]
	pub project: std::path::PathBuf,

	/// Context files to ingest
	#[clap(short, long)]
	pub context: Vec<std::path::PathBuf>,
}

fn main() -> anyhow::Result<()> {
	let app = App::parse();
	
	// Initialize logging, error handling, etc.
	setup_logging(&app.global)?;
	
	// Execute the appropriate command
	match app.command {
		Command::Init(args) => commands::init::exec(args),
		Command::Plan(args) => commands::plan::exec(args),
		Command::Prime(args) => commands::prime::exec(args),
	}
}
```

### Best Practices for `clap`

| Pattern | Benefit |
|---------|---------|
| Use `#[clap(flatten)]` | Reuse option groups across commands |
| Use `#[clap(global = true)]` | Make options available to all subcommands |
| Use `clap::ValueEnum` | Type-safe enum arguments |
| Use `action = clap::ArgAction::Count` | Count flag occurrences (e.g., `-vvv`) |
| Derive `Debug` on all structs | Easier debugging and logging |
| Use `value_name` | Better help text for positional args |

---

## Project Structure

### Recommended Layout for Multi-Command CLI

```
crocodile/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point (minimal)
│   ├── lib.rs                  # Library exports
│   ├── cli/
│   │   ├── mod.rs              # CLI parsing (App, Command enums)
│   │   ├── args.rs             # Argument structs
│   │   └── global.rs           # Global options
│   ├── commands/
│   │   ├── mod.rs              # Command module exports
│   │   ├── init.rs             # `croc init` implementation
│   │   ├── plan.rs             # `croc plan` implementation
│   │   └── prime.rs            # `croc prime` implementation
│   ├── engine/
│   │   ├── mod.rs              # Core engine logic
│   │   ├── croc_engine.rs       # CrocEngine implementation
│   │   └── storage.rs          # JSONL/SQLite storage
│   ├── error.rs                # Error types (thiserror)
│   ├── logging.rs              # Logging setup (tracing)
│   └── config.rs               # Configuration loading
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/
└── .env.example
```

### Why This Structure?

- **`cli/` module**: Keeps argument parsing separate from logic
- **`commands/` module**: Each command is independent, easy to test
- **`engine/` module**: Core business logic, reusable
- **`error.rs`**: Centralized error types
- **`logging.rs`**: Centralized logging setup
- **Minimal `main.rs`**: Only parses args and delegates

### `src/lib.rs` Pattern

```rust
//! Crocodile: AI agent orchestrator
//!
//! This library provides the core engine and CLI interface for orchestrating
//! AI agents in a human-in-the-loop workflow.

pub mod cli;
pub mod commands;
pub mod engine;
pub mod error;
pub mod logging;
pub mod config;

// Only export the top-level App struct
#[doc(hidden)]
pub use cli::App;
```

### `src/main.rs` Pattern

```rust
use clap::Parser;
use crocodile::App;

fn main() -> anyhow::Result<()> {
	// Install error handling (color_eyre for pretty errors)
	color_eyre::install()?;

	// Parse CLI arguments
	let app = App::parse();

	// Initialize logging based on verbosity
	crocodile::logging::init(&app.global)?;

	// Load configuration (env vars, config files, etc.)
	let config = crocodile::config::load(&app.global)?;

	// Execute the command
	app.execute(config).await
}
```

---

## Error Handling

### Pattern: `thiserror` + `anyhow`

- **`thiserror`**: Define custom error types for your domain
- **`anyhow`**: Use in `main()` and CLI functions for ergonomic error handling

### Custom Error Type Example

```rust
// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrocError {
	#[error("Project not found at {path}")]
	ProjectNotFound { path: String },

	#[error("Invalid configuration: {reason}")]
	InvalidConfig { reason: String },

	#[error("Engine error: {0}")]
	EngineError(String),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("JSON error: {0}")]
	Json(#[from] serde_json::Error),

	#[error("Database error: {0}")]
	Database(String),
}

pub type Result<T> = std::result::Result<T, CrocError>;
```

### Usage in Commands

```rust
// src/commands/init.rs
use anyhow::{Context, Result};
use crate::error::CrocError;

pub async fn exec(args: InitArgs) -> Result<()> {
	// Use anyhow::Result for ergonomic error handling
	let project_dir = args.dir
		.unwrap_or_else(|| std::env::current_dir().unwrap());

	// Add context to errors
	std::fs::create_dir_all(&project_dir)
		.context("Failed to create project directory")?;

	// Convert custom errors
	validate_project_name(&args.name)
		.map_err(|e| anyhow::anyhow!("Invalid project name: {}", e))?;

	println!("Project initialized at {}", project_dir.display());
	Ok(())
}

fn validate_project_name(name: &str) -> crate::error::Result<()> {
	if name.is_empty() {
		return Err(CrocError::InvalidConfig {
			reason: "Project name cannot be empty".to_string(),
		});
	}
	Ok(())
}
```

### Error Handling in `main()`

```rust
fn main() -> anyhow::Result<()> {
	color_eyre::install()?;

	let app = App::parse();
	crocodile::logging::init(&app.global)?;

	// Errors are automatically formatted nicely by color_eyre
	match app.command {
		Command::Init(args) => commands::init::exec(args),
		Command::Plan(args) => commands::plan::exec(args),
		Command::Prime(args) => commands::prime::exec(args),
	}
}
```

### Best Practices

| Pattern | Benefit |
|---------|---------|
| Use `#[from]` on error variants | Automatic `From` impl for conversion |
| Use `#[error(...)]` | Automatic `Display` impl |
| Use `.context()` | Add contextual information to errors |
| Use `bail!` macro | Quick error returns with formatting |
| Return `Result<T>` | Propagate errors with `?` operator |
| Use `color_eyre` | Pretty error output with backtraces |

---

## Environment Variables with `dotenvy`

### Setup Pattern

```rust
// src/config.rs
use anyhow::Result;
use std::path::PathBuf;

pub struct Config {
	pub log_level: String,
	pub engine_db_path: PathBuf,
	pub context_dir: PathBuf,
	pub api_key: Option<String>,
}

impl Config {
	/// Load configuration from environment, .env file, and defaults
	pub fn load(global_opts: &crate::cli::GlobalOpts) -> Result<Self> {
		// Load .env file (fails silently if not found)
		dotenvy::dotenv().ok();

		// Determine log level from verbosity flag or env var
		let log_level = match global_opts.verbose {
			0 => std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
			1 => "debug".to_string(),
			_ => "trace".to_string(),
		};

		// Load paths from env or use defaults
		let engine_db_path = std::env::var("CROC_DB_PATH")
			.map(PathBuf::from)
			.unwrap_or_else(|_| {
				let home = dirs::home_dir().expect("Could not determine home directory");
				home.join(".croc").join("engine.db")
			});

		let context_dir = std::env::var("CROC_CONTEXT_DIR")
			.map(PathBuf::from)
			.unwrap_or_else(|_| {
				let home = dirs::home_dir().expect("Could not determine home directory");
				home.join(".croc").join("context")
			});

		// Load optional API key (never log this!)
		let api_key = std::env::var("CROC_API_KEY").ok();

		Ok(Config {
			log_level,
			engine_db_path,
			context_dir,
			api_key,
		})
	}
}
```

### `.env.example` File

```bash
# Logging
RUST_LOG=info

# Engine configuration
CROC_DB_PATH=~/.croc/engine.db
CROC_CONTEXT_DIR=~/.croc/context

# API configuration (never commit actual keys!)
# CROC_API_KEY=your_key_here
```

### Best Practices

| Pattern | Benefit |
|---------|---------|
| Call `dotenvy::dotenv().ok()` early | Load .env before parsing config |
| Use `.ok()` to ignore missing .env | Graceful fallback to system env vars |
| Provide sensible defaults | CLI works without .env file |
| Never log sensitive values | Prevent accidental credential leaks |
| Use `Option<String>` for optional vars | Type-safe optional configuration |
| Document all env vars | Help users understand configuration |

---

## Structured Logging with `tracing`

### Setup Pattern

```rust
// src/logging.rs
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use anyhow::Result;

pub fn init(global_opts: &crate::cli::GlobalOpts) -> Result<()> {
	// Determine log level from verbosity
	let log_level = match global_opts.verbose {
		0 => "info",
		1 => "debug",
		_ => "trace",
	};

	// Build the env filter
	let env_filter = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(log_level))?;

	// Build the fmt layer
	let fmt_layer = fmt::layer()
		.with_writer(std::io::stderr)
		.with_target(global_opts.verbose > 0)
		.with_thread_ids(global_opts.verbose > 1)
		.with_line_number(global_opts.verbose > 1);

	// Compose and initialize
	tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt_layer)
		.init();

	Ok(())
}
```

### Usage in Commands

```rust
// src/commands/init.rs
use tracing::{info, debug, warn};

pub async fn exec(args: InitArgs) -> anyhow::Result<()> {
	info!("Initializing project: {}", args.name);
	debug!(template = %args.template, "Using template");

	let project_dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());

	// Structured logging with fields
	debug!(
		path = %project_dir.display(),
		"Creating project directory"
	);

	std::fs::create_dir_all(&project_dir)?;

	info!(
		name = %args.name,
		path = %project_dir.display(),
		"Project initialized successfully"
	);

	Ok(())
}
```

### Structured Logging Macros

```rust
// Basic logging
info!("Project initialized");
debug!("Processing file");
warn!("Deprecated option used");

// With structured fields
info!(
	project_name = %name,
	file_count = count,
	"Processed project"
);

// With debug formatting
debug!(
	config = ?config_struct,
	"Configuration loaded"
);

// With error context
warn!(
	error = %err,
	"Failed to load config, using defaults"
);
```

### Integration with `indicatif`

Use `tracing-indicatif` to automatically manage progress bars:

```rust
// Cargo.toml
[dependencies]
tracing-indicatif = "0.3"

// src/logging.rs
use tracing_indicatif::IndicatifLayer;

pub fn init(global_opts: &crate::cli::GlobalOpts) -> Result<()> {
	let env_filter = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(log_level))?;

	let fmt_layer = fmt::layer()
		.with_writer(std::io::stderr);

	// Add indicatif layer for progress bars
	let indicatif_layer = IndicatifLayer::new();

	tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt_layer)
		.with(indicatif_layer)
		.init();

	Ok(())
}
```

### Best Practices

| Pattern | Benefit |
|---------|---------|
| Use structured fields | Easier to parse and filter logs |
| Use `%` for Display formatting | Cleaner output than `?` (Debug) |
| Use `?` for Debug formatting | Full details for complex types |
| Log to stderr | Keeps stdout clean for output |
| Use `RUST_LOG` env var | Control verbosity at runtime |
| Use `#[instrument]` macro | Automatic span creation for functions |

---

## Progress Reporting with `indicatif`

### Basic Progress Bar

```rust
use indicatif::{ProgressBar, ProgressStyle};

pub async fn exec(args: PrimeArgs) -> anyhow::Result<()> {
	let files = collect_context_files(&args.context)?;

	// Create progress bar
	let pb = ProgressBar::new(files.len() as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
			.progress_chars("#>-")
	);

	for file in files {
		pb.set_message(format!("Processing {}", file.display()));
		process_file(&file).await?;
		pb.inc(1);
	}

	pb.finish_with_message("Context primed successfully");
	Ok(())
}
```

### Multi-Progress with Nested Operations

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub async fn exec(args: PrimeArgs) -> anyhow::Result<()> {
	let multi = MultiProgress::new();

	// Parent progress bar
	let parent = multi.add(ProgressBar::new(100));
	parent.set_style(
		ProgressStyle::default_bar()
			.template("{spinner:.green} {msg} [{bar:30}] {pos}/{len}")?
	);

	// Child progress bars
	for i in 0..10 {
		let child = multi.add(ProgressBar::new(10));
		child.set_style(
			ProgressStyle::default_bar()
				.template("  {spinner:.blue} Task {pos}/{len}")?
		);

		for _ in 0..10 {
			child.inc(1);
			parent.inc(1);
		}
		child.finish();
	}

	parent.finish_with_message("All tasks completed");
	Ok(())
}
```

### Spinner for Indeterminate Progress

```rust
use indicatif::ProgressBar;

pub async fn exec(args: PlanArgs) -> anyhow::Result<()> {
	let spinner = ProgressBar::new_spinner();
	spinner.set_message("Planning next phase...");

	// Long-running operation
	let plan = generate_plan(&args.project).await?;

	spinner.finish_with_message("Plan generated successfully");
	Ok(())
}
```

### Best Practices

| Pattern | Benefit |
|---------|---------|
| Use `MultiProgress` for nested operations | Clear hierarchy of progress |
| Set meaningful messages | Users understand what's happening |
| Use spinners for indeterminate work | Better UX than silent operations |
| Finish with summary message | Clear completion status |
| Use `{spinner:.color}` | Visual feedback of activity |
| Integrate with `tracing-indicatif` | Automatic progress bar management |

---

## Command Registration & Routing

### Pattern: Trait-Based Command Execution

```rust
// src/commands/mod.rs
pub mod init;
pub mod plan;
pub mod prime;

use anyhow::Result;
use crate::config::Config;

/// Trait for executable commands
pub trait Command: Send + Sync {
	async fn execute(&self, config: &Config) -> Result<()>;
}

// Implement for each command
impl Command for InitArgs {
	async fn execute(&self, config: &Config) -> Result<()> {
		init::exec(self.clone(), config).await
	}
}
```

### Pattern: Direct Enum Matching (Simpler)

```rust
// src/cli/mod.rs
impl App {
	pub async fn execute(self, config: Config) -> anyhow::Result<()> {
		match self.command {
			Command::Init(args) => commands::init::exec(args, &config).await,
			Command::Plan(args) => commands::plan::exec(args, &config).await,
			Command::Prime(args) => commands::prime::exec(args, &config).await,
		}
	}
}
```

### Pattern: Nested Subcommands

For deeper command hierarchies (e.g., `croc deployment list`):

```rust
#[derive(Debug, Subcommand)]
pub enum Command {
	Init(InitArgs),
	Deployment(DeploymentCmd),
}

#[derive(Debug, Subcommand)]
pub enum DeploymentCmd {
	List(DeploymentListArgs),
	Status(DeploymentStatusArgs),
	Delete(DeploymentDeleteArgs),
}

impl App {
	pub async fn execute(self, config: Config) -> anyhow::Result<()> {
		match self.command {
			Command::Init(args) => commands::init::exec(args, &config).await,
			Command::Deployment(cmd) => match cmd {
				DeploymentCmd::List(args) => commands::deployment::list(args, &config).await,
				DeploymentCmd::Status(args) => commands::deployment::status(args, &config).await,
				DeploymentCmd::Delete(args) => commands::deployment::delete(args, &config).await,
			},
		}
	}
}
```

---

## Complete Example: Crocodile CLI

### Full Project Structure

```
crocodile/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   └── args.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── plan.rs
│   │   └── prime.rs
│   ├── engine/
│   │   ├── mod.rs
│   │   └── croc_engine.rs
│   ├── error.rs
│   ├── logging.rs
│   └── config.rs
└── .env.example
```

### `Cargo.toml` Dependencies

```toml
[package]
name = "crocodile"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"

# Configuration
dotenvy = "0.15"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tracing-indicatif = "0.3"

# Progress
indicatif = "0.17"

# Error handling
color-eyre = "0.6"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Utilities
dirs = "5.0"
camino = "1.1"
```

### `src/main.rs`

```rust
use clap::Parser;
use crocodile::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	color_eyre::install()?;

	let app = App::parse();
	crocodile::logging::init(&app.global)?;

	let config = crocodile::config::Config::load(&app.global)?;

	app.execute(config).await
}
```

### `src/lib.rs`

```rust
//! Crocodile: AI agent orchestrator
//!
//! A simplified AI agent orchestrator inspired by Gas Town, focusing on
//! human-in-the-loop workflows with explicit planning and review gates.

pub mod cli;
pub mod commands;
pub mod engine;
pub mod error;
pub mod logging;
pub mod config;

#[doc(hidden)]
pub use cli::App;
```

### `src/cli/mod.rs`

```rust
use clap::{Parser, Subcommand, Args};
use crate::config::Config;

/// Crocodile: AI agent orchestrator
#[derive(Debug, Parser)]
#[clap(name = "croc", version, about)]
pub struct App {
	#[clap(flatten)]
	pub global: GlobalOpts,

	#[clap(subcommand)]
	pub command: Command,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
	#[clap(short, long, global = true, action = clap::ArgAction::Count)]
	pub verbose: u8,

	#[clap(long, global = true, default_value = "auto")]
	pub color: ColorMode,

	#[clap(short, long, global = true)]
	pub config: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ColorMode {
	Always,
	Auto,
	Never,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	/// Initialize a new Crocodile project
	Init(InitArgs),

	/// Plan the next phase of work
	Plan(PlanArgs),

	/// Prime the engine with context
	Prime(PrimeArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
	#[clap(value_name = "NAME")]
	pub name: String,

	#[clap(short, long)]
	pub dir: Option<std::path::PathBuf>,

	#[clap(short, long, default_value = "default")]
	pub template: String,
}

#[derive(Debug, Args)]
pub struct PlanArgs {
	#[clap(value_name = "PATH")]
	pub project: std::path::PathBuf,

	#[clap(short, long)]
	pub force: bool,
}

#[derive(Debug, Args)]
pub struct PrimeArgs {
	#[clap(value_name = "PATH")]
	pub project: std::path::PathBuf,

	#[clap(short, long)]
	pub context: Vec<std::path::PathBuf>,
}

impl App {
	pub async fn execute(self, config: Config) -> anyhow::Result<()> {
		match self.command {
			Command::Init(args) => crate::commands::init::exec(args, &config).await,
			Command::Plan(args) => crate::commands::plan::exec(args, &config).await,
			Command::Prime(args) => crate::commands::prime::exec(args, &config).await,
		}
	}
}
```

### `src/commands/init.rs`

```rust
use anyhow::Result;
use tracing::info;
use crate::cli::InitArgs;
use crate::config::Config;

pub async fn exec(args: InitArgs, config: &Config) -> Result<()> {
	info!("Initializing project: {}", args.name);

	let project_dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());

	std::fs::create_dir_all(&project_dir)?;

	// Initialize engine database
	crate::engine::CrocEngine::init(&project_dir, &config).await?;

	info!(
		name = %args.name,
		path = %project_dir.display(),
		"Project initialized successfully"
	);

	Ok(())
}
```

### `src/error.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrocError {
	#[error("Project not found at {path}")]
	ProjectNotFound { path: String },

	#[error("Invalid configuration: {reason}")]
	InvalidConfig { reason: String },

	#[error("Engine error: {0}")]
	EngineError(String),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("JSON error: {0}")]
	Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CrocError>;
```

### `src/logging.rs`

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use anyhow::Result;

pub fn init(global_opts: &crate::cli::GlobalOpts) -> Result<()> {
	let log_level = match global_opts.verbose {
		0 => "info",
		1 => "debug",
		_ => "trace",
	};

	let env_filter = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(log_level))?;

	let fmt_layer = fmt::layer()
		.with_writer(std::io::stderr)
		.with_target(global_opts.verbose > 0)
		.with_thread_ids(global_opts.verbose > 1);

	tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt_layer)
		.init();

	Ok(())
}
```

### `src/config.rs`

```rust
use anyhow::Result;
use std::path::PathBuf;

pub struct Config {
	pub log_level: String,
	pub engine_db_path: PathBuf,
	pub context_dir: PathBuf,
}

impl Config {
	pub fn load(global_opts: &crate::cli::GlobalOpts) -> Result<Self> {
		dotenvy::dotenv().ok();

		let log_level = match global_opts.verbose {
			0 => std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
			1 => "debug".to_string(),
			_ => "trace".to_string(),
		};

		let engine_db_path = std::env::var("CROC_DB_PATH")
			.map(PathBuf::from)
			.unwrap_or_else(|_| {
				let home = dirs::home_dir().expect("Could not determine home directory");
				home.join(".croc").join("engine.db")
			});

		let context_dir = std::env::var("CROC_CONTEXT_DIR")
			.map(PathBuf::from)
			.unwrap_or_else(|_| {
				let home = dirs::home_dir().expect("Could not determine home directory");
				home.join(".croc").join("context")
			});

		Ok(Config {
			log_level,
			engine_db_path,
			context_dir,
		})
	}
}
```

---

## Testing CLI Applications

### Unit Testing Commands

```rust
#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_init_creates_project() -> anyhow::Result<()> {
		let temp_dir = tempfile::tempdir()?;
		let args = InitArgs {
			name: "test-project".to_string(),
			dir: Some(temp_dir.path().to_path_buf()),
			template: "default".to_string(),
		};

		let config = Config {
			log_level: "debug".to_string(),
			engine_db_path: temp_dir.path().join("engine.db"),
			context_dir: temp_dir.path().join("context"),
		};

		exec(args, &config).await?;

		assert!(temp_dir.path().exists());
		Ok(())
	}
}
```

### Integration Testing CLI

```rust
#[cfg(test)]
mod integration_tests {
	use assert_cmd::Command;
	use predicates::prelude::*;

	#[test]
	fn test_init_command() {
		let mut cmd = Command::cargo_bin("croc").unwrap();
		cmd.arg("init")
			.arg("test-project")
			.assert()
			.success()
			.stdout(predicate::str::contains("Project initialized"));
	}
}
```

---

## Summary: Key Takeaways

### Architecture
- **Struct-based App** with enum-based Commands
- **Separate modules** for CLI, commands, engine, errors
- **Minimal main.rs** - only parse and delegate
- **Trait-based or match-based** command routing

### CLI Parsing
- Use **derive API** (not builder)
- Use **`#[clap(flatten)]`** for option composition
- Use **`#[clap(global = true)]`** for global options
- Use **`clap::ValueEnum`** for type-safe enums

### Error Handling
- **`thiserror`** for custom error types
- **`anyhow`** for ergonomic error handling in CLIs
- **`.context()`** to add contextual information
- **`color_eyre`** for pretty error output

### Configuration
- **`dotenvy`** to load `.env` files
- **Sensible defaults** for all configuration
- **Never log secrets** - use `Option<String>` for optional values
- **Document all env vars** in `.env.example`

### Logging
- **`tracing`** for structured logging
- **`tracing-subscriber`** for initialization
- **`RUST_LOG`** env var for runtime control
- **`tracing-indicatif`** for progress bar integration

### Progress
- **`indicatif`** for progress bars and spinners
- **`MultiProgress`** for nested operations
- **Meaningful messages** for user feedback
- **Finish with summary** for clear completion

---

## References

- [Rain's Rust CLI Recommendations](https://rust-cli-recommendations.sunshowers.io/)
- [Kevin K's CLI Structure Series](https://kbknapp.dev/cli-structure-01/)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [Indicatif Documentation](https://docs.rs/indicatif/latest/indicatif/)
- [Anyhow Documentation](https://docs.rs/anyhow/latest/anyhow/)
- [Thiserror Documentation](https://docs.rs/thiserror/latest/thiserror/)

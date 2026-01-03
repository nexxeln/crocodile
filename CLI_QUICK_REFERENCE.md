# Rust CLI Quick Reference

Fast lookup for common patterns when building the Crocodile CLI.

---

## Clap Derive Patterns

### Basic Command Structure
```rust
#[derive(Debug, Parser)]
#[clap(name = "croc", version, about)]
pub struct App {
	#[clap(flatten)]
	pub global: GlobalOpts,
	#[clap(subcommand)]
	pub command: Command,
}
```

### Global Options
```rust
#[derive(Debug, Args)]
pub struct GlobalOpts {
	#[clap(short, long, global = true, action = clap::ArgAction::Count)]
	pub verbose: u8,
	
	#[clap(long, global = true, default_value = "auto")]
	pub color: ColorMode,
}
```

### Subcommand Enum
```rust
#[derive(Debug, Subcommand)]
pub enum Command {
	/// Initialize project
	Init(InitArgs),
	
	/// Plan phase
	Plan(PlanArgs),
}
```

### Command Arguments
```rust
#[derive(Debug, Args)]
pub struct InitArgs {
	/// Project name
	#[clap(value_name = "NAME")]
	pub name: String,
	
	#[clap(short, long)]
	pub dir: Option<PathBuf>,
	
	#[clap(short, long, default_value = "default")]
	pub template: String,
}
```

### Enum Arguments
```rust
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ColorMode {
	Always,
	Auto,
	Never,
}
```

---

## Error Handling Patterns

### Define Custom Errors
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrocError {
	#[error("Not found: {path}")]
	NotFound { path: String },
	
	#[error("Invalid: {reason}")]
	Invalid { reason: String },
	
	#[error("IO: {0}")]
	Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CrocError>;
```

### Use in Functions
```rust
pub async fn exec(args: InitArgs) -> anyhow::Result<()> {
	// Add context
	std::fs::create_dir_all(&dir)
		.context("Failed to create directory")?;
	
	// Convert custom errors
	validate(&args)
		.map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;
	
	Ok(())
}
```

### Quick Error Returns
```rust
use anyhow::bail;

if name.is_empty() {
	bail!("Name cannot be empty");
}
```

---

## Logging Patterns

### Initialize Logging
```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init(verbose: u8) -> anyhow::Result<()> {
	let level = match verbose {
		0 => "info",
		1 => "debug",
		_ => "trace",
	};
	
	let env_filter = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(level))?;
	
	tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt::layer().with_writer(std::io::stderr))
		.init();
	
	Ok(())
}
```

### Log Messages
```rust
use tracing::{info, debug, warn, error};

info!("Starting operation");
debug!(file = %path.display(), "Processing file");
warn!(error = %err, "Operation failed, retrying");
error!("Critical error: {}", err);
```

### Structured Fields
```rust
info!(
	project = %name,
	count = items.len(),
	"Processed items"
);

debug!(
	config = ?config_struct,
	"Configuration loaded"
);
```

---

## Configuration Patterns

### Load from Environment
```rust
pub struct Config {
	pub db_path: PathBuf,
	pub api_key: Option<String>,
}

impl Config {
	pub fn load() -> anyhow::Result<Self> {
		// Load .env file
		dotenvy::dotenv().ok();
		
		// Read env vars with defaults
		let db_path = std::env::var("DB_PATH")
			.map(PathBuf::from)
			.unwrap_or_else(|_| {
				dirs::home_dir()
					.unwrap()
					.join(".croc")
					.join("engine.db")
			});
		
		// Optional values
		let api_key = std::env::var("API_KEY").ok();
		
		Ok(Config { db_path, api_key })
	}
}
```

### `.env.example`
```bash
# Database
DB_PATH=~/.croc/engine.db

# API (never commit actual keys!)
# API_KEY=your_key_here
```

---

## Progress Patterns

### Simple Progress Bar
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(items.len() as u64);
pb.set_style(
	ProgressStyle::default_bar()
		.template("{spinner:.green} [{bar:40}] {pos}/{len}")?
		.progress_chars("#>-")
);

for item in items {
	pb.set_message(format!("Processing {}", item));
	process(item).await?;
	pb.inc(1);
}

pb.finish_with_message("Done!");
```

### Spinner
```rust
use indicatif::ProgressBar;

let spinner = ProgressBar::new_spinner();
spinner.set_message("Working...");

// Long operation
do_work().await?;

spinner.finish_with_message("Complete!");
```

### Multi-Progress
```rust
use indicatif::MultiProgress;

let multi = MultiProgress::new();
let parent = multi.add(ProgressBar::new(100));
let child = multi.add(ProgressBar::new(10));

// Update both
parent.inc(1);
child.inc(1);
```

---

## Project Structure

### Minimal Layout
```
src/
├── main.rs          # Entry point
├── lib.rs           # Exports
├── cli.rs           # Argument parsing
├── commands/
│   ├── mod.rs
│   ├── init.rs
│   └── plan.rs
├── error.rs         # Error types
├── logging.rs       # Logging setup
└── config.rs        # Configuration
```

### Modular Layout
```
src/
├── main.rs
├── lib.rs
├── cli/
│   ├── mod.rs
│   └── args.rs
├── commands/
│   ├── mod.rs
│   ├── init.rs
│   └── plan.rs
├── engine/
│   ├── mod.rs
│   └── croc_engine.rs
├── error.rs
├── logging.rs
└── config.rs
```

---

## Main.rs Template

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

---

## Lib.rs Template

```rust
//! Crocodile: AI agent orchestrator

pub mod cli;
pub mod commands;
pub mod engine;
pub mod error;
pub mod logging;
pub mod config;

#[doc(hidden)]
pub use cli::App;
```

---

## Command Execution Pattern

### In `cli/mod.rs`
```rust
impl App {
	pub async fn execute(self, config: Config) -> anyhow::Result<()> {
		match self.command {
			Command::Init(args) => commands::init::exec(args, &config).await,
			Command::Plan(args) => commands::plan::exec(args, &config).await,
		}
	}
}
```

### In `commands/init.rs`
```rust
use anyhow::Result;
use crate::cli::InitArgs;
use crate::config::Config;

pub async fn exec(args: InitArgs, config: &Config) -> Result<()> {
	// Implementation
	Ok(())
}
```

---

## Testing Patterns

### Unit Test
```rust
#[cfg(test)]
mod tests {
	use super::*;
	
	#[tokio::test]
	async fn test_init() -> anyhow::Result<()> {
		let args = InitArgs {
			name: "test".to_string(),
			dir: None,
			template: "default".to_string(),
		};
		
		let config = Config::default();
		exec(args, &config).await?;
		
		Ok(())
	}
}
```

### Integration Test
```rust
#[cfg(test)]
mod integration {
	use assert_cmd::Command;
	use predicates::prelude::*;
	
	#[test]
	fn test_init_command() {
		Command::cargo_bin("croc")
			.unwrap()
			.arg("init")
			.arg("test")
			.assert()
			.success();
	}
}
```

---

## Cargo.toml Essentials

```toml
[package]
name = "crocodile"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
indicatif = "0.17"
color-eyre = "0.6"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
tempfile = "3.0"
assert_cmd = "2.0"
predicates = "3.0"
```

---

## Common Clap Attributes

| Attribute | Purpose |
|-----------|---------|
| `#[clap(short)]` | Single-letter flag (`-v`) |
| `#[clap(long)]` | Long flag (`--verbose`) |
| `#[clap(global = true)]` | Available to all subcommands |
| `#[clap(flatten)]` | Flatten nested struct fields |
| `#[clap(subcommand)]` | Delegate to subcommand enum |
| `#[clap(default_value = "...")]` | Default value |
| `#[clap(value_name = "...")]` | Help text name |
| `#[clap(action = ArgAction::Count)]` | Count occurrences |
| `#[clap(value_enum)]` | Enum with limited values |

---

## Common Tracing Macros

| Macro | Level | Use Case |
|-------|-------|----------|
| `trace!()` | Trace | Very detailed debugging |
| `debug!()` | Debug | Development debugging |
| `info!()` | Info | Important events |
| `warn!()` | Warn | Warnings |
| `error!()` | Error | Errors |

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `RUST_LOG` | Set log level (e.g., `info`, `debug`, `trace`) |
| `RUST_BACKTRACE` | Enable backtraces (`1` or `full`) |
| `CROC_DB_PATH` | Database path |
| `CROC_CONTEXT_DIR` | Context directory |

---

## Useful Commands

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- init my-project

# Run with trace logging
RUST_LOG=trace cargo run -- plan .

# Run tests
cargo test

# Run with backtrace
RUST_BACKTRACE=1 cargo run -- init

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build release
cargo build --release
```

---

## Common Patterns Checklist

- [ ] Use `#[derive(Parser)]` on App struct
- [ ] Use `#[clap(flatten)]` for GlobalOpts
- [ ] Use `#[clap(subcommand)]` for Command enum
- [ ] Define custom errors with `thiserror`
- [ ] Use `anyhow::Result<T>` in functions
- [ ] Initialize logging in main
- [ ] Load config with `dotenvy::dotenv().ok()`
- [ ] Use `tracing::info!()` for important events
- [ ] Use `indicatif` for long operations
- [ ] Add `.context()` to errors
- [ ] Never log secrets
- [ ] Use `color_eyre::install()?` in main
- [ ] Keep main.rs minimal
- [ ] Separate CLI from business logic
- [ ] Write tests for commands


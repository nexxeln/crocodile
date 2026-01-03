use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Main CLI application for crocodile.
///
/// Parses command-line arguments and dispatches to subcommands.
#[derive(Debug, Parser)]
#[clap(
    name = "croc",
    version,
    about = "Human-in-the-loop AI agent orchestrator"
)]
pub struct App {
    #[clap(flatten)]
    pub global: GlobalOpts,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    #[clap(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[clap(long, global = true, default_value = "auto")]
    pub color: ColorMode,
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum ColorMode {
    Always,
    #[default]
    Auto,
    Never,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),
    Prime(PrimeArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[clap(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct PrimeArgs {}

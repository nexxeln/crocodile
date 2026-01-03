use clap::Parser;
use crocodile::cli::{App, Command};
use crocodile::commands;
use crocodile::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().ok();

    let app = App::parse();

    let config = Config::from_current_dir().ok();
    let logs_dir = config.as_ref().map(|c| c.logs_dir());

    let _log_guard = crocodile::logging::init(app.global.verbose, logs_dir.as_deref())?;

    match app.command {
        Some(Command::Init(args)) => commands::init_exec(args).await,
        Some(Command::Prime(args)) => commands::prime_exec(args).await,
        None => {
            App::parse_from(["croc", "--help"]);
            Ok(())
        }
    }
}

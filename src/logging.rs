use anyhow::Result;
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub struct LogGuard {
    _file_guard: Option<WorkerGuard>,
}

pub fn init(verbose: u8, logs_dir: Option<&Path>) -> Result<LogGuard> {
    let log_level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(format!("crocodile={}", log_level)))?;

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(verbose > 1)
        .with_thread_ids(verbose > 2)
        .with_line_number(verbose > 2);

    let (file_layer, file_guard) = if let Some(dir) = logs_dir {
        if dir.exists() {
            let file_appender = tracing_appender::rolling::daily(dir, "croc.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let layer = fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true);

            (Some(layer), Some(guard))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();

    Ok(LogGuard {
        _file_guard: file_guard,
    })
}

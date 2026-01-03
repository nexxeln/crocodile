use crate::config::Config;
use crate::error::{CrocError, Result};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Storage {
    config: Config,
}

impl Storage {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn append_jsonl<T: Serialize>(&self, path: &Path, record: &T) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| CrocError::Storage {
                message: format!("Failed to open {}: {}", path.display(), e),
            })?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, record)?;
        writeln!(writer)?;
        writer.flush()?;

        Ok(())
    }

    pub fn create_empty_file(&self, path: &Path) -> Result<()> {
        File::create(path).map_err(|e| CrocError::Storage {
            message: format!("Failed to create {}: {}", path.display(), e),
        })?;
        Ok(())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

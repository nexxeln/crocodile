use crate::config::Config;
use crate::engine::cache::Cache;
use crate::engine::storage::Storage;
use crate::error::{CrocError, Result};
use crate::schemas::{ContextItem, Event, Plan, Review, Task};
use tracing::{debug, info};

pub struct CrocEngine {
    storage: Storage,
    cache: Cache,
    config: Config,
}

impl CrocEngine {
    pub async fn new(config: Config) -> Result<Self> {
        if !config.is_initialized() {
            return Err(CrocError::InvalidConfig {
                reason: format!(
                    "Crocodile not initialized at {}. Run 'croc init' first.",
                    config.croc_dir.display()
                ),
            });
        }

        let cache_path = config.croc_dir.join("cache.db");
        let cache = Cache::new(&cache_path).await?;
        let storage = Storage::new(config.clone());

        let engine = Self {
            storage,
            cache,
            config,
        };

        engine.ensure_cache_synced().await?;

        Ok(engine)
    }

    async fn ensure_cache_synced(&self) -> Result<()> {
        let plans_in_cache = self.cache.get_all_plans().await?;
        if plans_in_cache.is_empty() {
            let plans_in_storage = self.storage.read_plans()?;
            if !plans_in_storage.is_empty() {
                debug!("Cache empty but storage has data, syncing...");
                self.full_sync().await?;
            }
        }
        Ok(())
    }

    pub async fn full_sync(&self) -> Result<()> {
        info!("Running full sync from JSONL to SQLite cache");

        self.cache.clear_all().await?;

        for plan in self.storage.read_plans()? {
            self.cache.upsert_plan(&plan).await?;
        }

        for task in self.storage.read_tasks()? {
            self.cache.upsert_task(&task).await?;
        }

        for context in self.storage.read_context()? {
            self.cache.upsert_context(&context).await?;
        }

        for event in self.storage.read_events()? {
            self.cache.upsert_event(&event).await?;
        }

        for review in self.storage.read_reviews()? {
            self.cache.upsert_review(&review).await?;
        }

        info!("Full sync complete");
        Ok(())
    }

    pub async fn append_plan(&self, plan: &Plan) -> Result<()> {
        self.storage.append_plan(plan)?;
        self.cache.upsert_plan(plan).await?;
        Ok(())
    }

    pub async fn append_task(&self, task: &Task) -> Result<()> {
        self.storage.append_task(task)?;
        self.cache.upsert_task(task).await?;
        Ok(())
    }

    pub async fn append_context(&self, context: &ContextItem) -> Result<()> {
        self.storage.append_context(context)?;
        self.cache.upsert_context(context).await?;
        Ok(())
    }

    pub async fn append_event(&self, event: &Event) -> Result<()> {
        self.storage.append_event(event)?;
        self.cache.upsert_event(event).await?;
        Ok(())
    }

    pub async fn append_review(&self, review: &Review) -> Result<()> {
        self.storage.append_review(review)?;
        self.cache.upsert_review(review).await?;
        Ok(())
    }

    pub async fn get_plan(&self, id: &str) -> Result<Plan> {
        self.cache
            .get_plan(id)
            .await?
            .ok_or_else(|| CrocError::NotFound {
                entity_type: "Plan".to_string(),
                id: id.to_string(),
            })
    }

    pub async fn get_plan_opt(&self, id: &str) -> Result<Option<Plan>> {
        self.cache.get_plan(id).await
    }

    pub async fn get_task(&self, id: &str) -> Result<Task> {
        self.cache
            .get_task(id)
            .await?
            .ok_or_else(|| CrocError::NotFound {
                entity_type: "Task".to_string(),
                id: id.to_string(),
            })
    }

    pub async fn get_task_opt(&self, id: &str) -> Result<Option<Task>> {
        self.cache.get_task(id).await
    }

    pub async fn get_tasks_for_plan(&self, plan_id: &str) -> Result<Vec<Task>> {
        self.cache.get_tasks_for_plan(plan_id).await
    }

    pub async fn get_context_for_plan(&self, plan_id: &str) -> Result<Vec<ContextItem>> {
        self.cache.get_context_for_plan(plan_id).await
    }

    pub async fn get_context_for_task(&self, subtask_id: &str) -> Result<Vec<ContextItem>> {
        self.cache.get_context_for_task(subtask_id).await
    }

    pub async fn get_active_plans(&self) -> Result<Vec<Plan>> {
        self.cache.get_active_plans().await
    }

    pub async fn get_all_plans(&self) -> Result<Vec<Plan>> {
        self.cache.get_all_plans().await
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

use crate::error::{CrocError, Result};
use crate::schemas::{
    ContextItem, ContextType, Event, Plan, PlanStatus, Review, Task, TaskStatus, TaskType,
};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use tracing::debug;

pub struct Cache {
    pool: SqlitePool,
}

impl Cache {
    pub async fn new(db_path: &Path) -> Result<Self> {
        debug!(path = %db_path.display(), "Opening SQLite cache");

        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to open database: {}", e),
            })?;

        let cache = Self { pool };
        cache.run_migrations().await?;
        cache.set_pragmas().await?;

        Ok(cache)
    }

    async fn set_pragmas(&self) -> Result<()> {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&self.pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&self.pool)
            .await?;
        sqlx::query("PRAGMA cache_size = -64000")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn run_migrations(&self) -> Result<()> {
        debug!("Running SQLite migrations");

        sqlx::query(
            r#"
			CREATE TABLE IF NOT EXISTS plans (
				id TEXT PRIMARY KEY,
				title TEXT NOT NULL,
				description TEXT NOT NULL,
				subtasks_preview TEXT NOT NULL,
				considerations TEXT NOT NULL,
				status TEXT NOT NULL,
				approved_at TEXT,
				created_at TEXT NOT NULL,
				updated_at TEXT NOT NULL
			)
			"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
			CREATE TABLE IF NOT EXISTS tasks (
				id TEXT PRIMARY KEY,
				plan_id TEXT NOT NULL,
				parent_id TEXT,
				task_type TEXT NOT NULL,
				title TEXT NOT NULL,
				description TEXT,
				status TEXT NOT NULL,
				depends_on TEXT NOT NULL,
				worktree TEXT,
				assigned_worker TEXT,
				created_at TEXT NOT NULL,
				updated_at TEXT NOT NULL
			)
			"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
			CREATE TABLE IF NOT EXISTS context_items (
				id TEXT PRIMARY KEY,
				plan_id TEXT NOT NULL,
				subtask_id TEXT,
				item_type TEXT NOT NULL,
				content TEXT NOT NULL,
				source TEXT,
				reasoning TEXT,
				alternatives TEXT,
				confidence REAL,
				created_at TEXT NOT NULL
			)
			"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
			CREATE TABLE IF NOT EXISTS events (
				id TEXT PRIMARY KEY,
				event_type TEXT NOT NULL,
				plan_id TEXT,
				task_id TEXT,
				data TEXT,
				timestamp TEXT NOT NULL
			)
			"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
			CREATE TABLE IF NOT EXISTS reviews (
				id TEXT PRIMARY KEY,
				plan_id TEXT NOT NULL,
				reviewer_type TEXT NOT NULL,
				status TEXT NOT NULL,
				notes TEXT NOT NULL,
				created_at TEXT NOT NULL,
				updated_at TEXT NOT NULL
			)
			"#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_plan_id ON tasks(plan_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_context_plan_id ON context_items(plan_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_plans_status ON plans(status)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn upsert_plan(&self, plan: &Plan) -> Result<()> {
        sqlx::query(
			r#"
			INSERT OR REPLACE INTO plans 
			(id, title, description, subtasks_preview, considerations, status, approved_at, created_at, updated_at)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
			"#,
		)
		.bind(&plan.id)
		.bind(&plan.title)
		.bind(&plan.description)
		.bind(serde_json::to_string(&plan.subtasks_preview)?)
		.bind(serde_json::to_string(&plan.considerations)?)
		.bind(serde_json::to_string(&plan.status)?)
		.bind(plan.approved_at.map(|t| t.to_rfc3339()))
		.bind(plan.created_at.to_rfc3339())
		.bind(plan.updated_at.to_rfc3339())
		.execute(&self.pool)
		.await?;

        Ok(())
    }

    pub async fn upsert_task(&self, task: &Task) -> Result<()> {
        sqlx::query(
			r#"
			INSERT OR REPLACE INTO tasks 
			(id, plan_id, parent_id, task_type, title, description, status, depends_on, worktree, assigned_worker, created_at, updated_at)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
			"#,
		)
		.bind(&task.id)
		.bind(&task.plan_id)
		.bind(&task.parent_id)
		.bind(serde_json::to_string(&task.task_type)?)
		.bind(&task.title)
		.bind(&task.description)
		.bind(serde_json::to_string(&task.status)?)
		.bind(serde_json::to_string(&task.depends_on)?)
		.bind(&task.worktree)
		.bind(&task.assigned_worker)
		.bind(task.created_at.to_rfc3339())
		.bind(task.updated_at.to_rfc3339())
		.execute(&self.pool)
		.await?;

        Ok(())
    }

    pub async fn upsert_context(&self, context: &ContextItem) -> Result<()> {
        sqlx::query(
			r#"
			INSERT OR REPLACE INTO context_items 
			(id, plan_id, subtask_id, item_type, content, source, reasoning, alternatives, confidence, created_at)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
			"#,
		)
		.bind(&context.id)
		.bind(&context.plan_id)
		.bind(&context.subtask_id)
		.bind(serde_json::to_string(&context.item_type)?)
		.bind(&context.content)
		.bind(&context.source)
		.bind(&context.reasoning)
		.bind(context.alternatives.as_ref().and_then(|a| serde_json::to_string(a).ok()))
		.bind(context.confidence)
		.bind(context.created_at.to_rfc3339())
		.execute(&self.pool)
		.await?;

        Ok(())
    }

    pub async fn upsert_event(&self, event: &Event) -> Result<()> {
        sqlx::query(
            r#"
			INSERT OR REPLACE INTO events 
			(id, event_type, plan_id, task_id, data, timestamp)
			VALUES (?, ?, ?, ?, ?, ?)
			"#,
        )
        .bind(&event.id)
        .bind(serde_json::to_string(&event.event_type)?)
        .bind(&event.plan_id)
        .bind(&event.task_id)
        .bind(event.data.as_ref().map(|d| d.to_string()))
        .bind(event.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_review(&self, review: &Review) -> Result<()> {
        sqlx::query(
            r#"
			INSERT OR REPLACE INTO reviews 
			(id, plan_id, reviewer_type, status, notes, created_at, updated_at)
			VALUES (?, ?, ?, ?, ?, ?, ?)
			"#,
        )
        .bind(&review.id)
        .bind(&review.plan_id)
        .bind(serde_json::to_string(&review.reviewer_type)?)
        .bind(serde_json::to_string(&review.status)?)
        .bind(serde_json::to_string(&review.notes)?)
        .bind(review.created_at.to_rfc3339())
        .bind(review.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_plan(&self, id: &str) -> Result<Option<Plan>> {
        let row = sqlx::query(
			"SELECT id, title, description, subtasks_preview, considerations, status, approved_at, created_at, updated_at FROM plans WHERE id = ?",
		)
		.bind(id)
		.fetch_optional(&self.pool)
		.await?;

        match row {
            Some(row) => Ok(Some(self.row_to_plan(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let row = sqlx::query(
			"SELECT id, plan_id, parent_id, task_type, title, description, status, depends_on, worktree, assigned_worker, created_at, updated_at FROM tasks WHERE id = ?",
		)
		.bind(id)
		.fetch_optional(&self.pool)
		.await?;

        match row {
            Some(row) => Ok(Some(self.row_to_task(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_tasks_for_plan(&self, plan_id: &str) -> Result<Vec<Task>> {
        let rows = sqlx::query(
			"SELECT id, plan_id, parent_id, task_type, title, description, status, depends_on, worktree, assigned_worker, created_at, updated_at FROM tasks WHERE plan_id = ? ORDER BY created_at",
		)
		.bind(plan_id)
		.fetch_all(&self.pool)
		.await?;

        rows.iter().map(|r| self.row_to_task(r)).collect()
    }

    pub async fn get_context_for_plan(&self, plan_id: &str) -> Result<Vec<ContextItem>> {
        let rows = sqlx::query(
			"SELECT id, plan_id, subtask_id, item_type, content, source, reasoning, alternatives, confidence, created_at FROM context_items WHERE plan_id = ? ORDER BY created_at",
		)
		.bind(plan_id)
		.fetch_all(&self.pool)
		.await?;

        rows.iter().map(|r| self.row_to_context(r)).collect()
    }

    pub async fn get_context_for_task(&self, subtask_id: &str) -> Result<Vec<ContextItem>> {
        let rows = sqlx::query(
			"SELECT id, plan_id, subtask_id, item_type, content, source, reasoning, alternatives, confidence, created_at FROM context_items WHERE subtask_id = ? ORDER BY created_at",
		)
		.bind(subtask_id)
		.fetch_all(&self.pool)
		.await?;

        rows.iter().map(|r| self.row_to_context(r)).collect()
    }

    pub async fn get_active_plans(&self) -> Result<Vec<Plan>> {
        let rows = sqlx::query(
			r#"SELECT id, title, description, subtasks_preview, considerations, status, approved_at, created_at, updated_at FROM plans WHERE status IN ('"approved"', '"running"') ORDER BY created_at DESC"#,
		)
		.fetch_all(&self.pool)
		.await?;

        rows.iter().map(|r| self.row_to_plan(r)).collect()
    }

    pub async fn get_all_plans(&self) -> Result<Vec<Plan>> {
        let rows = sqlx::query(
			"SELECT id, title, description, subtasks_preview, considerations, status, approved_at, created_at, updated_at FROM plans ORDER BY created_at DESC",
		)
		.fetch_all(&self.pool)
		.await?;

        rows.iter().map(|r| self.row_to_plan(r)).collect()
    }

    pub async fn clear_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM plans").execute(&self.pool).await?;
        sqlx::query("DELETE FROM tasks").execute(&self.pool).await?;
        sqlx::query("DELETE FROM context_items")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM events")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM reviews")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn row_to_plan(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Plan> {
        let status_str: String = row.get("status");
        let status: PlanStatus = serde_json::from_str(&status_str)?;

        let subtasks_str: String = row.get("subtasks_preview");
        let subtasks_preview: Vec<String> = serde_json::from_str(&subtasks_str)?;

        let considerations_str: String = row.get("considerations");
        let considerations: Vec<String> = serde_json::from_str(&considerations_str)?;

        let approved_at: Option<String> = row.get("approved_at");
        let approved_at = approved_at
            .map(|s| chrono::DateTime::parse_from_rfc3339(&s))
            .transpose()
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to parse approved_at: {}", e),
            })?
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let created_at: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to parse created_at: {}", e),
            })?
            .with_timezone(&chrono::Utc);

        let updated_at: String = row.get("updated_at");
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to parse updated_at: {}", e),
            })?
            .with_timezone(&chrono::Utc);

        Ok(Plan {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            subtasks_preview,
            considerations,
            status,
            approved_at,
            created_at,
            updated_at,
        })
    }

    fn row_to_task(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Task> {
        let task_type_str: String = row.get("task_type");
        let task_type: TaskType = serde_json::from_str(&task_type_str)?;

        let status_str: String = row.get("status");
        let status: TaskStatus = serde_json::from_str(&status_str)?;

        let depends_on_str: String = row.get("depends_on");
        let depends_on: Vec<String> = serde_json::from_str(&depends_on_str)?;

        let created_at: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to parse created_at: {}", e),
            })?
            .with_timezone(&chrono::Utc);

        let updated_at: String = row.get("updated_at");
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to parse updated_at: {}", e),
            })?
            .with_timezone(&chrono::Utc);

        Ok(Task {
            id: row.get("id"),
            plan_id: row.get("plan_id"),
            parent_id: row.get("parent_id"),
            task_type,
            title: row.get("title"),
            description: row.get("description"),
            status,
            depends_on,
            worktree: row.get("worktree"),
            assigned_worker: row.get("assigned_worker"),
            created_at,
            updated_at,
        })
    }

    fn row_to_context(&self, row: &sqlx::sqlite::SqliteRow) -> Result<ContextItem> {
        let item_type_str: String = row.get("item_type");
        let item_type: ContextType = serde_json::from_str(&item_type_str)?;

        let alternatives_str: Option<String> = row.get("alternatives");
        let alternatives: Option<Vec<String>> = alternatives_str
            .map(|s| serde_json::from_str(&s))
            .transpose()?;

        let created_at: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| CrocError::Cache {
                message: format!("Failed to parse created_at: {}", e),
            })?
            .with_timezone(&chrono::Utc);

        Ok(ContextItem {
            id: row.get("id"),
            plan_id: row.get("plan_id"),
            subtask_id: row.get("subtask_id"),
            item_type,
            content: row.get("content"),
            source: row.get("source"),
            reasoning: row.get("reasoning"),
            alternatives,
            confidence: row.get("confidence"),
            created_at,
        })
    }
}

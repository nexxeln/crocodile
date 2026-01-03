use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Pending,
    Approved,
    Running,
    Complete,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Foreman,
    Subtask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    Fact,
    Decision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Initialized,
    PlanCreated,
    PlanApproved,
    ForemanSpawned,
    WorkerSpawned,
    WorkerProgress,
    WorkerComplete,
    WorkerFailed,
    ReviewRequested,
    ReviewApproved,
    ReviewChangesRequested,
    PlanComplete,
    PlanCancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerType {
    Agent,
    Human,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    ChangesRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Planner,
    Foreman,
    Worker,
    Reviewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub subtasks_preview: Vec<String>,
    pub considerations: Vec<String>,
    pub status: PlanStatus,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Plan {
    pub fn new(id: String, title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            subtasks_preview: Vec::new(),
            considerations: Vec::new(),
            status: PlanStatus::Pending,
            approved_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn generate_id() -> String {
        format!("plan-{}", uuid::Uuid::new_v4().as_simple())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub plan_id: String,
    pub parent_id: Option<String>,
    pub task_type: TaskType,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub depends_on: Vec<String>,
    pub worktree: Option<String>,
    pub assigned_worker: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new_foreman(plan_id: String, title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_foreman_id(&plan_id),
            plan_id,
            parent_id: None,
            task_type: TaskType::Foreman,
            title,
            description: None,
            status: TaskStatus::Pending,
            depends_on: Vec::new(),
            worktree: None,
            assigned_worker: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_subtask(
        plan_id: String,
        parent_id: String,
        subtask_num: u32,
        title: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("{}.{}", parent_id, subtask_num),
            plan_id,
            parent_id: Some(parent_id),
            task_type: TaskType::Subtask,
            title,
            description: None,
            status: TaskStatus::Pending,
            depends_on: Vec::new(),
            worktree: None,
            assigned_worker: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn generate_foreman_id(plan_id: &str) -> String {
        format!("task-{}", plan_id.strip_prefix("plan-").unwrap_or(plan_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub id: String,
    pub plan_id: String,
    pub subtask_id: Option<String>,
    pub item_type: ContextType,
    pub content: String,
    pub source: Option<String>,
    pub reasoning: Option<String>,
    pub alternatives: Option<Vec<String>>,
    pub confidence: Option<f32>,
    pub created_at: DateTime<Utc>,
}

impl ContextItem {
    pub fn new_fact(
        plan_id: String,
        subtask_id: Option<String>,
        content: String,
        source: Option<String>,
        confidence: Option<f32>,
    ) -> Self {
        Self {
            id: Self::generate_id("fact"),
            plan_id,
            subtask_id,
            item_type: ContextType::Fact,
            content,
            source,
            reasoning: None,
            alternatives: None,
            confidence,
            created_at: Utc::now(),
        }
    }

    pub fn new_decision(
        plan_id: String,
        subtask_id: Option<String>,
        content: String,
        reasoning: String,
        alternatives: Option<Vec<String>>,
    ) -> Self {
        Self {
            id: Self::generate_id("dec"),
            plan_id,
            subtask_id,
            item_type: ContextType::Decision,
            content,
            source: None,
            reasoning: Some(reasoning),
            alternatives,
            confidence: None,
            created_at: Utc::now(),
        }
    }

    fn generate_id(prefix: &str) -> String {
        format!("{}-{}", prefix, Utc::now().timestamp_millis())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub plan_id: Option<String>,
    pub task_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    pub fn new(event_type: EventType) -> Self {
        Self {
            id: Self::generate_id(),
            event_type,
            plan_id: None,
            task_id: None,
            data: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_plan(mut self, plan_id: String) -> Self {
        self.plan_id = Some(plan_id);
        self
    }

    pub fn with_task(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    fn generate_id() -> String {
        format!("evt-{}", Utc::now().timestamp_millis())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub plan_id: String,
    pub reviewer_type: ReviewerType,
    pub status: ReviewStatus,
    pub notes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Review {
    pub fn new(plan_id: String, reviewer_type: ReviewerType) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            plan_id,
            reviewer_type,
            status: ReviewStatus::Pending,
            notes: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn generate_id() -> String {
        format!("rev-{}", Utc::now().timestamp_millis())
    }
}

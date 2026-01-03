use crate::cli::PrimeArgs;
use crate::config::Config;
use crate::engine::CrocEngine;
use crate::error::CrocError;
use crate::schemas::{ContextItem, Plan, Role, Task};
use anyhow::{Context, Result};
use std::env;

pub async fn exec(_args: PrimeArgs) -> Result<()> {
    let role_str = env::var("CROC_ROLE").map_err(|_| CrocError::MissingEnvVar {
        name: "CROC_ROLE".to_string(),
    })?;

    let role: Role =
        serde_json::from_str(&format!("\"{}\"", role_str)).map_err(|_| CrocError::InvalidRole {
            role: role_str.clone(),
        })?;

    match role {
        Role::Planner => print_planner_context(),
        Role::Foreman => print_foreman_context().await,
        Role::Worker => print_worker_context().await,
        Role::Reviewer => print_reviewer_context().await,
    }
}

fn print_planner_context() -> Result<()> {
    println!(
        r#"# Crocodile Planner Mode

You are in **planning mode**. Your role is to collaborate with the user to create a clear, actionable plan.

## Your Responsibilities

1. **Understand the request** - Ask clarifying questions if needed
2. **Break down the work** - Identify subtasks and dependencies
3. **Consider constraints** - Note any technical considerations
4. **Propose a plan** - Present a structured plan with subtasks

## When the Plan is Ready

When the user approves the plan, they will use `/approve` or run `croc approve`.
The plan will then be handed off to the Foreman for execution.

## Guidelines

- Keep subtasks atomic and independently executable
- Identify dependencies between subtasks
- Note any files that will be touched
- Consider edge cases and error handling
- No code execution in this phase - planning only"#
    );

    Ok(())
}

async fn print_foreman_context() -> Result<()> {
    let plan_id = env::var("CROC_PLAN_ID").map_err(|_| CrocError::MissingEnvVar {
        name: "CROC_PLAN_ID".to_string(),
    })?;

    let config = Config::from_current_dir().context("Failed to load config")?;
    let engine = CrocEngine::new(config).await?;
    let plan = engine.get_plan(&plan_id).await?;
    let tasks = engine.get_tasks_for_plan(&plan_id).await?;
    let context = engine.get_context_for_plan(&plan_id).await?;

    println!("{}", build_foreman_prompt(&plan, &tasks, &context));
    Ok(())
}

async fn print_worker_context() -> Result<()> {
    let subtask_id = env::var("CROC_SUBTASK_ID").map_err(|_| CrocError::MissingEnvVar {
        name: "CROC_SUBTASK_ID".to_string(),
    })?;

    let plan_id = env::var("CROC_PLAN_ID").map_err(|_| CrocError::MissingEnvVar {
        name: "CROC_PLAN_ID".to_string(),
    })?;

    let config = Config::from_current_dir().context("Failed to load config")?;
    let engine = CrocEngine::new(config).await?;
    let task = engine.get_task(&subtask_id).await?;
    let plan = engine.get_plan(&plan_id).await?;
    let context = engine.get_context_for_task(&subtask_id).await?;
    let plan_context = engine.get_context_for_plan(&plan_id).await?;

    println!(
        "{}",
        build_worker_prompt(&plan, &task, &context, &plan_context)
    );
    Ok(())
}

async fn print_reviewer_context() -> Result<()> {
    let plan_id = env::var("CROC_PLAN_ID").map_err(|_| CrocError::MissingEnvVar {
        name: "CROC_PLAN_ID".to_string(),
    })?;

    let config = Config::from_current_dir().context("Failed to load config")?;
    let engine = CrocEngine::new(config).await?;
    let plan = engine.get_plan(&plan_id).await?;
    let tasks = engine.get_tasks_for_plan(&plan_id).await?;
    let context = engine.get_context_for_plan(&plan_id).await?;

    println!("{}", build_reviewer_prompt(&plan, &tasks, &context));
    Ok(())
}

fn build_foreman_prompt(plan: &Plan, tasks: &[Task], context: &[ContextItem]) -> String {
    let mut prompt = format!(
        r#"# Crocodile Foreman Mode

## Plan: {}

{}

## Considerations
{}

## Current Subtasks
"#,
        plan.title,
        plan.description,
        if plan.considerations.is_empty() {
            "- None specified".to_string()
        } else {
            plan.considerations
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    if tasks.is_empty() {
        prompt.push_str("\nNo subtasks created yet. Create the task DAG based on the plan.\n");
    } else {
        for task in tasks {
            let status = format!("{:?}", task.status).to_lowercase();
            let deps = if task.depends_on.is_empty() {
                String::new()
            } else {
                format!(" (depends: {})", task.depends_on.join(", "))
            };
            prompt.push_str(&format!(
                "- [{}] {}: {}{}\n",
                status, task.id, task.title, deps
            ));
        }
    }

    if !context.is_empty() {
        prompt.push_str("\n## Learned Context\n");
        for item in context {
            match item.item_type {
                crate::schemas::ContextType::Fact => {
                    prompt.push_str(&format!("- FACT: {}\n", item.content));
                }
                crate::schemas::ContextType::Decision => {
                    prompt.push_str(&format!(
                        "- DECISION: {} (reason: {})\n",
                        item.content,
                        item.reasoning.as_deref().unwrap_or("unspecified")
                    ));
                }
            }
        }
    }

    prompt.push_str(
        r#"
## Your Responsibilities

1. **Create/update the task DAG** - Break down work into parallelizable subtasks
2. **Spawn workers** - Start workers for ready subtasks (dependencies satisfied)
3. **Monitor progress** - Track worker status and handle handoffs
4. **Propagate context** - Pass learned facts/decisions to dependent workers
5. **Signal completion** - When all subtasks complete, hand off to Reviewer

## Status Block Format

End each response with a status block:

---CROC_STATUS---
FOREMAN: <plan_id>
SUBTASKS_TOTAL: <n>
SUBTASKS_COMPLETE: <n>
SUBTASKS_RUNNING: <ids>
NEXT_ACTION: <spawn_worker|wait|review_ready>
---END_CROC_STATUS---
"#,
    );

    prompt
}

fn build_worker_prompt(
    plan: &Plan,
    task: &Task,
    task_context: &[ContextItem],
    plan_context: &[ContextItem],
) -> String {
    let mut prompt = format!(
        r#"# Crocodile Worker Mode

## Plan: {}

## Your Subtask: {}

{}

"#,
        plan.title,
        task.title,
        task.description
            .as_deref()
            .unwrap_or("No description provided.")
    );

    if !plan_context.is_empty() || !task_context.is_empty() {
        prompt.push_str("## Context from Previous Work\n\n");
        for item in plan_context.iter().chain(task_context.iter()) {
            match item.item_type {
                crate::schemas::ContextType::Fact => {
                    let source = item.source.as_deref().unwrap_or("unknown");
                    prompt.push_str(&format!("- FACT ({}): {}\n", source, item.content));
                }
                crate::schemas::ContextType::Decision => {
                    prompt.push_str(&format!(
                        "- DECISION: {} (reason: {})\n",
                        item.content,
                        item.reasoning.as_deref().unwrap_or("unspecified")
                    ));
                }
            }
        }
        prompt.push('\n');
    }

    prompt.push_str(
        r#"## Your Responsibilities

1. **Execute this subtask** - Focus only on this specific piece of work
2. **Document learnings** - Record facts and decisions for other workers
3. **Signal completion** - Report when done with a status block

## Guidelines

- Stay focused on this subtask only
- If you discover something other workers need to know, document it as a FACT
- If you make a decision that affects other work, document it as a DECISION
- Do not modify files outside your subtask's scope

## Status Block Format

End each response with:

---CROC_STATUS---
SUBTASK: <task_id>
STATUS: <in_progress|complete|blocked>
CONTEXT_USAGE: <percentage>
FILES_MODIFIED: [<list>]
FACTS_LEARNED:
  - "<fact>" (<source>)
DECISIONS_MADE:
  - decision: "<what>"
    reasoning: "<why>"
WORK_COMPLETED: "<summary>"
EXIT_READY: <true|false>
---END_CROC_STATUS---
"#,
    );

    prompt
}

fn build_reviewer_prompt(plan: &Plan, tasks: &[Task], context: &[ContextItem]) -> String {
    let mut prompt = format!(
        r#"# Crocodile Reviewer Mode

## Plan: {}

{}

## Completed Subtasks
"#,
        plan.title, plan.description
    );

    for task in tasks {
        let status = format!("{:?}", task.status).to_lowercase();
        prompt.push_str(&format!("- [{}] {}: {}\n", status, task.id, task.title));
    }

    if !context.is_empty() {
        prompt.push_str("\n## Decisions Made During Implementation\n\n");
        for item in context
            .iter()
            .filter(|i| matches!(i.item_type, crate::schemas::ContextType::Decision))
        {
            prompt.push_str(&format!(
                "- {}: {}\n",
                item.content,
                item.reasoning.as_deref().unwrap_or("no reason given")
            ));
        }
    }

    prompt.push_str(
        r#"
## Your Responsibilities

1. **Review all changes** - Check code quality, consistency, and correctness
2. **Verify plan adherence** - Ensure implementation matches the approved plan
3. **Check for issues** - Look for bugs, security issues, missing tests
4. **Provide feedback** - Document any concerns or required changes

## Review Criteria

- Does the code follow project conventions?
- Are there any obvious bugs or edge cases missed?
- Is error handling appropriate?
- Are the changes well-tested?
- Does the implementation match the plan's intent?

## Status Block Format

End your review with:

---CROC_REVIEW---
STATUS: <approved|changes_requested>
SUMMARY: "<overall assessment>"
ISSUES:
  - severity: <critical|major|minor>
    description: "<issue>"
    location: "<file:line or general>"
RECOMMENDATIONS:
  - "<suggestion>"
---END_CROC_REVIEW---
"#,
    );

    prompt
}

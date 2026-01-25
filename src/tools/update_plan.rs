use crate::plan::{Plan, PlanManager, TaskStatus};
use crate::tools::FileToolError;
use colored::*;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UpdatePlanArgs {
    Create {
        title: String,
    },
    AddTask {
        description: String,
        #[serde(default)]
        dependencies: Vec<usize>,
    },
    UpdateStatus {
        task_id: usize,
        status: String,
    },
    Show,
    Clear,
}

#[derive(Serialize, Debug)]
pub struct UpdatePlanOutput {
    pub success: bool,
    pub message: String,
    pub plan: Option<String>,
}

pub struct UpdatePlanTool {
    manager: PlanManager,
}

impl UpdatePlanTool {
    pub fn new(manager: PlanManager) -> Self {
        Self { manager }
    }

    fn parse_status(status_str: &str) -> Option<TaskStatus> {
        match status_str.to_lowercase().as_str() {
            "pending" => Some(TaskStatus::Pending),
            "in_progress" | "inprogress" => Some(TaskStatus::InProgress),
            "completed" | "done" => Some(TaskStatus::Completed),
            "blocked" => Some(TaskStatus::Blocked),
            _ => None,
        }
    }
}

impl Tool for UpdatePlanTool {
    const NAME: &'static str = "update_plan";
    type Error = FileToolError;
    type Args = UpdatePlanArgs;
    type Output = UpdatePlanOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "update_plan".to_string(),
            description: "Manage execution plans with tasks and dependencies. Create plans, add tasks, update status, and track progress.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["create", "add_task", "update_status", "show", "clear"],
                        "description": "Action: create (new plan), add_task (add task), update_status (change task status), show (display plan), clear (remove plan)"
                    },
                    "title": {
                        "type": "string",
                        "description": "Plan title (for create action)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Task description (for add_task action)"
                    },
                    "dependencies": {
                        "type": "array",
                        "items": {"type": "number"},
                        "description": "Task IDs that must complete first (for add_task action)"
                    },
                    "task_id": {
                        "type": "number",
                        "description": "Task ID to update (for update_status action)"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["pending", "in_progress", "completed", "blocked"],
                        "description": "New status (for update_status action)"
                    }
                },
                "required": ["action"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args {
            UpdatePlanArgs::Create { title } => {
                let plan = Plan::new(title.clone());
                let formatted = plan.format();
                self.manager.set_plan(plan);
                Ok(UpdatePlanOutput {
                    success: true,
                    message: format!("Created plan: {}", title),
                    plan: Some(formatted),
                })
            }
            UpdatePlanArgs::AddTask {
                description,
                dependencies,
            } => {
                let mut task_id = 0;
                let success = self.manager.update_plan(|plan| {
                    task_id = plan.add_task(description.clone(), dependencies);
                });

                if success {
                    let plan = self.manager.get_plan().unwrap();
                    Ok(UpdatePlanOutput {
                        success: true,
                        message: format!("Added task [{}]: {}", task_id, description),
                        plan: Some(plan.format()),
                    })
                } else {
                    Err(FileToolError::InvalidInput("No active plan. Create one first.".to_string()))
                }
            }
            UpdatePlanArgs::UpdateStatus { task_id, status } => {
                let status_enum = Self::parse_status(&status)
                    .ok_or_else(|| FileToolError::InvalidInput(format!("Invalid status: {}", status)))?;

                let success = self.manager.update_plan(|plan| {
                    plan.update_status(task_id, status_enum);
                });

                if success {
                    let plan = self.manager.get_plan().unwrap();
                    Ok(UpdatePlanOutput {
                        success: true,
                        message: format!("Updated task [{}] to {}", task_id, status),
                        plan: Some(plan.format()),
                    })
                } else {
                    Err(FileToolError::InvalidInput("No active plan or task not found.".to_string()))
                }
            }
            UpdatePlanArgs::Show => {
                if let Some(plan) = self.manager.get_plan() {
                    Ok(UpdatePlanOutput {
                        success: true,
                        message: "Current plan".to_string(),
                        plan: Some(plan.format()),
                    })
                } else {
                    Err(FileToolError::InvalidInput("No active plan.".to_string()))
                }
            }
            UpdatePlanArgs::Clear => {
                self.manager.clear_plan();
                Ok(UpdatePlanOutput {
                    success: true,
                    message: "Plan cleared".to_string(),
                    plan: None,
                })
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WrappedUpdatePlanTool {
    #[serde(skip)]
    inner: Option<UpdatePlanTool>,
}

impl WrappedUpdatePlanTool {
    pub fn new(manager: PlanManager) -> Self {
        Self {
            inner: Some(UpdatePlanTool::new(manager)),
        }
    }
}

impl Tool for WrappedUpdatePlanTool {
    const NAME: &'static str = "update_plan";
    type Error = FileToolError;
    type Args = <UpdatePlanTool as Tool>::Args;
    type Output = <UpdatePlanTool as Tool>::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.as_ref().unwrap().definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let action_name = match &args {
            UpdatePlanArgs::Create { .. } => "Create Plan",
            UpdatePlanArgs::AddTask { .. } => "Add Task",
            UpdatePlanArgs::UpdateStatus { .. } => "Update Status",
            UpdatePlanArgs::Show => "Show Plan",
            UpdatePlanArgs::Clear => "Clear Plan",
        };

        println!("\n{} {}", "●".bright_blue(), action_name);

        let result = self.inner.as_ref().unwrap().call(args).await;

        match &result {
            Ok(output) => {
                println!("  └─ {}", output.message.green());
            }
            Err(e) => {
                println!("  └─ {}", format!("Error: {}", e).red());
            }
        }
        println!();
        result
    }
}

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub status: TaskStatus,
    pub dependencies: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub title: String,
    pub tasks: Vec<Task>,
    pub next_id: usize,
}

impl Plan {
    pub fn new(title: String) -> Self {
        Self {
            title,
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_task(&mut self, description: String, dependencies: Vec<usize>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task {
            id,
            description,
            status: TaskStatus::Pending,
            dependencies,
        });
        id
    }

    pub fn update_status(&mut self, task_id: usize, status: TaskStatus) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
            true
        } else {
            false
        }
    }

    pub fn get_next_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Pending))
            .filter(|t| {
                t.dependencies.iter().all(|dep_id| {
                    self.tasks
                        .iter()
                        .find(|task| task.id == *dep_id)
                        .map(|dep| matches!(dep.status, TaskStatus::Completed))
                        .unwrap_or(false)
                })
            })
            .collect()
    }

    pub fn format(&self) -> String {
        let mut output = format!("📋 Plan: {}\n\n", self.title);
        
        for task in &self.tasks {
            let status_icon = match task.status {
                TaskStatus::Pending => "⏳",
                TaskStatus::InProgress => "🔄",
                TaskStatus::Completed => "✅",
                TaskStatus::Blocked => "🚫",
            };
            
            output.push_str(&format!("{} [{}] {}\n", status_icon, task.id, task.description));
            
            if !task.dependencies.is_empty() {
                output.push_str(&format!("   Dependencies: {:?}\n", task.dependencies));
            }
        }
        
        let next_tasks = self.get_next_tasks();
        if !next_tasks.is_empty() {
            output.push_str("\n🎯 Next available tasks:\n");
            for task in next_tasks {
                output.push_str(&format!("   • [{}] {}\n", task.id, task.description));
            }
        }
        
        output
    }
}

#[derive(Clone)]
pub struct PlanManager {
    current_plan: Arc<Mutex<Option<Plan>>>,
}

impl PlanManager {
    pub fn new() -> Self {
        Self {
            current_plan: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_plan(&self) -> Option<Plan> {
        self.current_plan.lock().ok()?.clone()
    }

    pub fn set_plan(&self, plan: Plan) {
        *self.current_plan.lock().unwrap() = Some(plan);
    }

    pub fn clear_plan(&self) {
        *self.current_plan.lock().unwrap() = None;
    }

    pub fn update_plan<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut Plan),
    {
        let mut guard = self.current_plan.lock().unwrap();
        if let Some(plan) = guard.as_mut() {
            f(plan);
            true
        } else {
            false
        }
    }
}

impl Default for PlanManager {
    fn default() -> Self {
        Self::new()
    }
}

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub task_id: Uuid,
    pub run_at: DateTime<Utc>,
    pub is_executed: bool,
}

pub struct DownloadScheduler {
    tasks: Vec<ScheduledTask>,
}

impl DownloadScheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn schedule(&mut self, task_id: Uuid, run_at: DateTime<Utc>) {
        self.tasks.push(ScheduledTask {
            task_id,
            run_at,
            is_executed: false,
        });
    }

    pub fn get_due_tasks(&mut self, now: DateTime<Utc>) -> Vec<Uuid> {
        let mut due = Vec::new();
        for task in &mut self.tasks {
            if !task.is_executed && task.run_at <= now {
                task.is_executed = true;
                due.push(task.task_id);
            }
        }
        due
    }
}

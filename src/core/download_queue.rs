use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

#[derive(Debug, Clone)]
pub struct QueuedItem {
    pub task_id: Uuid,
    pub priority: Priority,
}

pub struct DownloadQueue {
    items: VecDeque<QueuedItem>,
    max_simultaneous: usize,
}

impl DownloadQueue {
    pub fn new(max_simultaneous: usize) -> Self {
        Self {
            items: VecDeque::new(),
            max_simultaneous,
        }
    }

    pub fn push(&mut self, task_id: Uuid, priority: Priority) {
        let item = QueuedItem { task_id, priority };
        let insert_idx = self
            .items
            .iter()
            .position(|it| it.priority < priority)
            .unwrap_or(self.items.len());
        self.items.insert(insert_idx, item);
    }

    pub fn pop_next(&mut self) -> Option<Uuid> {
        self.items.pop_front().map(|it| it.task_id)
    }

    pub fn remove(&mut self, task_id: Uuid) -> bool {
        if let Some(pos) = self.items.iter().position(|it| it.task_id == task_id) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn max_simultaneous(&self) -> usize {
        self.max_simultaneous
    }

    pub fn set_max_simultaneous(&mut self, limit: usize) {
        self.max_simultaneous = limit;
    }
}

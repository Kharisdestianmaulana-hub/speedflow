use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::download_queue::DownloadQueue;
use crate::core::download_task::DownloadTask;

pub struct DownloadManager {
    tasks: Arc<RwLock<HashMap<Uuid, DownloadTask>>>,
    queue: Arc<RwLock<DownloadQueue>>,
}

impl DownloadManager {
    pub fn new(max_simultaneous: usize) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(DownloadQueue::new(max_simultaneous))),
        }
    }

    pub async fn add_task(&self, task: DownloadTask) -> Uuid {
        let id = task.id;
        let mut tasks = self.tasks.write().await;
        tasks.insert(id, task);
        id
    }

    pub async fn get_task(&self, id: &Uuid) -> Option<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.get(id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub fn queue(&self) -> &Arc<RwLock<DownloadQueue>> {
        &self.queue
    }
}

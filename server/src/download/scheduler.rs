use super::{manager::DownloadManager, model::DownloadStatus, DownloadResult};
use tokio::{sync::watch, time::Duration};

impl DownloadManager {
    pub(super) fn spawn(&self, task_id: String) {
        let manager = self.clone();
        tokio::spawn(async move {
            let Some(generation) = manager.queued_generation(&task_id).await else {
                return;
            };
            let (cancel, mut cancelled) = watch::channel(false);
            let (finished, _) = watch::channel(false);
            {
                let mut workers = manager.workers.lock().await;
                if workers.contains_key(&task_id) {
                    return;
                }
                workers.insert(
                    task_id.clone(),
                    super::manager::WorkerHandle {
                        generation,
                        cancel,
                        finished,
                    },
                );
            }

            if !manager.is_queued_generation(&task_id, generation).await {
                manager.finish_worker(&task_id, generation).await;
                return;
            }

            let permit = tokio::select! {
                permit = manager.task_semaphore.clone().acquire_owned() => permit.ok(),
                _ = wait_for_cancel(&mut cancelled) => None,
            };
            if let Some(_permit) = permit {
                if let Err(error) = manager.run_task(&task_id, generation, &cancelled).await {
                    let _ = manager
                        .fail_generation(&task_id, generation, error.to_string())
                        .await;
                }
            }
            manager.finish_worker(&task_id, generation).await;
        });
    }

    async fn finish_worker(&self, task_id: &str, generation: u64) {
        {
            let mut workers = self.workers.lock().await;
            if workers
                .get(task_id)
                .is_some_and(|worker| worker.generation == generation)
            {
                if let Some(worker) = workers.get(task_id) {
                    let _ = worker.finished.send(true);
                }
                workers.remove(task_id);
            }
        }

        if self.queued_generation(task_id).await.is_some() {
            self.spawn(task_id.to_string());
        }
    }

    pub(super) async fn cancel_worker(&self, task_id: &str) {
        let cancel = self
            .workers
            .lock()
            .await
            .get(task_id)
            .map(|worker| worker.cancel.clone());
        if let Some(cancel) = cancel {
            let _ = cancel.send(true);
        }
    }

    pub(super) async fn cancel_worker_and_wait(&self, task_id: &str) -> DownloadResult<()> {
        let handles = self
            .workers
            .lock()
            .await
            .get(task_id)
            .map(|worker| (worker.cancel.clone(), worker.finished.subscribe()));
        let Some((cancel, mut finished)) = handles else {
            return Ok(());
        };
        let _ = cancel.send(true);
        if !*finished.borrow() {
            tokio::time::timeout(Duration::from_secs(30), finished.changed())
                .await
                .map_err(|_| anyhow::anyhow!("等待下载 worker 退出超时"))?
                .map_err(anyhow::Error::from)?;
        }
        Ok(())
    }

    async fn queued_generation(&self, task_id: &str) -> Option<u64> {
        self.tasks
            .read()
            .await
            .get(task_id)
            .and_then(|task| (task.status == DownloadStatus::Queued).then_some(task.generation))
    }

    async fn is_queued_generation(&self, task_id: &str, generation: u64) -> bool {
        self.tasks.read().await.get(task_id).is_some_and(|task| {
            task.status == DownloadStatus::Queued && task.generation == generation
        })
    }
}

async fn wait_for_cancel(cancelled: &mut watch::Receiver<bool>) {
    if *cancelled.borrow() {
        return;
    }
    let _ = cancelled.changed().await;
}

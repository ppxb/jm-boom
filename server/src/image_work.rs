use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

const NETWORK_TOTAL_CONCURRENCY: usize = 12;
const NETWORK_BACKGROUND_CONCURRENCY: usize = 8;
const NETWORK_PREFETCH_CONCURRENCY: usize = 4;
const DECODE_TOTAL_CONCURRENCY: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageWorkPriority {
    Foreground,
    Prefetch,
    Download,
}

#[derive(Clone)]
pub struct ImageWorkBudget {
    network: PriorityBudget,
    decode: PriorityBudget,
}

#[derive(Clone)]
struct PriorityBudget {
    inner: Arc<PriorityBudgetInner>,
}

struct PriorityBudgetInner {
    limits: BudgetLimits,
    state: Mutex<PriorityState>,
    notify: Notify,
}

#[derive(Clone, Copy)]
struct BudgetLimits {
    total: usize,
    background: usize,
    prefetch: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PriorityState {
    active_total: usize,
    active_prefetch: usize,
    active_download: usize,
    waiting_foreground: usize,
    waiting_prefetch: usize,
    waiting_download: usize,
}

pub struct ImageWorkPermit {
    inner: Arc<PriorityBudgetInner>,
    priority: ImageWorkPriority,
}

struct WaitingRegistration {
    inner: Arc<PriorityBudgetInner>,
    priority: ImageWorkPriority,
    active: bool,
}

impl ImageWorkBudget {
    pub fn new() -> Self {
        Self {
            network: PriorityBudget::new(BudgetLimits {
                total: NETWORK_TOTAL_CONCURRENCY,
                background: NETWORK_BACKGROUND_CONCURRENCY,
                prefetch: NETWORK_PREFETCH_CONCURRENCY,
            }),
            decode: PriorityBudget::new(BudgetLimits {
                total: DECODE_TOTAL_CONCURRENCY,
                background: DECODE_TOTAL_CONCURRENCY,
                prefetch: DECODE_TOTAL_CONCURRENCY,
            }),
        }
    }

    pub async fn acquire_network(&self, priority: ImageWorkPriority) -> ImageWorkPermit {
        self.network.acquire(priority).await
    }

    pub async fn acquire_decode(&self, priority: ImageWorkPriority) -> ImageWorkPermit {
        self.decode.acquire(priority).await
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(network_total: usize, decode_total: usize) -> Self {
        Self {
            network: PriorityBudget::new(BudgetLimits {
                total: network_total,
                background: network_total,
                prefetch: network_total,
            }),
            decode: PriorityBudget::new(BudgetLimits {
                total: decode_total,
                background: decode_total,
                prefetch: decode_total,
            }),
        }
    }
}

impl PriorityBudget {
    fn new(limits: BudgetLimits) -> Self {
        Self {
            inner: Arc::new(PriorityBudgetInner {
                limits,
                state: Mutex::new(PriorityState::default()),
                notify: Notify::new(),
            }),
        }
    }

    async fn acquire(&self, priority: ImageWorkPriority) -> ImageWorkPermit {
        let mut waiting = WaitingRegistration::new(self.inner.clone(), priority);

        loop {
            let notified = self.inner.notify.notified();
            let acquired = {
                let mut state = self.inner.state.lock().expect("image work state poisoned");
                if can_acquire(&state, self.inner.limits, priority) {
                    waiting.complete_locked(&mut state);
                    state.active_total += 1;
                    match priority {
                        ImageWorkPriority::Foreground => {}
                        ImageWorkPriority::Prefetch => state.active_prefetch += 1,
                        ImageWorkPriority::Download => state.active_download += 1,
                    }
                    true
                } else {
                    false
                }
            };

            if acquired {
                self.inner.notify.notify_waiters();
                return ImageWorkPermit {
                    inner: self.inner.clone(),
                    priority,
                };
            }

            notified.await;
        }
    }

    #[cfg(test)]
    fn state(&self) -> PriorityState {
        *self.inner.state.lock().expect("image work state poisoned")
    }
}

impl WaitingRegistration {
    fn new(inner: Arc<PriorityBudgetInner>, priority: ImageWorkPriority) -> Self {
        {
            let mut state = inner.state.lock().expect("image work state poisoned");
            match priority {
                ImageWorkPriority::Foreground => state.waiting_foreground += 1,
                ImageWorkPriority::Prefetch => state.waiting_prefetch += 1,
                ImageWorkPriority::Download => state.waiting_download += 1,
            }
        }

        Self {
            inner,
            priority,
            active: true,
        }
    }

    fn complete_locked(&mut self, state: &mut PriorityState) {
        if !self.active {
            return;
        }

        decrement_waiter(state, self.priority);
        self.active = false;
    }
}

impl Drop for WaitingRegistration {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let mut state = self.inner.state.lock().expect("image work state poisoned");
        decrement_waiter(&mut state, self.priority);
        drop(state);
        self.inner.notify.notify_waiters();
    }
}

impl Drop for ImageWorkPermit {
    fn drop(&mut self) {
        let mut state = self.inner.state.lock().expect("image work state poisoned");
        state.active_total = state.active_total.saturating_sub(1);
        match self.priority {
            ImageWorkPriority::Foreground => {}
            ImageWorkPriority::Prefetch => {
                state.active_prefetch = state.active_prefetch.saturating_sub(1)
            }
            ImageWorkPriority::Download => {
                state.active_download = state.active_download.saturating_sub(1)
            }
        }
        drop(state);
        self.inner.notify.notify_waiters();
    }
}

fn can_acquire(state: &PriorityState, limits: BudgetLimits, priority: ImageWorkPriority) -> bool {
    if state.active_total >= limits.total {
        return false;
    }

    match priority {
        ImageWorkPriority::Foreground => true,
        ImageWorkPriority::Prefetch => {
            state.waiting_foreground == 0
                && active_background(state) < limits.background
                && state.active_prefetch < limits.prefetch
        }
        ImageWorkPriority::Download => {
            state.waiting_foreground == 0
                && state.waiting_prefetch == 0
                && active_background(state) < limits.background
        }
    }
}

fn active_background(state: &PriorityState) -> usize {
    state.active_prefetch + state.active_download
}

fn decrement_waiter(state: &mut PriorityState, priority: ImageWorkPriority) {
    match priority {
        ImageWorkPriority::Foreground => {
            state.waiting_foreground = state.waiting_foreground.saturating_sub(1)
        }
        ImageWorkPriority::Prefetch => {
            state.waiting_prefetch = state.waiting_prefetch.saturating_sub(1)
        }
        ImageWorkPriority::Download => {
            state.waiting_download = state.waiting_download.saturating_sub(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BudgetLimits, ImageWorkPriority, PriorityBudget};
    use std::time::Duration;
    use tokio::{task::JoinHandle, time::timeout};

    #[tokio::test]
    async fn enforces_fixed_decode_concurrency() {
        let budget = PriorityBudget::new(BudgetLimits {
            total: 4,
            background: 4,
            prefetch: 4,
        });
        let mut permits = Vec::new();
        for _ in 0..4 {
            permits.push(budget.acquire(ImageWorkPriority::Prefetch).await);
        }
        let blocked_budget = budget.clone();
        let mut blocked =
            tokio::spawn(
                async move { blocked_budget.acquire(ImageWorkPriority::Foreground).await },
            );

        assert!(timeout(Duration::from_millis(30), &mut blocked)
            .await
            .is_err());
        blocked.abort();
        drop(permits);
    }

    #[tokio::test]
    async fn foreground_waiter_runs_before_background_waiters() {
        let budget = PriorityBudget::new(BudgetLimits {
            total: 1,
            background: 1,
            prefetch: 1,
        });
        let active = budget.acquire(ImageWorkPriority::Download).await;
        let download = spawn_acquire(budget.clone(), ImageWorkPriority::Download);
        wait_for_waiters(&budget, 0, 0, 1).await;
        let prefetch = spawn_acquire(budget.clone(), ImageWorkPriority::Prefetch);
        wait_for_waiters(&budget, 0, 1, 1).await;
        let foreground = spawn_acquire(budget.clone(), ImageWorkPriority::Foreground);
        wait_for_waiters(&budget, 1, 1, 1).await;
        drop(active);

        let foreground_permit = timeout(Duration::from_secs(1), foreground)
            .await
            .expect("foreground waiter timed out")
            .expect("foreground task failed");
        assert_eq!(budget.state().active_total, 1);
        drop(foreground_permit);

        let prefetch_permit = timeout(Duration::from_secs(1), prefetch)
            .await
            .expect("prefetch waiter timed out")
            .expect("prefetch task failed");
        drop(prefetch_permit);
        let download_permit = timeout(Duration::from_secs(1), download)
            .await
            .expect("download waiter timed out")
            .expect("download task failed");
        drop(download_permit);
    }

    #[tokio::test]
    async fn cancelling_waiter_releases_priority_registration() {
        let budget = PriorityBudget::new(BudgetLimits {
            total: 1,
            background: 1,
            prefetch: 1,
        });
        let active = budget.acquire(ImageWorkPriority::Foreground).await;
        let waiter = spawn_acquire(budget.clone(), ImageWorkPriority::Foreground);
        wait_for_waiters(&budget, 1, 0, 0).await;
        waiter.abort();
        let _ = waiter.await;

        assert_eq!(budget.state().waiting_foreground, 0);
        drop(active);
    }

    fn spawn_acquire(
        budget: PriorityBudget,
        priority: ImageWorkPriority,
    ) -> JoinHandle<super::ImageWorkPermit> {
        tokio::spawn(async move { budget.acquire(priority).await })
    }

    async fn wait_for_waiters(
        budget: &PriorityBudget,
        foreground: usize,
        prefetch: usize,
        download: usize,
    ) {
        timeout(Duration::from_secs(1), async {
            loop {
                let state = budget.state();
                if state.waiting_foreground == foreground
                    && state.waiting_prefetch == prefetch
                    && state.waiting_download == download
                {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("waiter registration timed out");
    }
}

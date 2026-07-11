use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

const TOTAL_CONCURRENCY: usize = 8;
const BACKGROUND_CONCURRENCY: usize = 6;
const PREFETCH_CONCURRENCY: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageWorkPriority {
    Foreground,
    Prefetch,
    Download,
}

#[derive(Clone)]
pub struct ImageWorkBudget {
    inner: Arc<ImageWorkBudgetInner>,
}

struct ImageWorkBudgetInner {
    state: Mutex<ImageWorkState>,
    notify: Notify,
}

#[derive(Default)]
struct ImageWorkState {
    active_total: usize,
    active_prefetch: usize,
    active_download: usize,
    waiting_foreground: usize,
    waiting_prefetch: usize,
}

pub struct ImageWorkPermit {
    inner: Arc<ImageWorkBudgetInner>,
    priority: ImageWorkPriority,
}

struct WaitingRegistration {
    inner: Arc<ImageWorkBudgetInner>,
    priority: ImageWorkPriority,
    active: bool,
}

impl ImageWorkBudget {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ImageWorkBudgetInner {
                state: Mutex::new(ImageWorkState::default()),
                notify: Notify::new(),
            }),
        }
    }

    pub async fn acquire(&self, priority: ImageWorkPriority) -> ImageWorkPermit {
        let mut waiting = WaitingRegistration::new(self.inner.clone(), priority);

        loop {
            let notified = self.inner.notify.notified();
            let acquired = {
                let mut state = self.inner.state.lock().expect("image work state poisoned");
                if can_acquire(&state, priority) {
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
}

impl WaitingRegistration {
    fn new(inner: Arc<ImageWorkBudgetInner>, priority: ImageWorkPriority) -> Self {
        {
            let mut state = inner.state.lock().expect("image work state poisoned");
            match priority {
                ImageWorkPriority::Foreground => state.waiting_foreground += 1,
                ImageWorkPriority::Prefetch => state.waiting_prefetch += 1,
                ImageWorkPriority::Download => {}
            }
        }

        Self {
            inner,
            priority,
            active: true,
        }
    }

    fn complete_locked(&mut self, state: &mut ImageWorkState) {
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

fn can_acquire(state: &ImageWorkState, priority: ImageWorkPriority) -> bool {
    if state.active_total >= TOTAL_CONCURRENCY {
        return false;
    }

    match priority {
        ImageWorkPriority::Foreground => true,
        ImageWorkPriority::Prefetch => {
            state.waiting_foreground == 0
                && active_background(state) < BACKGROUND_CONCURRENCY
                && state.active_prefetch < PREFETCH_CONCURRENCY
        }
        ImageWorkPriority::Download => {
            state.waiting_foreground == 0
                && state.waiting_prefetch == 0
                && active_background(state) < BACKGROUND_CONCURRENCY
        }
    }
}

fn active_background(state: &ImageWorkState) -> usize {
    state.active_prefetch + state.active_download
}

fn decrement_waiter(state: &mut ImageWorkState, priority: ImageWorkPriority) {
    match priority {
        ImageWorkPriority::Foreground => {
            state.waiting_foreground = state.waiting_foreground.saturating_sub(1)
        }
        ImageWorkPriority::Prefetch => {
            state.waiting_prefetch = state.waiting_prefetch.saturating_sub(1)
        }
        ImageWorkPriority::Download => {}
    }
}

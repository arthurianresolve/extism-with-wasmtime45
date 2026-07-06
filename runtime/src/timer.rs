use crate::*;

#[derive(Copy, Clone)]
pub(crate) enum TimeoutAdjustment {
    Extend(std::time::Duration),
    Reduce(std::time::Duration),
}

pub(crate) enum TimerAction {
    Start {
        id: uuid::Uuid,
        engine: Engine,
        duration: Option<std::time::Duration>,
    },
    Adjust {
        id: uuid::Uuid,
        adjustment: TimeoutAdjustment,
    },
    Pause {
        id: uuid::Uuid,
    },
    Resume {
        id: uuid::Uuid,
    },
    Stop {
        id: uuid::Uuid,
    },
    Cancel {
        id: uuid::Uuid,
    },
    Shutdown,
}

pub(crate) struct Timer {
    pub tx: std::sync::mpsc::Sender<TimerAction>,
    pub thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(not(target_family = "windows"))]
extern "C" fn cleanup_timer() {
    let mut timer = match TIMER.lock() {
        Ok(x) => x,
        Err(e) => e.into_inner(),
    };
    drop(timer.take());
}

static TIMER: std::sync::Mutex<Option<Timer>> = std::sync::Mutex::new(None);

type ActiveTimerMap = std::collections::BTreeMap<uuid::Uuid, (Engine, Option<std::time::Instant>)>;
type PausedTimerMap = std::collections::BTreeMap<uuid::Uuid, (Engine, Option<std::time::Duration>)>;

fn deadline_from_now(duration: std::time::Duration) -> Option<std::time::Instant> {
    let now = std::time::Instant::now();
    now.checked_add(duration).or(Some(now))
}

fn adjust_deadline(deadline: &mut std::time::Instant, adjustment: TimeoutAdjustment) {
    match adjustment {
        TimeoutAdjustment::Extend(duration) => {
            if let Some(next) = deadline.checked_add(duration) {
                *deadline = next;
            }
        }
        TimeoutAdjustment::Reduce(duration) => {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let remaining = remaining.checked_sub(duration).unwrap_or_default();
            if let Some(next) = deadline_from_now(remaining) {
                *deadline = next;
            }
        }
    }
}

fn adjust_remaining(remaining: &mut std::time::Duration, adjustment: TimeoutAdjustment) {
    match adjustment {
        TimeoutAdjustment::Extend(duration) => {
            if let Some(next) = remaining.checked_add(duration) {
                *remaining = next;
            }
        }
        TimeoutAdjustment::Reduce(duration) => {
            *remaining = remaining.checked_sub(duration).unwrap_or_default();
        }
    }
}

impl Timer {
    pub(crate) fn tx() -> std::sync::mpsc::Sender<TimerAction> {
        let mut timer = match TIMER.lock() {
            Ok(x) => x,
            Err(e) => e.into_inner(),
        };

        let timer = &mut *timer;

        match timer {
            None => Timer::init(timer),
            Some(t) => t.tx.clone(),
        }
    }

    pub fn init(timer: &mut Option<Timer>) -> std::sync::mpsc::Sender<TimerAction> {
        let (tx, rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut plugins = ActiveTimerMap::new();
            let mut paused = PausedTimerMap::new();

            macro_rules! handle {
                ($x:expr) => {
                    match $x {
                        TimerAction::Start {
                            id,
                            engine,
                            duration,
                        } => {
                            let timeout = duration.and_then(deadline_from_now);
                            trace!(
                                plugin = id.to_string(),
                                "start event with timeout: {:?}",
                                duration
                            );
                            paused.remove(&id);
                            plugins.insert(id, (engine, timeout));
                        }
                        TimerAction::Adjust { id, adjustment } => {
                            trace!(plugin = id.to_string(), "handling timeout adjustment");
                            if let Some((_engine, Some(deadline))) = plugins.get_mut(&id) {
                                adjust_deadline(deadline, adjustment);
                            } else if let Some((_engine, Some(remaining))) = paused.get_mut(&id) {
                                adjust_remaining(remaining, adjustment);
                            }
                        }
                        TimerAction::Pause { id } => {
                            trace!(plugin = id.to_string(), "handling timeout pause");
                            if let Some((engine, deadline)) = plugins.remove(&id) {
                                let remaining = deadline.map(|deadline| {
                                    deadline.saturating_duration_since(std::time::Instant::now())
                                });
                                paused.insert(id, (engine, remaining));
                            }
                        }
                        TimerAction::Resume { id } => {
                            trace!(plugin = id.to_string(), "handling timeout resume");
                            if let Some((engine, remaining)) = paused.remove(&id) {
                                let deadline = remaining.and_then(deadline_from_now);
                                plugins.insert(id, (engine, deadline));
                            }
                        }
                        TimerAction::Stop { id } => {
                            trace!(plugin = id.to_string(), "handling stop event");
                            plugins.remove(&id);
                            paused.remove(&id);
                        }
                        TimerAction::Cancel { id } => {
                            trace!(plugin = id.to_string(), "handling cancel event");
                            if let Some((engine, _)) = plugins.remove(&id) {
                                engine.increment_epoch();
                            }
                            if let Some((engine, _)) = paused.remove(&id) {
                                engine.increment_epoch();
                            }
                        }
                        TimerAction::Shutdown => {
                            trace!("Shutting down timer");
                            for (id, (engine, _)) in plugins.iter() {
                                trace!(plugin = id.to_string(), "handling shutdown event");
                                engine.increment_epoch();
                            }
                            for (id, (engine, _)) in paused.iter() {
                                trace!(plugin = id.to_string(), "handling shutdown event");
                                engine.increment_epoch();
                            }
                            return;
                        }
                    }
                };
            }

            loop {
                if plugins.is_empty() {
                    if let Ok(x) = rx.recv() {
                        handle!(x);
                    }
                }

                let mut timeout: Option<std::time::Duration> = None;

                plugins.retain(|_k, (engine, end)| {
                    if let Some(end) = end {
                        let now = std::time::Instant::now();
                        if *end <= now {
                            engine.increment_epoch();
                            return false;
                        } else {
                            let time_left =
                                (*end - now).saturating_sub(std::time::Duration::from_millis(1));
                            if let Some(t) = &timeout {
                                if time_left < *t {
                                    timeout = Some(time_left);
                                }
                            } else {
                                timeout = Some(time_left);
                            }
                        }
                    }

                    true
                });

                if let Some(timeout) = timeout {
                    if let Ok(x) = rx.recv_timeout(timeout) {
                        handle!(x)
                    }
                } else if let Ok(x) = rx.recv() {
                    handle!(x)
                }
            }
        });
        *timer = Some(Timer {
            thread: Some(thread),
            tx: tx.clone(),
        });
        trace!("Extism timer created");

        #[cfg(not(target_family = "windows"))]
        unsafe {
            libc::atexit(cleanup_timer);
        }

        tx
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let _ = self.tx.send(TimerAction::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

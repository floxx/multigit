//! Counting semaphore with an adjustable limit.

use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
struct State {
    in_use: usize,
    max: usize,
}

/// Limits how many jobs run at once; the limit can change at runtime.
#[derive(Debug)]
pub struct Semaphore {
    state: Mutex<State>,
    cv: Condvar,
}

/// Held while a job runs; releases its slot on drop.
#[derive(Debug)]
pub struct Permit {
    semaphore: Arc<Semaphore>,
}

impl Semaphore {
    pub fn new(max: usize) -> Self {
        Self {
            state: Mutex::new(State {
                in_use: 0,
                max: max.max(1),
            }),
            cv: Condvar::new(),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Block until a slot is free.
    pub fn acquire(self: &Arc<Self>) -> Permit {
        let mut state = self.state();
        while state.in_use >= state.max {
            state = self.cv.wait(state).unwrap_or_else(|p| p.into_inner());
        }
        state.in_use += 1;
        Permit {
            semaphore: self.clone(),
        }
    }

    /// New limit (at least 1); waiting jobs start if it grew.
    pub fn set_max(&self, max: usize) {
        self.state().max = max.max(1);
        self.cv.notify_all();
    }

    pub fn in_use(&self) -> usize {
        self.state().in_use
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.semaphore.state().in_use -= 1;
        self.semaphore.cv.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn never_exceeds_limit() {
        let sem = Arc::new(Semaphore::new(2));
        let peak = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let (sem, peak) = (sem.clone(), peak.clone());
                thread::spawn(move || {
                    let _p = sem.acquire();
                    peak.fetch_max(sem.in_use(), Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(10));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert!(peak.load(Ordering::SeqCst) <= 2);
        assert_eq!(sem.in_use(), 0);
    }

    #[test]
    fn raising_limit_wakes_waiters() {
        let sem = Arc::new(Semaphore::new(1));
        let first = sem.acquire();
        let sem2 = sem.clone();
        let waiter = thread::spawn(move || drop(sem2.acquire()));
        thread::sleep(Duration::from_millis(20));
        sem.set_max(2);
        waiter.join().unwrap();
        drop(first);
        assert_eq!(sem.in_use(), 0);
    }

    #[test]
    fn zero_is_clamped_to_one() {
        let sem = Arc::new(Semaphore::new(0));
        drop(sem.acquire());
    }
}

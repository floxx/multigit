//! Decides when the periodic status refresh and background fetch are due.

use std::time::{Duration, Instant};

/// Seconds between status refreshes of all cards.
pub const STATUS_INTERVAL: Duration = Duration::from_secs(10);

/// Timer state of the two periodic jobs. Both are due right at start.
#[derive(Debug, Clone, Default)]
pub struct Schedule {
    last_status: Option<Instant>,
    last_fetch: Option<Instant>,
}

/// What to start on this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Due {
    Nothing,
    Status,
    /// Fetch (which also refreshes status).
    Fetch,
}

impl Schedule {
    /// Check the timers at `now`; a due job resets its timer. A due fetch also
    /// counts as a status refresh.
    pub fn tick(&mut self, now: Instant, fetch_interval: Duration) -> Due {
        let elapsed = |last: Option<Instant>, every: Duration| {
            last.is_none_or(|t| now.saturating_duration_since(t) >= every)
        };
        if elapsed(self.last_fetch, fetch_interval) {
            self.last_fetch = Some(now);
            self.last_status = Some(now);
            Due::Fetch
        } else if elapsed(self.last_status, STATUS_INTERVAL) {
            self.last_status = Some(now);
            Due::Status
        } else {
            Due::Nothing
        }
    }

    /// Time until the next job is due, for scheduling a repaint.
    pub fn until_next(&self, now: Instant, fetch_interval: Duration) -> Duration {
        let left = |last: Option<Instant>, every: Duration| {
            last.map_or(Duration::ZERO, |t| {
                every.saturating_sub(now.saturating_duration_since(t))
            })
        };
        left(self.last_status, STATUS_INTERVAL).min(left(self.last_fetch, fetch_interval))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_first_then_status_every_ten_seconds() {
        let mut s = Schedule::default();
        let t0 = Instant::now();
        let minute = Duration::from_secs(60);
        assert_eq!(s.tick(t0, minute), Due::Fetch);
        assert_eq!(s.tick(t0 + Duration::from_secs(5), minute), Due::Nothing);
        assert_eq!(s.tick(t0 + Duration::from_secs(10), minute), Due::Status);
        assert_eq!(s.tick(t0 + Duration::from_secs(15), minute), Due::Nothing);
        assert_eq!(s.tick(t0 + Duration::from_secs(20), minute), Due::Status);
        assert_eq!(s.tick(t0 + Duration::from_secs(60), minute), Due::Fetch);
        assert_eq!(s.tick(t0 + Duration::from_secs(65), minute), Due::Nothing);
        assert_eq!(s.tick(t0 + Duration::from_secs(70), minute), Due::Status);
    }

    #[test]
    fn until_next() {
        let mut s = Schedule::default();
        let t0 = Instant::now();
        let minute = Duration::from_secs(60);
        assert_eq!(s.until_next(t0, minute), Duration::ZERO);
        s.tick(t0, minute);
        assert_eq!(
            s.until_next(t0 + Duration::from_secs(3), minute),
            Duration::from_secs(7)
        );
    }
}

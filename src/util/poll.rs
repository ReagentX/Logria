use std::collections::vec_deque::VecDeque;

use std::cmp::{max, min};
use std::time::Duration;

use crate::constants::cli::poll_rate::{DEFAULT, FASTEST, SLOWEST};

pub fn ms_per_message(timestamp: Duration, messages: u64) -> u64 {
    (timestamp.as_millis() as u64)
        .checked_div(messages)
        .unwrap_or(SLOWEST)
        .clamp(FASTEST, SLOWEST)
}

#[derive(Debug)]
pub struct Backoff {
    num_cycles: u64,    // The number of times we have increased the poll rate
    previous_base: u64, // The previous amount we increased the poll rate by
}

impl Backoff {
    pub fn new() -> Backoff {
        Backoff {
            num_cycles: 1,
            previous_base: DEFAULT,
        }
    }

    pub fn determine_poll_rate(&mut self, poll_rate: u64) -> u64 {
        // Poll rate is capped to SLOWEST in the reader
        if poll_rate > self.previous_base || poll_rate == SLOWEST {
            let increase = self.previous_base * self.num_cycles;
            self.num_cycles = self.num_cycles.checked_add(1).unwrap_or(1);
            self.previous_base = min(min(self.previous_base + increase, poll_rate), SLOWEST);
            self.previous_base
        } else {
            self.num_cycles = 1;
            self.previous_base = max(poll_rate, 1); // Ensure we are always positive
            poll_rate
        }
    }
}

#[derive(Debug)]
pub struct RollingMean {
    pub deque: VecDeque<u64>,
    sum: u64,
    size: u64,
    max_size: usize,
    tracker: Backoff,
}

impl RollingMean {
    pub fn new(max_size: usize) -> RollingMean {
        RollingMean {
            deque: VecDeque::with_capacity(max_size),
            sum: 0,
            size: 0,
            max_size,
            tracker: Backoff::new(),
        }
    }

    pub fn update(&mut self, item: u64) {
        if self.deque.len() >= self.max_size {
            self.sum -= self.deque.pop_back().unwrap_or(0);
        } else {
            self.size += 1;
        }
        let adjusted_item = self.tracker.determine_poll_rate(item);
        self.deque.push_front(adjusted_item);
        self.sum += adjusted_item;
    }

    pub fn mean(&self) -> u64 {
        self.sum.checked_div(self.size).unwrap_or(0)
    }

    pub fn reset(&mut self) {
        self.sum = 0;
        self.size = 0;
        self.deque.clear();
    }
}

#[cfg(test)]
mod mean_track_tests {
    use crate::util::poll::RollingMean;

    #[test]
    fn can_create() {
        let tracker = RollingMean::new(5);
        assert_eq!(tracker.sum, 0);
        assert_eq!(tracker.max_size, 5);
        assert_eq!(tracker.size, 0);
    }

    #[test]
    fn cant_exceed_capacity() {
        let mut tracker = RollingMean::new(2);
        tracker.update(1);
        tracker.update(2);
        tracker.update(3);
        tracker.update(4);
        assert_eq!(tracker.sum, 7);
        assert_eq!(tracker.deque.len(), 2);
        assert_eq!(tracker.size, 2);
    }

    #[test]
    fn can_get_mean_full() {
        let mut tracker = RollingMean::new(5);
        tracker.update(1);
        tracker.update(2);
        tracker.update(3);
        tracker.update(4);
        tracker.update(5);
        assert_eq!(tracker.sum, 15);
        assert_eq!(tracker.mean(), 3);
        assert_eq!(tracker.size, 5);
    }

    #[test]
    fn can_get_mean_under() {
        let mut tracker = RollingMean::new(5);
        tracker.update(1);
        tracker.update(2);
        tracker.update(3);
        assert_eq!(tracker.sum, 6);
        assert_eq!(tracker.mean(), 2);
        assert_eq!(tracker.size, 3);
    }

    #[test]
    fn can_get_mean_over() {
        let mut tracker = RollingMean::new(5);
        tracker.update(1);
        tracker.update(2);
        tracker.update(3);
        tracker.update(4);
        tracker.update(5);
        tracker.update(6);
        tracker.update(7);
        assert_eq!(tracker.sum, 25);
        assert_eq!(tracker.mean(), 5);
        assert_eq!(tracker.size, 5);
    }
}

#[cfg(test)]
mod tracker_tests {
    use crate::util::poll::Backoff;

    #[test]
    fn can_create() {
        let tracker = Backoff::new();
        assert_eq!(tracker.num_cycles, 1);
    }

    #[test]
    fn stays_low_no_slowest() {
        let mut tracker = Backoff::new();
        tracker.determine_poll_rate(25);
        tracker.determine_poll_rate(900);
        tracker.determine_poll_rate(34);
        assert_eq!(tracker.num_cycles, 1);
    }

    #[test]
    fn stays_low_when_less_than_max() {
        let mut tracker = Backoff::new();
        let result = tracker.determine_poll_rate(25);
        assert_eq!(result, 25);
        assert_eq!(tracker.num_cycles, 1);

        let result = tracker.determine_poll_rate(1000);
        assert_eq!(result, 50);
        assert_eq!(tracker.num_cycles, 2);

        let result = tracker.determine_poll_rate(900);
        assert_eq!(result, 150);
        assert_eq!(tracker.num_cycles, 3);
    }

    #[test]
    fn expands_slowly() {
        let mut tracker = Backoff::new();
        tracker.determine_poll_rate(34);
        assert_eq!(tracker.previous_base, 34);

        let mut result = tracker.determine_poll_rate(1000);
        assert_eq!(tracker.num_cycles, 2);
        assert_eq!(tracker.previous_base, 34 * 2);
        assert_eq!(result, 68);

        result = tracker.determine_poll_rate(1000);
        assert_eq!(tracker.num_cycles, 3);
        assert_eq!(tracker.previous_base, 68 + (68 * 2));
        assert_eq!(result, 68 + (68 * 2));

        result = tracker.determine_poll_rate(1000);
        assert_eq!(tracker.num_cycles, 4);
        assert_eq!(tracker.previous_base, 204 + (204 * 3));
        assert_eq!(result, 204 + (204 * 3));

        result = tracker.determine_poll_rate(1000);
        assert_eq!(tracker.num_cycles, 5);
        assert_eq!(tracker.previous_base, 1000);
        assert_eq!(result, 1000);

        result = tracker.determine_poll_rate(34);
        assert_eq!(tracker.num_cycles, 1);
        assert_eq!(tracker.previous_base, 34);
        assert_eq!(result, 34);
    }
}

use std::collections::vec_deque::VecDeque;

use std::cmp::{max, min};

use crate::constants::cli::poll_rate::SLOWEST;

#[derive(Debug)]
pub struct Tracker {
    num_cycles: u64, // The number of times we have increased, i.e.
    previous_base: u64,
}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker {
            num_cycles: 1,
            previous_base: 0,
        }
    }

    pub fn determine_poll_rate(&mut self, poll_rate: u64) -> u64 {
        if poll_rate == SLOWEST {
            let increase = self.previous_base * self.num_cycles;
            self.num_cycles = self.num_cycles.checked_add(1).unwrap_or(1);
            self.previous_base = min(max(self.previous_base + increase, 1), SLOWEST);
            // println!("cycles: {}, base: {}", self.num_cycles, self.previous_base);
            self.previous_base
        } else {
            self.num_cycles = 1;
            self.previous_base = poll_rate;
            poll_rate
        }
    }
}

#[derive(Debug)]
pub struct MeanTrack {
    pub deque: VecDeque<u64>,
    sum: u64,
    size: u64,
    max_size: usize,
    tracker: Tracker,
}

impl MeanTrack {
    pub fn new(max_size: usize) -> MeanTrack {
        MeanTrack {
            deque: VecDeque::with_capacity(max_size),
            sum: 0,
            size: 0,
            max_size,
            tracker: Tracker::new(),
        }
    }

    pub fn update(&mut self, item: u64) {
        if self.deque.len() >= self.max_size {
            self.sum -= self.deque.pop_back().unwrap();
        } else {
            self.size += 1;
        }
        let adjusted_item = self.tracker.determine_poll_rate(item);
        self.deque.push_front(adjusted_item);
        self.sum += adjusted_item;
    }

    pub fn mean(&self) -> u64 {
        self.sum / self.size
    }

    pub fn reset(&mut self) {
        self.sum = 0;
        self.size = 0;
        self.deque.clear();
    }
}

mod mean_track_tests {
    use crate::util::poll::MeanTrack;

    #[test]
    fn can_create() {
        let tracker = MeanTrack::new(5);
        assert_eq!(tracker.sum, 0);
        assert_eq!(tracker.max_size, 5);
        assert_eq!(tracker.size, 0);
    }

    #[test]
    fn cant_exceed_capacity() {
        let mut tracker = MeanTrack::new(2);
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
        let mut tracker = MeanTrack::new(5);
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
        let mut tracker = MeanTrack::new(5);
        tracker.update(1);
        tracker.update(2);
        tracker.update(3);
        assert_eq!(tracker.sum, 6);
        assert_eq!(tracker.mean(), 2);
        assert_eq!(tracker.size, 3);
    }

    #[test]
    fn can_get_mean_over() {
        let mut tracker = MeanTrack::new(5);
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
    use crate::util::poll::Tracker;

    #[test]
    fn can_create() {
        let tracker = Tracker::new();
        assert_eq!(tracker.num_cycles, 1);
    }

    #[test]
    fn stays_low_no_slowest() {
        let mut tracker = Tracker::new();
        tracker.determine_poll_rate(25);
        tracker.determine_poll_rate(900);
        tracker.determine_poll_rate(34);
        assert_eq!(tracker.num_cycles, 1);
    }

    #[test]
    fn expands_slowly() {
        let mut tracker = Tracker::new();
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

use std::collections::vec_deque::VecDeque;

#[derive(Debug)]
pub struct MeanTrack {
    pub deque: VecDeque<u64>,
    sum: u64,
    size: u64,
    max_size: usize, // Avoids casting `size` every insertion
}

impl MeanTrack {
    pub fn new(max_size: usize) -> MeanTrack {
        MeanTrack {
            deque: VecDeque::with_capacity(max_size),
            sum: 0,
            size: 0,
            max_size,
        }
    }

    pub fn update(&mut self, item: u64) {
        if self.deque.len() >= self.max_size {
            self.sum -= self.deque.pop_back().unwrap();
        } else {
            self.size += 1;
        }
        self.deque.push_front(item);
        self.sum += item;
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

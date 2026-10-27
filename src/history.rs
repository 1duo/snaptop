//! Fixed-capacity history buffers for the sparklines.

use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct History {
    data: VecDeque<f32>,
    capacity: usize,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, value: f32) {
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value.max(0.0));
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn as_slice(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_newest_values() {
        let mut h = History::new(3);
        for v in [1.0, 2.0, 3.0, 4.0] {
            h.push(v);
        }
        assert_eq!(h.as_slice(), vec![2.0, 3.0, 4.0]);
        assert_eq!(h.as_slice().len(), 3);
    }

    #[test]
    fn keeps_values_non_negative() {
        let mut h = History::new(2);
        h.push(-5.0);
        h.push(250.0);
        assert_eq!(h.as_slice(), vec![0.0, 250.0]);
    }
}

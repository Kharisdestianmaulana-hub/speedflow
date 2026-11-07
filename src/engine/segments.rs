use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Segment {
    pub index: u32,
    #[allow(dead_code)]
    pub start_byte: u64,
    #[allow(dead_code)]
    pub end_byte: u64, // inclusive
    pub downloaded_bytes: u64,
    pub is_completed: bool,
}

impl Segment {
    pub fn new(index: u32, start_byte: u64, end_byte: u64) -> Self {
        Self {
            index,
            start_byte,
            end_byte,
            downloaded_bytes: 0,
            is_completed: false,
        }
    }

    pub fn total_bytes(&self) -> u64 {
        if self.end_byte >= self.start_byte {
            (self.end_byte - self.start_byte) + 1
        } else {
            0
        }
    }

    pub fn current_byte_offset(&self) -> u64 {
        self.start_byte + self.downloaded_bytes
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.total_bytes().saturating_sub(self.downloaded_bytes)
    }
}

/// Divides a file of `total_bytes` into `num_segments` non-overlapping segments.
pub fn calculate_segments(total_bytes: u64, num_segments: u32) -> Vec<Segment> {
    if total_bytes == 0 || num_segments == 0 {
        return Vec::new();
    }

    let num_segments = num_segments.max(1) as u64;
    let segment_size = total_bytes / num_segments;
    let mut remainder = total_bytes % num_segments;

    let mut segments = Vec::with_capacity(num_segments as usize);
    let mut current_start = 0u64;

    for i in 0..num_segments {
        let extra = if remainder > 0 {
            remainder -= 1;
            1
        } else {
            0
        };
        let current_size = segment_size + extra;
        if current_size == 0 {
            continue;
        }

        let current_end = current_start + current_size - 1;
        segments.push(Segment::new(i as u32, current_start, current_end));
        current_start = current_end + 1;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_calculation_even() {
        let segs = calculate_segments(1000, 4);
        assert_eq!(segs.len(), 4);
        assert_eq!(segs[0], Segment::new(0, 0, 249));
        assert_eq!(segs[1], Segment::new(1, 250, 499));
        assert_eq!(segs[2], Segment::new(2, 500, 749));
        assert_eq!(segs[3], Segment::new(3, 750, 999));
    }

    #[test]
    fn test_segment_calculation_remainder() {
        let segs = calculate_segments(10, 3);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].total_bytes(), 4); // 0..=3
        assert_eq!(segs[1].total_bytes(), 3); // 4..=6
        assert_eq!(segs[2].total_bytes(), 3); // 7..=9
        assert_eq!(segs[0].start_byte, 0);
        assert_eq!(segs[2].end_byte, 9);
    }
}

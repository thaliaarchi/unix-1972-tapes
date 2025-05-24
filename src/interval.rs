use std::ops::Range;

use anyhow::{Result, bail};

/// A set of disjoint `Range<usize>` intervals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntervalSet {
    intervals: Vec<Range<usize>>,
}

impl IntervalSet {
    /// Creates a new interval set with the given bounds.
    pub fn new(bounds: Range<usize>) -> Self {
        IntervalSet {
            intervals: vec![0..bounds.start, bounds.end..usize::MAX],
        }
    }

    /// Attempts to inserts an interval and returns whether it could be
    /// inserted.
    pub fn insert(&mut self, interval: Range<usize>) -> Result<()> {
        let x = normalize(interval);
        if x.is_empty() {
            bail!("empty interval");
        }
        let i = match self.intervals.binary_search_by(|y| y.end.cmp(&x.start)) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let Some(after) = self.intervals.get(i) else {
            bail!("out of bounds");
        };
        let before = &self.intervals[i - 1];
        let gap = before.end..after.start;
        if !(gap.start <= x.start && x.end <= gap.end) {
            bail!("not disjoint");
        }
        if gap.start < x.start && x.end < gap.end {
            self.intervals.insert(i, x);
        } else if gap.start == x.start && x.end == gap.end {
            self.intervals[i - 1].end = self.intervals[i].end;
            self.intervals.remove(i);
        } else if gap.start == x.start {
            self.intervals[i - 1].end = x.end;
        } else if gap.end == x.end {
            self.intervals[i].start = x.start;
        } else {
            unreachable!();
        }
        Ok(())
    }

    pub fn get_disjoint(&self, interval: Range<usize>, out: &mut Vec<Range<usize>>) {
        let mut x = normalize(interval);
        let i = match self.intervals.binary_search_by(|y| y.end.cmp(&x.start)) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        for y in &self.intervals[i..] {
            debug_assert!(y.end > x.start);
            if x.start < y.start {
                out.push(x.start..y.start.min(x.end));
            }
            x.start = y.end;
            if x.start >= x.end {
                break;
            }
        }
    }
}

fn normalize(interval: Range<usize>) -> Range<usize> {
    interval.start..interval.end.max(interval.start)
}

#[cfg(test)]
mod tests {
    use super::*;

    const M: usize = usize::MAX;

    #[test]
    fn insert() {
        let tests = [
            (0..0, None),
            (0..1, Some(vec![0..3, 7..10, 20..M])),
            (1..3, None),
            (1..5, None),
            (2..6, None),
            (8..9, None),
            (3..5, Some(vec![0..0, 1..5, 7..10, 20..M])),
            (4..6, Some(vec![0..0, 1..3, 4..6, 7..10, 20..M])),
            (5..7, Some(vec![0..0, 1..3, 5..10, 20..M])),
            (10..20, Some(vec![0..0, 1..3, 7..M])),
            (19..21, None),
            (20..20, None),
            (M..M, None),
        ];
        for (interval, expect) in tests {
            let mut set = IntervalSet {
                intervals: vec![0..0, 1..3, 7..10, 20..M],
            };
            let inserted = set.insert(interval.clone());
            assert_eq!(
                inserted.ok().map(|()| set.intervals),
                expect,
                "insert({interval:?})"
            );
        }
    }

    #[test]
    fn get_disjoint() {
        let set = IntervalSet {
            intervals: vec![0..0, 1..3, 7..10, 20..M],
        };
        let tests = [
            (0..M, vec![0..1, 3..7, 10..20]),
            (4..6, vec![4..6]),
            (7..9, vec![]),
        ];
        for (interval, expect) in tests {
            let mut got = Vec::new();
            set.get_disjoint(interval.clone(), &mut got);
            assert_eq!(got, expect, "get_disjoint({interval:?})");
        }
    }
}

use std::io::BufRead;
use std::str::FromStr;

use ranges::Range;

use anyhow::{Result, anyhow};
use common_utils::get_buffered_input;
use itertools::Itertools;

fn main() -> Result<()> {
    let contains_count = get_buffered_input()
        .lines()
        .map(|line_res| {
            let line = line_res?;
            let mut split = line.split(',');
            match (split.next(), split.next(), split.next()) {
                (Some(a), Some(b), None) => Ok((Range::from_str(a)?, Range::from_str(b)?)),
                _ => Err(anyhow!("Expected RANGE,RANGE got {}", line))
            }
        })
        .filter_ok(|(a, b)| a.contains(*b) || b.contains(*a))
        .fold_ok(0usize, |acc, _| acc + 1)?;
    println!("{} contained pairs", contains_count);

    let overlapping_count = get_buffered_input()
        .lines()
        .map(|line_res| {
            let line = line_res?;
            let mut split = line.split(',');
            match (split.next(), split.next(), split.next()) {
                (Some(a), Some(b), None) => Ok((Range::from_str(a)?, Range::from_str(b)?)),
                _ => Err(anyhow!("Expected RANGE,RANGE got {}", line))
            }
        })
        .filter_ok(|(a, b)| a.overlaps(*b))
        .fold_ok(0usize, |acc, _| acc + 1)?;
    println!("{} overlapping pairs", overlapping_count);
    Ok(())
}

mod ranges {
    use std::str::FromStr;
    use anyhow::{anyhow, Result};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Range {
        start: usize,
        end: usize
    }

    impl FromStr for Range {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self> {
            let mut parts = s.split('-');
            if let (Some(a), Some(b), None) = (parts.next(), parts.next(), parts.next()) {
                Ok(Range {
                    start: a.parse()?,
                    end: b.parse()?
                })
            } else {
                Err(anyhow!("Expected A-B, got {}", s))
            }
        }
    }

    impl Range {
        pub fn contains(self, other: Self) -> bool {
            self.start <= other.start && self.end >= other.end
        }

        pub fn overlaps(self, other: Self) -> bool {
            let other_range = other.start..=other.end;
            other_range.contains(&self.start) ||
                other_range.contains(&self.end) ||
                self.contains(other)
        }
    }
}

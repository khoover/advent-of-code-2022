use std::io::BufRead;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use common_utils::get_buffered_input;
use itertools::Itertools;

fn main() -> Result<()> {
    let input = get_buffered_input();
    let mut lines = input.lines();
    let mut state = get_initial_state(lines.by_ref())?;
    lines.map(|line_res| MoveSpec::from_str(&(line_res?)))
        .try_for_each(|move_spec_res| {
            move_spec_res.and_then(|move_spec| move_spec.do_move(&mut state))
        })?;
    let output_utf8_bytes: Vec<u8> = state.into_iter()
        .map(|v| v.last().copied().unwrap_or(b' '))
        .collect();
    println!("Top crates are {}", std::str::from_utf8(&output_utf8_bytes)?);
    Ok(())
}

fn get_initial_state(iter: impl Iterator<Item = std::io::Result<String>>) -> Result<Vec<Vec<u8>>> {
    let mut lines: Vec<String> = iter.take_while(|line_res| {
            match line_res {
                Ok(s) => !s.is_empty(),
                Err(_) => true
            }
        })
        .try_collect()?;
    let last_line = lines.pop().ok_or_else(|| anyhow!("Should have at least one line."))?;
    let num_cols = last_line.split(' ').filter(|s| !s.is_empty()).count();
    let mut stacks = vec![Vec::new(); num_cols];
    let push_res = lines.into_iter().rev().try_for_each(|line| {
        line.as_bytes()
            .chunks(4)
            .into_iter()
            .enumerate()
            .try_for_each(|(i, chunk)| {
               match chunk.get(..3) {
                   Some(&[b' ', b' ', b' ']) => Ok(()),
                   Some(&[b'[', val, b']']) => {
                       stacks.get_mut(i).ok_or_else(|| anyhow!("Tried indexing out-of-bounds stack."))?.push(val);
                       Ok(())
                   },
                   _ => Err(anyhow!("Expected [N], got {:?}", std::str::from_utf8(chunk)?))
               }
            })
    });
    push_res.map(|_| stacks)
}

struct MoveSpec {
    count: usize,
    from: usize,
    to: usize
}

impl FromStr for MoveSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let (
            _mov,
            count,
            _from_text,
            from,
            _to_text,
            to
        ): (&str, &str, &str, &str, &str, &str) = s.split(' ')
             .collect_tuple()
             .ok_or_else(|| anyhow!("Expected 'move N from A to B', got {}", s))?;
        // skipping checking the text bits, we'll assume they're right
        Ok(Self {
            count: count.parse()?,
            from: from.parse()?,
            to: to.parse()?
        })
    }
}

impl MoveSpec {
    fn do_move(&self, stacks: &mut [Vec<u8>]) -> Result<()> {
        use index_many::generic::{UnsortedIndices, get_many_mut};

        let [src, dst] = get_many_mut(stacks, UnsortedIndices([self.from - 1, self.to - 1]))
            .ok_or_else(|| anyhow!("From/To are out-of-bounds or the same."))?;
        let tail = src.drain((src.len() - self.count)..);
        dst.extend(tail);
        Ok(())
    }
}

use std::collections::HashSet;
use std::io::BufRead;

use anyhow::{bail, Result};
use common_utils::get_buffered_input;
use tap::pipe::Pipe;

fn main() -> Result<()> {
    let count = get_buffered_input()
        .lines()
        .pipe(get_distinct_spaces::<2>)?;
    println!("Num distinct is {}", count);

    let count = get_buffered_input()
        .lines()
        .pipe(get_distinct_spaces::<10>)?;
    println!("Num long distinct is {}", count);
    Ok(())
}

#[test]
fn test_sample() {
    static SAMPLE: &str = "R 4\nU 4\nL 3\nD 1\nR 4\nD 1\nL 5\nR 2";
    assert_eq!(
        13,
        get_distinct_spaces::<2>(SAMPLE.lines().map(|line| { Ok(line.to_owned()) })).unwrap()
    );
    assert_eq!(
        1,
        get_distinct_spaces::<10>(SAMPLE.lines().map(|line| { Ok(line.to_owned()) })).unwrap()
    );
}

fn get_distinct_spaces<const KNOTS: usize>(
    iter: impl Iterator<Item = std::io::Result<String>>,
) -> Result<usize> {
    let mut visited_spaces: HashSet<(i16, i16)> = HashSet::new();
    visited_spaces.insert((0, 0));
    iter.map(|line_res| -> Result<Move> {
        let line = line_res?;
        let (left, mov) = parse::parse_move(&line)?;
        if !left.is_empty() {
            bail!("Shouldn't have had leftover text, had {}", left);
        }
        Ok(mov)
    })
    .try_fold([(0, 0); KNOTS], |mut knots, mov| {
        let Move { dir, steps } = mov?;
        let (x, y) = dir.into();
        for _ in 0..steps {
            knots[0].0 += x;
            knots[0].1 += y;
            for i in 0..KNOTS - 1 {
                if !is_adjacent(knots[i], knots[i + 1]) {
                    knots[i + 1] = move_towards_head(knots[i], knots[i + 1]);
                } else {
                    break;
                }
            }
            visited_spaces.insert(knots[KNOTS - 1]);
        }
        Ok::<_, anyhow::Error>(knots)
    })?;
    Ok(visited_spaces.len())
}

fn is_adjacent(head: (i16, i16), tail: (i16, i16)) -> bool {
    head.0.abs_diff(tail.0) <= 1 && head.1.abs_diff(tail.1) <= 1
}

fn move_towards_head(head: (i16, i16), tail: (i16, i16)) -> (i16, i16) {
    (
        tail.0 + (head.0 - tail.0).signum(),
        tail.1 + (head.1 - tail.1).signum(),
    )
}

#[derive(Debug, Clone, Copy)]
struct Move {
    pub dir: Direction,
    pub steps: u16,
}

impl From<Move> for (i16, i16) {
    fn from(mov: Move) -> (i16, i16) {
        let (x, y) = mov.dir.into();
        (x * mov.steps as i16, y * mov.steps as i16)
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for (i16, i16) {
    fn from(dir: Direction) -> (i16, i16) {
        match dir {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

mod parse {
    use super::*;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::u16 as nom_u16,
        combinator::{map, value},
        error::Error,
        sequence::separated_pair,
        Finish, IResult,
    };

    use anyhow::Result;

    fn direction(s: &str) -> IResult<&str, Direction> {
        alt((
            value(Direction::Up, tag("U")),
            value(Direction::Down, tag("D")),
            value(Direction::Left, tag("L")),
            value(Direction::Right, tag("R")),
        ))(s)
    }

    pub(super) fn parse_move(s: &str) -> Result<(&str, Move)> {
        map(
            separated_pair(direction, tag(" "), nom_u16),
            |(dir, steps)| Move { dir, steps },
        )(s)
        .finish()
        .map_err(|e| {
            Error {
                input: e.input.to_owned(),
                code: e.code,
            }
            .into()
        })
    }
}

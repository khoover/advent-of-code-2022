use std::io::BufRead;
use std::ops::ControlFlow;

use color_eyre::eyre::{eyre, Result};
use common_utils::get_buffered_input;
use itertools::{EitherOrBoth, Itertools};
use nom::{
    branch::alt,
    character::complete::char as nom_char,
    character::complete::u32 as nom_u32,
    combinator::{all_consuming, map},
    multi::separated_list0,
    sequence::delimited,
    Finish, IResult,
};
use tap::{pipe::Pipe, Tap};

fn main() -> Result<()> {
    color_eyre::install()?;

    // Part 1
    let index_sum = get_buffered_input()
        .lines()
        .batching(make_pairs)
        .enumerate()
        .map(|(i, val)| (i + 1, val))
        .filter(|(_, res)| {
            let (a, b) = match res {
                Ok((a, b)) => (a, b),
                Err(_) => return true,
            };
            match a.correct_order(b) {
                ControlFlow::Continue(_) => {
                    println!("Comparison ended undecided, dunno what to do.");
                    true
                }
                ControlFlow::Break(val) => val,
            }
        })
        .try_fold(0usize, |acc, (i, res)| match res {
            Ok(_) => Ok(acc + i),
            Err(e) => Err(e),
        })?;
    println!("{}", index_sum);

    // Part 2
    let divider1 = IntOrVec::Vec(vec![IntOrVec::Vec(vec![IntOrVec::Int(2)])]);
    let divider2 = IntOrVec::Vec(vec![IntOrVec::Vec(vec![IntOrVec::Int(6)])]);

    let ordered_packets = get_buffered_input()
        .lines()
        .batching(make_pairs)
        .map(|res| res.unwrap())
        .flat_map(|(a, b)| [a, b])
        .chain([divider1.clone(), divider2.clone()])
        .collect_vec()
        .tap_mut(|v| v.sort_unstable());
    let pos1 = ordered_packets.binary_search(&divider1).unwrap() + 1;
    let pos2 = ordered_packets.binary_search(&divider2).unwrap() + 1;
    println!("Dividers at {}", pos1 * pos2);
    Ok(())
}

fn make_pairs(
    line_iter: &mut impl Iterator<Item = std::io::Result<String>>,
) -> Option<Result<(IntOrVec, IntOrVec)>> {
    let first = match line_iter.next()?.pipe(parse_line) {
        Ok(x) => x,
        Err(e) => return Some(Err(e)),
    };
    let second = match line_iter
        .next()
        .ok_or_else(|| eyre!("Unexpected EOF."))
        .and_then(parse_line)
    {
        Ok(x) => x,
        Err(e) => return Some(Err(e)),
    };
    match line_iter.next() {
        Some(Err(e)) => return Some(Err(e.into())),
        Some(Ok(s)) if !s.is_empty() => {
            return Some(Err(eyre!("Expected empty string, got {}", s)));
        }
        _ => (),
    };
    Some(Ok((first, second)))
}

fn parse_line(s: std::io::Result<String>) -> Result<IntOrVec> {
    let s = s?;
    let mut full_line = all_consuming(IntOrVec::parse_list);
    match full_line(&s).finish() {
        Ok((_, val)) => Ok(val),
        Err(e) => Err(eyre!("Parsing error: {}", e)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IntOrVec {
    Int(u32),
    Vec(Vec<IntOrVec>),
}

impl Ord for IntOrVec {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for IntOrVec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match self.correct_order(other) {
            ControlFlow::Continue(_) => {
                if self == other {
                    Some(Equal)
                } else {
                    None
                }
            }
            ControlFlow::Break(b) => Some(if b { Less } else { Greater }),
        }
    }
}

impl IntOrVec {
    fn correct_order(&self, other: &IntOrVec) -> ControlFlow<bool, ()> {
        use IntOrVec::*;

        fn compare_iters<'a, 'b, A, B>(a: A, b: B) -> ControlFlow<bool, ()>
        where
            A: IntoIterator<Item = &'a IntOrVec>,
            B: IntoIterator<Item = &'b IntOrVec>,
        {
            a.into_iter()
                .zip_longest(b.into_iter())
                .try_fold((), |_, pair| match pair {
                    EitherOrBoth::Both(a, b) => a.correct_order(b),
                    EitherOrBoth::Left(_) => ControlFlow::Break(false),
                    EitherOrBoth::Right(_) => ControlFlow::Break(true),
                })
        }

        match (self, other) {
            (Int(a), Int(b)) => match a.cmp(b) {
                std::cmp::Ordering::Less => ControlFlow::Break(true),
                std::cmp::Ordering::Equal => ControlFlow::Continue(()),
                std::cmp::Ordering::Greater => ControlFlow::Break(false),
            },
            (Vec(a), Vec(b)) => compare_iters(a, b),
            (elt @ Int(_), Vec(b)) => compare_iters(std::iter::once(elt), b),
            (Vec(a), elt @ Int(_)) => compare_iters(a, std::iter::once(elt)),
        }
    }

    fn parse_list(s: &str) -> IResult<&str, Self> {
        map(
            delimited(
                nom_char('['),
                separated_list0(
                    nom_char(','),
                    alt((map(nom_u32, IntOrVec::Int), Self::parse_list)),
                ),
                nom_char(']'),
            ),
            IntOrVec::Vec,
        )(s)
    }
}

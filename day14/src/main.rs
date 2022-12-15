use std::ops::ControlFlow;
use std::{hint::unreachable_unchecked, io::BufRead};

use color_eyre::eyre::{eyre, Result};
use common_utils::get_buffered_input;
use itertools::Itertools;
use nom::{
    bytes::complete::tag, character::complete::char as nom_char,
    character::complete::u16 as nom_u16, combinator::all_consuming, multi::separated_list1,
    sequence::separated_pair, Finish, IResult,
};
use tap::{pipe::Pipe, Tap};

fn main() -> Result<()> {
    color_eyre::install()?;

    let paths: Vec<_> = get_buffered_input()
        .lines()
        .map(|line_res| {
            line_res
                .map_err(color_eyre::eyre::Report::from)
                .and_then(|line| {
                    parse_line(&line)
                        .finish()
                        .map_err(|e| eyre!("Parsing error: {}", e))
                        .map(|(_, x)| x)
                })
        })
        .try_collect()?;

    let max_x = paths.iter().flatten().map(|&pair| pair.1).max().unwrap() + 2;
    let (min_y, max_y) = paths
        .iter()
        .flatten()
        .map(|&pair| pair.0)
        .minmax()
        .into_option()
        .unwrap()
        .pipe(|(min, max)| (min.min(500) - max_x, max.max(500) + max_x));
    let grid_rows = max_x as usize + 1;
    let grid_cols = (max_y - min_y) as usize + 1;
    let grid = vec![vec![false; grid_cols]; grid_rows].tap_mut(|mut_grid| {
        paths.iter().for_each(|path| {
            path.iter().copied().reduce(|start, end| {
                let (start_x, end_x) = (start.1.min(end.1), start.1.max(end.1));
                let (start_y, end_y) = (start.0.min(end.0), start.0.max(end.0));
                for x in start_x..=end_x {
                    for y in (start_y..=end_y).map(|coord| translate_coord(coord, min_y)) {
                        mut_grid[x as usize][y] = true;
                    }
                }
                end
            });
        });
        mut_grid[max_x as usize].fill(true);
    });
    let ControlFlow::Break(count) = (1..).try_fold((0, grid), |(prev_count, mut grid), count| {
        if let Some((x, y)) = drop_sand(&grid, min_y, max_x) {
            grid[x][y] = true;
            if x == 0 && y == translate_coord(500, min_y) {
                ControlFlow::Break(count)
            } else {
                ControlFlow::Continue((count, grid))
            }
        } else {
            ControlFlow::Break(prev_count)
        }
    }) else { unsafe { unreachable_unchecked() } };
    println!("{} grains dropped", count);
    Ok(())
}

fn parse_line(s: &str) -> IResult<&str, Vec<(u16, u16)>> {
    all_consuming(separated_list1(
        tag(" -> "),
        separated_pair(nom_u16, nom_char(','), nom_u16),
    ))(s)
}

fn translate_coord(coord: u16, min: u16) -> usize {
    (coord - min) as usize
}

fn drop_sand(grid: &[Vec<bool>], min_y: u16, max_x: u16) -> Option<(usize, usize)> {
    let start_y = translate_coord(500, min_y) as isize;
    let res = (0..max_x as usize).try_fold(start_y, |y, x| {
        let next_row = &grid[x + 1];
        if !next_row[y as usize] {
            ControlFlow::Continue(y)
        } else if !usize::try_from(y - 1)
            .map(|new_y| next_row[new_y])
            .unwrap_or(false)
        {
            ControlFlow::Continue(y - 1)
        } else if !next_row.get(y as usize + 1).unwrap_or(&false) {
            ControlFlow::Continue(y + 1)
        } else {
            ControlFlow::Break((x, y as usize))
        }
    });
    match res {
        ControlFlow::Break(pair) => Some(pair),
        _ => None,
    }
}

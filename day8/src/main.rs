use std::{io::BufRead, ops::ControlFlow};

use anyhow::Result;
use arrayvec::ArrayVec;
use common_utils::get_buffered_input;
use itertools::Itertools;

fn main() -> Result<()> {
    let grid: Vec<Vec<u8>> = get_buffered_input()
        .lines()
        .map(|line_res| {
            line_res.map(|line| {
                line.as_bytes()
                    .iter()
                    .copied()
                    .map(|byte| match byte {
                        b'0'..=b'9' => byte - b'0',
                        _ => unreachable!(),
                    })
                    .collect_vec()
            })
        })
        .try_collect()?;

    let num_rows = grid.len();
    let num_cols = grid[0].len();
    let mut visible = vec![vec![false; num_cols]; num_rows];
    for row in visible.iter_mut() {
        row[0] = true;
        row[num_cols - 1] = true;
    }
    visible[0].fill(true);
    visible[num_rows - 1].fill(true);

    for (line, row_visibility) in grid.iter().zip(visible.iter_mut()) {
        set_visibility(line.iter().copied(), row_visibility.iter_mut());
        set_visibility(line.iter().rev().copied(), row_visibility.iter_mut().rev());
    }
    for i in 0..num_cols {
        let col = grid.iter().map(|row| row[i]);
        let visibility_col = visible.iter_mut().map(|row| &mut row[i]);
        set_visibility(col.clone(), visibility_col);
        let visibility_col_rev = visible.iter_mut().map(|row| &mut row[i]).rev();
        set_visibility(col.rev(), visibility_col_rev);
    }

    let num_visible = visible.iter().flatten().filter(|&&x| x).count();
    println!("There are {} visible from edges", num_visible);
    println!(
        "The max scenic score is {}",
        compute_max_scenic_score(&grid)
    );
    Ok(())
}

fn set_visibility<'a>(
    line: impl Iterator<Item = u8>,
    visibilities: impl Iterator<Item = &'a mut bool>,
) {
    line.zip(visibilities)
        .try_fold(0, |max_height, (height, visibility)| {
            if height > max_height {
                *visibility = true;
                if height == 9 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(height)
                }
            } else {
                ControlFlow::Continue(max_height)
            }
        });
}

fn compute_max_scenic_score(grid: &[Vec<u8>]) -> u32 {
    let mut products: Vec<Vec<u32>> = grid
        .iter()
        .map(|row| compute_scenic_score_line(row.iter().copied()).collect_vec())
        .collect_vec();
    products
        .iter_mut()
        .zip(grid)
        .for_each(|(product_row, grid_row)| {
            let rev_scenics = compute_scenic_score_line(grid_row.iter().rev().copied());
            product_row
                .iter_mut()
                .rev()
                .zip(rev_scenics)
                .for_each(|(elem, new_score)| *elem *= new_score);
        });

    let num_cols = grid[0].len();
    for i in 0..num_cols {
        let grid_col = grid.iter().map(|row| row[i]);
        let product_col = products.iter_mut().map(|row| &mut row[i]);
        product_col
            .zip(compute_scenic_score_line(grid_col))
            .for_each(|(elem, new_score)| *elem *= new_score);

        let grid_col_rev = grid.iter().rev().map(|row| row[i]);
        let product_col_rev = products.iter_mut().rev().map(|row| &mut row[i]);
        product_col_rev
            .zip(compute_scenic_score_line(grid_col_rev))
            .for_each(|(elem, new_score)| *elem *= new_score);
    }

    products.into_iter().flatten().max().unwrap()
}

fn compute_scenic_score_line(line: impl Iterator<Item = u8>) -> impl Iterator<Item = u32> {
    line.enumerate().scan(
        ArrayVec::<(usize, u8), 10>::new(),
        |previous_maxes, (i, height)| {
            previous_maxes.retain(|(_, previous_max)| *previous_max >= height);
            let previous_max = *previous_maxes.last().unwrap_or(&(0, 0));
            let score = (i - previous_max.0) as u32;
            if previous_max.1 == height {
                previous_maxes.pop();
            }
            previous_maxes.push((i, height));
            Some(score)
        },
    )
}

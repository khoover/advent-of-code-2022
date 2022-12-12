use std::{convert::TryInto, io::BufRead};

use common_utils::get_buffered_input;

use color_eyre::eyre::Result;
use itertools::Itertools;
fn main() -> Result<()> {
    color_eyre::install()?;

    let mut start_position: Option<(usize, usize)> = None;
    let data = get_buffered_input()
        .lines()
        .map(|line_res| line_res.unwrap())
        .enumerate()
        .map(|(row, line)| {
            line.as_bytes()
                .iter()
                .copied()
                .enumerate()
                .map(|(col, val)| match val {
                    b'S' => 0,
                    b'E' => {
                        start_position = Some((row, col));
                        b'z' - b'a'
                    }
                    b'a'..=b'z' => val - b'a',
                    _ => unreachable!(),
                })
                .collect_vec()
        })
        .collect_vec();
    let start_position = start_position.unwrap();
    let data_ref = &data;
    let path = pathfinding::directed::dijkstra::dijkstra(
        &start_position,
        |&(x, y)| {
            let curr_height = data[x][y];
            [(0isize, 1isize), (0, -1), (1, 0), (-1, 0)]
                .into_iter()
                .filter_map(move |(dx, dy)| {
                    let new_pos = (
                        dx.checked_add(x as isize)?.try_into().ok()?,
                        dy.checked_add(y as isize)?.try_into().ok()?,
                    );
                    let cell = data_ref
                        .get(new_pos.0)
                        .and_then(|row: &Vec<u8>| row.get(new_pos.1).copied())?;
                    if curr_height <= cell + 1 {
                        Some((new_pos, 1))
                    } else {
                        None
                    }
                })
        },
        |&pos| data[pos.0][pos.1] == 0,
    )
    .ok_or_else(|| color_eyre::eyre::eyre!("Could not find any path to the target"))?;
    println!("Path cost is {}", path.1);
    Ok(())
}

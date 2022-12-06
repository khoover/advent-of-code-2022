use std::collections::VecDeque;
use std::io::Read;

use anyhow::{Result, Context};
use common_utils::get_buffered_input;
use itertools::{FoldWhile, Itertools};

fn main() -> Result<()> {
    let sample1 = "mjqjpqmgbljsphdztnvjfqwrcgsmlb";
    let sample2 = "bvwbjplbgvbhsrlpgdmjqwftvncz";
    let sample3 = "nppdvjthqldpwncqszvftbrmjlhg";
    let sample4 = "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg";
    let sample5 = "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw";

    assert_eq!(get_start_of_packet(sample1.as_bytes().iter().map(|x| Ok(*x)))?, 7);
    assert_eq!(get_start_of_packet(sample2.as_bytes().iter().map(|x| Ok(*x)))?, 5);
    assert_eq!(get_start_of_packet(sample3.as_bytes().iter().map(|x| Ok(*x)))?, 6);
    assert_eq!(get_start_of_packet(sample4.as_bytes().iter().map(|x| Ok(*x)))?, 10);
    assert_eq!(get_start_of_packet(sample5.as_bytes().iter().map(|x| Ok(*x)))?, 11);

    assert_eq!(get_start_of_message(sample1.as_bytes().iter().map(|x| Ok(*x)))?, 19);
    assert_eq!(get_start_of_message(sample2.as_bytes().iter().map(|x| Ok(*x)))?, 23);
    assert_eq!(get_start_of_message(sample3.as_bytes().iter().map(|x| Ok(*x)))?, 23);
    assert_eq!(get_start_of_message(sample4.as_bytes().iter().map(|x| Ok(*x)))?, 29);
    assert_eq!(get_start_of_message(sample5.as_bytes().iter().map(|x| Ok(*x)))?, 26);

    let start_of_packet = get_start_of_packet(get_buffered_input().bytes())?;
    println!("First start of packet is {}", start_of_packet);

    let start_of_message = get_start_of_message(get_buffered_input().bytes())?;
    println!("First start of message is {}", start_of_message);

    Ok(())
}

fn get_start_of_packet(iter: impl IntoIterator<Item = std::io::Result<u8>>) -> Result<usize> {
    find_first_run_of_n_distinct(iter, 4)
}

fn get_start_of_message(iter: impl IntoIterator<Item = std::io::Result<u8>>) -> Result<usize> {
    find_first_run_of_n_distinct(iter, 14)
}

fn find_first_run_of_n_distinct(iter: impl IntoIterator<Item = std::io::Result<u8>>, n: usize) -> Result<usize> {
    let mut enumerated = iter.into_iter()
        .enumerate()
        .map(|(i, res)| match res {
            Ok(x) => Ok((i+1, x)),
            Err(e) => Err(e)
        });
    let mut last_n_seen: VecDeque<u8> = enumerated.by_ref()
        .take(n)
        .map_ok(|(_, x)| x)
        .try_collect()?;
    if last_n_seen.iter().all_unique() { return Ok(n); }
    enumerated.fold_while(Ok(None), |_, res| {
        match res {
            Err(e) => FoldWhile::Done(Err(e)),
            Ok((i, val)) => {
                last_n_seen.pop_front();
                last_n_seen.push_back(val);
                if last_n_seen.iter().all_unique() {
                    FoldWhile::Done(Ok(Some(i)))
                } else {
                    FoldWhile::Continue(Ok(None))
                }
            }
        }
    }).into_inner()?.with_context(|| format!("No run of {} encountered", n))
}

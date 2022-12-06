use std::collections::VecDeque;
use std::io::Read;

use anyhow::{Result, bail};
use common_utils::get_buffered_input;
use itertools::Itertools;

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
    let mut last_four_seen: VecDeque<u8> = enumerated.by_ref()
        .take(n)
        .map_ok(|(_, x)| x)
        .try_collect()?;
    if last_four_seen.iter().all_unique() { return Ok(n); }
    for res in enumerated {
        let (i, val) = res?;
        last_four_seen.pop_front();
        last_four_seen.push_back(val);
        if last_four_seen.iter().all_unique() {
            return Ok(i);
        }
    }
    bail!("No start of packet encountered.");
}

use std::io::BufRead;
use common_utils::get_buffered_input;
use itertools::Itertools;

fn main() {
    let priority_sum = get_buffered_input().lines()
        .map(|line_result| line_result.unwrap())
        .map(|line| find_shared_priority(&line))
        .sum::<u64>();
    println!("Sum of priorities is {}", priority_sum);

    let group_sum = get_buffered_input().lines()
        .map(|x| x.unwrap())
        .chunks(3)
        .into_iter()
        .map(find_group_priority)
        .sum::<u64>();
    println!("Group sum using itertools is {}", group_sum);
}

fn find_group_priority(groups: impl Iterator<Item = String>) -> u64 {
    groups.map(|line| {
            let mut appears = [false; 52];
            for &byte in line.as_bytes() {
                let priority = utf8_byte_to_priority(byte) - 1;
                appears[priority as usize] = true;
            }
            appears.map(|x| x as u8)
        })
        .reduce(|mut acc, appears| {
            acc.iter_mut().zip(appears)
                .for_each(|(acc_val, appears_val)| {
                    *acc_val += appears_val;
                });
            acc
        })
        .unwrap()
        .into_iter()
        .find_position(|count| *count == 3).unwrap().0 as u64 + 1
}

fn find_shared_priority(contents: &str) -> u64 {
    let mut has_seen = [false; 52];
    let bytes = contents.as_bytes();
    let (first_compartment, second_compartment) = bytes.split_at(bytes.len() / 2);
    for &byte in first_compartment {
        let priority = utf8_byte_to_priority(byte);
        unsafe {
            *has_seen.get_unchecked_mut((priority as usize) - 1) = true;
        }
    }
    for &byte in second_compartment {
        let priority = utf8_byte_to_priority(byte);
        if unsafe { *has_seen.get_unchecked((priority as usize) - 1) } {
            return priority;
        }
    }
    unreachable!()
}

fn utf8_byte_to_priority(byte: u8) -> u64 {
    1 + match byte {
        b'a'..=b'z' => byte - b'a',
        b'A'..=b'Z' => byte - b'A' + 26,
        _ => unreachable!()
    } as u64
}

use std::io::BufRead;

fn get_buffered_input() -> std::io::BufReader<std::fs::File> {
    let mut args = std::env::args();
    args.next().unwrap();
    let input_path = args.next().unwrap();
    let input_file = std::fs::OpenOptions::new()
        .read(true)
        .open(input_path)
        .unwrap();
    std::io::BufReader::new(input_file)
}

fn main() {
    let priority_sum = get_buffered_input().lines()
        .map(|line_result| line_result.unwrap())
        .map(|line| find_shared_priority(&line))
        .sum::<u64>();
    println!("Sum of priorities is {}", priority_sum);

    let buf_in = get_buffered_input();
    let mut lines = buf_in.lines();
    let mut sum = 0u64;
    loop {
        let line1 = lines.next();
        if line1.is_none() { break; }
        let group: [String; 3] = [
            line1.unwrap().unwrap(),
            lines.next().unwrap().unwrap(),
            lines.next().unwrap().unwrap()
        ];
        sum += group.into_iter().map(|line| {
                let mut appears = [false; 52];
                line.as_bytes().iter().for_each(|&byte| {
                    let priority = utf8_byte_to_priority(byte) - 1;
                    appears[priority as usize] = true;
                });
                appears
            })
            .fold([0u8; 52], |mut acc, appears| {
                acc.iter_mut().zip(appears)
                    .for_each(|(acc_val, appears_val)| {
                        if appears_val {
                            *acc_val += 1;
                        }
                    });
                acc
            }).into_iter().enumerate()
            .find(|(_, count)| *count == 3).unwrap().0 as u64 + 1;

    }
    println!("Shared group priorities sum is {}", sum);
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

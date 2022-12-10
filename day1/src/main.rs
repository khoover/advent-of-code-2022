use std::collections::BinaryHeap;
use std::io::BufRead;
use std::str::FromStr;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let input_path = args.next().unwrap();
    let input_file = std::fs::OpenOptions::new()
        .read(true)
        .open(input_path)
        .unwrap();
    let buffered_input = std::io::BufReader::new(input_file);
    let line_iter = buffered_input.lines().map(|line_result| {
        line_result.and_then(|line| {
            if line.is_empty() {
                Ok(None)
            } else {
                match u64::from_str(&line) {
                    Ok(val) => Ok(Some(val)),
                    Err(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Non-integer input received.",
                    )),
                }
            }
        })
    });
    //    println!("Max total calories is {}", max_total_calories(line_iter));
    println!("Total of top 3 is {}", top_three_total(line_iter));
}

#[allow(dead_code)]
fn max_total_calories(iter: impl IntoIterator<Item = std::io::Result<Option<u64>>>) -> u64 {
    let mut curr_max: Option<u64> = None;
    iter.into_iter()
        .try_fold(0u64, |acc, maybe_calorie_count_res| {
            maybe_calorie_count_res.map(|maybe_calorie_count| {
                if let Some(count) = maybe_calorie_count {
                    acc + count
                } else {
                    curr_max = Some(curr_max.map_or(acc, |last_max| last_max.max(acc)));
                    0
                }
            })
        })
        .expect("Unexpected error while reading input lines.");
    curr_max.unwrap()
}

fn top_three_total(iter: impl IntoIterator<Item = std::io::Result<Option<u64>>>) -> u64 {
    let mut heap: BinaryHeap<u64> = BinaryHeap::new();
    iter.into_iter().fold(0u64, |acc, maybe_calorie_count_res| {
        if let Some(count) = maybe_calorie_count_res.unwrap() {
            acc + count
        } else {
            heap.push(acc);
            0
        }
    });
    let top_three: [Option<u64>; 3] = [heap.pop(), heap.pop(), heap.pop()];
    top_three.into_iter().flatten().sum::<u64>()
}

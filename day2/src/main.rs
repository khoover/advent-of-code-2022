use std::io::BufRead;

#[derive(Clone, Copy)]
enum RPS {
    Rock,
    Paper,
    Scissors,
}

#[derive(Clone, Copy)]
enum Outcome {
    Win,
    Loss,
    Draw,
}

impl Outcome {
    fn from_utf8_byte(byte: u8) -> Self {
        match byte {
            b'X' => Outcome::Loss,
            b'Y' => Outcome::Draw,
            b'Z' => Outcome::Win,
            _ => unreachable!(),
        }
    }

    fn score(self, other_throw: RPS) -> u64 {
        match self {
            Outcome::Loss => other_throw.losing_throw().score(),
            Outcome::Draw => 3 + other_throw.score(),
            Outcome::Win => 6 + other_throw.beating_throw().score(),
        }
    }
}

impl RPS {
    fn from_utf8_byte(byte: u8) -> Self {
        match byte {
            b'A' | b'X' => RPS::Rock,
            b'B' | b'Y' => RPS::Paper,
            b'C' | b'Z' => RPS::Scissors,
            _ => unreachable!(),
        }
    }

    fn score(self) -> u64 {
        match self {
            RPS::Rock => 1,
            RPS::Paper => 2,
            RPS::Scissors => 3,
        }
    }

    fn round_value(mine: RPS, other: RPS) -> u64 {
        use RPS::*;

        mine.score()
            + match (mine, other) {
                (Rock, Paper) | (Paper, Scissors) | (Scissors, Rock) => 0,
                (Paper, Rock) | (Scissors, Paper) | (Rock, Scissors) => 6,
                _ => 3,
            }
    }

    fn beating_throw(self) -> Self {
        match self {
            RPS::Rock => RPS::Paper,
            RPS::Scissors => RPS::Rock,
            RPS::Paper => RPS::Scissors,
        }
    }

    fn losing_throw(self) -> Self {
        match self {
            RPS::Rock => RPS::Scissors,
            RPS::Scissors => RPS::Paper,
            RPS::Paper => RPS::Rock,
        }
    }
}

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
    let total = get_buffered_input()
        .lines()
        .map(|line_res| {
            let line_str = line_res.unwrap();
            let line_bytes = line_str.as_bytes();
            (
                RPS::from_utf8_byte(line_bytes[0]),
                RPS::from_utf8_byte(line_bytes[2]),
            )
        })
        .map(|(other, mine)| RPS::round_value(mine, other))
        .sum::<u64>();
    println!("Total score is {}", total);

    let part2_total = get_buffered_input()
        .lines()
        .map(|line_res| {
            let line_str = line_res.unwrap();
            let line_bytes = line_str.as_bytes();
            (
                RPS::from_utf8_byte(line_bytes[0]),
                Outcome::from_utf8_byte(line_bytes[2]),
            )
        })
        .map(|(other_throw, outcome)| outcome.score(other_throw))
        .sum::<u64>();
    println!("Total part 2 score is {}", part2_total);
}

use std::collections::BinaryHeap;

use arrayvec::ArrayVec;
use color_eyre::eyre::Result;
use tap::tap::Tap;
fn main() -> Result<()> {
    color_eyre::install()?;

    let mut sample_input: [Monkey; 4] = [
        Monkey {
            items: vec![79, 98],
            op: (Operation::Mult, Argument::Constant(19)),
            test: TestAndTargets {
                argument: 23,
                is_divisible: 2,
                is_not_divisible: 3,
            },
        },
        Monkey {
            items: vec![54, 65, 75, 74],
            op: (Operation::Add, Argument::Constant(6)),
            test: TestAndTargets {
                argument: 19,
                is_divisible: 2,
                is_not_divisible: 0,
            },
        },
        Monkey {
            items: vec![79, 60, 97],
            op: (Operation::Mult, Argument::Old),
            test: TestAndTargets {
                argument: 13,
                is_divisible: 1,
                is_not_divisible: 3,
            },
        },
        Monkey {
            items: vec![74],
            op: (Operation::Add, Argument::Constant(3)),
            test: TestAndTargets {
                argument: 17,
                is_divisible: 0,
                is_not_divisible: 1,
            },
        },
    ];

    let big_modulo: u64 = 17 * 13 * 19 * 23;
    println!("{:x}", big_modulo);

    let sample_times_inspecting = get_times_inspecting::<4, 10>(&mut sample_input, big_modulo);
    println!("sample: {:?}", sample_times_inspecting);

    let mut input = [
        Monkey {
            items: vec![74, 73, 57, 77, 74],
            op: (Operation::Mult, Argument::Constant(11)),
            test: TestAndTargets {
                argument: 19,
                is_divisible: 6,
                is_not_divisible: 7,
            },
        },
        Monkey {
            items: vec![99, 77, 79],
            op: (Operation::Add, Argument::Constant(8)),
            test: TestAndTargets {
                argument: 2,
                is_divisible: 6,
                is_not_divisible: 0,
            },
        },
        Monkey {
            items: vec![64, 67, 50, 96, 89, 82, 82],
            op: (Operation::Add, Argument::Constant(1)),
            test: TestAndTargets {
                argument: 3,
                is_divisible: 5,
                is_not_divisible: 3,
            },
        },
        Monkey {
            items: vec![88],
            op: (Operation::Mult, Argument::Constant(7)),
            test: TestAndTargets {
                argument: 17,
                is_divisible: 5,
                is_not_divisible: 4,
            },
        },
        Monkey {
            items: vec![80, 66, 98, 83, 70, 63, 57, 66],
            op: (Operation::Add, Argument::Constant(4)),
            test: TestAndTargets {
                argument: 13,
                is_divisible: 0,
                is_not_divisible: 1,
            },
        },
        Monkey {
            items: vec![81, 93, 90, 61, 62, 64],
            op: (Operation::Add, Argument::Constant(7)),
            test: TestAndTargets {
                argument: 7,
                is_divisible: 1,
                is_not_divisible: 4,
            },
        },
        Monkey {
            items: vec![69, 97, 88, 93],
            op: (Operation::Mult, Argument::Old),
            test: TestAndTargets {
                argument: 5,
                is_divisible: 7,
                is_not_divisible: 2,
            },
        },
        Monkey {
            items: vec![59, 80],
            op: (Operation::Add, Argument::Constant(6)),
            test: TestAndTargets {
                argument: 11,
                is_divisible: 2,
                is_not_divisible: 3,
            },
        },
    ];
    let big_modulo = input
        .iter()
        .map(|monkey| monkey.test.argument)
        .reduce(|prod, val| prod.checked_mul(val).unwrap())
        .unwrap();
    println!("{:x}", big_modulo);

    let input_times_inspecting = get_times_inspecting::<8, 36>(&mut input, big_modulo);
    println!("input is {:?}", input_times_inspecting);

    println!(
        "input monkey business is {}",
        input_times_inspecting
            .into_iter()
            .collect::<BinaryHeap<_>>()
            .into_iter()
            .take(2)
            .map(|x| x as u64)
            .product::<u64>()
    );

    Ok(())
}

fn get_times_inspecting<const N: usize, const LIMIT: usize>(
    monkeys: &mut [Monkey; N],
    big_modulo: u64,
) -> [usize; N] {
    let mut times_inspecting: [usize; N] = [0; N];

    for _ in 0..10000 {
        for i in 0..N {
            monkeys[i]
                .run_round(big_modulo)
                .collect::<ArrayVec<_, LIMIT>>()
                .tap(|vec| times_inspecting[i] += vec.len())
                .into_iter()
                .for_each(|(j, val)| monkeys[j].add_item(val));
        }
    }

    times_inspecting
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<u64>,
    op: (Operation, Argument),
    test: TestAndTargets,
}

impl Monkey {
    fn run_round(&'_ mut self, big_modulo: u64) -> impl Iterator<Item = (usize, u64)> + '_ {
        self.items
            .drain(..)
            .map(|old| self.op.0.run(old, self.op.1))
            .map(move |new| new % big_modulo)
            //.map(|new| new.div_euclid(3)) Part 1
            .map(|new| (self.test.get_next_monkey(new), new))
    }

    fn add_item(&mut self, item: u64) {
        self.items.push(item);
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add,
    Mult,
}

impl Operation {
    fn run(self, old: u64, arg: Argument) -> u64 {
        let arg = arg.into_value(old);
        match self {
            Operation::Add => old + arg,
            Operation::Mult => old * arg,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Argument {
    Constant(u64),
    Old,
}

impl Argument {
    fn into_value(self, old: u64) -> u64 {
        match self {
            Argument::Constant(x) => x,
            Argument::Old => old,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TestAndTargets {
    argument: u64,
    is_divisible: usize,
    is_not_divisible: usize,
}

impl TestAndTargets {
    fn get_next_monkey(&self, new_val: u64) -> usize {
        if new_val % self.argument == 0 {
            self.is_divisible
        } else {
            self.is_not_divisible
        }
    }
}

use std::convert::TryInto;
use std::io::BufRead;

use arrayvec::ArrayVec;
use color_eyre::eyre::{Result, WrapErr};
use common_utils::get_buffered_input;
use itertools::Itertools;
use tap::pipe::Pipe;
fn main() -> Result<()> {
    color_eyre::install()?;

    let mut reg_values = get_buffered_input()
        .lines()
        .map(|line| Ok(parser::instruction(&line?)?.1))
        .pipe(run_program);

    /* Part 1
    reg_values.by_ref().take(19).try_fold(0, |_, val| val)?;
    let signal_strength_sum = reg_values
        .step_by(40)
        .enumerate()
        .map(|(i, res)| res.map(|val| val * (20 + 40 * i as i32)))
        .try_fold(0, |sum, res| res.map(|reg_val| sum + reg_val))?;

    println!("signal strength sum is {}", signal_strength_sum);
    */

    /* Part 2 */
    reg_values.chunks(40).into_iter().try_for_each(|chunk| {
        let row: ArrayVec<u8, 40> = chunk
            .enumerate()
            .map(|(i, res)| {
                res.map(|val| {
                    let val: usize = match val.try_into() {
                        Ok(x) => x,
                        Err(_) => return b'.',
                    };
                    if (i.saturating_sub(1)..=usize::min(40, i + 1)).contains(&val) {
                        b'#'
                    } else {
                        b'.'
                    }
                })
            })
            .try_collect()?;
        let s = std::str::from_utf8(&row)?;
        println!("{}", s);
        Ok::<_, color_eyre::eyre::Report>(())
    })?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Add(i16),
    Noop,
}

fn run_program(
    iter: impl IntoIterator<Item = Result<Instruction>>,
) -> impl Iterator<Item = Result<i32>> {
    let mut value = 1i32;
    let mut to_add = None;
    iter.into_iter().batching(move |instructions| {
        if let Some(x) = to_add {
            to_add = None;
            let old_value = value;
            value += x as i32;
            Some(Ok(old_value))
        } else {
            instructions.next().map(|res| {
                res.map(|instruction| {
                    if let Instruction::Add(x) = instruction {
                        to_add = Some(x);
                    }
                    value
                })
            })
        }
    })
}

mod parser {
    use super::*;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::i16 as nom_i16,
        combinator::{map, value},
        error::Error,
        sequence::preceded,
        Finish, IResult,
    };

    pub(super) fn instruction(s: &str) -> Result<(&str, Instruction)> {
        alt((value(Instruction::Noop, tag("noop")), add))(s)
            .finish()
            .map_err(|e| Error {
                input: e.input.to_owned(),
                code: e.code,
            })
            .wrap_err_with(|| format!("Unable to parse '{}' as instruction", s))
            .and_then(|val| {
                color_eyre::eyre::ensure!(
                    val.0.is_empty(),
                    "Should have parsed entire line, had '{}' left over",
                    val.0
                );
                Ok(val)
            })
    }

    fn add(s: &str) -> IResult<&str, Instruction> {
        map(preceded(tag("addx "), nom_i16), Instruction::Add)(s)
    }
}

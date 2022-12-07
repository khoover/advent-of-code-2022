use std::{
    collections::HashMap,
    io::BufRead,
    cell::Cell
};

use anyhow::{Result, anyhow, Context};
use common_utils::get_buffered_input;
use itertools::Itertools;

fn main() -> Result<()> {
    let mut builder = TreeBuilder::new();
    builder.parse(get_buffered_input().lines()
        .map(|line| parser::line(&line?)))?;
    let root = builder.build();
    println!("Sum is {}", root.get_sum_size_under_threshold(100000));

    const TOTAL_SPACE: u64 = 70000000;
    const NEEDED_SPACE: u64 = 30000000;
    let currently_free = TOTAL_SPACE - root.get_recursive_size();
    let to_free = NEEDED_SPACE - currently_free;
    let smallest_dir = root.get_min_size_over_threshold(to_free);
    println!("Smallest dir to delete is {:?}", smallest_dir);
    Ok(())
}

#[derive(Debug)]
struct TreeBuilder {
    pwd: Vec<usize>,
    dirs: Vec<DirBuilder>
}

impl TreeBuilder {
    fn new() -> Self {
        Self {
            pwd: vec![0],
            dirs: vec![DirBuilder::new("/".to_owned())],
        }
    }

    fn parse(&mut self, mut lines: impl Iterator<Item = Result<Line>>) -> Result<()> {
        lines.try_for_each(|line_res| {
            let line = line_res?;
            match line {
                Line::Command(c) => {
                    match c {
                        Command::Dir(spec) => {
                            match spec {
                                DirSpec::Root => self.cd_root(),
                                DirSpec::Up => self.cd_up()?,
                                DirSpec::Down(name) => self.cd_down(name)?
                            }
                        },
                        Command::Ls => {}
                    }
                },
                Line::Entry(e) => {
                    match e {
                        DirOrFileEntry::File(size) => {
                            let pwd = &mut self.dirs[*self.pwd.last().unwrap()];
                            pwd.add_file(size);
                        },
                        DirOrFileEntry::Dir(name) => {
                            let child_idx = self.dirs.len();
                            let pwd = &mut self.dirs[*self.pwd.last().unwrap()];
                            pwd.add_subdir(name.clone(), child_idx);
                            self.dirs.push(DirBuilder::new(name));
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn cd_root(&mut self) {
        while self.pwd.len() > 1 {
            self.pwd.pop();
        }
    }

    fn cd_up(&mut self) -> Result<()> {
        match self.pwd.pop() {
            Some(_) => Ok(()),
            None => Err(anyhow!("PWD unexpectedly empty"))
        }
    }

    fn cd_down(&mut self, dir_name: String) -> Result<()> {
        let pwd = &mut self.dirs[*self.pwd.last().unwrap()];
        let next_idx = *pwd.children.get(&dir_name)
            .with_context(|| format!("Couldn't find a dir named {} under {}", dir_name, pwd.name))?;
        self.pwd.push(next_idx);
        Ok(())
    }

    fn build(self) -> Dir {
        let root = &self.dirs[0];
        root.build(&self.dirs)
    }
}

#[derive(Debug)]
struct DirBuilder {
    pub name: String,
    pub self_size: u64,
    pub children: HashMap<String, usize>
}

impl DirBuilder {
    fn new(name: String) -> Self {
        Self {
            name,
            self_size: 0,
            children: HashMap::new()
        }
    }

    fn add_file(&mut self, size: u64) {
        self.self_size += size;
    }

    fn add_subdir(&mut self, subdir_name: String, subdir_idx: usize) {
        self.children.insert(subdir_name, subdir_idx);
    }

    fn build(&self, other_builders: &[DirBuilder]) -> Dir {
        let children = self.children.values()
            .copied()
            .map(|idx| &other_builders[idx])
            .map(|builder| builder.build(other_builders))
            .collect_vec();
        Dir::new(self.name.clone(), self.self_size, children)
    }
}

#[derive(Debug, Clone)]
struct Dir {
    #[allow(dead_code)]
    pub name: String,
    pub self_size: u64,
    recursive_size: Cell<Option<u64>>,
    pub children: Vec<Dir>
}

impl Dir {
    fn new(name: String, self_size: u64, children: Vec<Dir>) -> Self {
        Self {
            name,
            self_size,
            recursive_size: Cell::new(None),
            children
        }
    }

    fn get_recursive_size(&self) -> u64 {
        if let Some(size) = self.recursive_size.get() {
            size
        } else {
            let size = self.self_size + self.children.iter().map(Dir::get_recursive_size).sum::<u64>();
            self.recursive_size.set(Some(size));
            size
        }
    }

    fn get_sum_size_under_threshold(&self, threshold: u64) -> u64 {
        let child_sizes = self.children.iter()
            .map(|child| child.get_sum_size_under_threshold(threshold))
            .sum::<u64>();
        child_sizes + if self.get_recursive_size() <= threshold { self.get_recursive_size() } else { 0 }
    }

    fn get_min_size_over_threshold(&self, threshold: u64) -> Option<u64> {
        self.children.iter()
            .filter_map(|child| child.get_min_size_over_threshold(threshold))
            .min()
            .or_else(|| {
                let own_size = self.get_recursive_size();
                if own_size >= threshold { Some(own_size) } else { None }
            })
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    Command(Command),
    Entry(DirOrFileEntry)
}

impl From<Command> for Line {
    fn from(val: Command) -> Self {
        Line::Command(val)
    }
}

impl From<DirOrFileEntry> for Line {
    fn from(val: DirOrFileEntry) -> Self {
        Line::Entry(val)
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Ls,
    Dir(DirSpec)
}

impl From<DirSpec> for Command {
    fn from(val: DirSpec) -> Self {
        Command::Dir(val)
    }
}

#[derive(Debug, Clone)]
pub enum DirSpec {
    Up,
    Root,
    Down(String)
}

#[derive(Debug, Clone)]
pub enum DirOrFileEntry {
    Dir(String),
    File(u64)
}

mod parser {
    use super::*;

    use nom::{
        error::Error,
        branch::alt,
        Finish,
        combinator::map,
        IResult,
        sequence::{preceded, separated_pair},
        bytes::complete::{tag, take_while},
        character::complete::{u64 as nom_u64}
    };
    use anyhow::{anyhow, Result};

    pub fn line(s: &str) -> Result<Line> {
        let (left, out) = alt((map(command, Into::into), map(entry, Into::into)))(s).finish()
            .map_err(|e| Error { input: e.input.to_owned(), code: e.code })?;
        if !left.is_empty() {
            Err(anyhow!("Did not consume all of line"))
        } else {
            Ok(out)
        }
    }

    fn command(s: &str) -> IResult<&str, Command> {
        preceded(
            tag("$ "),
            alt((
                map(tag("ls"), |_| Command::Ls),
                map(
                    preceded(tag("cd "), dir_spec),
                    Into::into
                )
            ))
        )(s)
    }

    fn dir_spec(s: &str) -> IResult<&str, DirSpec> {
        alt((
            map(tag(".."), |_| DirSpec::Up),
            map(tag("/"), |_| DirSpec::Root),
            map(take_while(|_| true), |name: &str| DirSpec::Down(name.to_owned()))
        ))(s)
    }

    fn entry(s: &str) -> IResult<&str, DirOrFileEntry> {
        alt((
            map(
                preceded(tag("dir "), take_while(|_| true)),
                |name: &str| DirOrFileEntry::Dir(name.to_owned())
            ),
            map(
                separated_pair(nom_u64::<&str, _>, tag(" "), take_while(|_| true)),
                |(size, _)| DirOrFileEntry::File(size)
            )
        ))(s)
    }
}
use std::rc::Rc;
use std::{collections::HashMap, cell::RefCell};
use std::io::BufRead;

use anyhow::{Result, Context, anyhow, bail};
use common_utils::get_buffered_input;
use itertools::{Either};

fn main() -> Result<()> {
    let root: Rc<RefCell<DirBuilder<u64>>> = Rc::new(RefCell::new(DirBuilder { name: "/".to_owned(), children: Default::default() }));
    get_buffered_input().lines()
        .map(|line| parser::line(&line.unwrap()).unwrap())
        .try_fold((vec![root.clone()], false), |(mut dir_stack, in_ls), line| {
            match line {
                Line::Command(c) => {
                    match c {
                        Command::Dir(name) => {
                            if name == ".." {
                                dir_stack.pop().with_context(|| anyhow!("Directory stack is empty."))?;
                            } else if name == "/" {
                                while dir_stack.len() > 1 {
                                    dir_stack.pop();
                                }
                            } else {
                                let pwd = dir_stack.last().with_context(|| anyhow!("Directory stack is empty"))?.clone();
                                let mut borrow = pwd.borrow_mut();
                                dir_stack.push(borrow.children
                                    .entry(name)
                                    .or_insert_with_key(|key| Either::Left(Rc::new(RefCell::new(DirBuilder { name: key.clone(), children: Default::default() })))).clone().expect_left("Should be a directory"));
                            }
                            Ok::<_, anyhow::Error>((dir_stack, false))
                        },
                        Command::Ls => {
                            Ok((dir_stack, true))
                        }
                    }
                },
                Line::Entry(e) => {
                    if !in_ls {
                        bail!("Unexpected dir entry.");
                    }
                    match e {
                        DirOrFile::Dir(d) => {
                            let pwd = dir_stack.last().with_context(|| anyhow!("Directory stack is empty"))?.clone();
                            let mut borrow = pwd.borrow_mut();
                            borrow.children
                                .entry(d.name)
                                .or_insert_with_key(|key| Either::Left(Rc::new(RefCell::new(DirBuilder { name: key.clone(), children: Default::default() }))));
                        },
                        DirOrFile::File(f) => {
                            let pwd = dir_stack.last().with_context(|| anyhow!("Directory stack is empty"))?.clone();
                            pwd.borrow_mut().children.insert(f.name.clone(), Either::Right(f));
                        }
                    }
                    Ok((dir_stack, true))
                }
            }
        })?;
    let dir_sizes: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    let root = Rc::try_unwrap(root).unwrap().into_inner().build();
    let total_used = DirOrFile::from(root).reduce(&mut |x| x, &|name, children| {
        let size = children.into_iter()
            .map(|file| file.data)
            .sum::<u64>();
        dir_sizes.borrow_mut().push(size);
        File { name, data: size }
    }).data;
    let dir_sizes = dir_sizes.into_inner();
    let total = dir_sizes.iter()
        .filter(|&&x| x <= 100000)
        .sum::<u64>();
    println!("Sum is {}", total);

    const TOTAL_SPACE: u64 = 70000000;
    const NEEDED_SPACE: u64 = 30000000;
    let currently_free = TOTAL_SPACE - total_used;
    let to_free = NEEDED_SPACE - currently_free;
    let smallest_dir = dir_sizes.into_iter().filter(|&x| x >= to_free).min();
    println!("Smallest dir to delete is {:?}", smallest_dir);
    Ok(())
}

#[derive(Debug)]
struct DirBuilder<T> {
    pub name: String,
    pub children: HashMap<String, Either<Rc<RefCell<DirBuilder<T>>>, File<T>>>
}

impl<T> DirBuilder<T> {
    fn build(self) -> Dir<T> {
        Dir {
            name: self.name,
            children: self.children.into_values()
                .map(|dir_or_file| match dir_or_file {
                    Either::Left(dir_builder) => match Rc::try_unwrap(dir_builder) {
                        Ok(inner) => inner.into_inner().build(),
                        Err(_) => unreachable!()
                    }.into(),
                    Either::Right(file) => file.into()
                })
                .collect()
        }
    }
}

#[derive(Clone)]
pub struct Dir<T> {
    pub name: String,
    pub children: Vec<DirOrFile<T>>
}

impl<T> Dir<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            children: Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct File<T> {
    pub name: String,
    pub data: T
}

#[derive(Clone)]
pub enum DirOrFile<T> {
    Dir(Dir<T>),
    File(File<T>)
}

impl<T> DirOrFile<T> {
    pub fn reduce<Acc>(
        self,
        file_map: &mut dyn FnMut(File<T>) -> File<Acc>,
        dir_reduce: &dyn Fn(String, Vec<File<Acc>>) -> File<Acc>
    ) -> File<Acc> {
        match self {
            DirOrFile::File(val) => file_map(val),
            DirOrFile::Dir(Dir { name, children }) => dir_reduce(
                name,
                children.into_iter()
                    .map(|child| child.reduce(file_map, dir_reduce))
                    .collect()
            )
        }
    }
}

pub enum Line<T> {
    Command(Command),
    Entry(DirOrFile<T>)
}

pub enum Command {
    Ls,
    Dir(String)
}

impl<T> From<Command> for Line<T> {
    fn from(val: Command) -> Self {
        Line::Command(val)
    }
}

impl<T> From<DirOrFile<T>> for Line<T> {
    fn from(val: DirOrFile<T>) -> Self {
        Line::Entry(val)
    }
}

impl<T> From<Dir<T>> for DirOrFile<T> {
    fn from(val: Dir<T>) -> Self {
        DirOrFile::Dir(val)
    }
}

impl<T> From<File<T>> for DirOrFile<T> {
    fn from(val: File<T>) -> Self {
        DirOrFile::File(val)
    }
}

mod parser {
    use super::*;

    use nom::{
        branch::alt,
        Finish,
        combinator::map,
        IResult,
        sequence::{preceded, separated_pair},
        bytes::complete::{tag, take_while},
        character::complete::{u64 as nom_u64}
    };
    use anyhow::{anyhow, Result};

    pub fn line(s: &str) -> Result<Line<u64>> {
        alt((map(command, Into::into), map(entry, Into::into)))(s).finish()
            .map_err(|_| anyhow!("Internal parsing error"))
            .and_then(|(left, out)| {
                if !left.is_empty() {
                    Err(anyhow!("Did not consume all of line"))
                } else {
                    Ok(out)
                }
            })
    }

    fn command(s: &str) -> IResult<&str, Command> {
        preceded(
            tag("$ "),
            alt((
                map(tag("ls"), |_| Command::Ls),
                map(
                    preceded(tag("cd "), take_while(|_| true)),
                    |name: &str| Command::Dir(name.to_owned())
                )
            ))
        )(s)
    }

    fn entry(s: &str) -> IResult<&str, DirOrFile<u64>> {
        alt((
            map(
                preceded(tag("dir "), take_while(|_| true)),
                |name: &str| DirOrFile::Dir(Dir::new(name.to_owned()))
            ),
            map(
                separated_pair(nom_u64::<&str, _>, tag(" "), take_while(|_| true)),
                |(size, name)| DirOrFile::File(File { name: name.to_owned(), data: size })
            )
        ))(s)
    }
}
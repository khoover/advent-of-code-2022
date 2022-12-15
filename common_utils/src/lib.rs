use color_eyre::eyre::{eyre, Result};
use nom::{combinator::all_consuming, error::ParseError, Finish, Parser};

pub fn get_buffered_input() -> std::io::BufReader<std::fs::File> {
    let mut args = std::env::args();
    args.next().unwrap();
    let input_path = args.next().unwrap();
    let input_file = std::fs::OpenOptions::new()
        .read(true)
        .open(input_path)
        .unwrap();
    std::io::BufReader::new(input_file)
}

pub fn parse_line<'s, O, E, ParseFn>(line: &'s str, parser: ParseFn) -> Result<O>
where
    E: ParseError<&'s str> + std::error::Error,
    ParseFn: Parser<&'s str, O, E>,
{
    match all_consuming(parser)(line).finish() {
        Ok((_, out)) => Ok(out),
        Err(e) => Err(eyre!("Parsing error: {}", e)),
    }
}

pub fn parse_io_line<O, E, ParseFn>(line: std::io::Result<String>, parser: ParseFn) -> Result<O>
where
    E: for<'s> ParseError<&'s str> + std::error::Error,
    ParseFn: for<'s> Parser<&'s str, O, E>,
{
    match line {
        Ok(s) => parse_line(&s, parser),
        Err(e) => Err(e.into()),
    }
}

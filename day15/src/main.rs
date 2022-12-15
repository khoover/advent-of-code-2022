use std::io::BufRead;

use color_eyre::eyre::{eyre, Result};
use common_utils::{get_buffered_input, parse_line};
use itertools::Itertools;

fn main() -> Result<()> {
    color_eyre::install()?;

    let sensors: Vec<Sensor> = get_buffered_input()
        .lines()
        .map(|line_res| {
            let line = line_res?;
            parse_line(&line, parser::sensor)
        })
        .try_collect()?;

    println!("There are {} blocked positions", part1(&sensors, 2_000_000));
    println!("Part 2 answer is {}", part2(&sensors, 4000000)?);
    Ok(())
}

#[test]
fn sample() -> Result<()> {
    const SAMPLE_INPUT: &str = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";
    let sensors: Vec<Sensor> = SAMPLE_INPUT
        .lines()
        .map(|line| parse_line(line, parser::sensor))
        .try_collect()?;

    assert_eq!(part1(&sensors, 10), 26);
    assert_eq!(part2(&sensors, 20)?, 56000011);
    Ok(())
}

fn part1(sensors: &[Sensor], y: i32) -> usize {
    let min_x = sensors
        .iter()
        .map(|sensor| sensor.sensor_location.x - (sensor.l_1_radius as i32))
        .min()
        .unwrap();
    let max_x = sensors
        .iter()
        .map(|sensor| sensor.sensor_location.x + (sensor.l_1_radius as i32))
        .max()
        .unwrap();
    (min_x..=max_x)
        .map(|x| Point { x, y })
        .filter(|&point| {
            sensors
                .iter()
                .any(|sensor| sensor.sensor_location.l_1_dist(point) <= sensor.l_1_radius)
                && sensors.iter().all(|sensor| sensor.nearest_beacon != point)
        })
        .count()
}

fn part2(sensors: &[Sensor], max_xy: i32) -> Result<u64> {
    let distress_location = sensors
        .iter()
        .flat_map(|sensor| sensor.sensor_location.l_1_circle(sensor.l_1_radius + 1))
        .filter(|&point| point.x >= 0 && point.x <= max_xy && point.y >= 0 && point.y <= max_xy)
        .filter(|&point| {
            sensors
                .iter()
                .all(|sensor| sensor.sensor_location.l_1_dist(point) > sensor.l_1_radius)
        })
        .dedup()
        .inspect(|p| println!("{:?}", p))
        .exactly_one()
        .map_err(|_| eyre!("Multiple points found"))?;
    Ok((4000000 * distress_location.x as u64) + distress_location.y as u64)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    fn l_1_dist(self, other: Point) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    fn l_1_circle(self, radius: u32) -> impl Iterator<Item = Point> {
        (0..=radius).flat_map(move |offset| {
            [
                Point {
                    x: self.x + (radius - offset) as i32,
                    y: self.y + offset as i32,
                }, // [12, 3)
                Point {
                    x: self.x + offset as i32,
                    y: self.y - (radius - offset) as i32,
                }, // [3, 6)
                Point {
                    x: self.x - (radius - offset) as i32,
                    y: self.y - offset as i32,
                }, // [6, 9)
                Point {
                    x: self.x - offset as i32,
                    y: self.y + (radius - offset) as i32,
                }, // [9, 12)
            ]
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Sensor {
    sensor_location: Point,
    nearest_beacon: Point,
    l_1_radius: u32,
}

impl Sensor {
    fn new(sensor_location: Point, nearest_beacon: Point) -> Self {
        Self {
            sensor_location,
            nearest_beacon,
            l_1_radius: sensor_location.l_1_dist(nearest_beacon),
        }
    }
}

mod parser {
    use nom::{
        bytes::complete::tag,
        character::complete::i32 as nom_i32,
        combinator::map,
        sequence::{preceded, separated_pair},
        IResult,
    };

    use super::*;

    pub(super) fn sensor(s: &str) -> IResult<&str, Sensor> {
        map(
            separated_pair(sensor_location, tag(": "), beacon_location),
            |(sensor_loc, beacon_loc)| Sensor::new(sensor_loc, beacon_loc),
        )(s)
    }

    fn sensor_location(s: &str) -> IResult<&str, Point> {
        preceded(tag("Sensor at "), point)(s)
    }

    fn beacon_location(s: &str) -> IResult<&str, Point> {
        preceded(tag("closest beacon is at "), point)(s)
    }

    fn point(s: &str) -> IResult<&str, Point> {
        map(
            separated_pair(
                preceded(tag("x="), nom_i32),
                tag(", "),
                preceded(tag("y="), nom_i32),
            ),
            |(x, y)| Point { x, y },
        )(s)
    }
}

use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, separated_pair};
use nom::IResult;

use std::fmt::Write;

#[derive(Debug)]
struct Race {
    time: u64,
    distance: u64,
}

impl Race {
    fn num_winning_times(&self) -> u64 {
        let t = self.time as f64;
        let d = (self.distance + 1) as f64;
        let zero1 = (-1.0 * t + (t * t - 4.0 * d).sqrt()) / -2.0;
        let zero2 = (-1.0 * t - (t * t - 4.0 * d).sqrt()) / -2.0;
        (zero2.floor() - zero1.ceil() + 1.0) as u64
    }
}

#[derive(Debug)]
struct Match {
    races: Vec<Race>,
}

impl Match {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_pair(
                preceded(
                    pair(tag("Time:"), character::complete::space1),
                    separated_list1(character::complete::space1, character::complete::u64),
                ),
                character::complete::newline,
                preceded(
                    pair(tag("Distance:"), character::complete::space1),
                    separated_list1(character::complete::space1, character::complete::u64),
                ),
            ),
            |(times, distances)| {
                assert!(times.len() == distances.len());
                Match {
                    races: times
                        .into_iter()
                        .zip(distances)
                        .map(|(time, distance)| Race { time, distance })
                        .collect(),
                }
            },
        )(s)
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let (remainder, game) = Match::parse(input.trim()).unwrap();
    if !remainder.is_empty() {
        anyhow::bail!("unparsed input: {:?}", remainder);
    }
    let part1 = game
        .races
        .iter()
        .map(|r| r.num_winning_times())
        .product::<u64>();
    println!("part 1: {}", part1);
    let mut time_acc = String::new();
    let mut distance_acc = String::new();
    game.races.iter().for_each(|race| {
        write!(time_acc, "{}", race.time).unwrap();
        write!(distance_acc, "{}", race.distance).unwrap();
    });
    let race_part2 = Race {
        time: time_acc.parse::<u64>().unwrap(),
        distance: distance_acc.parse::<u64>().unwrap(),
    };
    let part2 = race_part2.num_winning_times();
    println!("part 2: {}", part2);
    Ok(())
}

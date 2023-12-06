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
    fn winning_times(&self) -> impl Iterator<Item = u64> {
        let total_time = self.time;
        let target = self.distance;
        (0..self.time).filter(move |hold_time| {
            let achieved = hold_time * (total_time - hold_time);
            achieved > target
        })
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
                        .zip(distances.into_iter())
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
        .map(|r| r.winning_times().count() as u64)
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
    let part2 = race_part2.winning_times().count();
    println!("part 2: {}", part2);
    Ok(())
}

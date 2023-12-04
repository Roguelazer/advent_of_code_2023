use clap::{Parser, ValueEnum};
use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, separated_pair, tuple};
use nom::IResult;

use std::collections::BTreeSet;

#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_enum)]
    mode: Mode,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Card {
    id: u32,
    winning: BTreeSet<u32>,
    found: BTreeSet<u32>,
}

impl Card {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_pair(
                preceded(
                    tuple((tag("Card"), character::complete::space1)),
                    character::complete::u32,
                ),
                tuple((tag(":"), character::complete::space1)),
                separated_pair(
                    separated_list1(character::complete::space1, character::complete::u32),
                    delimited(
                        character::complete::space1,
                        tag("|"),
                        character::complete::space1,
                    ),
                    separated_list1(character::complete::space1, character::complete::u32),
                ),
            ),
            |(card_id, (winning, got))| Card {
                id: card_id,
                winning: winning.into_iter().collect::<BTreeSet<u32>>(),
                found: got.into_iter().collect::<BTreeSet<u32>>(),
            },
        )(s)
    }

    fn score(&self) -> u32 {
        self.winning.intersection(&self.found).count() as u32
    }

    fn points(&self) -> u32 {
        let ct = self.score();
        if ct == 0 {
            0
        } else {
            1 << (ct - 1)
        }
    }
}

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let cards = stdin
        .lines()
        .map(|l| Card::parse(&l.unwrap()).unwrap().1)
        .collect::<Vec<Card>>();

    if args.mode == Mode::Part1 {
        let score: u32 = cards.into_iter().map(|c| c.points()).sum();
        println!("{}", score);
    } else {
        let mut card_counts = vec![1u32; cards.len()];
        let max = cards.len();
        for (i, card) in cards.into_iter().enumerate() {
            let score = card.score();
            let my_count = card_counts[i];
            for j in 1..=score {
                let offset = i + j as usize;
                if offset < max {
                    card_counts[offset] += my_count;
                }
            }
        }
        println!("{}", card_counts.into_iter().sum::<u32>());
    }
}

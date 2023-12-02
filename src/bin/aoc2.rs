use clap::{Parser, ValueEnum};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character,
    combinator::map,
    multi::separated_list1,
    sequence::{pair, preceded, terminated},
    IResult,
};

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
struct Pull {
    red: u32,
    green: u32,
    blue: u32,
}

impl Pull {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_list1(
                tag(", "),
                pair(
                    terminated(character::complete::u32, tag(" ")),
                    alt((tag("red"), tag("blue"), tag("green"))),
                ),
            ),
            |pairs| {
                let mut pull = Pull {
                    red: 0,
                    green: 0,
                    blue: 0,
                };
                pairs.into_iter().for_each(|(val, tag)| match tag {
                    "red" => pull.red += val,
                    "blue" => pull.blue += val,
                    "green" => pull.green += val,
                    _ => panic!("what"),
                });
                pull
            },
        )(s)
    }

    fn power(&self) -> u64 {
        (self.red as u64) * (self.blue as u64) * (self.green as u64)
    }
}

#[derive(Debug)]
struct Game {
    id: u32,
    pulls: Vec<Pull>,
}

impl Game {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            nom::sequence::tuple((
                preceded(tag("Game "), character::complete::u32),
                preceded(tag(": "), separated_list1(tag("; "), Pull::parse)),
            )),
            |(id, pulls)| Game { id, pulls },
        )(s)
    }

    fn fewest_cubes(&self) -> Pull {
        let mut r = Pull {
            red: 0,
            blue: 0,
            green: 0,
        };
        self.pulls.iter().for_each(|p| {
            if p.red > r.red {
                r.red = p.red;
            }
            if p.green > r.green {
                r.green = p.green;
            }
            if p.blue > r.blue {
                r.blue = p.blue;
            }
        });
        r
    }
}

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let games = stdin
        .lines()
        .map(|l| {
            let s = l.unwrap();
            let (remainder, game) = Game::parse(&s).unwrap();
            if !remainder.is_empty() {
                panic!("unhandled remainder {:?}", remainder);
            }
            game
        })
        .collect::<Vec<_>>();
    if args.mode == Mode::Part1 {
        let value: u32 = games
            .into_iter()
            .filter(|g| {
                g.pulls
                    .iter()
                    .all(|p| p.red <= 12 && p.green <= 13 && p.blue <= 14)
            })
            .map(|g| g.id)
            .sum();
        println!("{:?}", value);
    } else {
        let value: u64 = games.into_iter().map(|g| g.fewest_cubes().power()).sum();
        println!("{:?}", value);
    }
}

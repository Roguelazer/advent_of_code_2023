use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;

use std::collections::{BTreeMap, HashSet};

struct LoopForever<'a, T: Copy> {
    inner: &'a Vec<T>,
    index: usize,
}

impl<'a, T: Copy> LoopForever<'a, T> {
    fn new(v: &'a Vec<T>) -> Self {
        LoopForever { inner: v, index: 0 }
    }
}

impl<'a, T: Copy + 'a> Iterator for LoopForever<'a, T> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.inner.len() {
            None
        } else {
            let index = self.index;
            self.index = (self.index + 1) % self.inner.len();
            self.inner.get(index).cloned().map(|v| (index, v))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum Instruction {
    Left,
    Right,
}

impl Instruction {
    fn from_char(c: char) -> Self {
        match c {
            'L' => Instruction::Left,
            'R' => Instruction::Right,
            _ => panic!("unhandled instruction {}", c),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
struct Place {
    value: [u8; 3],
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}{}{}",
            char::from(self.value[0]),
            char::from(self.value[1]),
            char::from(self.value[2])
        )
    }
}

impl Place {
    fn new(s: &str) -> Self {
        if s.len() != 3 {
            panic!("unhandled place {:?}", s);
        }
        let buf = s.as_bytes();
        Place {
            value: buf.try_into().unwrap(),
        }
    }

    fn part_2_source(&self) -> bool {
        self.value[2] == b'A'
    }

    fn part_2_dest(&self) -> bool {
        self.value[2] == b'Z'
    }
}

#[derive(Debug)]
struct Game {
    sequence: Vec<Instruction>,
    map: BTreeMap<Place, (Place, Place)>,
}

impl Game {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_pair(
                character::complete::alpha1::<&str, _>,
                character::complete::multispace1,
                separated_list1(
                    character::complete::newline,
                    tuple((
                        terminated(character::complete::alpha1, tag(" = (")),
                        terminated(character::complete::alpha1, tag(", ")),
                        terminated(character::complete::alpha1, tag(")")),
                    )),
                ),
            ),
            |(sequence, instructions)| {
                let sequence = sequence.chars().map(Instruction::from_char).collect();
                let map = instructions
                    .into_iter()
                    .map(|(source, ldest, rdest)| {
                        (Place::new(source), (Place::new(ldest), Place::new(rdest)))
                    })
                    .collect();
                Game { sequence, map }
            },
        )(s)
    }

    fn find_inner(&self, start: Place, target: Place) -> Option<u64> {
        let mut steps = 0;
        let mut state = start;
        let mut seen = HashSet::new();
        for (i, instruction) in LoopForever::new(&self.sequence) {
            if seen.contains(&(state, i)) {
                return None;
            }
            seen.insert((state, i));
            if let Some(choice) = self.map.get(&state) {
                let next_state = match instruction {
                    Instruction::Left => choice.0,
                    Instruction::Right => choice.1,
                };
                steps += 1;
                if next_state == target {
                    return Some(steps);
                }
                state = next_state;
            } else {
                panic!("uh oh");
            }
        }
        None
    }

    fn part1(&self) -> Option<u64> {
        self.find_inner(Place::new("AAA"), Place::new("ZZZ"))
    }

    fn part2(&self) -> Option<u64> {
        let starts = self
            .map
            .keys()
            .filter(|s| s.part_2_source())
            .collect::<Vec<_>>();
        let ends = self
            .map
            .keys()
            .filter(|s| s.part_2_dest())
            .collect::<Vec<_>>();
        // technically, we should be taking the cartesian product of all the valid paths from each
        // start; however, it turns out that there's only one valid path for each start, so we
        // are going to be lazy. YMMV
        let res = starts
            .iter()
            .cartesian_product(&ends)
            .filter_map(|(start, end)| self.find_inner(**start, **end))
            .fold(1, num::integer::lcm);
        Some(res)
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let (remainder, game) = Game::parse(input.trim()).unwrap();
    if !remainder.is_empty() {
        anyhow::bail!("unparsed input: {:?}", remainder);
    }
    println!("part 1: {:?}", game.part1());
    println!("part 2: {:?}", game.part2());
    Ok(())
}

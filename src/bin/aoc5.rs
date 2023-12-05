use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use petgraph::algo::astar::astar;
use petgraph::graph::{DiGraph, NodeIndex};

use std::collections::BTreeMap;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct RangeMapEntry {
    start_id: u64,
    length: u64,
    target_id: u64,
}

impl RangeMapEntry {
    fn translate(&self, value: u64) -> u64 {
        assert!(value >= self.start_id);
        if value > self.end() {
            value
        } else {
            let offset = value - self.start_id;
            self.target_id + offset
        }
    }

    fn end(&self) -> u64 {
        self.start_id + self.length
    }
}

#[derive(Debug)]
struct RangeMap {
    source_to_dest: Vec<RangeMapEntry>,
}

impl RangeMap {
    // translate a contiguous range into one or more contiguous ranges of outputs
    fn translate_range(&self, mut start: u64, mut length: u64) -> Vec<(u64, u64)> {
        let mut output = Vec::new();
        let first = self.source_to_dest[0].start_id;
        while length > 0 {
            let pp = self.source_to_dest.partition_point(|v| v.start_id <= start);
            let (new_start, len) = if pp == 0 {
                // we're before the first mapped segment
                assert!(start < first);
                let remaining_before_start = first - start;
                let this_chunk_len = std::cmp::min(length, remaining_before_start);
                (start, this_chunk_len)
            } else {
                let segment = &self.source_to_dest[pp - 1];
                if start < segment.end() {
                    // we're in the segment
                    let remaining = segment.end() - start;
                    let this_chunk_len = std::cmp::min(length, remaining);
                    (segment.translate(start), this_chunk_len)
                } else {
                    // we're outside the segment
                    let remaining = if pp == self.source_to_dest.len() {
                        length
                    } else {
                        let next = &self.source_to_dest[pp];
                        next.start_id - start
                    };
                    let this_chunk_len = std::cmp::min(length, remaining);
                    (start, this_chunk_len)
                }
            };
            output.push((new_start, len));
            start += len;
            length -= len;
        }
        output
    }

    fn from(raw: Vec<(u64, u64, u64)>) -> Self {
        let mut source_to_dest = raw
            .into_iter()
            .map({
                |(target_id, start_id, length)| RangeMapEntry {
                    start_id,
                    length,
                    target_id,
                }
            })
            .collect::<Vec<_>>();
        source_to_dest.sort();
        RangeMap { source_to_dest }
    }
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<u64>,
    maps: BTreeMap<(String, String), RangeMap>,
    nodes: BTreeMap<String, NodeIndex>,
    nodes_rev: BTreeMap<NodeIndex, String>,
    graph: DiGraph<String, ()>,
}

impl Almanac {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_pair(
                preceded(
                    tag("seeds: "),
                    separated_list1(character::complete::space1, character::complete::u64),
                ),
                character::complete::multispace1,
                separated_list1(
                    character::complete::multispace1,
                    pair(
                        terminated(
                            separated_pair(
                                character::complete::alpha1,
                                tag("-to-"),
                                character::complete::alpha1,
                            ),
                            tag(" map:\n"),
                        ),
                        separated_list1(
                            character::complete::newline,
                            tuple((
                                terminated(character::complete::u64, tag(" ")),
                                terminated(character::complete::u64, tag(" ")),
                                character::complete::u64,
                            )),
                        ),
                    ),
                ),
            ),
            |(seeds, ranges)| {
                let maps = ranges
                    .into_iter()
                    .map(|((from, to), map): ((&str, &str), Vec<_>)| {
                        ((from.to_string(), to.to_string()), RangeMap::from(map))
                    })
                    .collect::<BTreeMap<(String, String), RangeMap>>();

                let mut graph = petgraph::graph::DiGraph::new();
                let mut nodes = BTreeMap::new();
                maps.keys().for_each(|(s, d)| {
                    let source = *nodes
                        .entry(s.to_owned())
                        .or_insert_with(|| graph.add_node(s.to_owned()));
                    let dest = *nodes
                        .entry(d.to_owned())
                        .or_insert_with(|| graph.add_node(d.to_owned()));
                    graph.add_edge(source, dest, ());
                });
                let nodes_rev = nodes
                    .iter()
                    .map(|(k, v)| (v.to_owned(), k.to_owned()))
                    .collect();

                Almanac {
                    seeds,
                    maps,
                    graph,
                    nodes,
                    nodes_rev,
                }
            },
        )(s)
    }

    fn seed_to_location_path(&self) -> Vec<NodeIndex> {
        let finish_node = self.nodes["location"];
        let path = astar(
            &self.graph,
            self.nodes["seed"],
            |finish| finish == finish_node,
            |_| 1,
            |_| 0,
        );
        path.unwrap().1
    }

    fn location_for_seed(&self, seed: u64) -> u64 {
        self.seed_to_location_path()
            .into_iter()
            .tuple_windows()
            .fold(seed, |value, (from, to)| {
                let from_name = self.nodes_rev[&from].clone();
                let to_name = self.nodes_rev[&to].clone();
                let map = &self.maps[&(from_name, to_name)];
                map.translate_range(value, 1)[0].0
            })
    }

    fn lowest_for_seed_range(&self, start: u64, length: u64) -> u64 {
        // we model this as a series of flows between segments, which may split or recombine
        let wavefrom = vec![(start, length)];
        let final_wave = self
            .seed_to_location_path()
            .into_iter()
            .tuple_windows()
            .fold(wavefrom, |wave, (from, to)| {
                let from_name = self.nodes_rev[&from].clone();
                let to_name = self.nodes_rev[&to].clone();
                let map = &self.maps[&(from_name, to_name)];
                // TODO: deduplicate wavefronts? theoretically we could
                // get a combinatorial explosion here, but in practice we
                // don't seem to get more than 32 waves live at a time.
                wave.into_iter()
                    .flat_map(|(start, length)| map.translate_range(start, length))
                    .collect::<Vec<_>>()
            });
        assert!(final_wave.iter().map(|(_, l)| *l).sum::<u64>() == length);
        final_wave.into_iter().map(|(s, _)| s).min().unwrap()
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let (remainder, almanac) = Almanac::parse(input.trim()).unwrap();
    if !remainder.is_empty() {
        anyhow::bail!("unparsed input: {:?}", remainder);
    }
    let res1 = almanac
        .seeds
        .iter()
        .map(|seed| almanac.location_for_seed(*seed))
        .min();
    println!("part1: {:?}", res1);
    let res2 = almanac
        .seeds
        .iter()
        .tuples()
        .map(|(seed_from, length)| almanac.lowest_for_seed_range(*seed_from, *length))
        .min();
    println!("part2: {:?}", res2);
    Ok(())
}

use aoclib::{DenseGrid, Point};
use itertools::Itertools;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
enum Tile {
    #[default]
    Path,
    Forest,
    SlopeRight,
    SlopeDown,
    SlopeUp,
    SlopeLeft,
}

const NORTH: Point = Point::new(0, -1);
const EAST: Point = Point::new(1, 0);
const WEST: Point = Point::new(-1, 0);
const SOUTH: Point = Point::new(0, 1);

type Graph<N, E, T> = petgraph::matrix_graph::MatrixGraph<N, E, T>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Path,
            '#' => Tile::Forest,
            '>' => Tile::SlopeRight,
            '<' => Tile::SlopeLeft,
            'v' => Tile::SlopeDown,
            '^' => Tile::SlopeUp,
            _ => panic!("unhandled char {}", c),
        }
    }

    fn directions(&self, mode: Mode) -> &'static [Point] {
        if mode == Mode::Part1 {
            match self {
                Tile::Path => &[NORTH, EAST, SOUTH, WEST],
                Tile::Forest => &[],
                Tile::SlopeRight => &[EAST],
                Tile::SlopeLeft => &[WEST],
                Tile::SlopeUp => &[NORTH],
                Tile::SlopeDown => &[SOUTH],
            }
        } else {
            match self {
                Tile::Forest => &[],
                _ => &[NORTH, EAST, SOUTH, WEST],
            }
        }
    }
}

fn path_find(map: &DenseGrid<Tile>, mode: Mode) -> usize {
    let start = map
        .iter()
        .find(|(p, t)| *t == Tile::Path && p.y == 0)
        .map(|(p, _)| p)
        .unwrap();
    let goal = map
        .iter()
        .find(|(p, t)| *t == Tile::Path && p.y == map.max_y)
        .map(|(p, _)| p)
        .unwrap();
    let mut graph: Graph<Point, usize, petgraph::Directed> = Graph::new();
    let mut point_to_index = BTreeMap::new();
    let start_index = graph.add_node(start);
    point_to_index.insert(goal, graph.add_node(goal));
    let mut paths = VecDeque::new();
    paths.push_back((start, start_index, 0));
    while let Some((mut point, start_of_path, mut path_len)) = paths.pop_front() {
        let mut straight = true;
        let start_of_path_point = graph.node_weight(start_of_path);
        let mut this_run: Vec<Point> = vec![*start_of_path_point];
        while straight {
            if let Some(index) = point_to_index.get(&point) {
                if !graph.has_edge(start_of_path, *index) {
                    graph.add_edge(start_of_path, *index, path_len);
                }
                break;
            }
            this_run.push(point);
            let tile = map.get(point).unwrap();
            let neighbors = tile
                .directions(mode)
                .iter()
                .map(|d| point + *d)
                .filter(|npoint| match map.get(*npoint) {
                    Some(Tile::Forest) => false,
                    None => false,
                    _ => !this_run.contains(&npoint),
                })
                .collect::<smallvec::SmallVec<[Point; 4]>>();
            if neighbors.len() == 1 {
                path_len += 1;
                point = neighbors[0];
            } else {
                // this is an intersection
                let index = point_to_index
                    .entry(point)
                    .or_insert_with(|| graph.add_node(point));
                if !graph.has_edge(start_of_path, *index) {
                    graph.add_edge(start_of_path, *index, path_len);
                }
                if mode == Mode::Part2 {
                    if !graph.has_edge(*index, start_of_path) {
                        graph.add_edge(*index, start_of_path, path_len);
                    }
                }
                straight = false;
                for neighbor in neighbors {
                    paths.push_back((neighbor, *index, 1));
                }
            }
        }
    }
    let goal = point_to_index[&goal];
    petgraph::algo::simple_paths::all_simple_paths::<Vec<_>, _>(&graph, start_index, goal, 0, None)
        .map(|path| {
            let distance: usize = path
                .into_iter()
                .tuple_windows()
                .map(|(a, b)| graph.edge_weight(a, b))
                .sum();
            distance
        })
        .max()
        .unwrap()
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let map = DenseGrid::from_input(&input, Tile::from_char);
    println!("part 1: {}", path_find(&map, Mode::Part1));
    println!("part 2: {}", path_find(&map, Mode::Part2));
    Ok(())
}

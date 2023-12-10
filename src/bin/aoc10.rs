use aoclib::Point;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn invert(&self) -> Direction {
        use Direction::*;

        match self {
            North => South,
            West => East,
            South => North,
            East => West,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Tile {
    NorthSouth,
    EastWest,
    NorthEast,
    NorthWest,
    SouthWest,
    SouthEast,
    Ground,
    Start,
}

impl Tile {
    fn connects_from(&self, d: Direction) -> bool {
        use Direction::*;
        use Tile::*;

        match (d, self) {
            (_, Start) => true,
            (_, Ground) => false,
            (North, NorthSouth) => true,
            (North, NorthEast) => true,
            (North, NorthWest) => true,
            (West, NorthWest) => true,
            (West, SouthWest) => true,
            (West, EastWest) => true,
            (South, NorthSouth) => true,
            (South, SouthWest) => true,
            (South, SouthEast) => true,
            (East, EastWest) => true,
            (East, NorthEast) => true,
            (East, SouthEast) => true,
            _ => false,
        }
    }

    fn connects_to(&self) -> Vec<Direction> {
        use Direction::*;
        use Tile::*;

        match self {
            NorthSouth => vec![North, South],
            EastWest => vec![East, West],
            NorthEast => vec![North, East],
            NorthWest => vec![North, West],
            SouthWest => vec![South, West],
            SouthEast => vec![South, East],
            Ground => vec![],
            Start => vec![North, South, East, West],
        }
    }

    fn from_char(c: char) -> Self {
        match c {
            '|' => Tile::NorthSouth,
            '-' => Tile::EastWest,
            'L' => Tile::NorthEast,
            'J' => Tile::NorthWest,
            '7' => Tile::SouthWest,
            'F' => Tile::SouthEast,
            '.' => Tile::Ground,
            'S' => Tile::Start,
            _ => panic!("unhandled char {}", c),
        }
    }
}

struct PassabilityMap {
    inner: BTreeMap<Point<i32>, bool>,
    width: i32,
    height: i32,
}

impl PassabilityMap {
    fn from(m: &Map, loop_points: &[Point<i32>]) -> Self {
        // explode a regular map into a map that shows how flows can pass *between* pipes
        //
        // AB
        // CD
        //
        // goes to
        //
        // A?B
        // ?.?
        // C?D
        //
        // the ?'s can be determined by looking at adjacent cells. Remmber, pipes only count
        // if they're on the "main" loop; even if we have other smaller loops, we ignore them
        let mut out = BTreeMap::new();
        let mut width = 0;
        let mut height = 0;
        for y in 0..=m.height {
            for x in 0..=m.width {
                let point = Point::new(x, y);
                let new_point = Point::new(x * 2, y * 2);
                width = std::cmp::max(width, x * 2);
                let tile = if loop_points.contains(&point) {
                    m.tiles[&point]
                } else {
                    Tile::Ground
                };
                out.insert(new_point, tile == Tile::Ground);
                let right_point = point + Point::new(1, 0);
                let below_point = point + Point::new(0, 1);
                let right = if loop_points.contains(&right_point) {
                    m.tiles.get(&right_point).cloned()
                } else {
                    Some(Tile::Ground)
                };
                let below = if loop_points.contains(&below_point) {
                    m.tiles.get(&below_point).cloned()
                } else {
                    Some(Tile::Ground)
                };
                let tc = match (tile, right) {
                    (Tile::Ground, _) => true,
                    (_, None) => true,
                    (l, Some(r))
                        if l.connects_from(Direction::East) && r.connects_from(Direction::West) =>
                    {
                        false
                    }
                    _ => true,
                };
                out.insert(new_point + Point::new(1, 0), tc);
                let lm = match (tile, below) {
                    (Tile::Ground, _) => true,
                    (_, None) => true,
                    (l, Some(r))
                        if l.connects_from(Direction::South)
                            && r.connects_from(Direction::North) =>
                    {
                        false
                    }
                    _ => true,
                };
                out.insert(new_point + Point::new(0, 1), lm);
                let new_right_below = new_point + Point::new(1, 1);
                out.insert(new_right_below, true);
            }
            height = std::cmp::max(height, y * 2);
        }
        Self {
            inner: out,
            width: width as i32,
            height: height as i32,
        }
    }

    fn dump(&self) {
        for y in 0..=self.height {
            for x in 0..=self.width {
                let point = Point::new(x, y);
                let c = if *self.inner.get(&point).unwrap() {
                    '.'
                } else {
                    'x'
                };
                print!("{}", c);
            }
            println!()
        }
    }
}

#[derive(Debug)]
struct Map {
    tiles: BTreeMap<Point<i32>, Tile>,
    adjacencies: BTreeMap<Point<i32>, BTreeSet<Point<i32>>>,
    width: i32,
    height: i32,
}

impl Map {
    fn from_input(input: &str) -> Self {
        let mut tiles = BTreeMap::new();
        let mut width = 0;
        let mut height = 0;
        for (y, row) in input.lines().enumerate() {
            for (x, chr) in row.chars().enumerate() {
                let coord = Point::<i32>::new(x as i32, y as i32);
                let tile = Tile::from_char(chr);
                tiles.insert(coord, tile);
                width = std::cmp::max(width, x);
            }
            height = std::cmp::max(height, y);
        }
        let mut adjacencies = BTreeMap::new();
        for (point, tile) in tiles.iter() {
            for direction in tile.connects_to() {
                let that_way = match direction {
                    Direction::North => Point::new(point.x, point.y - 1),
                    Direction::West => Point::new(point.x - 1, point.y),
                    Direction::South => Point::new(point.x, point.y + 1),
                    Direction::East => Point::new(point.x + 1, point.y),
                };
                if that_way.x < 0 || that_way.y < 0 {
                    continue;
                }
                if that_way.x > (width as i32) || that_way.y > (height as i32) {
                    continue;
                }
                if tiles[&that_way].connects_from(direction.invert()) {
                    adjacencies
                        .entry(*point)
                        .or_insert_with(BTreeSet::new)
                        .insert(that_way);
                    adjacencies
                        .entry(that_way)
                        .or_insert_with(BTreeSet::new)
                        .insert(*point);
                }
            }
        }
        Map {
            tiles,
            adjacencies,
            width: width as i32,
            height: height as i32,
        }
    }

    fn loop_containing_start(&self) -> Option<Vec<Point<i32>>> {
        let start = self
            .tiles
            .iter()
            .find(|(_, t)| **t == Tile::Start)
            .unwrap()
            .0;
        let mut queue = VecDeque::new();
        queue.push_front((start, vec![]));
        while let Some((node, path)) = queue.pop_back() {
            let mut next = path.clone();
            next.push(*node);
            if let Some(neighbors) = self.adjacencies.get(node) {
                for neighbor in neighbors {
                    if neighbor == start && path.len() > 1 {
                        return Some(next);
                    }
                    if path.contains(neighbor) {
                        continue;
                    }
                    queue.push_front((neighbor, next.clone()));
                }
            }
        }
        None
    }

    fn tiles_contained(&self, loop_points: &[Point<i32>]) -> usize {
        let exploded = PassabilityMap::from(self, loop_points);
        if false {
            exploded.dump();
        }

        // for every tile, either it has a path to the edge, all paths lead
        // to the loop, or it's contained in some other loop. Or it is this this loop, lol.
        let mut path_to_edge = BTreeSet::new();
        let mut exploded_path_to_edge = BTreeSet::new();
        let mut exploded_contained = BTreeSet::new();
        let mut path_to_loop = BTreeSet::new();
        let mut contained: BTreeSet<Point<i32>> = BTreeSet::new();
        for (point, _tile) in self.tiles.iter() {
            //println!("examining {:?}", point);
            if loop_points.contains(point) {
                path_to_loop.insert(*point);
            } else if point.x == 0
                || point.y == 0
                || point.x == self.width
                || point.y == self.height
            {
                path_to_edge.insert(*point);
                exploded_path_to_edge.insert(Point::new(point.x * 2, point.y * 2));
                exploded_path_to_edge.insert(Point::new(point.x * 2 + 1, point.y * 2));
                exploded_path_to_edge.insert(Point::new(point.x * 2, point.y * 2 + 1));
                exploded_path_to_edge.insert(Point::new(point.x * 2 + 1, point.y * 2 + 1));
            } else {
                // bfs out in the exploded map
                let mut queue: VecDeque<Point<i32>> = VecDeque::new();
                let mut seen: BTreeSet<Point<i32>> = BTreeSet::new();
                let mut escaped = false;
                // can start at any of the 4 points that this point maps to
                queue.push_back(Point::new(point.x * 2, point.y * 2));
                queue.push_back(Point::new(point.x * 2 + 1, point.y * 2));
                queue.push_back(Point::new(point.x * 2, point.y * 2 + 1));
                queue.push_back(Point::new(point.x * 2 + 1, point.y * 2 + 1));
                while let Some(new_point) = queue.pop_front() {
                    if seen.contains(&new_point) {
                        continue;
                    }
                    seen.insert(new_point);
                    if let Some(false) = exploded.inner.get(&new_point) {
                        continue;
                    }
                    if exploded_path_to_edge.contains(&new_point)
                        || new_point.x == 0
                        || new_point.y == 0
                        || new_point.x == exploded.width
                        || new_point.y == exploded.height
                    {
                        exploded_path_to_edge.insert(new_point);
                        escaped = true;
                        break;
                    } else if exploded_contained.contains(&new_point) {
                        break;
                    } else {
                        queue.push_back(new_point + Point::new(0, 1));
                        queue.push_back(new_point + Point::new(1, 0));
                        queue.push_back(new_point + Point::new(-1, 0));
                        queue.push_back(new_point + Point::new(0, -1));
                    }
                }
                if !escaped {
                    exploded_contained.insert(Point::new(point.x * 2, point.y * 2));
                    exploded_contained.insert(Point::new(point.x * 2 + 1, point.y * 2));
                    exploded_contained.insert(Point::new(point.x * 2, point.y * 2 + 1));
                    exploded_contained.insert(Point::new(point.x * 2 + 1, point.y * 2 + 1));
                    contained.insert(*point);
                }
            }
        }
        if false {
            println!();
            for y in 0..=self.height {
                for x in 0..=self.width {
                    let point = Point::new(x, y);
                    if loop_points.contains(&point) {
                        print!(".");
                    } else if contained.contains(&point) {
                        print!("I");
                    } else {
                        print!("O");
                    }
                }
                println!();
            }
        }
        contained.len()
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let map = Map::from_input(&input);
    let loopp = map.loop_containing_start().unwrap();
    println!("part 1: {}", loopp.len() / 2);
    println!("part 2: {}", map.tiles_contained(&loopp));
    Ok(())
}

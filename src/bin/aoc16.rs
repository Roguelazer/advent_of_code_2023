use aoclib::{DenseGrid, Point};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum Tile {
    #[default]
    Empty,
    // \
    MirrorDown,
    // /
    MirrorUp,
    // -
    HorizontalSplitter,
    // |
    VerticalSplitter,
}

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Empty,
            '\\' => Tile::MirrorDown,
            '/' => Tile::MirrorUp,
            '-' => Tile::HorizontalSplitter,
            '|' => Tile::VerticalSplitter,
            _ => panic!("unhandled char {:?}", c),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Beam {
    coord: Point,
    direction: Point,
}

impl Beam {
    fn advance(&self) -> Beam {
        Beam {
            coord: self.coord + self.direction,
            direction: self.direction,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum UpDown {
    Up,
    Down,
}

fn rotate(angle: UpDown, vector: Point) -> Point {
    match (angle, vector.as_tuple()) {
        (UpDown::Down, (1, 0)) => Point::new(0, 1),
        (UpDown::Down, (0, 1)) => Point::new(1, 0),
        (UpDown::Down, (-1, 0)) => Point::new(0, -1),
        (UpDown::Down, (0, -1)) => Point::new(-1, 0),
        (UpDown::Down, _) => unreachable!(),
        (UpDown::Up, (1, 0)) => Point::new(0, -1),
        (UpDown::Up, (0, 1)) => Point::new(-1, 0),
        (UpDown::Up, (-1, 0)) => Point::new(0, 1),
        (UpDown::Up, (0, -1)) => Point::new(1, 0),
        (UpDown::Up, _) => unreachable!(),
    }
}

fn energized(map: &DenseGrid<Tile>, beam: Beam, seen: &mut HashSet<Beam>) -> HashSet<Point> {
    if seen.contains(&beam) {
        return HashSet::new();
    }
    seen.insert(beam.clone());
    let value = map.get(beam.coord);
    let mut new = match value {
        Some(Tile::Empty) => energized(map, beam.advance(), seen),
        Some(Tile::MirrorDown) => {
            let next = Beam {
                coord: beam.coord,
                direction: rotate(UpDown::Down, beam.direction),
            };
            energized(map, next.advance(), seen)
        }
        Some(Tile::MirrorUp) => {
            let next = Beam {
                coord: beam.coord,
                direction: rotate(UpDown::Up, beam.direction),
            };
            energized(map, next.advance(), seen)
        }
        Some(Tile::VerticalSplitter) => {
            if beam.direction.y == 0 {
                let above = Beam {
                    coord: beam.coord,
                    direction: Point::new(0, 1),
                };
                let below = Beam {
                    coord: beam.coord,
                    direction: Point::new(0, -1),
                };
                let above = energized(map, above.advance(), seen);
                let below = energized(map, below.advance(), seen);
                above.union(&below).cloned().collect::<HashSet<Point>>()
            } else {
                energized(map, beam.advance(), seen)
            }
        }
        Some(Tile::HorizontalSplitter) => {
            if beam.direction.x == 0 {
                let right = Beam {
                    coord: beam.coord,
                    direction: Point::new(1, 0),
                };
                let left = Beam {
                    coord: beam.coord,
                    direction: Point::new(-1, 0),
                };
                let right = energized(map, right.advance(), seen);
                let left = energized(map, left.advance(), seen);
                left.union(&right).cloned().collect::<HashSet<Point>>()
            } else {
                energized(map, beam.advance(), seen)
            }
        }
        None => return HashSet::new(),
    };
    new.insert(beam.coord);
    new
}

fn part1(map: &DenseGrid<Tile>) -> usize {
    let mut seen = HashSet::new();
    energized(
        map,
        Beam {
            coord: Point::new(0, 0),
            direction: Point::new(1, 0),
        },
        &mut seen,
    )
    .len()
}

fn starts(map: &DenseGrid<Tile>) -> Vec<Beam> {
    let mut s = vec![];
    for x in 0..map.width() {
        let x = x as i64;
        s.push(Beam {
            coord: Point::new(x, 0),
            direction: Point::new(0, 1),
        });
        s.push(Beam {
            coord: Point::new(x, map.height() as i64 - 1),
            direction: Point::new(0, -1),
        });
    }
    for y in 0..map.height() {
        let y = y as i64;
        s.push(Beam {
            coord: Point::new(0, y),
            direction: Point::new(1, 0),
        });
        s.push(Beam {
            coord: Point::new(map.width() as i64 - 1, y),
            direction: Point::new(-1, 0),
        });
    }
    s
}

fn part2(map: &DenseGrid<Tile>) -> usize {
    starts(map)
        .into_iter()
        .map(|b| {
            let mut seen = HashSet::new();
            energized(map, b, &mut seen).len()
        })
        .max()
        .unwrap()
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let map = DenseGrid::from_input(&input, Tile::from_char);
    println!("part 1: {:?}", part1(&map));
    println!("part 2: {:?}", part2(&map));
    Ok(())
}

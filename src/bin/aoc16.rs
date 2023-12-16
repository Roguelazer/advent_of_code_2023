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
    match angle {
        UpDown::Down => Point::new(vector.y, vector.x),
        UpDown::Up => Point::new(-1 * vector.y, -1 * vector.x),
    }
}

fn energized(map: &DenseGrid<Tile>, beam: Beam) -> HashSet<Point> {
    let mut beams = vec![beam];
    let mut energized = HashSet::new();
    let mut seen = HashSet::new();

    while let Some(mut beam) = beams.pop() {
        loop {
            if seen.contains(&beam) {
                break;
            }
            seen.insert(beam.clone());
            let value = map.get(beam.coord);
            if value.is_some() {
                energized.insert(beam.coord);
            }
            match value {
                Some(Tile::Empty) => beam = beam.advance(),
                Some(Tile::MirrorDown) => {
                    beam = Beam {
                        coord: beam.coord,
                        direction: rotate(UpDown::Down, beam.direction),
                    }
                    .advance();
                }
                Some(Tile::MirrorUp) => {
                    beam = Beam {
                        coord: beam.coord,
                        direction: rotate(UpDown::Up, beam.direction),
                    }
                    .advance();
                }
                Some(Tile::VerticalSplitter) => {
                    if beam.direction.y == 0 {
                        let above = Beam {
                            coord: beam.coord,
                            direction: Point::new(0, 1),
                        }
                        .advance();
                        let below = Beam {
                            coord: beam.coord,
                            direction: Point::new(0, -1),
                        }
                        .advance();
                        beams.push(above);
                        beams.push(below);
                        break;
                    } else {
                        beam = beam.advance();
                    }
                }
                Some(Tile::HorizontalSplitter) => {
                    if beam.direction.x == 0 {
                        let right = Beam {
                            coord: beam.coord,
                            direction: Point::new(1, 0),
                        }
                        .advance();
                        let left = Beam {
                            coord: beam.coord,
                            direction: Point::new(-1, 0),
                        }
                        .advance();
                        beams.push(left);
                        beams.push(right);
                        break;
                    } else {
                        beam = beam.advance()
                    }
                }
                None => break,
            };
        }
    }
    energized
}

fn part1(map: &DenseGrid<Tile>) -> usize {
    energized(
        map,
        Beam {
            coord: Point::new(0, 0),
            direction: Point::new(1, 0),
        },
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
        .map(|b| energized(map, b).len())
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

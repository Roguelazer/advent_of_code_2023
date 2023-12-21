use aoclib::{DenseGrid, Point};
use ndarray::prelude::*;
use ndarray_linalg::Solve;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tile {
    Rock,
    #[default]
    Garden,
    Start,
}

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Self::Garden,
            '#' => Self::Rock,
            'S' => Self::Start,
            _ => panic!("unknown char {}", c),
        }
    }
}

const DIRS: [Point<i64>; 4] = [
    Point::new(0, 1),
    Point::new(1, 0),
    Point::new(-1, 0),
    Point::new(0, -1),
];

fn append_neighbors(g: &DenseGrid<Tile>, p: Point, o: &mut BTreeSet<Point>) {
    for dir in DIRS {
        match g.get(p + dir) {
            Some(Tile::Garden) | Some(Tile::Start) => {
                o.insert(p + dir);
            }
            _ => {}
        }
    }
}

fn part1(g: &DenseGrid<Tile>, steps: usize) -> BTreeSet<Point> {
    let mut wave = BTreeSet::new();
    for (point, tile) in g.iter() {
        if tile == Tile::Start {
            wave.insert(point);
        }
    }
    for _ in 0..steps {
        let mut next_wave = BTreeSet::new();
        for point in &wave {
            append_neighbors(g, *point, &mut next_wave);
        }
        wave = next_wave;
    }
    wave
}

fn part2(map: &DenseGrid<Tile>, steps: usize) -> usize {
    let mut duplicated = DenseGrid::new_with(
        Point::new(-3 * map.width() as i64, -3 * map.height() as i64),
        Point::new(3 * map.max_x, 3 * map.max_y),
        Tile::Garden,
    );
    for (point, tile) in map.iter() {
        duplicated.set(point, tile);
        for x_offset in [-2, -1, 0, 1, 2] {
            for y_offset in [-2, -1, 0, 1, 2] {
                if x_offset == 0 && y_offset == 0 {
                    continue;
                }
                let real_offset = Point::new(
                    x_offset * map.width() as i64,
                    y_offset * map.height() as i64,
                );
                duplicated.set(
                    point + real_offset,
                    if tile == Tile::Start {
                        Tile::Garden
                    } else {
                        tile
                    },
                );
            }
        }
    }
    let point1 = part1(&duplicated, 65);
    let point2 = part1(&duplicated, 131 + 65);
    let point3 = part1(&duplicated, 131 * 2 + 65);
    let a: Array2<f64> = array![[0.0, 0.0, 1.0], [1.0, 1.0, 1.0], [4.0, 2.0, 1.0],];
    let b: Array1<f64> = array![
        point1.len() as f64,
        point2.len() as f64,
        point3.len() as f64
    ];
    let x = a.solve_into(b).unwrap();
    let n = steps / 131;
    let x0 = x[0] as usize;
    let x1 = x[1] as usize;
    let x2 = x[2] as usize;
    x0 * n * n + x1 * n + x2
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let map = DenseGrid::from_input(&input, Tile::from_char);
    println!("part 1: {}", part1(&map, 64).len());
    println!("part 2: {}", part2(&map, 26501365));
    Ok(())
}

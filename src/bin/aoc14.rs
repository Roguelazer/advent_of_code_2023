use aoclib::{DenseGrid, Point};
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default, Hash)]
enum Cell {
    #[default]
    Empty,
    Round,
    Cube,
}

impl Cell {
    fn passable(&self) -> bool {
        match self {
            Cell::Empty => true,
            _ => false,
        }
    }

    fn from_char(c: char) -> Self {
        match c {
            '.' => Self::Empty,
            'O' => Self::Round,
            '#' => Self::Cube,
            _ => panic!("unhandled char {:?}", c),
        }
    }
}

fn tilt(grid: &DenseGrid<Cell>, direction: Point) -> DenseGrid<Cell> {
    let mut new = grid.clone();
    loop {
        let mut moved = 0;
        for y in grid.row_numbers() {
            for x in grid.column_numbers() {
                let point = Point::new(x, y);
                let value = new.get(point);
                if let Some(Cell::Round) = value {
                    let above = point + direction;
                    if new.get(above).map(|x| x.passable()).unwrap_or(false) {
                        new.set(above, Cell::Round);
                        new.set(point, Cell::Empty);
                        moved += 1;
                    }
                }
            }
        }
        if moved == 0 {
            break;
        }
    }
    new
}

fn score(g: &DenseGrid<Cell>) -> usize {
    let height = g.height();
    g.iter()
        .filter_map(|(point, value)| {
            if let Cell::Round = value {
                Some(height - point.y as usize)
            } else {
                None
            }
        })
        .sum()
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let grid = DenseGrid::from_input(&input, Cell::from_char);
    let tilted = tilt(&grid, Point::new(0, -1));
    println!("part 1: {:?}", score(&tilted));
    let mut g = grid.clone();
    let mut seen = HashMap::<_, usize>::new();
    let mut intervals = HashMap::new();
    let mut first_repeat = None;
    for i in 0..=500 {
        g = tilt(&g, Point::new(0, -1));
        g = tilt(&g, Point::new(-1, 0));
        g = tilt(&g, Point::new(0, 1));
        g = tilt(&g, Point::new(1, 0));
        let hashable = g.clone();
        if let Some(last_seen) = seen.get(&hashable) {
            let last_seen = last_seen.clone();
            if first_repeat.is_none() {
                first_repeat = Some(last_seen);
            }
            let interval = i - last_seen;
            *intervals.entry(interval).or_insert(0) += 1;
        }
        seen.insert(hashable, i);
    }
    let best_interval = intervals
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, _)| k)
        .unwrap();
    let first_repeat = first_repeat.unwrap();
    let part2_target = (1_000_000_000 - first_repeat) % best_interval + first_repeat;
    let mut g = grid;
    for _ in 0..part2_target {
        g = tilt(&g, Point::new(0, -1));
        g = tilt(&g, Point::new(-1, 0));
        g = tilt(&g, Point::new(0, 1));
        g = tilt(&g, Point::new(1, 0));
    }
    println!("part 2: {:?}", score(&g));
    Ok(())
}

use aoclib::{DenseGrid, Point, Rotation};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

fn calc(map: &DenseGrid<usize>, start: Point, destination: Point, min: usize, max: usize) -> usize {
    let mut work = BinaryHeap::new();
    let mut seen = HashSet::new();

    work.push((Reverse(0), start, Point::new(0, 1)));
    work.push((Reverse(0), start, Point::new(1, 0)));

    while let Some((u, point, direction)) = work.pop() {
        if point == destination {
            return u.0;
        }
        if seen.contains(&(point, direction)) {
            continue;
        }
        seen.insert((point, direction));
        for new_direction in &[
            direction.rotate_by(Rotation::CW),
            direction.rotate_by(Rotation::CCW),
        ] {
            let mut u2 = u.0;
            let mut new_point = point;
            for j in 1..=max {
                new_point = new_point + *new_direction;
                if let Some(val) = map.get(new_point) {
                    u2 += val;
                    if j >= min {
                        work.push((Reverse(u2), new_point, *new_direction));
                    }
                }
            }
        }
    }
    panic!("failed to find a path!");
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let map = DenseGrid::from_input(&input, |c| c.to_digit(10).unwrap() as usize);
    let destination = Point::new(map.max_x, map.max_y);
    println!(
        "part 1: {:?}",
        calc(&map, Point::new(0, 0), destination, 1, 3)
    );
    println!(
        "part 2: {:?}",
        calc(&map, Point::new(0, 0), destination, 4, 10)
    );
    Ok(())
}

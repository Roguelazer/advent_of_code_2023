use aoclib::DenseGrid;
use aoclib::Point;
use itertools::Itertools;

fn read_map(s: &str) -> DenseGrid<bool> {
    DenseGrid::from_input(s, |chr| match chr {
        '.' => false,
        '#' => true,
        _ => panic!("what is {:?}", chr),
    })
}

fn translate(
    p: Point<i64>,
    empty_columns: &[i64],
    empty_rows: &[i64],
    expand_by: usize,
) -> Point<i64> {
    let x_offset = empty_columns.iter().filter(|c| **c <= p.x).count() * (expand_by - 1);
    let y_offset = empty_rows.iter().filter(|c| **c <= p.y).count() * (expand_by - 1);
    p + Point::new(x_offset as i64, y_offset as i64)
}

fn nonsense(g: &DenseGrid<bool>, expand_by: usize) -> usize {
    let empty_columns = g
        .columns()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.iter().all(|v| !v) {
                Some(i as i64)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let empty_rows = g
        .rows()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.iter().all(|v| !v) {
                Some(i as i64)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let coordinates = g
        .iter()
        .filter_map(|(p, v)| if v { Some(p) } else { None })
        .collect::<Vec<_>>();
    coordinates
        .into_iter()
        .combinations(2)
        .map(|k| {
            let a = translate(k[0], &empty_columns, &empty_rows, expand_by);
            let b = translate(k[1], &empty_columns, &empty_rows, expand_by);
            a.manhattan_distance_to(b)
        })
        .sum()
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let g = read_map(&input);
    println!("part 1: {}", nonsense(&g, 1));
    println!("part 2: {}", nonsense(&g, 1000000));
    Ok(())
}

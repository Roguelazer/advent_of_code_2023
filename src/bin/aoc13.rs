use aoclib::DenseGrid;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
enum Cell {
    #[default]
    Ash,
    Rock,
}

fn rc2i(rc: &[Cell]) -> u64 {
    rc.iter().fold(0, |acc, cell| {
        if *cell == Cell::Rock {
            (acc << 1) + 1
        } else {
            acc << 1
        }
    })
}

fn eq_bw(lhs: &[u64], rhs: &[u64]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    if lhs.len() == 0 {
        return false;
    }
    let len = lhs.len() - 1;
    for i in 0..=len {
        if lhs[i] != rhs[len - i] {
            return false;
        }
    }
    true
}

fn approx_eq_bw(lhs: &[u64], rhs: &[u64]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    if lhs.len() == 0 {
        return false;
    }
    let len = lhs.len() - 1;
    let mut num_smudges_required = 0;
    for i in 0..=len {
        // try smudging this column
        if lhs[i] == rhs[len - i] {
            continue;
        } else if (lhs[i] ^ rhs[len - i]).count_ones() == 1 {
            num_smudges_required += 1;
        } else {
            return false;
        }
    }
    num_smudges_required == 1
}

fn find_reflection(vals: &[u64]) -> Option<usize> {
    vals.iter()
        .enumerate()
        .skip(1)
        .take(vals.len() - 1)
        .filter_map(|(offset, _)| {
            let to_consider = std::cmp::min(offset, vals.len() - offset);
            let lhs = &vals[offset - to_consider..offset];
            let rhs = &vals[offset..offset + to_consider];
            if eq_bw(lhs, rhs) {
                Some(offset)
            } else {
                None
            }
        })
        .max()
}

fn find_smudge_reflection(vals: &[u64]) -> Option<usize> {
    vals.iter()
        .enumerate()
        .skip(1)
        .take(vals.len() - 1)
        .find_map(|(offset, _)| {
            let to_consider = std::cmp::min(offset, vals.len() - offset);
            let lhs = &vals[offset - to_consider..offset];
            let rhs = &vals[offset..offset + to_consider];
            if approx_eq_bw(lhs, rhs) {
                Some(offset)
            } else {
                None
            }
        })
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let as_ints = input
        .split("\n\n")
        .map(|segment| {
            DenseGrid::from_input(segment, |f| match f {
                '.' => Cell::Ash,
                '#' => Cell::Rock,
                _ => panic!("what is {:?}", f),
            })
        })
        .map(|grid| {
            let column_ints = grid.columns().map(|c| rc2i(&c)).collect::<Vec<u64>>();
            let row_ints = grid.rows().map(|r| rc2i(&r)).collect::<Vec<u64>>();
            (column_ints, row_ints)
        });
    let part1: usize = as_ints
        .clone()
        .map(|(column_ints, row_ints)| {
            let column_reflection = find_reflection(&column_ints);
            let row_reflection = find_reflection(&row_ints);
            column_reflection.unwrap_or(0) + row_reflection.unwrap_or(0) * 100
        })
        .sum();
    println!("part 1: {}", part1);
    let part2: usize = as_ints
        .map(|(column_ints, row_ints)| {
            find_smudge_reflection(&row_ints).unwrap_or(0) * 100
                + find_smudge_reflection(&column_ints).unwrap_or(0)
        })
        .sum();
    println!("part 2: {}", part2);
    Ok(())
}

use itertools::Itertools;

fn is_zeros(l: &[i32]) -> bool {
    l.iter().all(|c| *c == 0)
}

fn delta(l: &[i32]) -> Vec<i32> {
    l.iter().tuple_windows().map(|(a, b)| b - a).collect()
}

fn build_dims(s: Vec<i32>) -> Vec<Vec<i32>> {
    let mut dims = vec![];
    let mut current = s;
    loop {
        if is_zeros(&current) {
            dims.push(current);
            break;
        } else {
            let next = delta(&current);
            dims.push(current);
            current = next;
        }
    }
    dims.reverse();
    dims
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let sequences = input
        .lines()
        .map(|l| {
            l.split_whitespace()
                .map(|w| w.parse::<i32>().unwrap())
                .collect::<Vec<i32>>()
        })
        .collect::<Vec<_>>();
    let part1: i32 = sequences
        .clone()
        .into_iter()
        .map(|s| {
            let mut dims = build_dims(s);
            let mut last = 0;
            dims.iter_mut().for_each(|r| {
                let v = r.last().cloned().unwrap_or(0) + last;
                r.push(v);
                last = v;
            });
            last
        })
        .sum();
    println!("part1: {}", part1);
    let part2: i32 = sequences
        .clone()
        .into_iter()
        .map(|s| {
            let mut dims = build_dims(s);
            let mut last = 0;
            dims.iter_mut().for_each(|r| {
                let v = r.first().cloned().unwrap_or(0) - last;
                r.insert(0, v);
                last = v;
            });
            last
        })
        .sum();
    println!("part2: {}", part2);
    Ok(())
}

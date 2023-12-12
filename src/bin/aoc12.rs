use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Spring {
    Broken,
    Working,
    Unknown,
}

impl Spring {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Self::Working,
            '#' => Self::Broken,
            '?' => Self::Unknown,
            _ => panic!("unknown char {:?}", c),
        }
    }

    fn unknown(&self) -> bool {
        match *self {
            Self::Broken => false,
            Self::Working => false,
            Self::Unknown => true,
        }
    }
}

fn s2i(ss: &[Spring]) -> u64 {
    let mut i = 0;
    for s in ss {
        if *s == Spring::Broken {
            i += 1;
        }
        i <<= 1;
    }
    i
}

#[derive(Debug, Clone)]
struct Row {
    springs: Vec<Spring>,
    groups: Vec<usize>,
    total_broken: usize,
}

impl Row {
    fn from_line(s: &str) -> Self {
        if let Some((first, rest)) = s.split_once(' ') {
            let springs = first.chars().map(Spring::from_char).collect();
            let groups: Vec<usize> = rest.split(',').map(|w| w.parse().unwrap()).collect();
            let total_broken = groups.iter().sum();
            Row {
                springs,
                groups,
                total_broken,
            }
        } else {
            panic!("unhandled line {:?}", s);
        }
    }

    fn valid(&self) -> bool {
        if self
            .springs
            .iter()
            .filter(|s| **s == Spring::Broken)
            .count()
            != self.total_broken
        {
            return false;
        }
        let mut git = self.groups.iter();
        let mut current = 0;
        let mut current_goal: usize = git.next().cloned().unwrap_or(0);
        let mut saw_w = true;
        for spring in &self.springs {
            if *spring == Spring::Broken {
                if current_goal == 0 {
                    return false;
                } else if current == 0 && !saw_w {
                    // a new group can only start after a gap
                    return false;
                } else {
                    current += 1;
                }
                saw_w = false;
            } else {
                saw_w = true;
                if current != 0 {
                    if current_goal != current {
                        return false;
                    } else {
                        current = 0;
                        current_goal = git.next().cloned().unwrap_or(0);
                    }
                }
            }
        }
        let next = git.next();
        current == current_goal && next.is_none()
    }

    fn part1_solutions(&self) -> usize {
        let unknown_indices = self
            .springs
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if s.unknown() { Some(i) } else { None })
            .collect::<Vec<_>>();
        // println!("row: {:?}", self);

        assert!(unknown_indices.len() > 0);
        assert!(unknown_indices.len() < 32);

        println!("got {:?} unknowns", unknown_indices.len());

        let mut candidate = self.clone();

        let mut options = HashSet::new();

        for mut ival in 0..=(1 << unknown_indices.len()) {
            for sindex in &unknown_indices {
                let val = (ival & 0x1) == 0x1;
                ival >>= 1;
                candidate.springs[*sindex] = if val { Spring::Broken } else { Spring::Working }
            }
            if candidate.valid() {
                // println!("valid solution: {:?}", candidate.springs);
                options.insert(s2i(&candidate.springs));
            }
        }
        // println!("found {:?} options", options.len());
        options.len()
    }

    fn part2_solutions(&self) -> usize {
        let unknown_indices = self
            .springs
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if s.unknown() { Some(i) } else { None })
            .collect::<Vec<_>>();

        let mut git = self.groups.iter();

        todo!();
    }

    fn to_part2(&self) -> Self {
        let mut springs = vec![];
        let mut groups = vec![];
        for _ in 0..5 {
            springs.extend_from_slice(&self.springs);
            springs.push(Spring::Unknown);
            groups.extend_from_slice(&self.groups);
        }
        springs.pop();
        let total_broken = groups.iter().sum();
        Self {
            springs,
            groups,
            total_broken,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let rows = input.lines().map(Row::from_line).collect::<Vec<_>>();
    let part1: usize = rows.iter().map(|r| r.part1_solutions()).sum();
    println!("part 1: {}", part1);
    let part2_rows = rows.iter().map(|r| r.to_part2()).collect::<Vec<_>>();
    let part2: usize = part2_rows.iter().map(|r| r.part2_solutions()).sum();
    println!("part 2: {}", part2);
    Ok(())
}

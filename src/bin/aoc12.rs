use std::collections::BTreeMap;

const MAX_GROUPS: usize = 32;

type GroupType = u8;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
}

fn s2i(ss: &[Spring]) -> u128 {
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
    groups: Vec<GroupType>,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct CacheKey {
    current_broken: GroupType,
    springs_len: usize,
    springs: u128,
    groups: [GroupType; MAX_GROUPS],
}

impl CacheKey {
    fn new(s: &[Spring], g: &[GroupType], current_broken: GroupType) -> Self {
        assert!(g.len() < MAX_GROUPS);
        let mut groups = [0; MAX_GROUPS];
        for (i, gp) in g.iter().enumerate() {
            groups[i] = *gp;
        }
        CacheKey {
            springs: s2i(s),
            springs_len: s.len(),
            groups,
            current_broken,
        }
    }
}

impl Row {
    fn from_line(s: &str) -> Self {
        if let Some((first, rest)) = s.split_once(' ') {
            let springs = first.chars().map(Spring::from_char).collect();
            let groups = rest.split(',').map(|w| w.parse().unwrap()).collect();
            Row { springs, groups }
        } else {
            panic!("unhandled line {:?}", s);
        }
    }

    fn solutions_inner(
        &self,
        dp: &mut BTreeMap<CacheKey, usize>,
        springs: &[Spring],
        groups: &[GroupType],
        current_broken: GroupType,
    ) -> usize {
        let key = CacheKey::new(springs, groups, current_broken);
        if let Some(found) = dp.get(&key) {
            return *found;
        };

        let val = match springs.split_first() {
            Some((Spring::Working, rs)) => {
                if current_broken == 0 {
                    self.solutions_inner(dp, rs, groups, 0)
                } else {
                    match groups {
                        [g, rg @ ..] if *g == current_broken => self.solutions_inner(dp, rs, rg, 0),
                        _ => 0,
                    }
                }
            }
            Some((Spring::Broken, rs)) => {
                return match groups {
                    [g, ..] if current_broken > *g => 0,
                    [] => 0,
                    _ => self.solutions_inner(dp, rs, groups, current_broken + 1),
                }
            }
            Some((Spring::Unknown, _)) => {
                let mut new_working = springs.to_vec();
                new_working[0] = Spring::Working;
                let working_count = self.solutions_inner(dp, &new_working, groups, current_broken);
                let mut new_broken = springs.to_vec();
                new_broken[0] = Spring::Broken;
                let broken_count = self.solutions_inner(dp, &new_broken, groups, current_broken);
                working_count + broken_count
            }
            None => match groups {
                [] if current_broken == 0 => 1,
                [g] if *g == current_broken => 1,
                _ => 0,
            },
        };

        dp.insert(key, val);

        val
    }

    fn num_solutions(&self) -> usize {
        let mut cache = BTreeMap::new();
        self.solutions_inner(&mut cache, &self.springs, &self.groups, 0)
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
        Self { springs, groups }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let rows = input.lines().map(Row::from_line).collect::<Vec<_>>();
    let part1: usize = rows.iter().map(|r| r.num_solutions()).sum();
    println!("part 1: {}", part1);
    let part2: usize = rows.iter().map(|r| r.to_part2().num_solutions()).sum();
    println!("part 2: {}", part2);
    Ok(())
}

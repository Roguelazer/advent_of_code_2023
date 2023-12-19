use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::{map, opt, value};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Field {
    A,
    S,
    M,
    X,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Field::A => 'a',
                Field::S => 's',
                Field::M => 'm',
                Field::X => 'x',
            }
        )
    }
}

impl Field {
    fn parse(s: &str) -> IResult<&str, Self> {
        alt((
            value(Field::A, tag("a")),
            value(Field::S, tag("s")),
            value(Field::M, tag("m")),
            value(Field::X, tag("x")),
        ))(s)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Target {
    Workflow(String),
    Accept,
    Reject,
}

#[derive(Debug)]
enum BinOp {
    Gt,
    Lt,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                BinOp::Gt => '>',
                BinOp::Lt => '<',
            }
        )
    }
}

impl BinOp {
    fn parse(s: &str) -> IResult<&str, Self> {
        alt((map(tag(">"), |_| BinOp::Gt), map(tag("<"), |_| BinOp::Lt)))(s)
    }
}

#[derive(Debug)]
struct Comparison {
    field: Field,
    op: BinOp,
    value: i32,
}

impl std::fmt::Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{} {} {}", self.field, self.op, self.value)
    }
}

impl Comparison {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            tuple((Field::parse, BinOp::parse, character::complete::i32)),
            |(field, op, value)| Comparison { field, op, value },
        )(s)
    }

    fn matches(&self, part: &BTreeMap<Field, i32>) -> bool {
        let value = part[&self.field];
        match self.op {
            BinOp::Gt => value > self.value,
            BinOp::Lt => value < self.value,
        }
    }
}

#[derive(Debug)]
struct Rule {
    comparison: Option<Comparison>,
    target: Target,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        if let Some(ref c) = self.comparison {
            write!(f, "{} => {:?}", c, self.target)
        } else {
            write!(f, "all => {:?}", self.target)
        }
    }
}

impl Rule {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            pair(
                opt(terminated(Comparison::parse, tag(":"))),
                alt((
                    map(tag("R"), |_| Target::Reject),
                    map(tag("A"), |_| Target::Accept),
                    map(character::complete::alpha1, |w: &str| {
                        Target::Workflow(w.to_string())
                    }),
                )),
            ),
            |(comparison, target)| Rule { comparison, target },
        )(s)
    }

    fn apply(&self, part: &BTreeMap<Field, i32>) -> Option<&Target> {
        if self
            .comparison
            .as_ref()
            .map(|c| c.matches(part))
            .unwrap_or(true)
        {
            Some(&self.target)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Workflow {
    name: String,
    rules: Vec<Rule>,
}

impl Workflow {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            pair(
                character::complete::alpha1,
                delimited(tag("{"), separated_list1(tag(","), Rule::parse), tag("}")),
            ),
            |(name, rules)| Workflow {
                name: name.to_string(),
                rules,
            },
        )(s)
    }

    fn apply(&self, part: &BTreeMap<Field, i32>) -> &Target {
        for rule in &self.rules {
            if let Some(t) = rule.apply(part) {
                return t;
            }
        }
        panic!("fell off ruleset");
    }
}

#[derive(Debug)]
struct Program {
    workflows: BTreeMap<String, Workflow>,
    parts: Vec<BTreeMap<Field, i32>>,
}

impl Program {
    fn parse_parts(s: &str) -> IResult<&str, BTreeMap<Field, i32>> {
        map(
            delimited(
                tag("{"),
                separated_list1(
                    tag(","),
                    separated_pair(Field::parse, tag("="), character::complete::i32),
                ),
                tag("}"),
            ),
            |r| r.into_iter().map(|(s, v)| (s, v)).collect(),
        )(s)
    }

    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_pair(
                separated_list1(character::complete::newline, Workflow::parse),
                character::complete::multispace1,
                separated_list1(character::complete::newline, Self::parse_parts),
            ),
            |(workflows, parts)| {
                let workflows = workflows.into_iter().map(|w| (w.name.clone(), w)).collect();
                Program { workflows, parts }
            },
        )(s)
    }

    fn run(&self, part: &BTreeMap<Field, i32>) -> anyhow::Result<Target> {
        let mut workflow = self.workflows.get("in").expect("Could not find first rule");
        loop {
            match workflow.apply(part) {
                Target::Accept => return Ok(Target::Accept),
                Target::Reject => return Ok(Target::Reject),
                Target::Workflow(ref wf) => {
                    workflow = self.workflows.get(wf).expect("Could not find rule")
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundRange {
    inner: Vec<(i32, i32)>,
}

impl BoundRange {
    fn new() -> Self {
        BoundRange {
            inner: vec![(1, 4000)],
        }
    }

    fn len(&self) -> usize {
        self.inner.iter().map(|&(l, u)| (u - l) as usize + 1).sum()
    }

    fn empty() -> Self {
        BoundRange { inner: vec![] }
    }

    fn intersect_comparison(&self, comparison: &Comparison) -> Self {
        let inner = self
            .inner
            .iter()
            .filter_map(|&(l, u)| match comparison.op {
                BinOp::Gt => {
                    if comparison.value > u {
                        None
                    } else {
                        Some((std::cmp::max(comparison.value, l), u))
                    }
                }
                BinOp::Lt => {
                    if comparison.value < l {
                        None
                    } else {
                        Some((l, std::cmp::min(comparison.value - 1, u)))
                    }
                }
            })
            .collect();
        Self { inner }
    }

    fn subtract_comparison(&self, comparison: &Comparison) -> Self {
        match comparison.op {
            BinOp::Gt => {
                // (1, 4000) - (> 3000) -> (1, 3000)
                // (2500, 3500) - (> 3000) -> (2500, 3000)
                // (3500, 4000) - (> 3000) -> None
                let inner = self
                    .inner
                    .iter()
                    .filter_map(|(lower, upper)| {
                        if *lower > comparison.value {
                            None
                        } else {
                            Some((*lower, std::cmp::min(comparison.value, *upper)))
                        }
                    })
                    .collect();
                BoundRange { inner }
            }
            BinOp::Lt => {
                // (1, 4000) - (<4000) -> None
                // (1, 4000) - (<2000) -> (2000, 4000)
                // (2000, 4000) -  (<2000) -> (2000, 4000)
                let inner = self
                    .inner
                    .iter()
                    .filter_map(|(lower, upper)| {
                        if *lower > comparison.value {
                            None
                        } else {
                            Some((std::cmp::max(comparison.value, *lower), *upper))
                        }
                    })
                    .collect();
                BoundRange { inner }
            }
        }
    }
}

impl std::fmt::Display for BoundRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "[ ")?;
        for (l, u) in self.inner.iter() {
            write!(f, "{}-{} ", l, u)?;
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone)]
struct Constraints {
    inner: BTreeMap<Field, BoundRange>,
}

impl Constraints {
    fn new() -> Self {
        let mut c = BTreeMap::new();
        c.insert(Field::A, BoundRange::new());
        c.insert(Field::S, BoundRange::new());
        c.insert(Field::M, BoundRange::new());
        c.insert(Field::X, BoundRange::new());
        Constraints { inner: c }
    }

    fn empty() -> Self {
        let mut c = BTreeMap::new();
        c.insert(Field::A, BoundRange::empty());
        c.insert(Field::S, BoundRange::empty());
        c.insert(Field::M, BoundRange::empty());
        c.insert(Field::X, BoundRange::empty());
        Constraints { inner: c }
    }

    fn len(&self) -> usize {
        self.inner.values().map(|v| v.len()).product()
    }

    fn subtract_comparison(&self, comparison: &Option<Comparison>) -> Self {
        if let Some(comparison) = comparison {
            let mut new = self.clone();
            new.inner.insert(
                comparison.field,
                self.inner[&comparison.field].subtract_comparison(comparison),
            );
            new
        } else {
            Constraints::empty()
        }
    }

    fn intersect_comparison(&self, comparison: &Option<Comparison>) -> Self {
        let res = if let Some(comparison) = comparison {
            let new = self
                .inner
                .iter()
                .map(|(k, v)| {
                    if *k == comparison.field {
                        (*k, v.intersect_comparison(comparison))
                    } else {
                        (*k, v.clone())
                    }
                })
                .collect();
            Constraints { inner: new }
        } else {
            self.clone()
        };
        res
    }
}

impl std::fmt::Display for Constraints {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{{")?;
        for (key, values) in self.inner.iter() {
            write!(f, "{} {}  ", key, values)?;
        }
        write!(f, "}}")
    }
}

fn find_constraints(
    p: &Program,
    workflow: &Workflow,
    mut constraints: Constraints,
) -> Vec<Constraints> {
    let mut accepts = vec![];
    for rule in &workflow.rules {
        match rule.target {
            Target::Reject => {
                // if it goes to reject, then subtract whatever that set is from the "current"
                // constraint set
                let next = constraints.subtract_comparison(&rule.comparison);
                constraints = next;
            }
            Target::Accept => {
                let accepted = if rule.comparison.is_some() {
                    constraints.intersect_comparison(&rule.comparison)
                } else {
                    constraints.clone()
                };
                accepts.push(accepted);
                let next = constraints.subtract_comparison(&rule.comparison);
                constraints = next;
            }
            Target::Workflow(ref w) => {
                let nw = &p.workflows[w];
                let next = constraints.intersect_comparison(&rule.comparison);
                let next_accept = find_constraints(p, nw, next.clone());
                accepts.extend_from_slice(&next_accept);
                constraints = constraints.subtract_comparison(&rule.comparison);
            }
        }
    }
    accepts
}

fn part2(p: &Program) -> usize {
    // use petgraph to find all paths that end with an accept
    // for each path, built a set of constraints evaluated along it
    let accepts = find_constraints(p, p.workflows.get("in").unwrap(), Constraints::new());
    accepts.iter().map(|a| a.len()).sum()
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let (remainder, program) = Program::parse(input.trim()).unwrap();
    if !remainder.is_empty() {
        anyhow::bail!("unparsed input: {:?}", remainder);
    };
    let part1: i32 = program
        .parts
        .iter()
        .filter(|part| program.run(part).unwrap() == Target::Accept)
        .map(|part| part.values().sum::<i32>())
        .sum();
    println!("part 1: {}", part1);
    let part2 = part2(&program);
    println!("part 2: {}", part2);
    Ok(())
}

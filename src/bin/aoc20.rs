use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;
use petgraph::graph::DiGraph;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Pulse {
    Low,
    High,
}

impl std::ops::Not for Pulse {
    type Output = Pulse;

    fn not(self) -> Self::Output {
        match self {
            Pulse::Low => Pulse::High,
            Pulse::High => Pulse::Low,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Type {
    Broadcast,
    Conj,
    FlipFlop,
    Output,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct RawElement {
    name: String,
    etype: Type,
}

impl RawElement {
    fn parse(s: &str) -> IResult<&str, Self> {
        alt((
            map(tag("broadcaster"), |_| RawElement {
                name: "broadcaster".to_string(),
                etype: Type::Broadcast,
            }),
            map(
                preceded(tag("%"), character::complete::alpha1),
                |n: &str| RawElement {
                    name: n.to_string(),
                    etype: Type::FlipFlop,
                },
            ),
            map(
                preceded(tag("&"), character::complete::alpha1),
                |n: &str| RawElement {
                    name: n.to_string(),
                    etype: Type::Conj,
                },
            ),
            map(character::complete::alpha1, |n: &str| RawElement {
                name: n.to_string(),
                etype: Type::Output,
            }),
        ))(s)
    }

    fn finalize(self, index: petgraph::graph::NodeIndex) -> Element {
        Element {
            name: self.name,
            etype: self.etype,
            index,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Element {
    name: String,
    etype: Type,
    index: petgraph::graph::NodeIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ElementState {
    Broadcast,
    FlipFlop(bool),
    Conj(BTreeMap<String, Pulse>),
    Output(Option<Pulse>),
}

#[derive(Debug, Clone)]
struct State {
    elements: BTreeMap<String, ElementState>,
}

impl State {
    fn new_for(circuit: &Circuit) -> State {
        let mut elements = BTreeMap::new();
        for (name, value) in circuit.elements.iter() {
            let new_value = match value.etype {
                Type::Broadcast => ElementState::Broadcast,
                Type::FlipFlop => ElementState::FlipFlop(false),
                Type::Output => ElementState::Output(None),
                Type::Conj => {
                    let mut inner = BTreeMap::new();
                    let neighbors = circuit
                        .graph
                        .neighbors_directed(value.index, petgraph::Direction::Incoming);
                    for neighbor in neighbors {
                        let n = circuit.graph.node_weight(neighbor).unwrap();
                        inner.insert(n.to_string(), Pulse::Low);
                    }
                    ElementState::Conj(inner)
                }
            };
            elements.insert(name.to_string(), new_value);
        }
        State { elements }
    }

    #[allow(dead_code)]
    fn get(&self, s: &str) -> Option<&ElementState> {
        self.elements.get(s)
    }

    fn handle(&mut self, source: &str, dest: &str, pulse: Pulse) -> Option<Pulse> {
        match self.elements.get_mut(dest) {
            Some(ElementState::Broadcast) => panic!("sent a pulse into broadcast"),
            Some(ElementState::Output(ref mut old_state)) => {
                let _ = old_state.insert(pulse);
                None
            }
            Some(ElementState::FlipFlop(ref mut was)) => {
                if pulse == Pulse::High {
                    None
                } else {
                    if *was {
                        *was = false;
                        Some(Pulse::Low)
                    } else {
                        *was = true;
                        Some(Pulse::High)
                    }
                }
            }
            Some(ElementState::Conj(ref mut inputs)) => {
                inputs.insert(source.to_string(), pulse);
                if inputs.values().all(|p| *p == Pulse::High) {
                    Some(Pulse::Low)
                } else {
                    Some(Pulse::High)
                }
            }
            None => panic!("received pulse for unknown {:?}", dest),
        }
    }
}

#[derive(Debug, Clone)]
struct Circuit {
    elements: BTreeMap<String, Element>,
    graph: DiGraph<String, ()>,
}

impl Circuit {
    fn parse(s: &str) -> IResult<&str, Self> {
        map(
            separated_list1(
                character::complete::newline,
                separated_pair(
                    RawElement::parse,
                    tag(" -> "),
                    separated_list1(tag(", "), character::complete::alpha1),
                ),
            ),
            |elems| {
                let mut graph = DiGraph::new();
                let mut elements = BTreeMap::new();

                let edges = elems
                    .into_iter()
                    .map(|(elem, neighbors)| {
                        let name = elem.name.clone();
                        let index = graph.add_node(name.clone());
                        let finalied = elem.finalize(index);
                        elements.insert(name.clone(), finalied);
                        (index, neighbors)
                    })
                    .collect::<Vec<_>>();

                for (index, targets) in edges {
                    for target in targets {
                        let tindex = if let Some(target_e) = elements.get(target) {
                            target_e.index
                        } else {
                            let raw = RawElement {
                                name: target.to_string(),
                                etype: Type::Output,
                            };
                            let idx = graph.add_node(target.to_string());
                            let finalized = raw.finalize(idx);
                            elements.insert(target.to_string(), finalized);
                            idx
                        };
                        graph.add_edge(index, tindex, ());
                    }
                }

                Circuit { graph, elements }
            },
        )(s)
    }

    fn simulate_one<'a, 'b, F>(
        &'a self,
        state: &'b mut State,
        pulse: Pulse,
        mut f: Option<F>,
    ) -> (usize, usize)
    where
        F: FnMut(&'a str, Pulse) -> bool,
        'b: 'a,
    {
        let mut low_pulses = 1;
        let mut high_pulses = 0;
        let mut pulses = VecDeque::new();
        let broadcaster_ix = self.elements["broadcaster"].index;
        for neighbor in self
            .graph
            .neighbors_directed(broadcaster_ix, petgraph::Direction::Outgoing)
        {
            pulses.push_back((
                "broadcast".to_string(),
                self.graph.node_weight(neighbor).unwrap().to_string(),
                pulse,
            ));
        }
        while let Some((source_name, dest_name, pulse)) = pulses.pop_front() {
            if pulse == Pulse::High {
                high_pulses += 1;
            } else {
                low_pulses += 1;
            }
            if let Some(outgoing) = state.handle(&source_name, &dest_name, pulse) {
                let my_ix = self.elements[&dest_name].index;
                for neighbor in self
                    .graph
                    .neighbors_directed(my_ix, petgraph::Direction::Outgoing)
                {
                    if let Some(ref mut f) = f {
                        let node = self.graph.node_weight(neighbor).unwrap();
                        if f(node, outgoing) {
                            return (0, 0);
                        }
                    }
                    pulses.push_back((
                        dest_name.clone(),
                        self.graph.node_weight(neighbor).unwrap().to_string(),
                        outgoing,
                    ));
                }
            }
        }
        (low_pulses, high_pulses)
    }

    fn simulate_part1(&self, pulse: Pulse) -> usize {
        let mut state = State::new_for(self);
        let (low, high) = (0..1000)
            .map(|_| self.simulate_one(&mut state, pulse, Some(|_, _| false)))
            .reduce(|(acc_low, acc_high), (low, high)| (acc_low + low, acc_high + high))
            .unwrap();
        low * high
    }

    fn find_period(&self, target: &str) -> usize {
        let mut state = State::new_for(&self);
        let mut found = None;
        for iter in 1..20000 {
            let checker = |node, pulse| {
                if (node == target) && (pulse == Pulse::Low) {
                    let _ = found.insert(iter);
                    true
                } else {
                    false
                }
            };
            self.simulate_one(&mut state, Pulse::Low, Some(checker));
            if let Some(found) = found {
                return found;
            }
        }
        panic!("unable to find period for {}", target);
    }

    fn part2(&self) -> usize {
        // TODO: walk the graph to figure out the inputs
        ["pg", "sp", "sv", "qs"]
            .iter()
            .map(|input| self.find_period(input))
            .reduce(|acc, e| num::integer::lcm(acc, e))
            .unwrap()
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let (remainder, circuit) = Circuit::parse(input.trim()).unwrap();
    if !remainder.is_empty() {
        anyhow::bail!("unparsed input: {:?}", remainder);
    };
    println!("part 1: {}", circuit.simulate_part1(Pulse::Low));
    println!("part 2: {}", circuit.part2());
    Ok(())
}

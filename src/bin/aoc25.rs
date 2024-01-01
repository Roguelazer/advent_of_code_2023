use petgraph::visit::EdgeRef;
use std::collections::{BTreeMap, BTreeSet};

type GTy = petgraph::stable_graph::StableGraph<String, usize, petgraph::Undirected>;

struct Problem {
    graph: GTy,
}

impl Problem {
    fn read(input: &str) -> Self {
        let mut g = GTy::default();
        let mut nodes = std::collections::BTreeMap::new();
        for line in input.lines() {
            if let Some((lhs, rhs)) = line.split_once(": ") {
                let lhs_index = nodes
                    .entry(lhs.to_string())
                    .or_insert_with(|| g.add_node(lhs.to_string()))
                    .clone();
                for target in rhs.split_whitespace() {
                    let target_index = nodes
                        .entry(target.to_string())
                        .or_insert_with(|| g.add_node(target.to_string()));
                    g.add_edge(lhs_index, *target_index, 1);
                }
            }
        }
        Self { graph: g }
    }
}

fn stoer_wagner(g: &GTy) -> Vec<BTreeSet<String>> {
    // largely translated from the NetworkX implementation
    //
    assert!(g.node_count() > 2);
    let mut scratch = g.clone();

    let mut cut_value: Option<usize> = None;
    let mut best_round: Option<usize> = None;
    let initial_nodes = scratch
        .node_indices()
        .map(|i| scratch.node_weight(i).unwrap().clone())
        .collect::<BTreeSet<_>>();
    let mut contractions = vec![];

    let n = scratch.node_count();

    for i in 0..(n - 1) {
        let mut u = scratch.node_indices().next().unwrap();
        let mut a = [u].into_iter().collect::<BTreeSet<_>>();
        let mut h = scratch
            .edges(u)
            .map(|edge| {
                let weight = edge.weight();
                let neighbor = if edge.source() == u {
                    edge.target()
                } else {
                    edge.source()
                };
                (neighbor, *weight)
            })
            .collect::<mut_binary_heap::BinaryHeap<_, _>>();
        let n = scratch.node_count();
        while a.len() < n - 1 {
            // pretend it's a fucked-up heap
            let (nu, _) = h.pop_with_key().unwrap();
            u = nu;
            a.insert(u);
            for e in scratch.edges(u) {
                let n = if e.source() == u {
                    e.target()
                } else {
                    e.source()
                };
                if !a.contains(&n) {
                    // too bad there's no .entry...
                    let updated = if let Some(ref mut r) = h.get_mut(&n) {
                        **r += *e.weight();
                        true
                    } else {
                        false
                    };
                    if !updated {
                        h.push(n, *e.weight());
                    }
                }
            }
        }
        if let Some((v, w)) = h.pop_with_key() {
            if let Some(cv) = cut_value {
                if w < cv {
                    cut_value = Some(w);
                    best_round = Some(i);
                }
            } else {
                cut_value = Some(w);
                best_round = Some(i);
            }
            let u_label = scratch.node_weight(u).unwrap().clone();
            let v_label = scratch.node_weight(v).unwrap().clone();
            contractions.push((u_label, v_label));
            let mut to_change = vec![];
            let mut to_add = vec![];
            for edge in scratch.edges(v) {
                let peer = if edge.source() == v {
                    edge.target()
                } else {
                    edge.source()
                };
                if peer != u {
                    let weight = *edge.weight();
                    if let Some(edge) = scratch.find_edge(u, peer) {
                        to_change.push((edge, weight));
                    } else {
                        to_add.push((peer, weight));
                    }
                }
            }
            for (edge_id, weight_delta) in to_change {
                *scratch.edge_weight_mut(edge_id).unwrap() += weight_delta;
            }
            for (peer, weight) in to_add {
                scratch.add_edge(u, peer, weight);
            }
            scratch.remove_node(v);
        } else {
            panic!("uh oh");
        }
    }
    if let Some(best_phase) = best_round {
        let mut subgraph = GTy::default();
        let mut nodes = BTreeMap::new();
        for (lhs, rhs) in contractions.iter().take(best_phase) {
            let lc = lhs.clone();
            let rc = rhs.clone();
            let lhs_index = nodes
                .entry(lhs)
                .or_insert_with(|| subgraph.add_node(lc))
                .clone();
            let rhs_index = nodes
                .entry(rhs)
                .or_insert_with(|| subgraph.add_node(rc))
                .clone();
            subgraph.add_edge(lhs_index, rhs_index, 1);
        }
        let start = nodes.get(&contractions[best_phase].1).unwrap();
        let mut bfs = petgraph::visit::Bfs::new(&subgraph, *start);
        let mut reachable = BTreeSet::new();
        while let Some(nx) = bfs.next(&subgraph) {
            reachable.insert(subgraph.node_weight(nx).unwrap().to_owned());
        }
        vec![&initial_nodes - &reachable, reachable]
    } else {
        panic!("uh oh #2");
    }
}

fn part1(problem: &Problem) -> usize {
    let partitions = stoer_wagner(&problem.graph);
    assert_eq!(partitions.len(), 2);
    partitions.into_iter().map(|l| l.len()).product()
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let p = Problem::read(&input);
    println!("part 1: {}", part1(&p));
    Ok(())
}

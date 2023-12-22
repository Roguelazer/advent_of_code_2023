use aoclib::Vec3;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    index: usize,
    lower: Vec3,
    upper: Vec3,
}

fn poifi<'a, I: Iterator<Item = &'a str>>(i: &mut I) -> anyhow::Result<i64> {
    let lx: i64 = i.next().ok_or(anyhow::anyhow!("missing x[0]"))?.parse()?;
    Ok(lx)
}

impl Block {
    fn from_str(s: &str, index: usize) -> anyhow::Result<Self> {
        if let Some((lhs, rhs)) = s.trim().split_once('~') {
            let mut lhs_i = lhs.split(',');
            let mut rhs_i = rhs.split(',');
            let (lx, ly, lz) = (poifi(&mut lhs_i)?, poifi(&mut lhs_i)?, poifi(&mut lhs_i)?);
            let (rx, ry, rz) = (poifi(&mut rhs_i)?, poifi(&mut rhs_i)?, poifi(&mut rhs_i)?);
            let mut lhs = Vec3::new(lx, ly, lz);
            let mut rhs = Vec3::new(rx, ry, rz);
            if rz < lz {
                (lhs, rhs) = (rhs, lhs);
            }
            Ok(Block {
                index,
                lower: lhs,
                upper: rhs,
            })
        } else {
            anyhow::bail!("unhandled input in {}", s);
        }
    }

    #[inline(always)]
    fn min_x(&self) -> i64 {
        std::cmp::min(self.lower.x, self.upper.x)
    }

    #[inline(always)]
    fn min_y(&self) -> i64 {
        std::cmp::min(self.lower.y, self.upper.y)
    }

    #[inline(always)]
    fn min_z(&self) -> i64 {
        std::cmp::min(self.lower.z, self.upper.z)
    }

    #[inline(always)]
    fn max_x(&self) -> i64 {
        std::cmp::max(self.lower.x, self.upper.x)
    }

    #[inline(always)]
    fn max_y(&self) -> i64 {
        std::cmp::max(self.lower.y, self.upper.y)
    }

    #[inline(always)]
    fn max_z(&self) -> i64 {
        std::cmp::max(self.lower.z, self.upper.z)
    }

    fn drop_to(&mut self, z: i64) {
        let distance = self.min_z() - z;
        let vec = Vec3::new(0, 0, -1 * distance);
        self.lower += vec;
        self.upper += vec;
    }

    fn occludes(&self, other: &Block) -> bool {
        if self.min_z() > other.min_z() {
            return false;
        }
        let x_ok = other.max_x() >= self.min_x() && other.min_x() <= self.max_x();
        let y_ok = other.max_y() >= self.min_y() && other.min_y() <= self.max_y();
        x_ok && y_ok
    }
}

struct Problem {
    blocks: BTreeMap<usize, Block>,
    occlusion: petgraph::graph::DiGraph<usize, ()>,
    block_index_to_graph_index: BTreeMap<usize, petgraph::graph::NodeIndex>,
    supporting: BTreeMap<usize, Vec<usize>>,
    supported_by: BTreeMap<usize, Vec<usize>>,
}

impl Problem {
    fn new(mut blocks: BTreeMap<usize, Block>) -> Self {
        let mut occlusion = petgraph::Graph::new();
        let mut block_index_to_graph_index = BTreeMap::new();
        for block in blocks.values() {
            let graph_index = occlusion.add_node(block.index);
            block_index_to_graph_index.insert(block.index, graph_index);
        }
        for block in blocks.values() {
            let parent_graph_index = block_index_to_graph_index.get(&block.index).unwrap();
            for candidate in blocks.values() {
                if candidate == block {
                    continue;
                }
                if block.occludes(&candidate) {
                    let child_graph_index =
                        block_index_to_graph_index.get(&candidate.index).unwrap();
                    occlusion.add_edge(*parent_graph_index, *child_graph_index, ());
                }
            }
        }
        let mut visitor = petgraph::visit::Topo::new(&occlusion);
        while let Some(graph_index) = visitor.next(&occlusion) {
            let incoming = occlusion
                .neighbors_directed(graph_index, petgraph::Direction::Incoming)
                .map(|n| {
                    let block_index = occlusion.node_weight(n).unwrap();
                    blocks[block_index].max_z()
                })
                .max()
                .unwrap_or(0);
            if let Some(me) = occlusion
                .node_weight(graph_index)
                .and_then(|u| blocks.get_mut(u))
            {
                me.drop_to(incoming + 1);
            }
        }
        let mut supported_by = BTreeMap::new();
        for (child_u, child_block) in blocks.iter() {
            let child_graph_index = block_index_to_graph_index.get(&child_block.index).unwrap();
            for parent_graph_index in
                occlusion.neighbors_directed(*child_graph_index, petgraph::Direction::Incoming)
            {
                if *child_graph_index == parent_graph_index {
                    continue;
                }
                let u = occlusion.node_weight(parent_graph_index).unwrap();
                let block = &blocks[u];
                if block.max_z() + 1 == child_block.min_z() {
                    supported_by
                        .entry(*child_u)
                        .or_insert_with(Vec::new)
                        .push(*u);
                }
            }
        }
        let mut supporting: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (block, supporters) in supported_by.iter() {
            for k in supporters {
                supporting.entry(*k).or_insert_with(Vec::new).push(*block);
            }
        }
        Self {
            occlusion,
            blocks,
            block_index_to_graph_index,
            supporting,
            supported_by,
        }
    }

    fn part1(&self) -> usize {
        let destructable = self
            .blocks
            .keys()
            .filter(|u| !self.supported_by.values().any(|b| b == &[**u]))
            .cloned()
            .collect::<Vec<usize>>();
        destructable.len()
    }

    fn part2_helper(&self, root: usize) -> usize {
        println!("running for {}", root);
        let mut removed = BTreeSet::<usize>::new();
        removed.insert(root);
        let mut work = VecDeque::new();
        work.push_back(root);
        while let Some(todo) = work.pop_front() {
            let num_supporters = self
                .supported_by
                .get(&todo)
                .map(|sb| sb.iter().filter(|node| !removed.contains(node)).count())
                .unwrap_or(0);
            if num_supporters == 0 {
                removed.insert(todo);
            }
            if let Some(children) = self.supporting.get(&todo) {
                for item in children {
                    work.push_back(*item);
                }
            }
        }
        removed.remove(&root);
        return removed.len();
    }

    fn part2(&self) -> usize {
        self.blocks.keys().map(|u| self.part2_helper(*u)).sum()
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let blocks = input
        .lines()
        .enumerate()
        .map(|(i, l)| Block::from_str(l, i))
        .collect::<anyhow::Result<Vec<Block>>>()?
        .into_iter()
        .map(|b| (b.index, b))
        .collect::<BTreeMap<usize, Block>>();
    let problem = Problem::new(blocks);
    println!("part 1: {}", problem.part1());
    println!("part 2: {}", problem.part2());
    Ok(())
}

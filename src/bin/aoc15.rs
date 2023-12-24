fn hash(s: &str) -> usize {
    let mut v = 0usize;
    for c in s.as_bytes().iter() {
        v += *c as usize;
        v *= 17;
        v %= 256;
    }
    v
}

#[derive(Debug)]
enum Command<'a> {
    Assign(&'a str, u8),
    Remove(&'a str),
}

impl<'a> Command<'a> {
    fn parse(s: &'a str) -> Self {
        if s.contains('=') {
            let (first, rest) = s.split_once('=').unwrap();
            Command::Assign(first, rest.parse().unwrap())
        } else if s.contains('-') {
            let label = &s[..s.len() - 1];
            Command::Remove(label)
        } else {
            panic!("what is {:?}", s);
        }
    }

    #[allow(dead_code)]
    fn label(&self) -> &str {
        match self {
            Command::Assign(s, _) => s,
            Command::Remove(s) => s,
        }
    }
}

#[derive(Debug)]
struct Box {
    label: String,
    value: u8,
}

#[derive(Debug)]
struct Boxes {
    inner: Vec<Vec<Box>>,
}

impl Boxes {
    fn new() -> Self {
        let mut inner = Vec::with_capacity(256);
        for _ in 0..256 {
            inner.push(vec![]);
        }
        Boxes { inner }
    }

    fn insert(&mut self, label: &str, value: u8) {
        let h = hash(label);
        if let Some(ref mut r) = self.inner[h].iter_mut().find(|i| i.label == label) {
            r.value = value;
        } else {
            self.inner[h].push(Box {
                label: label.to_string(),
                value,
            });
        }
    }

    fn remove(&mut self, label: &str) {
        let h = hash(label);
        if let Some((i, _)) = self.inner[h]
            .iter()
            .enumerate()
            .find(|(_, b)| b.label == label)
        {
            self.inner[h].remove(i);
        }
    }

    fn score(&self) -> u64 {
        self.inner
            .iter()
            .enumerate()
            .map(|(box_number, boxes)| {
                let bni = box_number + 1;
                boxes
                    .iter()
                    .enumerate()
                    .map(|(offset, bx)| {
                        let ofi = offset + 1;
                        (ofi as u64) * (bni as u64) * (bx.value as u64)
                    })
                    .sum::<u64>()
            })
            .sum()
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let part1: usize = input.trim().split(',').map(hash).sum();
    println!("part 1: {}", part1);
    let mut boxes = Boxes::new();
    for command_str in input.trim().split(',') {
        let command = Command::parse(command_str);
        match command {
            Command::Assign(label, value) => boxes.insert(label, value),
            Command::Remove(label) => boxes.remove(label),
        }
    }
    println!("part 2: {}", boxes.score());
    Ok(())
}

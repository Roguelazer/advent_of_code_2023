use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Card(u8);

impl Card {
    fn is_joker(&self) -> bool {
        self.0 == 1
    }
}

impl TryFrom<char> for Card {
    type Error = anyhow::Error;

    fn try_from(s: char) -> anyhow::Result<Self> {
        let value = match s {
            'O' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'T' => 10,
            'J' => 11,
            'Q' => 12,
            'K' => 13,
            'A' => 14,
            _ => anyhow::bail!("invalid character {:?}", s),
        };
        Ok(Card(value))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

impl HandType {
    fn adjust_by(&self, n: usize) -> Self {
        match n {
            0 => *self,
            1 => match self {
                Self::HighCard => Self::OnePair,
                Self::OnePair => Self::ThreeOfAKind,
                Self::TwoPair => Self::FullHouse,
                Self::ThreeOfAKind => Self::FourOfAKind,
                Self::FourOfAKind => Self::FiveOfAKind,
                _ => panic!("unreachable 1"),
            },
            2 => match self {
                Self::HighCard => Self::ThreeOfAKind,
                Self::OnePair => Self::FourOfAKind,
                Self::ThreeOfAKind => Self::FiveOfAKind,
                _ => panic!("unreachable 2"),
            },
            3 => match self {
                Self::HighCard => Self::FourOfAKind,
                Self::OnePair => Self::FiveOfAKind,
                _ => panic!("unreachable 3"),
            },
            4 => Self::FiveOfAKind,
            5 => Self::FiveOfAKind,
            _ => panic!("unhandled increment {}", n),
        }
    }

    fn for_sorted_cards(cards: Vec<Card>) -> Self {
        let mut groups = Vec::new();
        let mut last_card = Card(0);
        let mut current_group_size = 0;
        let mut jokers = 0usize;
        for card in &cards {
            let card = *card;
            if card.is_joker() {
                jokers += 1;
                continue;
            }
            if card != last_card && current_group_size > 0 {
                groups.push(current_group_size);
                current_group_size = 0;
            }
            current_group_size += 1;
            last_card = card;
        }
        if current_group_size > 0 {
            groups.push(current_group_size)
        }
        groups.sort();
        let hand_type = if let Some(best) = groups.last() {
            match best {
                5 => HandType::FiveOfAKind,
                4 => HandType::FourOfAKind,
                3 => {
                    if groups == &[2, 3] {
                        HandType::FullHouse
                    } else {
                        HandType::ThreeOfAKind
                    }
                }
                2 => {
                    if groups == &[2, 2] || groups == &[1, 2, 2] {
                        HandType::TwoPair
                    } else {
                        HandType::OnePair
                    }
                }
                1 => HandType::HighCard,
                _ => panic!("got unexpected groups {:?}", groups),
            }
        } else {
            HandType::HighCard
        };
        if jokers > 0 {
            hand_type.adjust_by(jokers)
        } else {
            hand_type
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord)]
struct Hand {
    cards: Vec<Card>,
    hand_type: HandType,
}

impl Hand {
    fn new(cards: Vec<Card>) -> Self {
        let mut sorted_cards = cards.clone();
        sorted_cards.sort();
        let hand_type = HandType::for_sorted_cards(sorted_cards);
        Hand { cards, hand_type }
    }
}

impl PartialOrd<Hand> for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.hand_type.cmp(&other.hand_type) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => {
                for (lhs, rhs) in self.cards.iter().zip(other.cards.iter()) {
                    if lhs > rhs {
                        return Some(Ordering::Greater);
                    } else if lhs < rhs {
                        return Some(Ordering::Less);
                    }
                }
                Some(Ordering::Equal)
            }
        }
    }
}

impl std::str::FromStr for Hand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if s.len() != 5 {
            anyhow::bail!("expected a string of length 5");
        }
        let cards = s
            .chars()
            .map(Card::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Hand::new(cards))
    }
}

#[derive(Debug)]
struct Game {
    bids: Vec<(Hand, u32)>,
}

impl std::str::FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let bids: anyhow::Result<Vec<_>> = s
            .lines()
            .filter(|s| !s.is_empty())
            .map(|line| {
                if let Some((hand, rest)) = line.split_once(' ') {
                    let hand = hand.parse::<Hand>()?;
                    let bid = rest.trim().parse::<u32>()?;
                    Ok((hand, bid))
                } else {
                    Err(anyhow::anyhow!("invalid line {}", line))
                }
            })
            .collect();
        let mut bids = bids?;
        bids.sort();
        Ok(Game { bids })
    }
}

impl Game {
    fn score(&self) -> u64 {
        self.bids
            .iter()
            .enumerate()
            .map(|(i, bid)| (i as u64 + 1) * bid.1 as u64)
            .sum()
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let game = input.parse::<Game>()?;
    let part1 = game.score();
    println!("part 1: {}", part1);
    let game2 = input.replace('J', "O").parse::<Game>()?;
    let part2 = game2.score();
    println!("part 2: {}", part2);
    Ok(())
}

use clap::{Parser, ValueEnum};
use std::io::BufRead;

#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_enum)]
    mode: Mode,
}

const DIGITS: &[&[u8]] = &[
    b"one", b"two", b"three", b"four", b"five", b"six", b"seven", b"eight", b"nine",
];

enum Direction {
    Forward,
    Backward,
}

enum DigitResult {
    Found(u8),
    AdvanceBy(usize),
    Empty,
}

// Tricky! Consider something like "oneight"; read forward, it's one, and read
// backward, it's eight.
fn find_next_digit(s: &[u8], direction: Direction) -> DigitResult {
    if s.is_empty() {
        return DigitResult::Empty;
    }
    let first = match direction {
        Direction::Forward => s.first().unwrap(),
        Direction::Backward => s.last().unwrap(),
    };
    if first.is_ascii_digit() {
        return DigitResult::Found(*first - 0x30);
    }
    for (i, prefix) in DIGITS.iter().enumerate() {
        if s.len() < prefix.len() {
            continue;
        }
        match direction {
            Direction::Forward => {
                if s.starts_with(prefix) {
                    return DigitResult::Found((i + 1) as u8);
                }
            }
            Direction::Backward => {
                if s.ends_with(prefix) {
                    return DigitResult::Found((i + 1) as u8);
                }
            }
        }
    }
    DigitResult::AdvanceBy(1)
}

fn find_digits_mode2(s: &str) -> (u8, u8) {
    let mut forward_slice = s.as_bytes();
    let c0 = loop {
        match find_next_digit(forward_slice, Direction::Forward) {
            DigitResult::Found(i) => break i,
            DigitResult::AdvanceBy(i) => forward_slice = &forward_slice[i..],
            DigitResult::Empty => {
                panic!("oops")
            }
        }
    };
    let mut rev_slice = s.as_bytes();
    let c1 = loop {
        match find_next_digit(rev_slice, Direction::Backward) {
            DigitResult::Found(i) => break i,
            DigitResult::AdvanceBy(i) => rev_slice = &rev_slice[..(rev_slice.len() - i)],
            DigitResult::Empty => {
                panic!("oops")
            }
        }
    };
    (c0, c1)
}

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut line_buf = String::new();
    let mut buf = [0u8; 2];
    let mut acc = 0i64;
    loop {
        let res = handle.read_line(&mut line_buf);
        match res {
            Ok(0) => break,
            Err(_) => break,
            _ => {}
        }
        if args.mode == Mode::Part1 {
            let mut digits = line_buf
                .as_bytes()
                .iter()
                .filter(|c| c.is_ascii_digit())
                .map(|d| *d - 0x30)
                .peekable();
            if let Some(digit) = digits.peek() {
                buf[0] = *digit;
            }
            if let Some(digit) = digits.next_back() {
                buf[1] = digit;
            }
        } else {
            let (c1, c2) = find_digits_mode2(&line_buf);
            buf[0] = c1;
            buf[1] = c2;
        }
        assert!(buf[0] != 0);
        assert!(buf[1] != 0);
        let num = (buf[0] * 10 + buf[1]) as i64;
        acc += num;
        line_buf.clear();
    }
    println!("{}", acc);
}

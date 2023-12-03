use clap::{Parser, ValueEnum};
use std::io::Read;

use aoclib::Point;

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

#[derive(Debug)]
struct Number {
    col_start: i64,
    col_end: i64,
    row: i64,
    value: u32,
}

impl Number {
    fn is_adjacent_to(&self, sym: &Symbol) -> bool {
        let y = sym.loc.y >= (self.row - 1) && sym.loc.y <= self.row + 1;
        let x = sym.loc.x >= (self.col_start - 1) && sym.loc.x <= self.col_end + 1;
        y && x
    }
}

#[derive(Debug)]
struct Symbol {
    c: char,
    loc: Point,
}

#[derive(Debug)]
struct Board {
    symbols: Vec<Symbol>,
    numbers: Vec<Number>,
}

impl Board {
    fn parse(s: &str) -> Self {
        let mut symbols = Vec::new();
        let mut numbers = Vec::new();
        s.lines().enumerate().for_each(|(row_num, row)| {
            let mut num_buf = String::new();
            let mut current_number: Option<Number> = None;
            let row_num = row_num as i64;
            row.chars().enumerate().for_each(|(col_num, chr)| {
                let col_num = col_num as i64;
                if chr.is_ascii_digit() {
                    num_buf.push(chr);
                    if let Some(ref mut num) = current_number {
                        num.col_end = col_num;
                    } else {
                        current_number = Some(Number {
                            col_start: col_num,
                            col_end: col_num,
                            row: row_num,
                            value: 0,
                        })
                    }
                } else {
                    if let Some(mut number) = current_number.take() {
                        number.value = num_buf.parse().unwrap();
                        numbers.push(number);
                        num_buf.clear();
                    }
                    if chr != '.' {
                        symbols.push(Symbol {
                            c: chr,
                            loc: Point::new(col_num, row_num),
                        })
                    }
                }
            });
            if let Some(mut number) = current_number.take() {
                number.value = num_buf.parse().unwrap();
                numbers.push(number);
            }
        });
        Board { symbols, numbers }
    }

    fn number_is_adjacent_to_symbol(&self, num: &Number) -> bool {
        self.symbols.iter().any(|sym| num.is_adjacent_to(sym))
    }

    fn gear_ratio(&self, sym: &Symbol) -> u64 {
        let adjacent = self
            .numbers
            .iter()
            .filter(|num| num.is_adjacent_to(sym))
            .map(|num| num.value)
            .collect::<Vec<u32>>();
        if adjacent.len() == 2 {
            (adjacent[0] as u64) * (adjacent[1] as u64)
        } else {
            0
        }
    }
}

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    let board = Board::parse(&input);

    if args.mode == Mode::Part1 {
        let val: u32 = board
            .numbers
            .iter()
            .filter(|number| board.number_is_adjacent_to_symbol(number))
            .map(|number| number.value)
            .sum();
        println!("{:?}", val);
    } else {
        let val: u64 = board
            .symbols
            .iter()
            .filter(|s| s.c == '*')
            .map(|s| board.gear_ratio(s))
            .sum();
        println!("{:?}", val);
    }
}

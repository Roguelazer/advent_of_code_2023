use aoclib::Point;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}

impl Direction {
    fn as_point(&self) -> Point {
        match self {
            Direction::Right => Point::new(1, 0),
            Direction::Left => Point::new(-1, 0),
            Direction::Down => Point::new(0, 1),
            Direction::Up => Point::new(0, -1),
        }
    }
}

#[derive(Debug)]
struct Command {
    direction: Direction,
    distance: usize,
}

impl Command {
    fn from_line_part1(s: &str) -> Self {
        use Direction::*;

        let mut fields = s.split_whitespace();
        let direction = match fields.next().unwrap() {
            "R" => Right,
            "L" => Left,
            "U" => Up,
            "D" => Down,
            other => panic!("unhandled direction {:?}", other),
        };
        let distance = fields.next().unwrap().parse().unwrap();
        Command {
            direction,
            distance,
        }
    }

    fn from_line_part2(s: &str) -> Self {
        let hex = s.split('#').last().unwrap();
        let distance = usize::from_str_radix(&hex[0..5], 16).unwrap();
        let direction = match &hex[5..6] {
            "0" => Direction::Right,
            "1" => Direction::Down,
            "2" => Direction::Left,
            "3" => Direction::Up,
            other => panic!("unhandled {}", other),
        };
        Command {
            direction,
            distance,
        }
    }
}

fn area(commands: &[Command]) -> f64 {
    let mut point = Point::new(0, 0);
    let mut total_area = 0f64;
    let mut perimeter = 0f64;
    for command in commands {
        let next = point + (command.direction.as_point() * command.distance as i64);
        let area = (point.x * next.y) as f64 - (point.y * next.x) as f64;
        perimeter += (next.x - point.x).abs() as f64 + (next.y - point.y).abs() as f64;
        total_area += area;
        point = next;
    }
    assert_eq!(point, Point::new(0, 0));
    total_area /= 2.0;
    total_area += perimeter / 2.0;
    total_area + 1.0
}

fn part1(input: &str) {
    let commands = input
        .lines()
        .map(Command::from_line_part1)
        .collect::<Vec<_>>();
    let val = area(&commands);
    println!("part 1: {}", val);
}

fn part2(input: &str) {
    let commands = input
        .lines()
        .map(Command::from_line_part2)
        .collect::<Vec<_>>();
    let val = area(&commands);
    println!("total area: {}", val);
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    part1(&input);
    part2(&input);
    Ok(())
}

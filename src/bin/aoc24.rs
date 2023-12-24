use aoclib::{Point, Vec3};
use ndarray::prelude::*;
use ndarray_linalg::Solve;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Hailstone {
    index: usize,
    position: Vec3<i64>,
    velocity: Vec3<i64>,
}

impl Hailstone {
    fn from_str(i: usize, s: &str) -> Self {
        let (pos, vel) = s.trim().split_once(" @ ").unwrap();
        let mut pos = pos.split(", ");
        let mut vel = vel.split(", ");
        let x = pos.next().unwrap().trim().parse().unwrap();
        let y = pos.next().unwrap().trim().parse().unwrap();
        let z = pos.next().unwrap().trim().parse().unwrap();
        let vx = vel.next().unwrap().trim().parse().unwrap();
        let vy = vel.next().unwrap().trim().parse().unwrap();
        let vz = vel.next().unwrap().trim().parse().unwrap();
        let position = Vec3::new(x, y, z);
        let velocity = Vec3::new(vx, vy, vz);
        Hailstone {
            index: i,
            position,
            velocity,
        }
    }

    fn intersection_2d(&self, other: &Hailstone) -> Option<Point<f64>> {
        if self.velocity == other.velocity {
            return None;
        }
        // given self.position = <a, b, _> and self.velocity = <q, w>
        // and other.position = <c, d> and other.velocity = <x, y>
        // we have a + qt = c + xs
        // and= b + wt = d + ys
        // solve for t, s
        // qt - xs = c - a
        // wc - ys = d - b

        let a = self.position.x as f64;
        let b = self.position.y as f64;
        let q = self.velocity.x as f64;
        let w = self.velocity.y as f64;
        let c = other.position.x as f64;
        let d = other.position.y as f64;
        let x = other.velocity.x as f64;
        let y = other.velocity.y as f64;

        let matx: Array2<f64> = array![[q, -1.0 * x], [w, -1.0 * y]];
        let vec: Array1<f64> = array![c - a, d - b];
        match matx.solve_into(vec) {
            Ok(solution) => {
                let t = solution[0];
                let s = solution[1];
                if t > 0.0 && s > 0.0 {
                    Some(Point::new(a + q * t, b + w * t))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

fn all_pairs<T: Clone>(slice: &[T]) -> impl Iterator<Item = (&T, &T)> {
    (0..slice.len()).flat_map(move |i| ((i + 1)..slice.len()).map(move |j| (&slice[i], &slice[j])))
}

fn part1(hailstones: &[Hailstone]) -> usize {
    let (min_x, max_x): (f64, f64) = if hailstones.len() == 5 {
        (7.0, 27.0)
    } else {
        (200_000_000_000_000.0, 400_000_000_000_000.0)
    };
    all_pairs(hailstones)
        .filter_map(|(lhs, rhs)| match lhs.intersection_2d(rhs) {
            Some(pt) if pt.x >= min_x && pt.y >= min_x && pt.x <= max_x && pt.y <= max_x => {
                Some(pt)
            }
            _ => None,
        })
        .count()
}

fn initial_velocity_range(min: i64, max: i64) -> BTreeSet<i64> {
    (min..=max).collect()
}

fn part2(hailstones: &[Hailstone]) -> i64 {
    // had to look up hints for this one; linear algebra was an 8am class 15 years ago
    // and unlike graph theory and general algorithms, has never ever come up professionally
    let x_velocity = all_pairs(hailstones)
        .filter(|(h1, h2)| h1.velocity.x == h2.velocity.x)
        .fold(initial_velocity_range(-500, 500), |mut vels, (h1, h2)| {
            let delta_x = h2.position.x - h1.position.x;
            vels.retain(|xvel| *xvel != h1.velocity.x && delta_x % (xvel - h1.velocity.x) == 0);
            vels
        });
    let y_velocity = all_pairs(hailstones)
        .filter(|(h1, h2)| h1.velocity.y == h2.velocity.y)
        .fold(initial_velocity_range(-500, 500), |mut vels, (h1, h2)| {
            let delta_y = h2.position.y - h1.position.y;
            vels.retain(|yvel| *yvel != h1.velocity.y && delta_y % (yvel - h1.velocity.y) == 0);
            vels
        });
    let z_velocity = all_pairs(hailstones)
        .filter(|(h1, h2)| h1.velocity.z == h2.velocity.z)
        .fold(initial_velocity_range(-500, 500), |mut vels, (h1, h2)| {
            let delta_z = h2.position.z - h1.position.z;
            vels.retain(|zvel| *zvel != h1.velocity.z && delta_z % (zvel - h1.velocity.z) == 0);
            vels
        });
    assert_eq!(x_velocity.len(), 1);
    assert_eq!(y_velocity.len(), 1);
    assert_eq!(z_velocity.len(), 1);
    let velocity = Vec3::new(
        *x_velocity.first().unwrap(),
        *y_velocity.first().unwrap(),
        *z_velocity.first().unwrap(),
    );

    // assume that the 2d intersection is sufficient to uniquely identify the thrown rock;
    // otherwise

    // H[0].p.x + H[0].v.x * t = R.x + V.x * t
    // H[0].p.y + H[0].v.y * t = R.y + V.y * t
    // H[1].p.x + H[1].v.x * s = R.x + V.x * s
    // H[1].p.y + H[1].v.y * s = R.y + V.y * s
    //
    // (H[0].v.x - V.x) * t + 0s - R.x + 0= -H[0].p.x
    // (H[0].v.y - V.y) * t + 0s + 0 - R.y = -H[0].p.y

    let matx: Array2<f64> = array![
        [
            (hailstones[0].velocity.x - velocity.x) as f64,
            0.0f64,
            -1.0f64,
            0.0f64
        ],
        [
            (hailstones[0].velocity.y - velocity.y) as f64,
            0.0,
            0.0,
            -1.0
        ],
        [
            0.0,
            (hailstones[1].velocity.x - velocity.x) as f64,
            -1.0,
            0.0
        ],
        [
            0.0,
            (hailstones[1].velocity.y - velocity.y) as f64,
            0.0,
            -1.0
        ],
    ];
    let matb: Array1<f64> = array![
        -1.0 * hailstones[0].position.x as f64,
        -1.0 * hailstones[0].position.y as f64,
        -1.0 * hailstones[1].position.x as f64,
        -1.0 * hailstones[1].position.y as f64
    ];

    let solv = matx.solve_into(matb).unwrap();
    let t = solv[0];
    let px = solv[2].round();
    let py = solv[3].round();

    // given the time, x, and y coordinate, the z coordinate is pretty straightforward

    let pz = hailstones[0].position.z as f64 + (hailstones[0].velocity.z - velocity.z) as f64 * t;

    (px + py + pz) as i64
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let hailstones = input
        .lines()
        .enumerate()
        .map(|(i, l)| Hailstone::from_str(i, l))
        .collect::<Vec<_>>();
    println!("part 1: {}", part1(&hailstones));
    println!("part 2: {}", part2(&hailstones));
    Ok(())
}

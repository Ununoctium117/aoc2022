use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::{Add, Mul},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Vec3(i64, i64, i64);
impl Vec3 {
    fn in_range(&self, min: i64, max: i64) -> bool {
        (min <= self.0 && self.0 < max)
            && (min <= self.1 && self.1 < max)
            && (min <= self.2 && self.2 < max)
    }
}
impl<B: Borrow<Vec3>> Add<B> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: B) -> Self::Output {
        let rhs = rhs.borrow();
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}
impl<B: Borrow<Vec3>> Add<B> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: B) -> Self::Output {
        let rhs = rhs.borrow();
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}
impl Mul<i64> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: i64) -> Self::Output {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

// Note that the normal vector doesn't participate in equality or hashing
#[derive(Debug)]
struct Face {
    coords: [Vec3; 4],
}
impl Face {
    fn new(mut coords: [Vec3; 4]) -> Self {
        coords.sort_unstable();
        Face { coords }
    }
}
impl<B: Borrow<Vec3>> Add<B> for &Face {
    type Output = Face;

    fn add(self, rhs: B) -> Self::Output {
        let rhs = rhs.borrow();
        Face {
            coords: [
                &self.coords[0] + rhs,
                &self.coords[1] + rhs,
                &self.coords[2] + rhs,
                &self.coords[3] + rhs,
            ],
        }
    }
}
impl PartialEq for Face {
    fn eq(&self, other: &Self) -> bool {
        self.coords == other.coords
    }
}
impl Eq for Face {}
impl Hash for Face {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.coords.hash(state);
    }
}
impl PartialOrd for Face {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.coords.partial_cmp(&other.coords)
    }
}
impl Ord for Face {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct Cube {
    faces: [Face; 6],
}
impl Cube {
    const NORMALS: [Vec3; 6] = [
        Vec3(0, -1, 0),
        Vec3(0, 1, 0),
        Vec3(0, 0, 1),
        Vec3(0, 0, -1),
        Vec3(-1, 0, 0),
        Vec3(1, 0, 0),
    ];

    fn new(coords: Vec3) -> Self {
        let faces = Self::cube_faces(&coords);
        Self { faces }
    }

    fn cube_faces(&Vec3(x, y, z): &Vec3) -> [Face; 6] {
        [
            Face::new([
                Vec3(x, y, z),
                Vec3(x + 1, y, z),
                Vec3(x + 1, y, z + 1),
                Vec3(x, y, z + 1),
            ]), // bottom
            Face::new([
                Vec3(x, y + 1, z),
                Vec3(x + 1, y + 1, z),
                Vec3(x + 1, y + 1, z + 1),
                Vec3(x, y + 1, z + 1),
            ]), // top
            Face::new([
                Vec3(x, y, z + 1),
                Vec3(x + 1, y, z + 1),
                Vec3(x + 1, y + 1, z + 1),
                Vec3(x, y + 1, z + 1),
            ]), // back
            Face::new([
                Vec3(x, y, z),
                Vec3(x + 1, y, z),
                Vec3(x + 1, y + 1, z),
                Vec3(x, y + 1, z),
            ]), // front
            Face::new([
                Vec3(x, y, z),
                Vec3(x, y, z + 1),
                Vec3(x, y + 1, z + 1),
                Vec3(x, y + 1, z),
            ]), // left
            Face::new([
                Vec3(x + 1, y, z),
                Vec3(x + 1, y, z + 1),
                Vec3(x + 1, y + 1, z + 1),
                Vec3(x + 1, y + 1, z),
            ]), // right
        ]
    }
}

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let cubes = input
        .lines()
        .map(|line| {
            let mut nums = line.split(",").map(|num| num.parse().unwrap());
            let coords = Vec3(
                nums.next().unwrap(),
                nums.next().unwrap(),
                nums.next().unwrap(),
            );
            (coords.clone(), Cube::new(coords))
        })
        .collect::<HashMap<_, _>>();

    let unique_faces = cubes
        .values()
        .map(|cube| cube.faces.iter())
        .flatten()
        .collect::<HashSet<_>>();

    // Part 1
    {
        let total_faces = cubes.len() * 6;
        let duplicated_faces = total_faces - unique_faces.len();
        let surface_area = total_faces - (2 * duplicated_faces);

        println!("part 1 solution: {surface_area}");
    }

    // Part 2
    {
        let mut exposed_surfaces = 0;

        // Do a search to find all external cubes
        let mut queue = Vec::new();
        let mut checked_coords = HashSet::new();
        queue.push(Vec3(24, 24, 24));

        while let Some(coordinate) = queue.pop() {
            if !checked_coords.insert(coordinate.clone()) {
                continue;
            }

            let mut adjacent_faces = 0;
            for direction in Cube::NORMALS {
                let new_coordinate = &coordinate + direction;

                if !new_coordinate.in_range(-1, 25) {
                    continue;
                }

                if cubes.contains_key(&new_coordinate) {
                    adjacent_faces += 1;
                } else {
                    queue.push(new_coordinate);
                }
            }

            exposed_surfaces += adjacent_faces;
            // if adjacent_faces > 0 {
            //     println!("found {adjacent_faces} faces adjacent to {coordinate:?} (total found so far: {exposed_surfaces})");
            // }
        }

        println!("part 2 solution: {exposed_surfaces}");
    }
}

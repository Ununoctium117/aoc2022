use std::{
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Debug, Default, Clone)]
struct TypedItems<T> {
    items: [T; 4],
}
impl TypedItems<u32> {
    // returns remaining resources
    fn subtract_cost(&self, cost: &TypedItems<u32>) -> Option<TypedItems<u32>> {
        Some(TypedItems {
            items: [
                self.items[0].checked_sub(cost.items[0])?,
                self.items[1].checked_sub(cost.items[1])?,
                self.items[2].checked_sub(cost.items[2])?,
                self.items[3].checked_sub(cost.items[3])?,
            ],
        })
    }
}
impl<T> Index<ResourceType> for TypedItems<T> {
    type Output = T;

    fn index(&self, index: ResourceType) -> &Self::Output {
        &self.items[index as usize]
    }
}
impl<T> IndexMut<ResourceType> for TypedItems<T> {
    fn index_mut(&mut self, index: ResourceType) -> &mut Self::Output {
        &mut self.items[index as usize]
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
enum ResourceType {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}
impl ResourceType {
    const RESOURCES: [ResourceType; 4] = [
        ResourceType::Ore,
        ResourceType::Clay,
        ResourceType::Obsidian,
        ResourceType::Geode,
    ];
}

#[derive(Debug)]
struct Blueprint {
    robot_costs: TypedItems<TypedItems<u32>>,
    most_expensive_costs: TypedItems<u32>,
}
impl Blueprint {
    fn max_geodes(&self, time_limit: u32) -> u32 {
        #[derive(Debug, Clone)]
        struct State {
            resources: TypedItems<u32>,
            robots_producing: TypedItems<u32>,
            time_left: u32,
            // build_sequence: Vec<ResourceType>,
        }
        impl State {
            fn run_robots(&self, times: u32) -> Option<Self> {
                let mut result = self.clone();
                result.time_left = self.time_left.checked_sub(times)?;
                for material in ResourceType::RESOURCES {
                    result.resources[material] += self.robots_producing[material] * times;
                }
                Some(result)
            }

            fn time_till_buildable(&self, costs: &TypedItems<u32>) -> Option<u32> {
                let mut time = 0;
                for material in ResourceType::RESOURCES {
                    let missing = costs[material].saturating_sub(self.resources[material]);
                    if missing > 0 {
                        if self.robots_producing[material] == 0 {
                            return None;
                        }
                        time = time.max(
                            (missing + self.robots_producing[material] - 1)
                                .checked_div(self.robots_producing[material])?,
                        );
                    }
                }
                Some(time)
            }
        }

        let mut queue = Vec::new();
        queue.push(State {
            resources: TypedItems::default(),
            robots_producing: {
                let mut robots = TypedItems::default();
                robots[ResourceType::Ore] = 1;
                robots
            },
            time_left: time_limit,
            // build_sequence: Vec::with_capacity(20),
        });

        let mut max_geodes = 0;
        while let Some(state) = queue.pop() {
            max_geodes = max_geodes.max(state.resources[ResourceType::Geode]);

            if state.time_left > 0 {
                let mut any_robot_built = false;
                for material in ResourceType::RESOURCES {
                    // if we already have enough, don't bother producing more
                    if state.robots_producing[material] > self.most_expensive_costs[material] {
                        continue;
                    }

                    let cost = &self.robot_costs[material];
                    if let Some(time_till_buildable) = state.time_till_buildable(cost) {
                        if let Some(mut new_state) = state.run_robots(time_till_buildable + 1) {
                            new_state.resources = new_state.resources.subtract_cost(cost).unwrap();
                            new_state.robots_producing[material] += 1;
                            // new_state.build_sequence.push(material);
                            // println!("{:?}", new_state.build_sequence);
                            queue.push(new_state);
                            any_robot_built = true;
                        }
                    }
                }

                // If we can't build any more robots before time runs out, just simulate doing nothing.
                if !any_robot_built {
                    if let Some(new_state) = state.run_robots(1) {
                        queue.push(new_state);
                    }
                }
            }
        }

        max_geodes
    }
}
impl FromStr for Blueprint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_ascii_whitespace();
        let ore_robot_cost = TypedItems {
            items: [words.nth(6).unwrap().parse().unwrap(), 0, 0, 0],
        };
        let clay_robot_cost = TypedItems {
            items: [words.nth(5).unwrap().parse().unwrap(), 0, 0, 0],
        };
        let obsidian_robot_cost = TypedItems {
            items: [
                words.nth(5).unwrap().parse().unwrap(),
                words.nth(2).unwrap().parse().unwrap(),
                0,
                0,
            ],
        };
        let geode_robot_cost = TypedItems {
            items: [
                words.nth(5).unwrap().parse().unwrap(),
                0,
                words.nth(2).unwrap().parse().unwrap(),
                0,
            ],
        };

        let costs = [
            ore_robot_cost,
            clay_robot_cost,
            obsidian_robot_cost,
            geode_robot_cost,
        ];
        let mut most_expensive_costs = TypedItems {
            items: ResourceType::RESOURCES
                .map(|material| costs.iter().map(|cost| cost[material]).max().unwrap()),
        };
        most_expensive_costs[ResourceType::Geode] = u32::MAX;

        Ok(Self {
            robot_costs: TypedItems { items: costs },
            most_expensive_costs,
        })
    }
}

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let blueprints = input
        .lines()
        .map(|line| line.parse().unwrap())
        .collect::<Vec<Blueprint>>();

    let mut sum = 0;
    for (idx, blueprint) in blueprints.iter().enumerate() {
        let max_geodes = blueprint.max_geodes(24);
        sum += ((idx + 1) as u32) * max_geodes;
        println!("blueprint ID {} produced {} geodes", idx + 1, max_geodes);
    }

    println!("part 1 answer: {sum}");

    let mut product = 1;
    for (idx, blueprint) in blueprints.iter().take(3).enumerate() {
        let max_geodes = blueprint.max_geodes(32);
        product *= max_geodes;
        println!("blueprint ID {} produced {} geodes", idx + 1, max_geodes);
    }

    println!("part 2 answer: {product}");
}

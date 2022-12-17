use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use itertools::Itertools;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct ValveId([u8; 2]);
impl Debug for ValveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = std::str::from_utf8(&self.0[..]).unwrap();
        f.write_fmt(format_args!("Valve {}", text))
    }
}

#[derive(Debug, Clone)]
struct Valve {
    flow_rate: u64,
    adjacent_valves: Vec<ValveId>,
}
impl Valve {
    fn parse(str: &str) -> (ValveId, Self) {
        let mut words = str.split_ascii_whitespace();
        let valve_id = ValveId(words.nth(1).unwrap().as_bytes().try_into().unwrap());
        let flow_rate = words
            .nth(2)
            .unwrap()
            .split("=")
            .nth(1)
            .unwrap()
            .strip_suffix(";")
            .unwrap()
            .parse()
            .unwrap();
        let adjacent_valves = words
            .skip(4)
            .map(|word| ValveId(word.as_bytes()[..2].try_into().unwrap()))
            .collect();

        (
            valve_id,
            Valve {
                flow_rate,
                adjacent_valves,
            },
        )
    }
}

// Floyd-Warshall Algorithm
fn all_pairs_shortest_paths(valves: &HashMap<ValveId, Valve>) -> HashMap<(ValveId, ValveId), u64> {
    let mut result = HashMap::new();
    let keys = valves.keys().copied().collect::<Vec<_>>();

    for v1 in &keys {
        result.insert((*v1, *v1), 0);
        for v2 in &keys {
            result.insert((*v1, *v2), u64::MAX);
        }

        let valve = valves.get(v1).unwrap();
        for v2 in &valve.adjacent_valves {
            result.insert((*v1, *v2), 1);
        }
    }

    for k in &keys {
        for i in &keys {
            for j in &keys {
                let new_dist = result[&(*i, *k)].saturating_add(result[&(*k, *j)]);
                result.entry((*i, *j)).and_modify(|old_dist| {
                    if *old_dist > new_dist {
                        *old_dist = new_dist;
                    }
                });
            }
        }
    }

    result
}

#[derive(Debug, Clone, Copy)]
struct TravellingState {
    destination: ValveId,
    reactivates_on: u64, // started - path_len - 1 (time taken to open valve)
}

#[derive(Debug, Clone)]
struct AgentStates<const AGENTS: usize> {
    // Agents who just opened a valuable valve
    active: [Option<ValveId>; AGENTS],
    // Agents who are en route to a valuable valve
    inactive: [Option<TravellingState>; AGENTS],
}
impl<const AGENTS: usize> AgentStates<AGENTS> {
    fn active_count(&self) -> usize {
        self.active.iter().filter(|t| t.is_some()).count()
    }

    fn active_agents(&self) -> impl Iterator<Item = (usize, ValveId)> + '_ {
        self.active
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(idx, t)| t.map(|t| (idx, t)))
    }

    fn start_travelling(
        &mut self,
        agent_idx: usize,
        current_time: u64,
        destination: ValveId,
        path_length: u64,
    ) {
        self.active[agent_idx] = None;
        self.inactive[agent_idx] = Some(TravellingState {
            destination,
            reactivates_on: current_time - path_length - 1, // -1 for the time to open the valve
        });
    }

    // Returns (amount of time waited, amount of pressure released)
    fn wait_until_any_active(
        &mut self,
        current_time_remaining: u64,
        valves: &HashMap<ValveId, Valve>,
        history: &mut Vec<String>,
    ) -> (u64, u64) {
        let time_to_wait = self
            .inactive
            .iter()
            .filter_map(|t| t.map(|t| current_time_remaining.saturating_sub(t.reactivates_on)))
            .min()
            .unwrap();

        let new_time_remaining = current_time_remaining - time_to_wait;

        // Mark any agents that got there on that timestamp active
        let mut pressure_released = 0;
        for idx in 0..AGENTS {
            if let Some(travel_state) = self.inactive[idx] {
                if new_time_remaining.saturating_sub(travel_state.reactivates_on) == 0 {
                    history.push(format!(
                        "Agent {} opens {:?} with {} minutes left",
                        idx, travel_state.destination, new_time_remaining
                    ));
                    pressure_released += new_time_remaining
                        * valves.get(&travel_state.destination).unwrap().flow_rate;

                    self.active[idx] = Some(travel_state.destination);
                    self.inactive[idx] = None;
                }
            }
        }

        (time_to_wait, pressure_released)
    }
}
impl<const AGENTS: usize> Default for AgentStates<AGENTS> {
    fn default() -> Self {
        Self {
            active: [Some(ValveId([b'A', b'A'])); AGENTS],
            inactive: [None; AGENTS],
        }
    }
}

#[derive(Debug)]
struct VolcanoState<const AGENTS: usize> {
    total_pressure_released: u64,
    remaining_time: u64,
    agents: AgentStates<AGENTS>,
    remaining_valuable_unqueued_closed_valves: HashSet<ValveId>,
    history: Vec<String>,
}

fn do_the_solve<const AGENTS: usize>(
    valves: &HashMap<ValveId, Valve>,
    shortest_paths: &HashMap<(ValveId, ValveId), u64>,
    time_allowed: u64,
) {
    let mut queue = Vec::new();
    queue.push(VolcanoState::<AGENTS> {
        total_pressure_released: 0,
        remaining_time: time_allowed,
        agents: Default::default(),
        remaining_valuable_unqueued_closed_valves: valves
            .iter()
            .filter_map(|(id, valve)| {
                if valve.flow_rate == 0 {
                    None
                } else {
                    Some(*id)
                }
            })
            .collect(),
        history: Vec::new(),
    });

    let mut total_pressure_released = 0;
    let mut history = Vec::new();

    while let Some(state) = queue.pop() {
        if state.total_pressure_released > total_pressure_released {
            total_pressure_released = state.total_pressure_released;
            history = state.history.clone();
        }

        let active_agents = state.agents.active_count();

        for next_valve_ids in state
            .remaining_valuable_unqueued_closed_valves
            .iter()
            .combinations(active_agents)
        {
            // TODO: skip symmetric permutatations (ie if all the active agents are on the same valve, no point
            // queueing up (0->A, 1->B) AND (0->B, 1->A).)

            'permutations: for agent_permutation in
                next_valve_ids.into_iter().permutations(active_agents)
            {
                // If any agent would take too long to get to its destination, this is an invalid combination
                let agent_destinations = state.agents.active_agents().zip(agent_permutation);

                let mut new_agent_states = state.agents.clone();
                let mut new_closed_valves = state.remaining_valuable_unqueued_closed_valves.clone();
                for ((agent_idx, agent_cur_valve), dest_valve) in agent_destinations {
                    let path_len = shortest_paths[&(agent_cur_valve, *dest_valve)];

                    // If any agent would take too long to reach their destination, try a different permutation instead.
                    if path_len >= state.remaining_time {
                        continue 'permutations;
                    }

                    new_agent_states.start_travelling(
                        agent_idx,
                        state.remaining_time,
                        *dest_valve,
                        path_len,
                    );

                    new_closed_valves.remove(dest_valve);
                }

                let mut new_history = state.history.clone();
                let (time_to_wait, pressure_released) = new_agent_states.wait_until_any_active(
                    state.remaining_time,
                    valves,
                    &mut new_history,
                );
                let remaining_time = state.remaining_time - time_to_wait;

                queue.push(VolcanoState {
                    total_pressure_released: state.total_pressure_released + pressure_released,
                    remaining_time,
                    agents: new_agent_states,
                    remaining_valuable_unqueued_closed_valves: new_closed_valves,
                    history: new_history,
                });
            }
        }
    }

    for history in history {
        println!("{}", history);
    }
    println!("{}", total_pressure_released);
}

fn main() {
    let input = std::fs::read_to_string("input2.txt").unwrap();
    let valves: HashMap<_, _> = input.lines().map(|line| Valve::parse(line)).collect();

    let shortest_paths = all_pairs_shortest_paths(&valves);
    // dbg!(&shortest_paths);

    // do_the_solve::<1>(&valves, &shortest_paths, 30);
    do_the_solve::<2>(&valves, &shortest_paths, 26);
}

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    hash::Hash,
};

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct ValveId(usize);
impl ValveId {
    const MAX: usize = 702;

    fn from_chars(chars: &[u8]) -> Self {
        ValveId((((chars[0] - b'A') as u16) * 26 + (chars[1] - b'A') as u16).into())
    }
}
impl Debug for ValveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b0 = (self.0 / 26) as u8 + b'A';
        let b1 = (self.0 % 26) as u8 + b'A';
        let bytes = [b0, b1];
        let text = std::str::from_utf8(&bytes).unwrap();
        f.write_fmt(format_args!("Valve {}", text))
    }
}

const NULL_VALVE_ID: ValveId = ValveId(0xFFFF);

fn build_valve_mask_mapping(mut valves: Vec<ValveId>) -> Box<[u64; ValveId::MAX]> {
    let mut result = Box::new([0; ValveId::MAX]);

    valves.sort();
    for (idx, valve_id) in valves.iter().enumerate() {
        result[valve_id.0] = 1 << idx;
    }

    result
}

#[derive(Debug, Clone)]
struct Valve {
    flow_rate: u64,
    adjacent_valves: Vec<ValveId>,
}
impl Valve {
    fn parse(str: &str) -> (ValveId, Self) {
        let mut words = str.split_ascii_whitespace();
        let valve_id = ValveId::from_chars(words.nth(1).unwrap().as_bytes());
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
            .map(|word| ValveId::from_chars(&word.as_bytes()[..2]))
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
fn all_pairs_shortest_paths(valves: &HashMap<ValveId, Valve>) -> Vec<[u64; ValveId::MAX]> {
    let mut result = vec![[u64::MAX; ValveId::MAX]; ValveId::MAX];
    let keys = valves.keys().copied().collect::<Vec<_>>();

    for v1 in &keys {
        result[v1.0][v1.0] = 0;

        let valve = valves.get(v1).unwrap();
        for v2 in &valve.adjacent_valves {
            result[v1.0][v2.0] = 1;
        }
    }

    for k in &keys {
        for i in &keys {
            for j in &keys {
                let new_dist = result[i.0][k.0].saturating_add(result[k.0][j.0]);
                if result[i.0][j.0] > new_dist {
                    result[i.0][j.0] = new_dist;
                }
            }
        }
    }

    result
}

fn combinations<const SIZE: usize>(
    ids: &[ValveId],
    active_agents: usize,
    dest: &mut Vec<[ValveId; SIZE]>,
) {
    dest.clear();

    fn helper<const SIZE: usize>(
        ids: &[ValveId],
        mut tmp: [ValveId; SIZE],
        max_non_null: usize,
        idx: usize,
        i: usize,
        dest: &mut Vec<[ValveId; SIZE]>,
    ) {
        if idx == max_non_null {
            dest.push(tmp.clone());
        } else if i < ids.len() {
            tmp[idx] = ids[i];
            helper(ids, tmp, max_non_null, idx + 1, i + 1, dest);
            helper(ids, tmp, max_non_null, idx, i + 1, dest);
        }
    }

    helper(ids, [NULL_VALVE_ID; SIZE], active_agents, 0, 0, dest);
}

fn permutations<const SIZE: usize>(mut ids: [ValveId; SIZE], dest: &mut Vec<[ValveId; SIZE]>) {
    dest.clear();
    let mut stack = [0; SIZE];

    dest.push(ids.clone());

    let mut i = 1;
    while i < SIZE {
        if stack[i] < i {
            if i % 2 == 0 {
                ids.swap(0, i);
            } else {
                ids.swap(stack[i], i);
            }

            dest.push(ids.clone());

            stack[i] += 1;
            i = 1;
        } else {
            stack[i] = 0;
            i += 1;
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AgentState {
    Travelling {
        destination: ValveId,
        reactivates_on: u64, // started - path_len - 1 (time taken to open valve)
    },
    Active(ValveId),
}

#[derive(Debug, Clone)]
struct AgentStates<const AGENTS: usize> {
    agents: [AgentState; AGENTS],
}
impl<const AGENTS: usize> AgentStates<AGENTS> {
    fn active_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|t| matches!(t, AgentState::Active(_)))
            .count()
    }

    fn active_agents(&self) -> impl Iterator<Item = (usize, ValveId)> + '_ {
        self.agents
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(idx, t)| {
                if let AgentState::Active(valve_id) = t {
                    Some((idx, valve_id))
                } else {
                    None
                }
            })
    }

    fn inactive_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|t| matches!(t, AgentState::Travelling { .. }))
            .count()
    }

    fn start_travelling(
        &mut self,
        agent_idx: usize,
        current_time: u64,
        destination: ValveId,
        path_length: u64,
    ) {
        self.agents[agent_idx] = AgentState::Travelling {
            destination,
            reactivates_on: current_time - path_length - 1, // -1 for the time to open the valve
        };
    }

    // Returns (amount of time waited, amount of pressure released)
    fn wait_until_any_active(
        &mut self,
        current_time_remaining: u64,
        valves: &HashMap<ValveId, Valve>,
        // history: &mut Vec<String>,
    ) -> (u64, u64) {
        let time_to_wait = self
            .agents
            .iter()
            .filter_map(|t| {
                if let AgentState::Travelling { reactivates_on, .. } = t {
                    Some(current_time_remaining.saturating_sub(*reactivates_on))
                } else {
                    None
                }
            })
            .min()
            .unwrap();

        let new_time_remaining = current_time_remaining - time_to_wait;

        // Mark any agents that got there on that timestamp active
        let mut pressure_released = 0;
        for idx in 0..AGENTS {
            if let AgentState::Travelling {
                destination,
                reactivates_on,
            } = self.agents[idx]
            {
                if new_time_remaining.saturating_sub(reactivates_on) == 0 {
                    // history.push(format!(
                    //     "Agent {} opens {:?} with {} minutes left",
                    //     idx, destination, new_time_remaining
                    // ));
                    pressure_released +=
                        new_time_remaining * valves.get(&destination).unwrap().flow_rate;

                    self.agents[idx] = AgentState::Active(destination);
                }
            }
        }

        (time_to_wait, pressure_released)
    }
}
impl<const AGENTS: usize> Default for AgentStates<AGENTS> {
    fn default() -> Self {
        Self {
            agents: [AgentState::Active(ValveId::from_chars(&[b'A', b'A'])); AGENTS],
        }
    }
}

#[derive(Debug)]
struct VolcanoState<const AGENTS: usize> {
    total_pressure_released: u64,
    remaining_time: u64,
    agents: AgentStates<AGENTS>,
    remaining_valuable_unqueued_closed_valves: u64,
    // history: Vec<String>,
}

fn do_the_solve<const AGENTS: usize>(
    valves: &HashMap<ValveId, Valve>,
    shortest_paths: &Vec<[u64; ValveId::MAX]>,
    masks: &Box<[u64; ValveId::MAX]>,
    time_allowed: u64,
) {
    let mut queue = VecDeque::with_capacity(1_000);
    queue.push_back(VolcanoState::<AGENTS> {
        total_pressure_released: 0,
        remaining_time: time_allowed,
        agents: Default::default(),
        remaining_valuable_unqueued_closed_valves: {
            let mut result = 0;
            for (valve_id, valve) in valves.iter() {
                if valve.flow_rate != 0 {
                    result |= masks[valve_id.0];
                }
            }
            result
        },
        // history: Vec::new(),
    });

    let mut combination_buffer = Vec::with_capacity(100);
    let mut permutation_buffer = Vec::with_capacity(100);
    let mut valve_id_buffer = Vec::with_capacity(100);

    let mut total_pressure_released = 0;
    // let mut history = Vec::new();

    while let Some(state) = queue.pop_back() {
        if state.total_pressure_released > total_pressure_released {
            total_pressure_released = state.total_pressure_released;
            // history = state.history.clone();
        }

        let active_agents = state.agents.active_count();

        // Edge case: All valves are assigned, but some agents are still travelling to their assigned valve
        if state.remaining_valuable_unqueued_closed_valves == 0
            && state.agents.inactive_count() > 0
        {
            let mut new_agent_states = state.agents.clone();
            // let mut new_history = state.history.clone();
            let (time_to_wait, pressure_released) = new_agent_states.wait_until_any_active(
                state.remaining_time,
                valves,
                // &mut new_history,
            );
            let remaining_time = state.remaining_time - time_to_wait;

            queue.push_back(VolcanoState {
                total_pressure_released: state.total_pressure_released + pressure_released,
                remaining_time,
                agents: new_agent_states,
                remaining_valuable_unqueued_closed_valves: state
                    .remaining_valuable_unqueued_closed_valves
                    .clone(),
                // history: new_history,
            });
        }

        valve_id_buffer.clear();
        for valve_id in valves.keys() {
            if state.remaining_valuable_unqueued_closed_valves & masks[valve_id.0] != 0 {
                valve_id_buffer.push(*valve_id);
            }
        }
        combinations::<AGENTS>(&valve_id_buffer[..], active_agents, &mut combination_buffer);

        for next_valve_ids in &combination_buffer {
            permutations::<AGENTS>(next_valve_ids.clone(), &mut permutation_buffer);
            'permutations: for agent_permutation in &permutation_buffer {
                let agent_destinations = state.agents.active_agents().zip(agent_permutation);

                let mut new_agent_states = state.agents.clone();
                let mut new_closed_valves = state.remaining_valuable_unqueued_closed_valves.clone();
                for ((agent_idx, agent_cur_valve), dest_valve) in agent_destinations {
                    // If any agent was assigned a null ID, try a different permutation. This can happen if there are
                    // less active agents than agents; in this case, the combination assigns a number of null destinations
                    // equal to the number of inactive agents.
                    if *dest_valve == NULL_VALVE_ID {
                        continue 'permutations;
                    }

                    let path_len = shortest_paths[agent_cur_valve.0][dest_valve.0];

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

                    new_closed_valves &= !masks[dest_valve.0];
                }

                // let mut new_history = state.history.clone();
                let (time_to_wait, pressure_released) = new_agent_states.wait_until_any_active(
                    state.remaining_time,
                    valves,
                    // &mut new_history,
                );
                let remaining_time = state.remaining_time - time_to_wait;

                queue.push_back(VolcanoState {
                    total_pressure_released: state.total_pressure_released + pressure_released,
                    remaining_time,
                    agents: new_agent_states,
                    remaining_valuable_unqueued_closed_valves: new_closed_valves,
                    // history: new_history,
                });
            }
        }
    }

    // for history in history {
    //     println!("{}", history);
    // }
    println!("{}", total_pressure_released);
}

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let valves: HashMap<_, _> = input.lines().map(|line| Valve::parse(line)).collect();
    let masks = build_valve_mask_mapping(valves.keys().copied().collect());

    let shortest_paths = all_pairs_shortest_paths(&valves);
    // dbg!(&shortest_paths);

    do_the_solve::<1>(&valves, &shortest_paths, &masks, 30);
    do_the_solve::<2>(&valves, &shortest_paths, &masks, 26);
}

use std::collections::{HashSet, VecDeque};

use super::sasplus::{SASPlus, State};

impl SASPlus {
    pub fn bfs_shortest_length(&self, start_state: State) -> Option<usize> {
        if self.is_goal(&start_state) {
            return Some(0);
        }

        let mut visited: HashSet<Vec<usize>> = HashSet::new();
        visited.insert(start_state.values.clone());

        let mut queue: VecDeque<(State, usize)> = VecDeque::new();
        queue.push_back((start_state, 0));

        while let Some((state, depth)) = queue.pop_front() {
            for op in &self.operators {
                if op.is_applicable(&state) {
                    let new_state = op.apply(&state);
                    if self.is_goal(&new_state) {
                        return Some(depth + 1);
                    }
                    if visited.insert(new_state.values.clone()) {
                        queue.push_back((new_state, depth + 1));
                    }
                }
            }
        }

        None
    }
}

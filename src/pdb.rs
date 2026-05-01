use std::collections::BinaryHeap;
use std::cmp::Reverse;

use super::sasplus::SASPlus;

/// An operator projected onto the pattern variables.
struct AbstractOp {
    /// (pattern_index, required_value) from prevail + effect pre-values on pattern vars.
    preconditions: Vec<(usize, usize)>,
    /// For each abstract effect: (pattern_index, pre_value, post_value, conditions).
    #[allow(clippy::type_complexity)]
    effects: Vec<(usize, i32, usize, Vec<(usize, usize)>)>,
    cost: usize,
}

/// Result of a PDB construction.
pub struct PDBResult {
    pub pattern: Vec<usize>,
    pub domain_sizes: Vec<usize>,
    pub multipliers: Vec<usize>,
    pub total_states: usize,
    /// goal distance for each abstract state (usize::MAX = unreachable from goal).
    pub distance: Vec<usize>,
    /// whether each abstract state is forward-reachable from the initial state.
    pub reachable: Vec<bool>,
}

impl PDBResult {
    /// Decode a hash back to the tuple of variable values.
    pub fn decode(&self, hash: usize) -> Vec<usize> {
        let mut vals = Vec::with_capacity(self.pattern.len());
        let mut h = hash;
        for &d in &self.domain_sizes {
            vals.push(h % d);
            h /= d;
        }
        vals
    }

    pub fn hash(&self, vals: &[usize]) -> usize {
        vals.iter()
            .zip(self.multipliers.iter())
            .map(|(&v, &m)| v * m)
            .sum()
    }
}

impl SASPlus {
    /// Build a pattern database for the given pattern variables.
    /// `pattern` must be sorted in ascending order.
    pub fn build_pdb(&self, state_values: &[usize], pattern: &[usize]) -> PDBResult {
        let s = pattern.len();

        // Map global variable index -> pattern index (None if not in pattern).
        let n = self.variables.len();
        let mut var_to_pidx = vec![None; n];
        for (pidx, &v) in pattern.iter().enumerate() {
            var_to_pidx[v] = Some(pidx);
        }

        let domain_sizes: Vec<usize> = pattern
            .iter()
            .map(|&v| self.variables[v].values.len())
            .collect();

        let mut multipliers = vec![1usize; s];
        for i in 1..s {
            multipliers[i] = multipliers[i - 1] * domain_sizes[i - 1];
        }
        let total_states: usize = domain_sizes.iter().product();

        // Project operators.
        let abstract_ops = self.project_operators(pattern, &var_to_pidx);

        // Initial abstract state.
        let init_abs: Vec<usize> = pattern.iter().map(|&v| state_values[v]).collect();
        let init_hash: usize = init_abs
            .iter()
            .zip(multipliers.iter())
            .map(|(&v, &m)| v * m)
            .sum();

        // Forward BFS for reachability.
        let reachable =
            Self::forward_bfs(init_hash, total_states, &domain_sizes, &multipliers, &abstract_ops);

        // Abstract goal states.
        let mut goal_constraints: Vec<(usize, usize)> = Vec::new();
        for &(var, val) in &self.goal {
            if let Some(pidx) = var_to_pidx[var] {
                goal_constraints.push((pidx, val));
            }
        }

        // Backward Dijkstra for goal distances.
        let distance = Self::backward_dijkstra(
            total_states,
            &domain_sizes,
            &multipliers,
            &abstract_ops,
            &goal_constraints,
        );

        PDBResult {
            pattern: pattern.to_vec(),
            domain_sizes,
            multipliers,
            total_states,
            distance,
            reachable,
        }
    }

    fn project_operators(
        &self,
        _pattern: &[usize],
        var_to_pidx: &[Option<usize>],
    ) -> Vec<AbstractOp> {
        let mut ops = Vec::new();
        for op in &self.operators {
            let mut preconditions: Vec<(usize, usize)> = Vec::new();

            // Prevail conditions on pattern variables.
            for &(var, val) in &op.prevail {
                if let Some(pidx) = var_to_pidx[var] {
                    preconditions.push((pidx, val));
                }
            }

            let mut effects = Vec::new();
            for eff in &op.effects {
                if let Some(pidx) = var_to_pidx[eff.variable] {
                    let conds: Vec<(usize, usize)> = eff
                        .conditions
                        .iter()
                        .filter_map(|&(v, d)| var_to_pidx[v].map(|pi| (pi, d)))
                        .collect();
                    effects.push((pidx, eff.pre_value, eff.post_value, conds));
                }
            }

            if effects.is_empty() {
                continue;
            }

            // Pre-values of effects on pattern variables are also preconditions.
            for &(pidx, pre_val, _, _) in &effects {
                if pre_val >= 0 {
                    preconditions.push((pidx, pre_val as usize));
                }
            }

            ops.push(AbstractOp {
                preconditions,
                effects,
                cost: op.cost,
            });
        }
        ops
    }

    fn decode_state(hash: usize, domain_sizes: &[usize]) -> Vec<usize> {
        let mut vals = Vec::with_capacity(domain_sizes.len());
        let mut h = hash;
        for &d in domain_sizes {
            vals.push(h % d);
            h /= d;
        }
        vals
    }

    fn encode_state(vals: &[usize], multipliers: &[usize]) -> usize {
        vals.iter()
            .zip(multipliers.iter())
            .map(|(&v, &m)| v * m)
            .sum()
    }

    fn apply_abstract_forward(
        state: &[usize],
        op: &AbstractOp,
    ) -> Option<Vec<usize>> {
        for &(pidx, val) in &op.preconditions {
            if state[pidx] != val {
                return None;
            }
        }
        let mut succ = state.to_vec();
        for &(pidx, _pre, post, ref conds) in &op.effects {
            let conds_met = conds.iter().all(|&(pi, d)| state[pi] == d);
            if conds_met {
                succ[pidx] = post;
            }
        }
        Some(succ)
    }

    fn forward_bfs(
        init_hash: usize,
        total_states: usize,
        domain_sizes: &[usize],
        multipliers: &[usize],
        abstract_ops: &[AbstractOp],
    ) -> Vec<bool> {
        let mut reachable = vec![false; total_states];
        reachable[init_hash] = true;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(init_hash);

        while let Some(h) = queue.pop_front() {
            let state = Self::decode_state(h, domain_sizes);
            for op in abstract_ops {
                if let Some(succ) = Self::apply_abstract_forward(&state, op) {
                    let sh = Self::encode_state(&succ, multipliers);
                    if !reachable[sh] {
                        reachable[sh] = true;
                        queue.push_back(sh);
                    }
                }
            }
        }
        reachable
    }

    /// Backward Dijkstra from all abstract goal states.
    fn backward_dijkstra(
        total_states: usize,
        domain_sizes: &[usize],
        multipliers: &[usize],
        abstract_ops: &[AbstractOp],
        goal_constraints: &[(usize, usize)],
    ) -> Vec<usize> {
        let s = domain_sizes.len();

        // Build reverse edges: for each forward transition (src -> dst, cost),
        // store (src, cost) in rev_adj[dst].
        let mut rev_adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); total_states];

        for h in 0..total_states {
            let state = Self::decode_state(h, domain_sizes);
            for op in abstract_ops {
                if let Some(succ) = Self::apply_abstract_forward(&state, op) {
                    let sh = Self::encode_state(&succ, multipliers);
                    if sh != h {
                        rev_adj[sh].push((h, op.cost));
                    }
                }
            }
        }

        // Also add self-loops for operators that don't change the state
        // (not needed for Dijkstra -- a self-loop with positive cost never improves).

        // Enumerate goal states.
        let mut dist = vec![usize::MAX; total_states];
        let mut heap: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();

        // Generate all abstract goal states.
        let mut free_vars: Vec<usize> = Vec::new();
        let mut free_sizes: Vec<usize> = Vec::new();
        for (i, &ds) in domain_sizes.iter().enumerate() {
            if !goal_constraints.iter().any(|&(pi, _)| pi == i) {
                free_vars.push(i);
                free_sizes.push(ds);
            }
        }

        let num_free: usize = free_sizes.iter().product::<usize>().max(1);
        for combo in 0..num_free {
            let mut vals = vec![0usize; s];
            for &(pi, val) in goal_constraints {
                vals[pi] = val;
            }
            let mut c = combo;
            for (idx, &fi) in free_vars.iter().enumerate() {
                vals[fi] = c % free_sizes[idx];
                c /= free_sizes[idx];
            }
            let h = Self::encode_state(&vals, multipliers);
            if dist[h] == usize::MAX {
                dist[h] = 0;
                heap.push(Reverse((0, h)));
            }
        }

        // Dijkstra on reversed graph.
        while let Some(Reverse((d, u))) = heap.pop() {
            if d > dist[u] {
                continue;
            }
            for &(pred, cost) in &rev_adj[u] {
                let nd = d.saturating_add(cost);
                if nd < dist[pred] {
                    dist[pred] = nd;
                    heap.push(Reverse((nd, pred)));
                }
            }
        }

        dist
    }
}

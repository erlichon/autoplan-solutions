use super::sasplus::{SASPlus, State};

impl SASPlus {
    /// h^2 heuristic. Fixed-point regression over a
    /// symmetric table `cost[f1][f2]` of fact-pair reachability costs.
    /// Returns `None` if the goal is unreachable.
    pub fn h2(&self, state: &State) -> Option<usize> {
        let n = self.variables.len();

        let mut offset = vec![0usize; n + 1];
        for i in 0..n {
            offset[i + 1] = offset[i] + self.variables[i].values.len();
        }
        let num_facts = offset[n];
        let fid = |v: usize, d: usize| offset[v] + d;

        let mut fact_var = vec![0usize; num_facts];
        let mut fact_val = vec![0usize; num_facts];
        for v in 0..n {
            for d in 0..self.variables[v].values.len() {
                fact_var[offset[v] + d] = v;
                fact_val[offset[v] + d] = d;
            }
        }

        const INF: usize = usize::MAX;
        let mut cost = vec![vec![INF; num_facts]; num_facts];

        let init_facts: Vec<usize> = (0..n).map(|v| fid(v, state.values[v])).collect();
        for &f1 in &init_facts {
            for &f2 in &init_facts {
                cost[f1][f2] = 0;
            }
        }

        loop {
            let mut changed = false;

            for op in &self.operators {
                let mut assigned = vec![false; n];
                for e in &op.effects {
                    assigned[e.variable] = true;
                }

                // prevail + every effect's pre-value (same as h1/hff).
                let mut op_pre: Vec<usize> = Vec::new();
                for &(v, d) in &op.prevail {
                    op_pre.push(fid(v, d));
                }
                for e in &op.effects {
                    if e.pre_value >= 0 {
                        op_pre.push(fid(e.variable, e.pre_value as usize));
                    }
                }
                if !facts_consistent(&op_pre, &fact_var, &fact_val) {
                    continue;
                }

                // Both post-state facts come from effects of `op`.
                // i == j covers the singleton update cost[p][p].
                for (i, e1) in op.effects.iter().enumerate() {
                    for (j, e2) in op.effects.iter().enumerate().skip(i) {
                        if i != j && e1.variable == e2.variable {
                            continue;
                        }
                        let p1 = fid(e1.variable, e1.post_value);
                        let p2 = fid(e2.variable, e2.post_value);

                        let mut pre_set = op_pre.clone();
                        for &(v, d) in &e1.conditions {
                            pre_set.push(fid(v, d));
                        }
                        if i != j {
                            for &(v, d) in &e2.conditions {
                                pre_set.push(fid(v, d));
                            }
                        }
                        if !facts_consistent(&pre_set, &fact_var, &fact_val) {
                            continue;
                        }

                        let h_pre = h2_of_set(&pre_set, &cost);
                        if h_pre == INF {
                            continue;
                        }
                        let new_cost = op.cost.saturating_add(h_pre);
                        if new_cost < cost[p1][p2] {
                            cost[p1][p2] = new_cost;
                            cost[p2][p1] = new_cost;
                            changed = true;
                        }
                    }
                }

                // One fact from an effect; the other persists (its variable
                // is untouched by `op` and must already hold pre-state).
                for e in &op.effects {
                    let p1 = fid(e.variable, e.post_value);
                    let mut pre_base = op_pre.clone();
                    for &(v, d) in &e.conditions {
                        pre_base.push(fid(v, d));
                    }
                    if !facts_consistent(&pre_base, &fact_var, &fact_val) {
                        continue;
                    }

                    for v in 0..n {
                        if assigned[v] {
                            continue;
                        }
                        for d in 0..self.variables[v].values.len() {
                            let p2 = fid(v, d);
                            let mut pre_set = pre_base.clone();
                            pre_set.push(p2);
                            if !facts_consistent(&pre_set, &fact_var, &fact_val) {
                                continue;
                            }
                            let h_pre = h2_of_set(&pre_set, &cost);
                            if h_pre == INF {
                                continue;
                            }
                            let new_cost = op.cost.saturating_add(h_pre);
                            if new_cost < cost[p1][p2] {
                                cost[p1][p2] = new_cost;
                                cost[p2][p1] = new_cost;
                                changed = true;
                            }
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        let goal_facts: Vec<usize> = self.goal.iter().map(|&(v, d)| fid(v, d)).collect();
        let h = h2_of_set(&goal_facts, &cost);
        if h == INF { None } else { Some(h) }
    }
}

fn facts_consistent(facts: &[usize], fact_var: &[usize], fact_val: &[usize]) -> bool {
    for i in 0..facts.len() {
        for j in (i + 1)..facts.len() {
            if fact_var[facts[i]] == fact_var[facts[j]]
                && fact_val[facts[i]] != fact_val[facts[j]]
            {
                return false;
            }
        }
    }
    true
}

/// Max cost over all ordered pairs in `set x set`. Singletons `(p, p)`
/// are included, so this subsumes single-fact costs.
fn h2_of_set(set: &[usize], cost: &[Vec<usize>]) -> usize {
    const INF: usize = usize::MAX;
    let mut m = 0usize;
    for &p in set {
        for &q in set {
            let c = cost[p][q];
            if c == INF {
                return INF;
            }
            if c > m {
                m = c;
            }
        }
    }
    m
}

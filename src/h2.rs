use super::sasplus::{SASPlus, State};

const INF: usize = usize::MAX;

struct EffPc {
    p1: usize,
    pre_base: Vec<usize>,
    /// required[v] = Some(d) iff pre_base forces variable v to value d.
    required: Vec<Option<usize>>,
    consistent: bool,
}

struct PairPc {
    p1: usize,
    p2: usize,
    pre_set: Vec<usize>,
    consistent: bool,
}

struct OpPc {
    cost: usize,
    assigned: Vec<bool>,
    effs: Vec<EffPc>,
    pairs: Vec<PairPc>,
}

impl SASPlus {
    /// h^2 heuristic (Haslum & Geffner). Fixed-point regression over a
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

        let ops_pc: Vec<OpPc> = self
            .operators
            .iter()
            .map(|op| precompute_op(op, n, &offset, &fact_var, &fact_val))
            .collect();

        let mut cost = vec![vec![INF; num_facts]; num_facts];
        let init_facts: Vec<usize> = (0..n).map(|v| fid(v, state.values[v])).collect();
        for &f1 in &init_facts {
            for &f2 in &init_facts {
                cost[f1][f2] = 0;
            }
        }

        loop {
            let mut changed = false;

            for opc in &ops_pc {
                for pd in &opc.pairs {
                    if !pd.consistent {
                        continue;
                    }
                    let h = h2_of_set(&pd.pre_set, &cost);
                    if h == INF {
                        continue;
                    }
                    let new = opc.cost.saturating_add(h);
                    if new < cost[pd.p1][pd.p2] {
                        cost[pd.p1][pd.p2] = new;
                        cost[pd.p2][pd.p1] = new;
                        changed = true;
                    }
                }

                for ed in &opc.effs {
                    if !ed.consistent {
                        continue;
                    }
                    let base_h = h2_of_set(&ed.pre_base, &cost);
                    if base_h == INF {
                        continue;
                    }
                    let new_single = opc.cost.saturating_add(base_h);
                    if new_single < cost[ed.p1][ed.p1] {
                        cost[ed.p1][ed.p1] = new_single;
                        changed = true;
                    }

                    for v in 0..n {
                        if opc.assigned[v] {
                            continue;
                        }
                        match ed.required[v] {
                            Some(d) => {
                                // (v, d) already in pre_base, so h^2 is unchanged.
                                let p2 = fid(v, d);
                                if new_single < cost[ed.p1][p2] {
                                    cost[ed.p1][p2] = new_single;
                                    cost[p2][ed.p1] = new_single;
                                    changed = true;
                                }
                            }
                            None => {
                                for d in 0..self.variables[v].values.len() {
                                    let p2 = fid(v, d);
                                    let self_c = cost[p2][p2];
                                    if self_c == INF {
                                        continue;
                                    }
                                    // h^2(pre_base ∪ {p2}) = max over pairs
                                    // involving p2 and base_h.
                                    let mut m = if self_c > base_h { self_c } else { base_h };
                                    let mut any_inf = false;
                                    for &f in &ed.pre_base {
                                        let c = cost[f][p2];
                                        if c == INF {
                                            any_inf = true;
                                            break;
                                        }
                                        if c > m {
                                            m = c;
                                        }
                                    }
                                    if any_inf {
                                        continue;
                                    }
                                    let new = opc.cost.saturating_add(m);
                                    if new < cost[ed.p1][p2] {
                                        cost[ed.p1][p2] = new;
                                        cost[p2][ed.p1] = new;
                                        changed = true;
                                    }
                                }
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

fn precompute_op(
    op: &super::sasplus::Operator,
    n: usize,
    offset: &[usize],
    fact_var: &[usize],
    fact_val: &[usize],
) -> OpPc {
    let fid = |v: usize, d: usize| offset[v] + d;

    let mut op_pre: Vec<usize> = Vec::new();
    for &(v, d) in &op.prevail {
        op_pre.push(fid(v, d));
    }
    // prevail + every effect's pre-value (same as h1/hff).
    for e in &op.effects {
        if e.pre_value >= 0 {
            op_pre.push(fid(e.variable, e.pre_value as usize));
        }
    }
    let op_pre_ok = facts_consistent(&op_pre, fact_var, fact_val);

    let mut assigned = vec![false; n];
    for e in &op.effects {
        assigned[e.variable] = true;
    }

    let effs: Vec<EffPc> = op
        .effects
        .iter()
        .map(|e| {
            let p1 = fid(e.variable, e.post_value);
            let mut pre_base = op_pre.clone();
            for &(v, d) in &e.conditions {
                pre_base.push(fid(v, d));
            }
            let consistent = op_pre_ok && facts_consistent(&pre_base, fact_var, fact_val);
            let mut required = vec![None; n];
            if consistent {
                for &f in &pre_base {
                    required[fact_var[f]] = Some(fact_val[f]);
                }
            }
            EffPc {
                p1,
                pre_base,
                required,
                consistent,
            }
        })
        .collect();

    let mut pairs: Vec<PairPc> = Vec::new();
    for i in 0..op.effects.len() {
        for j in (i + 1)..op.effects.len() {
            let e1 = &op.effects[i];
            let e2 = &op.effects[j];
            if e1.variable == e2.variable {
                continue;
            }
            let p1 = fid(e1.variable, e1.post_value);
            let p2 = fid(e2.variable, e2.post_value);
            let mut pre_set = op_pre.clone();
            for &(v, d) in &e1.conditions {
                pre_set.push(fid(v, d));
            }
            for &(v, d) in &e2.conditions {
                pre_set.push(fid(v, d));
            }
            let consistent = op_pre_ok && facts_consistent(&pre_set, fact_var, fact_val);
            pairs.push(PairPc {
                p1,
                p2,
                pre_set,
                consistent,
            });
        }
    }

    OpPc {
        cost: op.cost,
        assigned,
        effs,
        pairs,
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

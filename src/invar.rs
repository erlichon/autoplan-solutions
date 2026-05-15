use super::SASPlus;
use super::sasplus::State;

impl SASPlus {
    /// Extract binary invariants using the Rintanen (1998) filter-refinement
    /// algorithm as presented in lecture 12.
    ///
    /// 1. Start with all binary disjunctions ℓ₁ ∨ ℓ₂ true in the initial state.
    /// 2. For each action, filter: keep ℓ₁ ∨ ℓ₂ only if the action preserves it
    ///    (using unit propagation with the current invariant set).
    /// 3. Repeat until fixpoint.
    pub fn h2_invariants(&self, state: &State) -> Vec<String> {
        let n = self.variables.len();
        let mut offset = vec![0usize; n + 1];
        for i in 0..n {
            offset[i + 1] = offset[i] + self.variables[i].values.len();
        }
        let nf = offset[n];
        let nl = 2 * nf;

        // Literal encoding: positive (v=d) → 2*(offset[v]+d), negative ¬(v=d) → 2*(offset[v]+d)+1

        // Which literals are true in the initial state?
        let mut init_true = vec![false; nl];
        for v in 0..n {
            let d = state.values[v];
            init_true[2 * (offset[v] + d)] = true;
            for d2 in 0..self.variables[v].values.len() {
                if d2 != d {
                    init_true[2 * (offset[v] + d2) + 1] = true;
                }
            }
        }

        // inv[l1][l2] = true means l1 ∨ l2 is a candidate invariant (symmetric)
        let mut inv = vec![vec![false; nl]; nl];
        for l1 in 0..nl {
            for l2 in l1..nl {
                if init_true[l1] || init_true[l2] {
                    inv[l1][l2] = true;
                    inv[l2][l1] = true;
                }
            }
        }

        // Precompute per-operator: the literal indices known directly from
        // preconditions (before invariant-based unit propagation).
        let op_pre_lits: Vec<Vec<usize>> = self
            .operators
            .iter()
            .map(|op| {
                let mut seen = vec![false; nl];
                let mut lits = Vec::new();
                let mut add_fact = |f: usize| {
                    let v = f; // fact index
                    let var = {
                        let mut var = 0;
                        while var + 1 < n && offset[var + 1] <= v {
                            var += 1;
                        }
                        var
                    };
                    let val = v - offset[var];
                    let pl = 2 * f;
                    if !seen[pl] {
                        seen[pl] = true;
                        lits.push(pl);
                    }
                    for d2 in 0..self.variables[var].values.len() {
                        if d2 != val {
                            let nl_idx = 2 * (offset[var] + d2) + 1;
                            if !seen[nl_idx] {
                                seen[nl_idx] = true;
                                lits.push(nl_idx);
                            }
                        }
                    }
                };
                for &(v, d) in &op.prevail {
                    add_fact(offset[v] + d);
                }
                for e in &op.effects {
                    if e.pre_value >= 0 {
                        add_fact(offset[e.variable] + e.pre_value as usize);
                    }
                }
                lits
            })
            .collect();

        let mut known = vec![false; nl];
        let mut queue: Vec<usize> = Vec::new();
        let mut affected = vec![false; nl];
        let mut all_eff = vec![false; nl];
        let mut definite_eff = vec![false; nl];
        let mut u = vec![false; nl];

        loop {
            let mut changed = false;

            for (oi, op) in self.operators.iter().enumerate() {
                // Reset working buffers
                known.iter_mut().for_each(|x| *x = false);
                queue.clear();

                for &l in &op_pre_lits[oi] {
                    if !known[l] {
                        known[l] = true;
                        queue.push(l);
                    }
                }

                // Unit propagation using current invariant set
                let mut contradiction = false;
                let mut qi = 0;
                while qi < queue.len() {
                    let k = queue[qi];
                    qi += 1;
                    let nk = k ^ 1;
                    for l2 in 0..nl {
                        if inv[nk][l2] && !known[l2] {
                            known[l2] = true;
                            if known[l2 ^ 1] {
                                contradiction = true;
                                break;
                            }
                            queue.push(l2);
                        }
                    }
                    if contradiction {
                        break;
                    }
                }

                if contradiction {
                    continue;
                }

                // Determine which effects fire, and build affected / eff / U
                affected.iter_mut().for_each(|x| *x = false);
                all_eff.iter_mut().for_each(|x| *x = false);
                definite_eff.iter_mut().for_each(|x| *x = false);

                for e in &op.effects {
                    let is_uncertain;
                    if e.conditions.is_empty() {
                        is_uncertain = false;
                    } else {
                        let all_known =
                            e.conditions.iter().all(|&(cv, cd)| known[2 * (offset[cv] + cd)]);
                        if all_known {
                            is_uncertain = false;
                        } else {
                            let any_false = e.conditions
                                .iter()
                                .any(|&(cv, cd)| known[2 * (offset[cv] + cd) + 1]);
                            if any_false {
                                continue;
                            }
                            is_uncertain = true;
                        }
                    }

                    let add_f = offset[e.variable] + e.post_value;
                    affected[2 * add_f + 1] = true;
                    all_eff[2 * add_f] = true;

                    if e.pre_value >= 0 {
                        let del_f = offset[e.variable] + e.pre_value as usize;
                        affected[2 * del_f] = true;
                        all_eff[2 * del_f + 1] = true;
                    } else {
                        for d in 0..self.variables[e.variable].values.len() {
                            if d != e.post_value {
                                let del_f = offset[e.variable] + d;
                                affected[2 * del_f] = true;
                                all_eff[2 * del_f + 1] = true;
                            }
                        }
                    }

                    if !is_uncertain {
                        definite_eff[2 * add_f] = true;
                        if e.pre_value >= 0 {
                            definite_eff[2 * (offset[e.variable] + e.pre_value as usize) + 1] =
                                true;
                        } else {
                            for d in 0..self.variables[e.variable].values.len() {
                                if d != e.post_value {
                                    definite_eff[2 * (offset[e.variable] + d) + 1] = true;
                                }
                            }
                        }
                    }
                }

                // U = (known \ {negate(l) | l ∈ all_eff}) ∪ definite_eff
                for l in 0..nl {
                    u[l] = (known[l] && !all_eff[l ^ 1]) || definite_eff[l];
                }

                // Filter: for each affected literal l, check invariants l ∨ l2
                for l in 0..nl {
                    if !affected[l] {
                        continue;
                    }
                    for l2 in 0..nl {
                        if inv[l][l2] && !u[l2] {
                            inv[l][l2] = false;
                            inv[l2][l] = false;
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        // Format output: for each ordered fact pair (v1,x1) < (v2,x2),
        // check all four sign combinations.
        let mut out: Vec<String> = Vec::new();
        for v1 in 0..n {
            for x1 in 0..self.variables[v1].values.len() {
                let f1 = offset[v1] + x1;
                for v2 in v1..n {
                    let x2_start = if v2 == v1 { x1 + 1 } else { 0 };
                    for x2 in x2_start..self.variables[v2].values.len() {
                        let f2 = offset[v2] + x2;
                        if inv[2 * f1][2 * f2] {
                            out.push(format!("+ {} {} + {} {}", v1, x1, v2, x2));
                        }
                        if inv[2 * f1][2 * f2 + 1] {
                            out.push(format!("+ {} {} - {} {}", v1, x1, v2, x2));
                        }
                        if inv[2 * f1 + 1][2 * f2] {
                            out.push(format!("- {} {} + {} {}", v1, x1, v2, x2));
                        }
                        if inv[2 * f1 + 1][2 * f2 + 1] {
                            out.push(format!("- {} {} - {} {}", v1, x1, v2, x2));
                        }
                    }
                }
            }
        }

        out.sort();
        out
    }
}

use super::sasplus::{SASPlus, State};

/// Result of the h^FF heuristic computation.
///
/// - `heuristic` is the h^FF heuristic value for the initial state, defined
///   as the **cost** of the extracted relaxed plan (the sum of operator
///   costs). Under unit-cost problems this equals the plan length.
/// - `plan` is the list of operator indices (0-indexed into
///   `SASPlus::operators`) in layer order, low-to-high (i.e. an order that
///   is applicable under delete relaxation).
pub struct HFF {
    pub heuristic: usize,
    pub plan: Vec<usize>,
}

impl SASPlus {
    /// Compute the h^FF heuristic and extract a delete-relaxed plan.
    ///
    /// Returns `None` if the goal is unreachable under delete relaxation.
    pub fn h_ff(&self, state: &State) -> Option<HFF> {
        // -----------------------------------------------------------------
        // Phase 1: layered reachability (h^max-style levels + best achiever)
        // -----------------------------------------------------------------
        //
        // level[v][d] = earliest layer at which fact (v, d) is achievable
        //               (0 if true in the initial state, usize::MAX if not
        //               yet known to be reachable).
        // achiever[v][d] = (op_idx, eff_idx) that first achieved (v, d)
        //                  at layer level[v][d]. None for facts that are
        //                  true in the initial state.
        let mut level: Vec<Vec<usize>> = self
            .variables
            .iter()
            .enumerate()
            .map(|(i, var)| {
                (0..var.values.len())
                    .map(|v| if state.values[i] == v { 0 } else { usize::MAX })
                    .collect()
            })
            .collect();

        let mut achiever: Vec<Vec<Option<(usize, usize)>>> = self
            .variables
            .iter()
            .map(|var| vec![None; var.values.len()])
            .collect();

        loop {
            let mut changed = false;
            for (op_idx, op) in self.operators.iter().enumerate() {
                for (eff_idx, eff) in op.effects.iter().enumerate() {
                    let Some(precond_level) = Self::effect_precond_level(op, eff, &level)
                    else {
                        continue;
                    };
                    let new_level = precond_level + 1;
                    if new_level < level[eff.variable][eff.post_value] {
                        level[eff.variable][eff.post_value] = new_level;
                        achiever[eff.variable][eff.post_value] = Some((op_idx, eff_idx));
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        for &(v, d) in &self.goal {
            if level[v][d] == usize::MAX {
                return None;
            }
        }

        // -----------------------------------------------------------------
        // Phase 2: relaxed-plan extraction (FF-style backward pass)
        // -----------------------------------------------------------------
        //
        // Starting from goal facts at their layers, walk DOWN the layers.
        // For each subgoal at layer t > 0, pick its recorded achiever and
        // add that operator to the plan (once per operator). Enqueue the
        // achiever's preconditions as new subgoals at their own layers.
        // Facts with level 0 are already true in the initial state and
        // need no achiever.

        let max_layer = self
            .goal
            .iter()
            .map(|&(v, d)| level[v][d])
            .max()
            .unwrap_or(0);

        let mut subgoals: Vec<Vec<(usize, usize)>> = vec![Vec::new(); max_layer + 1];
        let mut is_subgoal: Vec<Vec<bool>> = self
            .variables
            .iter()
            .map(|var| vec![false; var.values.len()])
            .collect();

        for &(v, d) in &self.goal {
            let l = level[v][d];
            if l > 0 && !is_subgoal[v][d] {
                subgoals[l].push((v, d));
                is_subgoal[v][d] = true;
            }
        }

        let mut plan: Vec<usize> = Vec::new();
        let mut used_op: Vec<bool> = vec![false; self.operators.len()];

        for t in (1..=max_layer).rev() {
            // subgoals[t] may grow while we iterate (if some achiever's
            // precondition happens to also sit at layer t-- which cannot
            // happen in our h^max-style layering, but we still use index
            // iteration to be robust).
            let mut idx = 0;
            while idx < subgoals[t].len() {
                let (v, d) = subgoals[t][idx];
                idx += 1;

                let (op_idx, eff_idx) = achiever[v][d]
                    .expect("reachable non-initial fact must have an achiever");

                if !used_op[op_idx] {
                    plan.push(op_idx);
                    used_op[op_idx] = true;
                }

                // Enqueue this achiever-effect's preconditions. As in
                // `effect_precond_level`, the precondition for firing
                // `eff_idx` is the union of the operator's prevail, every
                // effect's pre-value (>=0), and this effect's conditional
                // conditions.
                let op = &self.operators[op_idx];
                let eff = &op.effects[eff_idx];

                for &(pv, pd) in &op.prevail {
                    Self::push_subgoal(pv, pd, &level, &mut is_subgoal, &mut subgoals);
                }
                for e in &op.effects {
                    if e.pre_value >= 0 {
                        let pd = e.pre_value as usize;
                        Self::push_subgoal(e.variable, pd, &level, &mut is_subgoal, &mut subgoals);
                    }
                }
                for &(cv, cd) in &eff.conditions {
                    Self::push_subgoal(cv, cd, &level, &mut is_subgoal, &mut subgoals);
                }
            }
        }

        // We appended operators top-down (high layer first). Reverse to get a
        // forward-executable (under delete relaxation) ordering.
        plan.reverse();

        // h^FF value = cost of the extracted relaxed plan.
        let heuristic = plan
            .iter()
            .map(|&op_idx| self.operators[op_idx].cost)
            .sum();
        Some(HFF { heuristic, plan })
    }

    /// Max of the levels of an effect's preconditions. An effect of an
    /// operator can only fire when the whole operator is applicable, so the
    /// precondition for firing effect `e` is the union of:
    ///   * all prevail conditions of the operator,
    ///   * the pre-value of **every** effect of the operator that has
    ///     `pre_value >= 0` (SAS+ operator applicability as a whole),
    ///   * the conditional-effect conditions of `e` itself.
    /// Returns `None` if any of these facts is still unreachable.
    fn effect_precond_level(
        op: &super::sasplus::Operator,
        eff: &super::sasplus::Effect,
        level: &[Vec<usize>],
    ) -> Option<usize> {
        let mut m = 0usize;
        for &(v, d) in &op.prevail {
            match level[v][d] {
                usize::MAX => return None,
                l => m = m.max(l),
            }
        }
        for e in &op.effects {
            if e.pre_value >= 0 {
                let d = e.pre_value as usize;
                match level[e.variable][d] {
                    usize::MAX => return None,
                    l => m = m.max(l),
                }
            }
        }
        for &(v, d) in &eff.conditions {
            match level[v][d] {
                usize::MAX => return None,
                l => m = m.max(l),
            }
        }
        Some(m)
    }

    fn push_subgoal(
        v: usize,
        d: usize,
        level: &[Vec<usize>],
        is_subgoal: &mut [Vec<bool>],
        subgoals: &mut [Vec<(usize, usize)>],
    ) {
        let l = level[v][d];
        if l > 0 && !is_subgoal[v][d] {
            subgoals[l].push((v, d));
            is_subgoal[v][d] = true;
        }
    }
}

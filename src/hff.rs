use super::sasplus::{SASPlus, State};

pub struct HFF {
    /// Sum of operator costs in `plan` (the h^FF value).
    pub heuristic: usize,
    /// Relaxed plan, applicable in order under delete relaxation.
    pub plan: Vec<usize>,
}

impl SASPlus {
    /// Extract a delete-relaxed plan via FF heuristic.
    /// Returns `None` if the goal is unreachable under delete relaxation.
    pub fn h_ff(&self, state: &State) -> Option<HFF> {
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

        plan.reverse();

        let heuristic = plan
            .iter()
            .map(|&op_idx| self.operators[op_idx].cost)
            .sum();
        Some(HFF { heuristic, plan })
    }

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
        // Operator applicability needs every effect's pre-value, not just this one.
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

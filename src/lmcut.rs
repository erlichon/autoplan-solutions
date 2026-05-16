use super::sasplus::{SASPlus, State};
use std::collections::VecDeque;

/// One round of the LM-cut iterative landmark extraction.
pub struct LMCutRound {
    /// h^max value of the goal at the start of this round.
    pub hmax: usize,
    /// Operator indices forming the disjunctive action landmark (sorted).
    pub landmark: Vec<usize>,
    /// Minimum operator cost within the landmark (= cost partitioning weight).
    pub cost: usize,
}

/// Unary operator: a single (operator, effect) pair used for h^max and
/// justification-graph construction.
struct UnaryOp {
    op_idx: usize,
    effect_fid: usize,
    /// Sorted, deduplicated fact-IDs of all preconditions
    /// (prevail + all effects' pre-values + this effect's conditions).
    pre_facts: Vec<usize>,
}

impl SASPlus {
    /// Compute the LM-cut heuristic (slide 27, Ch.11).
    ///
    /// Iteratively extracts disjunctive action landmarks from the
    /// delete-relaxed problem, returning one `LMCutRound` per iteration
    /// (the final round has `hmax == 0` and an empty landmark).
    pub fn lm_cut(&self, state: &State) -> Vec<LMCutRound> {
        let nv = self.variables.len();
        let mut offset = vec![0usize; nv + 1];
        for i in 0..nv {
            offset[i + 1] = offset[i] + self.variables[i].values.len();
        }
        let nf = offset[nv];
        let virtual_init = nf;

        let uops = self.build_unary_ops(&offset);
        let goal_fids: Vec<usize> = self.goal.iter().map(|&(v, d)| offset[v] + d).collect();

        let mut costs: Vec<usize> = self.operators.iter().map(|o| o.cost).collect();
        let mut rounds = Vec::new();

        let mut hmax = vec![0usize; nf];
        let mut in_v0 = vec![false; nf + 1];
        let mut supporters: Vec<usize> = vec![0; uops.len()];
        let mut in_landmark = vec![false; self.operators.len()];
        let mut zero_rev: Vec<Vec<usize>> = vec![vec![]; nf + 1];
        let mut queue = VecDeque::new();

        loop {
            self.hmax_into(state, &costs, &offset, &mut hmax);

            let hmax_goal = goal_fids
                .iter()
                .map(|&fid| hmax[fid])
                .max()
                .unwrap_or(0);

            if hmax_goal == 0 {
                rounds.push(LMCutRound {
                    hmax: 0,
                    landmark: vec![],
                    cost: 0,
                });
                break;
            }
            if hmax_goal == usize::MAX {
                break;
            }

            Self::find_supporters(&uops, &hmax, virtual_init, &mut supporters);

            for v in &mut zero_rev {
                v.clear();
            }
            for (i, uo) in uops.iter().enumerate() {
                if costs[uo.op_idx] != 0 {
                    continue;
                }
                let sup = supporters[i];
                if hmax[uo.effect_fid] == usize::MAX {
                    continue;
                }
                if sup < nf && hmax[sup] == usize::MAX {
                    continue;
                }
                zero_rev[uo.effect_fid].push(sup);
            }

            for i in 0..=nf {
                in_v0[i] = false;
            }
            queue.clear();
            // Start V0 from the single goal fact with max hmax cost.
            // First-max-wins tie-breaking (models the artificial goal operator).
            let mut start_fid = goal_fids[0];
            let mut start_h = hmax[start_fid];
            for &fid in &goal_fids[1..] {
                if hmax[fid] > start_h {
                    start_h = hmax[fid];
                    start_fid = fid;
                }
            }
            in_v0[start_fid] = true;
            queue.push_back(start_fid);
            while let Some(f) = queue.pop_front() {
                for &sup in &zero_rev[f] {
                    if !in_v0[sup] {
                        in_v0[sup] = true;
                        queue.push_back(sup);
                    }
                }
            }

            for b in &mut in_landmark {
                *b = false;
            }
            for (i, uo) in uops.iter().enumerate() {
                if hmax[uo.effect_fid] == usize::MAX {
                    continue;
                }
                let sup = supporters[i];
                if sup < nf && hmax[sup] == usize::MAX {
                    continue;
                }
                if !in_v0[sup] && in_v0[uo.effect_fid] {
                    in_landmark[uo.op_idx] = true;
                }
            }

            let landmark: Vec<usize> = (0..self.operators.len())
                .filter(|&i| in_landmark[i])
                .collect();

            let lm_cost = landmark
                .iter()
                .map(|&a| costs[a])
                .min()
                .expect("landmark must not be empty when hmax > 0");

            rounds.push(LMCutRound {
                hmax: hmax_goal,
                landmark,
                cost: lm_cost,
            });

            for i in 0..self.operators.len() {
                if in_landmark[i] {
                    costs[i] -= lm_cost;
                }
            }
        }

        rounds
    }

    /// Decompose each (operator, effect) pair into a unary operator whose
    /// preconditions are the full operator preconditions (prevail + all
    /// effects' pre-values + this effect's conditions).
    fn build_unary_ops(&self, offset: &[usize]) -> Vec<UnaryOp> {
        let mut uops = Vec::new();
        for (op_idx, op) in self.operators.iter().enumerate() {
            for eff in &op.effects {
                let effect_fid = offset[eff.variable] + eff.post_value;
                let mut pre_facts = Vec::new();
                for &(v, d) in &op.prevail {
                    pre_facts.push(offset[v] + d);
                }
                for e in &op.effects {
                    if e.pre_value >= 0 {
                        pre_facts.push(offset[e.variable] + e.pre_value as usize);
                    }
                }
                for &(v, d) in &eff.conditions {
                    pre_facts.push(offset[v] + d);
                }
                uops.push(UnaryOp {
                    op_idx,
                    effect_fid,
                    pre_facts: {
                        pre_facts.sort_unstable();
                        pre_facts.dedup();
                        pre_facts
                    },
                });
            }
        }
        uops
    }

    /// For each unary op, pick the precondition with maximum h^max as its
    /// supporter (smallest fact-ID breaks ties).
    fn find_supporters(
        uops: &[UnaryOp],
        hmax: &[usize],
        virtual_init: usize,
        supporters: &mut [usize],
    ) {
        for (i, uo) in uops.iter().enumerate() {
            if uo.pre_facts.is_empty() {
                supporters[i] = virtual_init;
                continue;
            }
            let mut best = uo.pre_facts[0];
            let mut best_h = hmax[best];
            for &fid in &uo.pre_facts[1..] {
                let h = hmax[fid];
                if h > best_h {
                    best_h = h;
                    best = fid;
                }
            }
            supporters[i] = best;
        }
    }
}

use super::SASPlus;
use super::sasplus::State;

impl SASPlus {
    /// Extract binary invariants using the Rintanen (1998) filter-refinement
    /// algorithm with bitset-optimised inner loops.
    pub fn h2_invariants(&self, state: &State) -> Vec<String> {
        let n = self.variables.len();
        let mut offset = vec![0usize; n + 1];
        for i in 0..n {
            offset[i + 1] = offset[i] + self.variables[i].values.len();
        }
        let nf = offset[n];
        let nl = 2 * nf;
        let nw = (nl + 63) / 64;

        let mut init_mask = vec![0u64; nw];
        for v in 0..n {
            let d = state.values[v];
            bset(&mut init_mask, 2 * (offset[v] + d));
            for d2 in 0..self.variables[v].values.len() {
                if d2 != d {
                    bset(&mut init_mask, 2 * (offset[v] + d2) + 1);
                }
            }
        }

        let full_mask = {
            let mut m = vec![!0u64; nw];
            let rem = nl % 64;
            if rem != 0 {
                m[nw - 1] = (1u64 << rem) - 1;
            }
            m
        };

        // inv[l] is the bitset of l' where l ∨ l' is a candidate invariant.
        let mut inv: Vec<Vec<u64>> = (0..nl)
            .map(|l| {
                if bget(&init_mask, l) {
                    full_mask.clone()
                } else {
                    init_mask.clone()
                }
            })
            .collect();

        // Precompute per-operator precondition literals
        let op_pre: Vec<Vec<usize>> = self
            .operators
            .iter()
            .map(|op| {
                let mut seen = vec![false; nl];
                let mut lits = Vec::new();
                let mut add_fact = |fid: usize, var: usize, val: usize| {
                    let pl = 2 * fid;
                    if !seen[pl] {
                        seen[pl] = true;
                        lits.push(pl);
                    }
                    for d2 in 0..self.variables[var].values.len() {
                        if d2 != val {
                            let ni = 2 * (offset[var] + d2) + 1;
                            if !seen[ni] {
                                seen[ni] = true;
                                lits.push(ni);
                            }
                        }
                    }
                };
                for &(v, d) in &op.prevail {
                    add_fact(offset[v] + d, v, d);
                }
                for e in &op.effects {
                    if e.pre_value >= 0 {
                        add_fact(
                            offset[e.variable] + e.pre_value as usize,
                            e.variable,
                            e.pre_value as usize,
                        );
                    }
                }
                lits
            })
            .collect();

        let mut known = vec![0u64; nw];
        let mut u_bits = vec![0u64; nw];
        let mut aff = vec![0u64; nw];
        let mut all_eff = vec![0u64; nw];
        let mut def_eff = vec![0u64; nw];
        let mut queue: Vec<usize> = Vec::with_capacity(nl);

        loop {
            let mut changed = false;

            for (oi, op) in self.operators.iter().enumerate() {
                for w in known.iter_mut() {
                    *w = 0;
                }
                queue.clear();
                for &l in &op_pre[oi] {
                    if !bget(&known, l) {
                        bset(&mut known, l);
                        queue.push(l);
                    }
                }

                // Unit propagation using current invariant set
                let mut contradiction = false;
                let mut qi = 0;
                while qi < queue.len() {
                    let nk = queue[qi] ^ 1;
                    qi += 1;
                    for wi in 0..nw {
                        let fresh = inv[nk][wi] & !known[wi];
                        if fresh == 0 {
                            continue;
                        }
                        known[wi] |= fresh;
                        let mut w = fresh;
                        while w != 0 {
                            let bit = w.trailing_zeros() as usize;
                            let l2 = wi * 64 + bit;
                            w &= w - 1;
                            if l2 >= nl {
                                continue;
                            }
                            if bget(&known, l2 ^ 1) {
                                contradiction = true;
                                break;
                            }
                            queue.push(l2);
                        }
                        if contradiction {
                            break;
                        }
                    }
                    if contradiction {
                        break;
                    }
                }
                if contradiction {
                    continue;
                }

                // Build affected / all_eff / def_eff from effects
                for w in aff.iter_mut() {
                    *w = 0;
                }
                for w in all_eff.iter_mut() {
                    *w = 0;
                }
                for w in def_eff.iter_mut() {
                    *w = 0;
                }

                for e in &op.effects {
                    let uncertain = if e.conditions.is_empty() {
                        false
                    } else {
                        let ok = e
                            .conditions
                            .iter()
                            .all(|&(cv, cd)| bget(&known, 2 * (offset[cv] + cd)));
                        if ok {
                            false
                        } else {
                            let bad = e
                                .conditions
                                .iter()
                                .any(|&(cv, cd)| bget(&known, 2 * (offset[cv] + cd) + 1));
                            if bad {
                                continue;
                            }
                            true
                        }
                    };

                    let af = offset[e.variable] + e.post_value;
                    bset(&mut aff, 2 * af + 1);
                    bset(&mut all_eff, 2 * af);

                    if e.pre_value >= 0 {
                        let df = offset[e.variable] + e.pre_value as usize;
                        bset(&mut aff, 2 * df);
                        bset(&mut all_eff, 2 * df + 1);
                    } else {
                        for d in 0..self.variables[e.variable].values.len() {
                            if d != e.post_value {
                                let df = offset[e.variable] + d;
                                bset(&mut aff, 2 * df);
                                bset(&mut all_eff, 2 * df + 1);
                            }
                        }
                    }

                    if !uncertain {
                        bset(&mut def_eff, 2 * af);
                        if e.pre_value >= 0 {
                            bset(
                                &mut def_eff,
                                2 * (offset[e.variable] + e.pre_value as usize) + 1,
                            );
                        } else {
                            for d in 0..self.variables[e.variable].values.len() {
                                if d != e.post_value {
                                    bset(&mut def_eff, 2 * (offset[e.variable] + d) + 1);
                                }
                            }
                        }
                    }
                }

                if aff.iter().all(|&w| w == 0) {
                    continue;
                }

                // U = (known AND NOT negate(all_eff)) OR def_eff
                // negate swaps adjacent bit pairs (bit b ↔ bit b^1)
                const EVEN: u64 = 0x5555_5555_5555_5555;
                for wi in 0..nw {
                    let neg =
                        ((all_eff[wi] >> 1) & EVEN) | ((all_eff[wi] & EVEN) << 1);
                    u_bits[wi] = (known[wi] & !neg) | def_eff[wi];
                }

                // Filter: for each affected literal l, remove invariants
                // l ∨ l2 where l2 ∉ U.
                for awi in 0..nw {
                    let mut aw = aff[awi];
                    while aw != 0 {
                        let ab = aw.trailing_zeros() as usize;
                        let l = awi * 64 + ab;
                        aw &= aw - 1;
                        if l >= nl {
                            break;
                        }
                        for wi in 0..nw {
                            let rm = inv[l][wi] & !u_bits[wi];
                            if rm == 0 {
                                continue;
                            }
                            inv[l][wi] ^= rm;
                            changed = true;
                            let mut rw = rm;
                            while rw != 0 {
                                let rb = rw.trailing_zeros() as usize;
                                let l2 = wi * 64 + rb;
                                rw &= rw - 1;
                                if l2 < nl {
                                    bclr(&mut inv[l2], l);
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

        // Format output
        let mut out = Vec::new();
        for v1 in 0..n {
            for x1 in 0..self.variables[v1].values.len() {
                let f1 = offset[v1] + x1;
                for v2 in v1..n {
                    let x2s = if v2 == v1 { x1 + 1 } else { 0 };
                    for x2 in x2s..self.variables[v2].values.len() {
                        let f2 = offset[v2] + x2;
                        if bget(&inv[2 * f1], 2 * f2) {
                            out.push(format!("+ {} {} + {} {}", v1, x1, v2, x2));
                        }
                        if bget(&inv[2 * f1], 2 * f2 + 1) {
                            out.push(format!("+ {} {} - {} {}", v1, x1, v2, x2));
                        }
                        if bget(&inv[2 * f1 + 1], 2 * f2) {
                            out.push(format!("- {} {} + {} {}", v1, x1, v2, x2));
                        }
                        if bget(&inv[2 * f1 + 1], 2 * f2 + 1) {
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

#[inline(always)]
fn bget(bits: &[u64], i: usize) -> bool {
    bits[i / 64] & (1u64 << (i % 64)) != 0
}

#[inline(always)]
fn bset(bits: &mut [u64], i: usize) {
    bits[i / 64] |= 1u64 << (i % 64);
}

#[inline(always)]
fn bclr(bits: &mut [u64], i: usize) {
    bits[i / 64] &= !(1u64 << (i % 64));
}

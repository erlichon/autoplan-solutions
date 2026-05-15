use super::h2::H2Table;
use super::SASPlus;
use super::sasplus::State;

const INF: usize = usize::MAX;

impl SASPlus {
    /// Extract binary invariants from the h^2 cost table.
    ///
    /// An invariant L1 \/ L2 holds iff the conjunction (not L1) /\ (not L2)
    /// is unreachable.  Four sign combinations are checked for each ordered
    /// fact pair (v1,x1) < (v2,x2):
    ///
    ///   -- : cost[(v1,x1)][(v2,x2)] = inf
    ///   +- : for all a != x1: cost[(v1,a)][(v2,x2)] = inf
    ///   -+ : for all b != x2: cost[(v1,x1)][(v2,b)] = inf
    ///   ++ : for all a != x1, b != x2: cost[(v1,a)][(v2,b)] = inf
    ///
    /// Returns sorted invariant strings ready for output.
    pub fn h2_invariants(&self, state: &State) -> Vec<String> {
        let table = self.h2_cost_table(state);
        extract_invariants(self, &table)
    }
}

fn extract_invariants(sas: &SASPlus, table: &H2Table) -> Vec<String> {
    let n = sas.variables.len();
    let cost = &table.cost;
    let offset = &table.offset;
    let mut out: Vec<String> = Vec::new();

    for v1 in 0..n {
        let d1 = sas.variables[v1].values.len();
        for x1 in 0..d1 {
            for v2 in v1..n {
                let d2 = sas.variables[v2].values.len();
                let x2_start = if v2 == v1 { x1 + 1 } else { 0 };
                for x2 in x2_start..d2 {
                    let f1 = offset[v1] + x1;
                    let f2 = offset[v2] + x2;

                    // -- : (v1=x1) and (v2=x2) mutex
                    if cost[f1][f2] == INF {
                        out.push(format!("- {} {} - {} {}", v1, x1, v2, x2));
                    }

                    // +- : for all a != x1 in domain(v1), (v1,a) and (v2,x2) mutex
                    if (0..d1).all(|a| a == x1 || cost[offset[v1] + a][f2] == INF) {
                        out.push(format!("+ {} {} - {} {}", v1, x1, v2, x2));
                    }

                    // -+ : for all b != x2 in domain(v2), (v1,x1) and (v2,b) mutex
                    if (0..d2).all(|b| b == x2 || cost[f1][offset[v2] + b] == INF) {
                        out.push(format!("- {} {} + {} {}", v1, x1, v2, x2));
                    }

                    // ++ : for all a != x1, b != x2, (v1,a) and (v2,b) mutex
                    let pp = (0..d1).all(|a| {
                        a == x1
                            || (0..d2)
                                .all(|b| b == x2 || cost[offset[v1] + a][offset[v2] + b] == INF)
                    });
                    if pp {
                        out.push(format!("+ {} {} + {} {}", v1, x1, v2, x2));
                    }
                }
            }
        }
    }

    out.sort();
    out
}

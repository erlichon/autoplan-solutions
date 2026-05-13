use std::collections::HashSet;

use super::SASPlus;

/// Parse the landmark section appended after the SAS+ file.
pub fn parse_landmarks(input: &str) -> Vec<Vec<usize>> {
    let mut lines = input.lines().filter(|l| !l.is_empty());
    let n: usize = lines.next().unwrap().trim().parse().unwrap();
    let mut landmarks = Vec::with_capacity(n);
    for _ in 0..n {
        let line = lines.next().unwrap();
        let nums: Vec<usize> = line
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        landmarks.push(nums[1..].to_vec());
    }
    landmarks
}

/// Canonical landmark heuristic h^C.
///
/// Per-landmark value = min_{a in L_i} cost(a).
/// Two landmarks are additive iff they share no action.
/// h^C = max over all pairwise-additive subsets S of sum of values(S).
///
/// Solved via MWIS on the conflict graph, decomposed into connected
/// components with branch-and-bound per component.
pub fn canonical_lm_heuristic(sas: &SASPlus, landmarks: &[Vec<usize>]) -> usize {
    let n = landmarks.len();
    if n == 0 {
        return 0;
    }

    let values: Vec<usize> = landmarks
        .iter()
        .map(|lm| lm.iter().map(|&a| sas.operators[a].cost).min().unwrap_or(0))
        .collect();

    let action_sets: Vec<HashSet<usize>> =
        landmarks.iter().map(|lm| lm.iter().copied().collect()).collect();

    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for i in 0..n {
        for j in i + 1..n {
            if !action_sets[i].is_disjoint(&action_sets[j]) {
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }

    // Sum MWIS over connected components.
    let mut visited = vec![false; n];
    let mut total = 0usize;
    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut comp = vec![];
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(v) = stack.pop() {
            comp.push(v);
            for &u in &adj[v] {
                if !visited[u] {
                    visited[u] = true;
                    stack.push(u);
                }
            }
        }
        total += mwis(&values, &adj, &comp);
    }
    total
}

/// Maximum weight independent set via branch-and-bound.
fn mwis(values: &[usize], adj: &[Vec<usize>], comp: &[usize]) -> usize {
    let k = comp.len();
    // Build local adjacency bitmasks for the component.
    let local_values: Vec<usize> = comp.iter().map(|&i| values[i]).collect();
    let global_to_local: std::collections::HashMap<usize, usize> =
        comp.iter().enumerate().map(|(li, &gi)| (gi, li)).collect();
    let mut local_adj = vec![0u64; k];
    for (li, &gi) in comp.iter().enumerate() {
        for &gj in &adj[gi] {
            if let Some(&lj) = global_to_local.get(&gj) {
                local_adj[li] |= 1u64 << lj;
            }
        }
    }

    struct Ctx<'a> {
        k: usize,
        values: &'a [usize],
        adj: &'a [u64],
        best: usize,
    }
    fn solve(ctx: &mut Ctx, i: usize, excluded: u64, current: usize, remaining: usize) {
        if current + remaining <= ctx.best {
            return;
        }
        if i >= ctx.k {
            ctx.best = ctx.best.max(current);
            return;
        }
        let new_remaining = remaining - ctx.values[i];
        if excluded & (1u64 << i) != 0 {
            solve(ctx, i + 1, excluded, current, new_remaining);
            return;
        }
        solve(ctx, i + 1, excluded, current, new_remaining);
        solve(ctx, i + 1, excluded | ctx.adj[i], current + ctx.values[i], new_remaining);
    }

    let mut ctx = Ctx {
        k,
        values: &local_values,
        adj: &local_adj,
        best: 0,
    };
    let total: usize = local_values.iter().sum();
    solve(&mut ctx, 0, 0, 0, total);
    ctx.best
}

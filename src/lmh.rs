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
        let actions = nums[1..].to_vec();
        landmarks.push(actions);
    }
    landmarks
}

/// Canonical landmark heuristic via optimal LP cost partitioning.
/// max Σ x_i  s.t.  Σ_{i: a ∈ L_i} x_i ≤ cost(a),  x_i ≥ 0
pub fn canonical_lm_heuristic(sas: &SASPlus, landmarks: &[Vec<usize>]) -> usize {
    if landmarks.is_empty() {
        return 0;
    }

    let n = landmarks.len();

    let mut action_set = std::collections::BTreeSet::new();
    for lm in landmarks {
        for &a in lm {
            action_set.insert(a);
        }
    }
    let actions: Vec<usize> = action_set.into_iter().collect();
    let m = actions.len();
    let action_idx: std::collections::HashMap<usize, usize> =
        actions.iter().enumerate().map(|(i, &a)| (a, i)).collect();

    let mut a_matrix = vec![vec![0.0f64; n]; m];
    for (col, lm) in landmarks.iter().enumerate() {
        for &act in lm {
            a_matrix[action_idx[&act]][col] = 1.0;
        }
    }

    let b: Vec<f64> = actions.iter().map(|&a| sas.operators[a].cost as f64).collect();

    simplex_max(n, m, &a_matrix, &b)
}

/// Full-tableau primal simplex.  Initial BFS: all slacks basic.
fn simplex_max(n: usize, m: usize, a: &[Vec<f64>], b: &[f64]) -> usize {
    let cols = n + m + 1; // x_j | slack_i | RHS
    let mut tab = vec![vec![0.0f64; cols]; m + 1];

    for (i, row) in tab.iter_mut().take(m).enumerate() {
        row[..n].copy_from_slice(&a[i][..n]);
        row[n + i] = 1.0;
        row[cols - 1] = b[i];
    }
    for j in 0..n {
        tab[m][j] = -1.0;
    }

    let eps = 1e-9;

    loop {
        // Pricing: most negative reduced cost enters.
        let mut pivot_col = 0;
        let mut best_rc = -eps;
        for j in 0..n + m {
            if tab[m][j] < best_rc {
                best_rc = tab[m][j];
                pivot_col = j;
            }
        }
        if best_rc >= -eps {
            break;
        }

        // Ratio test.
        let mut pivot_row = usize::MAX;
        let mut best_ratio = f64::INFINITY;
        for (i, row) in tab.iter().enumerate().take(m) {
            if row[pivot_col] > eps {
                let ratio = row[cols - 1] / row[pivot_col];
                if ratio < best_ratio - eps || (ratio < best_ratio + eps && i < pivot_row) {
                    best_ratio = ratio;
                    pivot_row = i;
                }
            }
        }
        if pivot_row == usize::MAX {
            break;
        }

        // Pivot.
        let piv = tab[pivot_row][pivot_col];
        for j in 0..cols {
            tab[pivot_row][j] /= piv;
        }
        for i in 0..=m {
            if i == pivot_row {
                continue;
            }
            let factor = tab[i][pivot_col];
            if factor.abs() < eps {
                continue;
            }
            for j in 0..cols {
                tab[i][j] -= factor * tab[pivot_row][j];
            }
        }
    }

    let val = tab[m][cols - 1];
    (val + 0.5) as usize
}

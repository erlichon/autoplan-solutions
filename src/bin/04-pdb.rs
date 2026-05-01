/// week 04: 04-pdb -- pattern database heuristic.
use autoplan::SASPlus;
use std::{
    error::Error,
    io::{Read as _, Write as _, stdin, stdout},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    stdin().read_to_string(&mut data)?;
    let s: &str = data.as_ref();

    let (rest, (problem, state)) = SASPlus::parse(s).expect("could not parse SAS+");

    // Parse pattern line: "s v0 v1 ... v_{s-1}"
    let pattern_line = rest.trim();
    let nums: Vec<usize> = pattern_line
        .split_whitespace()
        .map(|tok| tok.parse().expect("bad pattern integer"))
        .collect();
    let _s = nums[0];
    let pattern: Vec<usize> = nums[1..].to_vec();

    let pdb = problem.build_pdb(&state.values, &pattern);

    let out = stdout();
    let mut out = out.lock();

    // Collect reachable states, sorted lexicographically by their value tuple.
    // Since decode gives values in pattern order (which is ascending variable
    // order) and hash is little-endian, we sort by the decoded tuple.
    let mut entries: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    for h in 0..pdb.total_states {
        if !pdb.reachable[h] {
            continue;
        }
        let vals = pdb.decode(h);
        entries.push((vals, h, pdb.distance[h]));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (vals, hash, dist) in &entries {
        for v in vals {
            write!(out, "{} ", v)?;
        }
        if *dist == usize::MAX {
            writeln!(out, "{} inf", hash)?;
        } else {
            writeln!(out, "{} {}", hash, dist)?;
        }
    }

    Ok(())
}

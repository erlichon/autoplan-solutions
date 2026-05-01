/// week 04: 04-pdb -- validates PDB output against reference .ans files.
/// The output is uniquely defined (lexicographic order, exact hash, exact
/// distances), so we compare the full output string.
use autoplan::SASPlus;
use std::io::Write as _;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/04-PDB-easy" as p04_pdb_easy => test_pdb_exact }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/04-PDB-hard" as p04_pdb_hard => test_pdb_exact }

fn test_pdb_exact([input, expected]: [&'static str; 2]) {
    let (rest, (problem, state)) = SASPlus::parse(input).expect("parse error");

    let pattern_line = rest.trim();
    let nums: Vec<usize> = pattern_line
        .split_whitespace()
        .map(|tok| tok.parse().expect("bad pattern integer"))
        .collect();
    let pattern: Vec<usize> = nums[1..].to_vec();

    let pdb = problem.build_pdb(&state.values, &pattern);

    let mut entries: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    for h in 0..pdb.total_states {
        if !pdb.reachable[h] {
            continue;
        }
        let vals = pdb.decode(h);
        entries.push((vals, h, pdb.distance[h]));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut buf: Vec<u8> = Vec::new();
    for (vals, hash, dist) in &entries {
        for v in vals {
            write!(buf, "{} ", v).unwrap();
        }
        if *dist == usize::MAX {
            writeln!(buf, "{} inf", hash).unwrap();
        } else {
            writeln!(buf, "{} {}", hash, dist).unwrap();
        }
    }
    let got = String::from_utf8(buf).unwrap();
    assert_eq!(expected, got);
}

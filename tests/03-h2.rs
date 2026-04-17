/// week 03: 03-h2 -- validates that h^2 is a tighter admissible heuristic
/// than h^1 on all BFS samples, and matches a hand-traced value on the
/// problem's published sample.
use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-bfs-simple" as p03_h2_bfs_simple => test_h2_vs_h1 }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-BFS-medium" as p03_h2_bfs_medium => test_h2_vs_h1 }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-BFS-hard" as p03_h2_bfs_hard => test_h2_vs_h1 }

// Official sample packs for this problem -- the .ans is the expected
// h^2 value, which is uniquely defined so we compare for exact equality.
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-H2-easy" as p03_h2_official_easy => test_h2_exact }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-H2-hard" as p03_h2_official_hard => test_h2_exact }

fn test_h2_exact([input, expected]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");
    let got = match problem.h2(&state) {
        Some(h) => format!("{h}\n"),
        None => "infinity\n".to_string(),
    };
    assert_eq!(expected, got);
}

fn test_h2_vs_h1([input, expected_bfs_len]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");

    let h1 = problem.h1(&state).expect("h1 should be finite");
    let h2 = problem.h2(&state).expect("h2 should be finite");

    // h^2 is by construction >= h^1 (both are admissible; h^2 dominates h^1).
    assert!(
        h2 >= h1,
        "h^2 ({h2}) must be >= h^1 ({h1}) -- h^2 dominates h^1"
    );

    // Both are admissible: they must not exceed the true optimal plan
    // length (the BFS ground truth).
    let expected_h_star: usize = expected_bfs_len
        .trim()
        .parse()
        .expect("BFS answer is an integer");
    assert!(
        h1 <= expected_h_star,
        "h^1 ({h1}) must be <= h* ({expected_h_star}) (admissible)"
    );
    assert!(
        h2 <= expected_h_star,
        "h^2 ({h2}) must be <= h* ({expected_h_star}) (admissible)"
    );
}

#[test]
fn sample_1_hand_traced() {
    // `tests/samples/01-bfs-simple/1.in` is the identical miconic instance
    // given as Sample Input 1 in the h^2 problem statement. A hand trace
    // gives h^1 = 3 (the shortest single-fact chain to `served`) and
    // h^2 = 4 (the pair {lift-at-f0, boarded} costs 3 under relaxation,
    // and serving requires lift-at-f0 AND boarded simultaneously).
    let input = include_str!("../tests/samples/01-bfs-simple/1.in");
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");

    assert_eq!(problem.h1(&state), Some(3), "h^1 sanity on sample 1");
    assert_eq!(problem.h2(&state), Some(4), "h^2 hand-traced value on sample 1");
}

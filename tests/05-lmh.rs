/// week 05: 05-lmh -- validates canonical landmark heuristic against .ans files.
use autoplan::{canonical_lm_heuristic, parse_landmarks, SASPlus};

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/05-LMH-easy" as p05_lmh_easy => test_lmh }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/05-LMH-hard" as p05_lmh_hard => test_lmh }

fn test_lmh([input, expected]: [&'static str; 2]) {
    let (rest, (problem, _state)) = SASPlus::parse(input).expect("parse error");
    let landmarks = parse_landmarks(rest);
    let h = canonical_lm_heuristic(&problem, &landmarks);
    let expected: usize = expected.trim().parse().expect("bad expected value");
    assert_eq!(h, expected, "h^C mismatch");
}
